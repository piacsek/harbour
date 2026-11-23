use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::os::fd::AsFd;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use nix::poll::{PollFd, PollFlags, PollTimeout};

const DEADLINE: Duration = Duration::from_millis(100);
const POLL: Duration = Duration::from_millis(2);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Below,
    Above,
}

pub struct Tmux {
    program: Vec<OsString>,
    pane: OsString,
    tty: OsString,
    deadline: Duration,
}

impl Tmux {
    pub fn from_env() -> Option<Tmux> {
        let pane = Tmux::pane(std::env::var_os("TMUX"), std::env::var_os("TMUX_PANE"))?;
        let tty = nix::unistd::ttyname(std::io::stdin()).ok()?;
        let opened = std::env::current_dir().ok()?;
        let path = std::env::var_os("PATH").unwrap_or_default();
        let program = located("tmux", &path, &opened)?;
        Some(Tmux::with_program(
            vec![program.into()],
            pane,
            tty.into(),
            DEADLINE,
        ))
    }

    pub fn pane(tmux: Option<OsString>, pane: Option<OsString>) -> Option<OsString> {
        tmux.filter(|tmux| !tmux.is_empty())?;
        pane.filter(|pane| !pane.is_empty())
    }

    pub fn with_program(
        program: Vec<OsString>,
        pane: OsString,
        tty: OsString,
        deadline: Duration,
    ) -> Tmux {
        Tmux {
            program,
            pane,
            tty,
            deadline,
        }
    }

    pub fn select(&self, direction: Direction) -> bool {
        let deadline = Instant::now() + self.deadline;
        let (key, edge, flag) = match direction {
            Direction::Below => (
                "C-j",
                "#{pane_active}#{window_active}#{pane_at_bottom} #{pane_tty}",
                "-D",
            ),
            Direction::Above => (
                "C-k",
                "#{pane_active}#{window_active}#{pane_at_top} #{pane_tty}",
                "-U",
            ),
        };
        let pane = self.pane.as_os_str();
        let listed: [&OsStr; 3] = ["list-keys".as_ref(), "-T".as_ref(), "root".as_ref()];
        if !self
            .output(&listed, deadline)
            .is_some_and(|listed| binds(&listed, key))
        {
            return false;
        }
        let check: [&OsStr; 5] = [
            "display-message".as_ref(),
            "-p".as_ref(),
            "-t".as_ref(),
            pane,
            edge.as_ref(),
        ];
        let answer = self.output(&check, deadline);
        let expected = [b"110 ", self.tty.as_bytes()].concat();
        if answer.is_none_or(|answer| answer.trim_ascii_end() != expected) {
            return false;
        }
        let select: [&OsStr; 4] = ["select-pane".as_ref(), flag.as_ref(), "-t".as_ref(), pane];
        let Some(mut child) = self.spawn(&select, Stdio::null()) else {
            return false;
        };
        waited(&mut child, Instant::now() + self.deadline).is_none_or(|status| status.success())
    }

    fn output(&self, args: &[&OsStr], deadline: Instant) -> Option<Vec<u8>> {
        let mut child = self.spawn(args, Stdio::piped())?;
        waited(&mut child, deadline)?.success().then_some(())?;
        answer(child.stdout.take()?, deadline)
    }

    fn spawn(&self, args: &[&OsStr], stdout: Stdio) -> Option<Child> {
        let (program, before) = self.program.split_first()?;
        Command::new(program)
            .args(before)
            .args(args)
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(Stdio::null())
            .spawn()
            .ok()
    }
}

pub fn located(program: &str, path: &OsStr, opened: &Path) -> Option<PathBuf> {
    std::env::split_paths(path)
        .map(|directory| opened.join(directory).join(program))
        .find(|candidate| crate::picker::executable(candidate))
}

fn binds(listed: &[u8], key: &str) -> bool {
    listed.split(|&byte| byte == b'\n').any(|line| {
        let mut words = line
            .split(u8::is_ascii_whitespace)
            .filter(|word| !word.is_empty())
            .skip_while(|word| *word != b"-T");
        words.nth(1) == Some(b"root".as_slice()) && words.next() == Some(key.as_bytes())
    })
}

fn answer(mut stdout: ChildStdout, deadline: Instant) -> Option<Vec<u8>> {
    let mut answer = Vec::new();
    let mut buffer = [0; 4096];
    loop {
        let timeout = if answer.is_empty() {
            PollTimeout::try_from(deadline.saturating_duration_since(Instant::now()))
                .unwrap_or(PollTimeout::ZERO)
        } else {
            PollTimeout::ZERO
        };
        let mut watched = [PollFd::new(stdout.as_fd(), PollFlags::POLLIN)];
        if nix::poll::poll(&mut watched, timeout).ok()? == 0 {
            break;
        }
        let read = stdout.read(&mut buffer).ok()?;
        if read == 0 {
            break;
        }
        answer.extend_from_slice(&buffer[..read]);
    }
    (!answer.is_empty()).then_some(answer)
}

fn waited(child: &mut Child, deadline: Instant) -> Option<ExitStatus> {
    loop {
        if let Some(status) = child.try_wait().ok()? {
            return Some(status);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        std::thread::sleep(POLL);
    }
}
