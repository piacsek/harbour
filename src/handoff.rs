use std::ffi::{OsStr, OsString};
use std::fs::{self, DirBuilder, Permissions};
use std::io::{self, Read, Write};
use std::net::Shutdown;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use nix::errno::Errno;
use nix::sys::socket::{AddressFamily, SockFlag, SockType, UnixAddr};
use nix::unistd::Uid;

const LONGEST_PATH: usize = 4096;
const LONGEST_REQUEST: usize = LONGEST_PATH + 64;
const LONGEST_REPLY: usize = 4096;
const ASKING: Duration = Duration::from_secs(2);
const OPENING: Duration = Duration::from_secs(5);
const ARRIVING: Duration = Duration::from_millis(100);
const ANSWERING: Duration = Duration::from_millis(100);
const RETRY: Duration = Duration::from_secs(5);
const RETRY_CONNECT: Duration = Duration::from_millis(2);
const SOCKET: &str = "sock";
const STAGED: &str = "new";

static BOUND: AtomicU64 = AtomicU64::new(0);

pub fn directory_from_env() -> Option<PathBuf> {
    [
        std::env::var_os("XDG_STATE_HOME").map(PathBuf::from),
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")),
    ]
    .into_iter()
    .flatten()
    .find(|dir| dir.is_absolute())
    .map(|dir| dir.join("harbour/edit"))
}

pub fn edit(argument: &OsStr) -> Result<(), String> {
    let file = resolve(argument)?;
    send(directory_from_env().as_deref(), &file)
}

pub fn send(dir: Option<&Path>, file: &Path) -> Result<(), String> {
    let none_running = || "no harbour is running".to_string();
    let dir = dir.ok_or_else(none_running)?;
    match fs::symlink_metadata(dir) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Err(none_running()),
        Err(error) => return Err(format!("{}: {}", dir.display(), plain(&error))),
        Ok(meta) if !private(&meta) => {
            return Err(format!(
                "{} is not private; refusing to use it",
                dir.display()
            ));
        }
        Ok(_) => {}
    }
    let sockets = sockets(dir, &[SOCKET]);
    let asking = Instant::now() + ASKING;
    let replies: Vec<Option<String>> = thread::scope(|scope| {
        let asked: Vec<_> = sockets
            .iter()
            .map(|socket| scope.spawn(move || exchange(socket, "ask", file, asking)))
            .collect();
        asked
            .into_iter()
            .map(|asked| asked.join().ok().flatten())
            .collect()
    });
    if replies.iter().all(Option::is_none) {
        return Err(none_running());
    }
    let mut chosen: Option<(&Path, u128)> = None;
    for (socket, reply) in sockets.iter().zip(&replies) {
        let used = reply
            .as_deref()
            .and_then(|reply| reply.strip_prefix("holds "))
            .and_then(|used| used.parse::<u128>().ok());
        if let Some(used) = used
            && chosen.is_none_or(|(_, best)| used > best)
        {
            chosen = Some((socket, used));
        }
    }
    let Some((socket, _)) = chosen else {
        return Err(format!("no harbour session holds {}", file.display()));
    };
    match exchange(socket, "open", file, Instant::now() + OPENING).as_deref() {
        Some("opened") => Ok(()),
        Some(reply) => match reply.strip_prefix("refused ") {
            Some(message) => Err(message.to_string()),
            None => Err("no answer from harbour".to_string()),
        },
        None => Err("no answer from harbour".to_string()),
    }
}

fn resolve(argument: &OsStr) -> Result<PathBuf, String> {
    let shown = Path::new(argument).display();
    let too_long = || format!("the path is longer than {LONGEST_PATH} bytes");
    if argument.as_bytes().len() > LONGEST_PATH {
        return Err(too_long());
    }
    let given = Path::new(argument);
    let resolved = match given.canonicalize() {
        Ok(path) if path.is_dir() => return Err(format!("{shown} is a directory")),
        Ok(path) => path,
        Err(_) => {
            let name = given
                .file_name()
                .ok_or_else(|| format!("{shown} is a directory"))?;
            let folder = match given.parent() {
                Some(parent) if !parent.as_os_str().is_empty() => parent,
                _ => Path::new("."),
            };
            folder
                .canonicalize()
                .map_err(|error| format!("{shown}: {}", plain(&error)))?
                .join(name)
        }
    };
    if resolved.as_os_str().as_bytes().len() > LONGEST_PATH {
        return Err(too_long());
    }
    Ok(resolved)
}

