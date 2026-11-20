use std::ffi::OsStr;
use std::fs::{self, DirBuilder, Permissions};
use std::io::{self, Read, Write};
use std::net::Shutdown;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
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
const RETRY: Duration = Duration::from_secs(5);
const RETRY_CONNECT: Duration = Duration::from_millis(2);
const POLL: Duration = Duration::from_millis(5);
const IN_FLIGHT: usize = 16;
const LONGEST_CHAIN: usize = 40;
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
    let replies: Vec<Exchange> = thread::scope(|scope| {
        let asked: Vec<_> = sockets
            .iter()
            .map(|socket| scope.spawn(move || exchange(socket, "ask", file, asking)))
            .collect();
        asked
            .into_iter()
            .map(|asked| asked.join().unwrap_or(Exchange::Silent))
            .collect()
    });
    if replies
        .iter()
        .all(|reply| matches!(reply, Exchange::Unreached))
    {
        return Err(none_running());
    }
    if !replies
        .iter()
        .any(|reply| matches!(reply, Exchange::Replied(_)))
    {
        return Err("no harbour answered".to_string());
    }
    let mut chosen: Option<(&Path, u128)> = None;
    for (socket, reply) in sockets.iter().zip(&replies) {
        let used = reply
            .text()
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
    match exchange(socket, "open", file, Instant::now() + OPENING).text() {
        Some("opened") => Ok(()),
        Some(reply) => match reply.strip_prefix("refused ") {
            Some(message) => Err(message.to_string()),
            None => Err("no answer from harbour".to_string()),
        },
        None => Err("no answer from harbour".to_string()),
    }
}

fn resolve(argument: &OsStr) -> Result<PathBuf, String> {
    if argument.as_bytes().len() > LONGEST_PATH {
        return Err(too_long());
    }
    located(Path::new(argument))
}

fn located(given: &Path) -> Result<PathBuf, String> {
    let shown = given.display();
    let directory = || format!("{shown} is a directory");
    let mut path = given.to_path_buf();
    let mut failure = None;
    for _ in 0..LONGEST_CHAIN {
        let error = match path.canonicalize() {
            Ok(found) if found.is_dir() => return Err(directory()),
            Ok(found) => return within_length(found),
            Err(error) => error,
        };
        let name = path.file_name().ok_or_else(directory)?;
        let folder = match path.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent,
            _ => Path::new("."),
        };
        let folder = folder
            .canonicalize()
            .map_err(|error| format!("{shown}: {}", plain(&error)))?;
        let joined = folder.join(name);
        match fs::read_link(&joined) {
            Ok(target) => path = folder.join(target),
            Err(_) => return within_length(joined),
        }
        failure = Some(error);
    }
    Err(format!(
        "{shown}: {}",
        failure.as_ref().map_or_else(String::new, plain)
    ))
}

fn within_length(path: PathBuf) -> Result<PathBuf, String> {
    if path.as_os_str().as_bytes().len() > LONGEST_PATH {
        return Err(too_long());
    }
    Ok(path)
}

fn too_long() -> String {
    format!("the path is longer than {LONGEST_PATH} bytes")
}

enum Exchange {
    Unreached,
    Silent,
    Replied(String),
}

impl Exchange {
    fn text(&self) -> Option<&str> {
        match self {
            Exchange::Replied(text) => Some(text),
            _ => None,
        }
    }
}

fn exchange(socket: &Path, verb: &str, file: &Path, deadline: Instant) -> Exchange {
    let Ok(mut stream) = connect(socket, deadline) else {
        return Exchange::Unreached;
    };
    replied(&mut stream, verb, file, deadline).map_or(Exchange::Silent, Exchange::Replied)
}

fn replied(stream: &mut UnixStream, verb: &str, file: &Path, deadline: Instant) -> Option<String> {
    let expires = unix_millis(SystemTime::now() + deadline.checked_duration_since(Instant::now())?);
    let mut request = format!("{verb} {expires}\0").into_bytes();
    request.extend_from_slice(file.as_os_str().as_bytes());
    let left = deadline.checked_duration_since(Instant::now())?;
    stream.set_write_timeout(Some(left)).ok()?;
    stream.set_read_timeout(Some(left)).ok()?;
    stream.write_all(&request).ok()?;
    stream.shutdown(Shutdown::Write).ok()?;
    let reply = read_until_closed(stream, LONGEST_REPLY, deadline)?;
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
    pub deadline: Instant,
    stream: UnixStream,
    _slot: Slot,
}

