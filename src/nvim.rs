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

pub struct Mode {
    pub mode: String,
    pub blocking: bool,
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

pub fn edit(socket: &Path, file: &Path, deadline: Instant) -> Option<Result<(), String>> {
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
    match call(socket, "nvim_cmd", arguments, deadline)? {
        Ok(_) => Some(Ok(())),
        Err(error) => Some(Err(message(&error))),
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
    read_reply(&mut stream, deadline)
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

fn read_reply(stream: &mut UnixStream, deadline: Instant) -> Option<Result<Value, Value>> {
    let mut received = Vec::new();
    let mut buffer = [0; 4096];
    loop {
        while let Some((message, used)) = next_message(&received) {
            received.drain(..used);
            if let Some(answer) = response(&message) {
                return Some(answer);
            }
        }
        if received.len() > READ_AT_MOST {
            return None;
        }
        let left = deadline.checked_duration_since(Instant::now())?;
        let _ = stream.set_read_timeout(Some(left.max(Duration::from_millis(1))));
        match stream.read(&mut buffer) {
            Ok(0) => return None,
            Ok(read) => received.extend_from_slice(&buffer[..read]),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(_) => return None,
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
