use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use nix::errno::Errno;
use nix::sys::socket::{AddressFamily, SockFlag, SockType, UnixAddr};
use rmpv::Value;

pub const PATIENCE: Duration = Duration::from_millis(100);

const READ_AT_MOST: usize = 1024 * 1024;
const RETRY_CONNECT: Duration = Duration::from_millis(2);
const REQUEST_ID: i64 = 1;
const WATCH: Duration = Duration::from_millis(50);

pub struct Mode {
    pub mode: String,
    pub blocking: bool,
}

impl Mode {
    pub fn waiting(&self) -> bool {
        self.blocking || self.mode.starts_with(['c', 'r']) || self.mode.starts_with("no")
    }
}

pub enum Edited {
    Done,
    Refused(String),
    Waiting,
}

enum Reading {
    Answer(Result<Value, Value>),
    Pending,
    Gone,
}

pub fn mode(socket: &Path, deadline: Instant) -> Option<Mode> {
    let reply = request(socket, "nvim_get_mode", Vec::new(), deadline)?;
    let field = |name: &str| {
        reply
            .as_map()?
            .iter()
            .find(|(key, _)| key.as_str() == Some(name))
            .map(|(_, value)| value.clone())
    };
    Some(Mode {
        mode: field("mode")?.as_str()?.to_string(),
        blocking: field("blocking")?.as_bool()?,
    })
}

pub fn eval(socket: &Path, expression: &str, deadline: Instant) -> Option<i64> {
    request(socket, "nvim_eval", vec![Value::from(expression)], deadline)?.as_i64()
}

pub fn edit(socket: &Path, file: &Path, deadline: Instant) -> Option<Edited> {
    let off = Value::Map(vec![
        (Value::from("file"), Value::from(false)),
        (Value::from("bar"), Value::from(false)),
    ]);
    let command = Value::Map(vec![
        (Value::from("cmd"), Value::from("edit")),
        (
            Value::from("args"),
            Value::Array(vec![Value::Binary(file.as_os_str().as_bytes().to_vec())]),
        ),
        (Value::from("magic"), off),
    ]);
    let arguments = vec![command, Value::Map(Vec::new())];
    let mut stream = send(socket, "nvim_cmd", arguments, deadline)?;
    let mut received = Vec::new();
    loop {
        let slice = deadline.min(Instant::now() + WATCH);
        match read_reply(&mut stream, &mut received, slice) {
            Reading::Answer(Ok(_)) => return Some(Edited::Done),
            Reading::Answer(Err(error)) => return Some(Edited::Refused(message(&error))),
            Reading::Gone => return None,
            Reading::Pending if Instant::now() >= deadline => return None,
            Reading::Pending => {
                let probe = deadline.min(Instant::now() + WATCH);
                if mode(socket, probe).is_some_and(|mode| mode.waiting()) {
                    return Some(Edited::Waiting);
                }
            }
        }
    }
}

fn message(error: &Value) -> String {
    match error.as_array().map(Vec::as_slice) {
        Some([_, text]) if text.is_str() => text.as_str().unwrap_or_default().to_string(),
        _ => error.to_string(),
    }
}

fn request(socket: &Path, method: &str, arguments: Vec<Value>, deadline: Instant) -> Option<Value> {
    call(socket, method, arguments, deadline)?.ok()
}

fn call(
    socket: &Path,
    method: &str,
    arguments: Vec<Value>,
    deadline: Instant,
) -> Option<Result<Value, Value>> {
    let mut stream = send(socket, method, arguments, deadline)?;
    match read_reply(&mut stream, &mut Vec::new(), deadline) {
        Reading::Answer(answer) => Some(answer),
        Reading::Pending | Reading::Gone => None,
    }
}

fn send(
    socket: &Path,
    method: &str,
    arguments: Vec<Value>,
    deadline: Instant,
) -> Option<UnixStream> {
    let mut stream = connect(socket, deadline)?;
    let patience = deadline.checked_duration_since(Instant::now())?;
    let request = Value::Array(vec![
        Value::from(0),
        Value::from(REQUEST_ID),
        Value::from(method),
        Value::Array(arguments),
    ]);
    let mut bytes = Vec::new();
    rmpv::encode::write_value(&mut bytes, &request).ok()?;
    stream.set_write_timeout(Some(patience)).ok()?;
    stream.write_all(&bytes).ok()?;
    Some(stream)
}

fn connect(socket: &Path, deadline: Instant) -> Option<UnixStream> {
    let address = UnixAddr::new(socket).ok()?;
    let descriptor = nix::sys::socket::socket(
        AddressFamily::Unix,
        SockType::Stream,
        SockFlag::empty(),
        None,
    )
    .ok()?;
    let stream = UnixStream::from(descriptor);
    stream.set_nonblocking(true).ok()?;
    loop {
        match nix::sys::socket::connect(stream.as_raw_fd(), &address) {
            Ok(()) | Err(Errno::EISCONN) => break,
            Err(Errno::EAGAIN | Errno::EINPROGRESS | Errno::EALREADY | Errno::EINTR) => {
                if Instant::now() >= deadline {
                    return None;
                }
                thread::sleep(RETRY_CONNECT);
            }
            Err(_) => return None,
        }
    }
    stream.set_nonblocking(false).ok()?;
    Some(stream)
}

fn read_reply(stream: &mut UnixStream, received: &mut Vec<u8>, until: Instant) -> Reading {
    let mut buffer = [0; 4096];
    loop {
        while let Some((message, used)) = next_message(received) {
            received.drain(..used);
            if let Some(answer) = response(&message) {
                return Reading::Answer(answer);
            }
        }
        if received.len() > READ_AT_MOST {
            return Reading::Gone;
        }
        let Some(left) = until.checked_duration_since(Instant::now()) else {
            return Reading::Pending;
        };
        let _ = stream.set_read_timeout(Some(left.max(Duration::from_millis(1))));
        match stream.read(&mut buffer) {
            Ok(0) => return Reading::Gone,
            Ok(read) => received.extend_from_slice(&buffer[..read]),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                return Reading::Pending;
            }
            Err(_) => return Reading::Gone,
        }
    }
}

fn next_message(received: &[u8]) -> Option<(Value, usize)> {
    let mut rest = received;
    let message = rmpv::decode::read_value(&mut rest).ok()?;
    Some((message, received.len() - rest.len()))
}

fn response(message: &Value) -> Option<Result<Value, Value>> {
    match message.as_array()?.as_slice() {
        [kind, id, error, result]
            if kind.as_i64() == Some(1) && id.as_i64() == Some(REQUEST_ID) =>
        {
            Some(if error.is_nil() {
                Ok(result.clone())
            } else {
                Err(error.clone())
            })
        }
        _ => None,
    }
}