fn exchange(socket: &Path, verb: &str, file: &Path, deadline: Instant) -> Option<String> {
    let mut stream = connect(socket, deadline).ok()?;
    let expires = unix_millis(SystemTime::now() + deadline.checked_duration_since(Instant::now())?);
    let mut request = format!("{verb} {expires}\0").into_bytes();
    request.extend_from_slice(file.as_os_str().as_bytes());
    let left = deadline.checked_duration_since(Instant::now())?;
    stream.set_write_timeout(Some(left)).ok()?;
    stream.set_read_timeout(Some(left)).ok()?;
    stream.write_all(&request).ok()?;
    stream.shutdown(Shutdown::Write).ok()?;
    let reply = read_until_closed(&mut stream, LONGEST_REPLY, deadline)?;
    let line = reply.split(|byte| *byte == b'\n').next()?;
    Some(String::from_utf8_lossy(line).into_owned())
}

fn connect(socket: &Path, deadline: Instant) -> Result<UnixStream, Errno> {
    let address = UnixAddr::new(socket)?;
    let descriptor = nix::sys::socket::socket(
        AddressFamily::Unix,
        SockType::Stream,
        SockFlag::empty(),
        None,
    )?;
    let stream = UnixStream::from(descriptor);
    stream.set_nonblocking(true).map_err(|_| Errno::EIO)?;
    loop {
        match nix::sys::socket::connect(stream.as_raw_fd(), &address) {
            Ok(()) | Err(Errno::EISCONN) => break,
            Err(Errno::EAGAIN | Errno::EINPROGRESS | Errno::EALREADY | Errno::EINTR) => {
                if Instant::now() >= deadline {
                    return Err(Errno::ETIMEDOUT);
                }
                thread::sleep(RETRY_CONNECT);
            }
            Err(error) => return Err(error),
        }
    }
    stream.set_nonblocking(false).map_err(|_| Errno::EIO)?;
    Ok(stream)
}

fn read_until_closed(stream: &mut UnixStream, most: usize, deadline: Instant) -> Option<Vec<u8>> {
    let mut received = Vec::new();
    let mut buffer = [0; 4096];
    loop {
        let left = deadline.checked_duration_since(Instant::now())?;
        let _ = stream.set_read_timeout(Some(left.max(Duration::from_millis(1))));
        match stream.read(&mut buffer) {
            Ok(0) => return Some(received),
            Ok(read) => {
                received.extend_from_slice(&buffer[..read]);
                if received.len() > most {
                    return None;
                }
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(_) => return None,
        }
    }
}

fn sockets(dir: &Path, extensions: &[&str]) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_socket()))
        .map(|entry| entry.path())
        .filter(|path| harbours_name(path, extensions))
        .collect();
    found.sort();
    found
}

fn harbours_name(path: &Path, extensions: &[&str]) -> bool {
    let Some(name) = path.file_name().and_then(OsStr::to_str) else {
        return false;
    };
    let Some((stem, extension)) = name.rsplit_once('.') else {
        return false;
    };
    let parts: Vec<&str> = stem.split('-').collect();
    extensions.contains(&extension)
        && parts.len() == 2
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

fn private(meta: &fs::Metadata) -> bool {
    meta.is_dir() && meta.uid() == Uid::current().as_raw() && meta.mode() & 0o077 == 0
}

fn unix_millis(at: SystemTime) -> u128 {
    at.duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |since| since.as_millis())
}

