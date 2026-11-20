use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
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

fn request(socket: &Path, method: &str, arguments: Vec<Value>, deadline: Instant) -> Option<Value> {
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

fn read_reply(stream: &mut UnixStream, deadline: Instant) -> Option<Value> {
    let mut received = Vec::new();
    let mut buffer = [0; 4096];
    loop {
        while let Some((message, used)) = next_message(&received) {
            received.drain(..used);
            if let Some(answer) = response(&message) {
                return answer;
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

fn response(message: &Value) -> Option<Option<Value>> {
    match message.as_array()?.as_slice() {
        [kind, id, error, result]
            if kind.as_i64() == Some(1) && id.as_i64() == Some(REQUEST_ID) =>
        {
            Some(error.is_nil().then(|| result.clone()))
        }
        _ => None,
    }
}