impl Request {
    pub fn answer(mut self, reply: Reply) {
        say(&mut self.stream, reply);
    }
}

fn say(stream: &mut UnixStream, reply: Reply) {
    let line = match reply {
        Reply::Holds(used) => format!("holds {}", unix_millis(used)),
        Reply::None => "none".to_string(),
        Reply::Opened => "opened".to_string(),
        Reply::Refused(message) => format!("refused {}", message.replace('\n', " ")),
    };
    let _ = stream.set_nonblocking(true);
    let _ = stream.write_all(format!("{line}\n").as_bytes());
}

struct Slot(Arc<AtomicUsize>);

impl Drop for Slot {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

pub struct Listener {
    requests: Receiver<Request>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Listener {
    pub fn new(dir: &Path) -> Listener {
        Listener::retrying_every(dir, RETRY)
    }

    pub fn retrying_every(dir: &Path, retry: Duration) -> Listener {
        let (sender, requests) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let watching = Watch {
            dir: dir.to_path_buf(),
            retry,
            bound: Bound::create(dir).ok(),
            stop: Arc::clone(&stop),
            sender,
            in_flight: Arc::new(AtomicUsize::new(0)),
        };
        let thread = thread::Builder::new()
            .name("harbour-handoff".to_string())
            .spawn(move || watching.run())
            .ok();
        Listener {
            requests,
            stop,
            thread,
        }
    }

    pub fn incoming(&mut self) -> Option<Request> {
        self.requests.try_recv().ok()
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

struct Watch {
    dir: PathBuf,
    retry: Duration,
    bound: Option<Bound>,
    stop: Arc<AtomicBool>,
    sender: Sender<Request>,
    in_flight: Arc<AtomicUsize>,
}

impl Watch {
    fn run(mut self) {
        let mut due = Instant::now() + self.retry;
        while !self.stop.load(Ordering::Acquire) {
            if Instant::now() >= due {
                due = Instant::now() + self.retry;
                if self.bound.as_ref().is_some_and(|bound| !bound.intact()) {
                    self.bound = None;
                }
                if self.bound.is_none() {
                    self.bound = Bound::create(&self.dir).ok();
                }
            }
            match self.bound.as_ref().map(|bound| bound.listener.accept()) {
                Some(Ok((stream, _))) => self.admit(stream),
                _ => thread::sleep(POLL),
            }
        }
    }

    fn admit(&self, stream: UnixStream) {
        if peer(&stream) != Some(Uid::current()) {
            return;
        }
        if self.in_flight.fetch_add(1, Ordering::AcqRel) >= IN_FLIGHT {
            self.in_flight.fetch_sub(1, Ordering::AcqRel);
            return;
        }
        let slot = Slot(Arc::clone(&self.in_flight));
        let sender = self.sender.clone();
        let _ = thread::Builder::new()
            .name("harbour-handoff-read".to_string())
            .spawn(move || {
                if let Some(request) = received(stream, slot) {
                    let _ = sender.send(request);
                }
            });
    }
}

fn received(stream: UnixStream, slot: Slot) -> Option<Request> {
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
    let left = expires
        .parse::<u128>()
        .ok()?
        .checked_sub(unix_millis(SystemTime::now()))?;
    if left == 0 || file.first() != Some(&b'/') || file.len() > LONGEST_PATH {
        return None;
    }
    let deadline = Instant::now() + Duration::from_millis(u64::try_from(left).ok()?).min(OPENING);
    match located(Path::new(OsStr::from_bytes(file))) {
        Ok(file) => Some(Request {
            verb,
            file,
            deadline,
            stream,
            _slot: slot,
        }),
        Err(refused) => {
            let reply = match verb {
                Verb::Ask => Reply::None,
                Verb::Open => Reply::Refused(refused),
            };
            say(&mut stream, reply);
            None
        }
    }
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