fn plain(error: &io::Error) -> String {
    let text = error.to_string();
    match text.find(" (os error") {
        Some(end) => text[..end].to_string(),
        None => text,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    Ask,
    Open,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Reply {
    Holds(SystemTime),
    None,
    Opened,
    Refused(String),
}

pub struct Request {
    pub verb: Verb,
    pub file: PathBuf,
    stream: UnixStream,
}

impl Request {
    pub fn answer(mut self, reply: Reply) {
        let line = match reply {
            Reply::Holds(used) => format!("holds {}", unix_millis(used)),
            Reply::None => "none".to_string(),
            Reply::Opened => "opened".to_string(),
            Reply::Refused(message) => format!("refused {}", message.replace('\n', " ")),
        };
        let _ = self.stream.set_write_timeout(Some(ANSWERING));
        let _ = self.stream.write_all(format!("{line}\n").as_bytes());
    }
}

pub struct Listener {
    dir: PathBuf,
    retry: Duration,
    due: Instant,
    bound: Option<Bound>,
}

impl Listener {
    pub fn new(dir: &Path) -> Listener {
        Listener {
            dir: dir.to_path_buf(),
            retry: RETRY,
            due: Instant::now(),
            bound: None,
        }
    }

    pub fn retrying_every(mut self, retry: Duration) -> Listener {
        self.retry = retry;
        self
    }

    pub fn incoming(&mut self) -> Option<Request> {
        let now = Instant::now();
        if now >= self.due {
            self.due = now + self.retry;
            if self.bound.as_ref().is_some_and(|bound| !bound.intact()) {
                self.bound = None;
            }
            if self.bound.is_none() {
                self.bound = Bound::create(&self.dir).ok();
            }
        }
        let bound = self.bound.as_ref()?;
        loop {
            let (stream, _) = bound.listener.accept().ok()?;
            if let Some(request) = received(stream) {
                return Some(request);
            }
        }
    }
}

fn received(stream: UnixStream) -> Option<Request> {
    if peer(&stream)? != Uid::current() {
        return None;
    }
    stream.set_nonblocking(false).ok()?;
    let mut stream = stream;
    let bytes = read_until_closed(&mut stream, LONGEST_REQUEST, Instant::now() + ARRIVING)?;
    let split = bytes.iter().position(|byte| *byte == 0)?;
    let (head, file) = (&bytes[..split], &bytes[split + 1..]);
    let (verb, expires) = std::str::from_utf8(head).ok()?.split_once(' ')?;
    let verb = match verb {
        "ask" => Verb::Ask,
        "open" => Verb::Open,
        _ => return None,
    };
    if expires.parse::<u128>().ok()? <= unix_millis(SystemTime::now()) {
        return None;
    }
    if file.first() != Some(&b'/') || file.len() > LONGEST_PATH {
        return None;
    }
    Some(Request {
        verb,
        file: PathBuf::from(OsString::from_vec(file.to_vec())),
        stream,
    })
}

#[cfg(target_os = "linux")]
fn peer(stream: &UnixStream) -> Option<Uid> {
    let credentials =
        nix::sys::socket::getsockopt(stream, nix::sys::socket::sockopt::PeerCredentials).ok()?;
    Some(Uid::from_raw(credentials.uid()))
}

#[cfg(not(target_os = "linux"))]
fn peer(stream: &UnixStream) -> Option<Uid> {
    nix::unistd::getpeereid(stream).ok().map(|(uid, _)| uid)
}

struct Bound {
    path: PathBuf,
    listener: UnixListener,
    inode: u64,
}

impl Bound {
    fn create(dir: &Path) -> io::Result<Bound> {
        DirBuilder::new().recursive(true).mode(0o700).create(dir)?;
        let meta = fs::symlink_metadata(dir)?;
        if !meta.is_dir() || meta.uid() != Uid::current().as_raw() {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        fs::set_permissions(dir, Permissions::from_mode(0o700))?;
        sweep(dir);
        let name = format!(
            "{}-{}",
            std::process::id(),
            BOUND.fetch_add(1, Ordering::Relaxed)
        );
        let staged = dir.join(format!("{name}.{STAGED}"));
        let path = dir.join(format!("{name}.{SOCKET}"));
        let listener = UnixListener::bind(&staged)?;
        let placed = fs::set_permissions(&staged, Permissions::from_mode(0o600))
            .and_then(|()| listener.set_nonblocking(true))
            .and_then(|()| fs::rename(&staged, &path));
        if let Err(error) = placed {
            let _ = fs::remove_file(&staged);
            return Err(error);
        }
        let inode = fs::symlink_metadata(&path)?.ino();
        Ok(Bound {
            path,
            listener,
            inode,
        })
    }

    fn intact(&self) -> bool {
        fs::symlink_metadata(&self.path).is_ok_and(|meta| meta.ino() == self.inode)
    }
}

impl Drop for Bound {
    fn drop(&mut self) {
        if self.intact() {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn sweep(dir: &Path) {
    let soon = Instant::now() + ARRIVING;
    for socket in sockets(dir, &[SOCKET, STAGED]) {
        if matches!(connect(&socket, soon), Err(Errno::ECONNREFUSED)) {
            let _ = fs::remove_file(&socket);
        }
    }
}
