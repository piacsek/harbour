use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::{Duration, Instant};

use rmpv::Value;

pub const PATIENCE: Duration = Duration::from_millis(100);

pub fn blocking(socket: &Path, deadline: Instant) -> Option<bool> {
    let mode = request(socket, "nvim_get_mode", Vec::new(), deadline)?;
    mode.as_map()?
        .iter()
        .find(|(key, _)| key.as_str() == Some("blocking"))?
        .1
        .as_bool()
}

pub fn eval(socket: &Path, expression: &str, deadline: Instant) -> Option<i64> {
    request(socket, "nvim_eval", vec![Value::from(expression)], deadline)?.as_i64()
}

fn request(socket: &Path, method: &str, arguments: Vec<Value>, deadline: Instant) -> Option<Value> {
    let patience = deadline.checked_duration_since(Instant::now())?;
    let mut stream = UnixStream::connect(socket).ok()?;
    let request = Value::Array(vec![
        Value::from(0),
        Value::from(1),
        Value::from(method),
        Value::Array(arguments),
    ]);
    let mut bytes = Vec::new();
    rmpv::encode::write_value(&mut bytes, &request).ok()?;
    stream.set_write_timeout(Some(patience)).ok()?;
    stream.write_all(&bytes).ok()?;
    let reply = read_reply(&mut stream, deadline)?;
    match reply.as_array()?.as_slice() {
        [kind, id, Value::Nil, result] if kind.as_i64() == Some(1) && id.as_i64() == Some(1) => {
            Some(result.clone())
        }
        _ => None,
    }
}

fn read_reply(stream: &mut UnixStream, deadline: Instant) -> Option<Value> {
    let mut received = Vec::new();
    let mut buffer = [0; 4096];
    loop {
        if let Ok(value) = rmpv::decode::read_value(&mut received.as_slice()) {
            return Some(value);
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
