use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::{Duration, Instant};

use rmpv::Value;

const PATIENCE: Duration = Duration::from_millis(100);

pub fn eval(socket: &Path, expression: &str) -> Option<i64> {
    let deadline = Instant::now() + PATIENCE;
    let mut stream = UnixStream::connect(socket).ok()?;
    let request = Value::Array(vec![
        Value::from(0),
        Value::from(1),
        Value::from("nvim_eval"),
        Value::Array(vec![Value::from(expression)]),
    ]);
    let mut bytes = Vec::new();
    rmpv::encode::write_value(&mut bytes, &request).ok()?;
    stream.set_write_timeout(Some(PATIENCE)).ok()?;
    stream.write_all(&bytes).ok()?;
    let reply = read_reply(&mut stream, deadline)?;
    match reply.as_array()?.as_slice() {
        [kind, id, Value::Nil, result] if kind.as_i64() == Some(1) && id.as_i64() == Some(1) => {
            result.as_i64()
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
        stream
            .set_read_timeout(Some(left.max(Duration::from_millis(1))))
            .ok()?;
        match stream.read(&mut buffer) {
            Ok(0) => return None,
            Ok(read) => received.extend_from_slice(&buffer[..read]),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(_) => return None,
        }
    }
}
