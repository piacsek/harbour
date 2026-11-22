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
        let bound: [&OsStr; 4] = [
            "list-keys".as_ref(),
            "-T".as_ref(),
            "root".as_ref(),
            key.as_ref(),
        ];
        if self.run(&bound, Stdio::null(), deadline).is_none() {
            return false;
        }
        let check: [&OsStr; 5] = [
            "display-message".as_ref(),
            "-p".as_ref(),
            "-t".as_ref(),
            pane,
            edge.as_ref(),
        ];
        let answer = self
            .run(&check, Stdio::piped(), deadline)
            .and_then(|mut child| child.stdout.take())
            .and_then(|stdout| answer(stdout, deadline));
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

    fn run(&self, args: &[&OsStr], stdout: Stdio, deadline: Instant) -> Option<Child> {
        let mut child = self.spawn(args, stdout)?;
        waited(&mut child, deadline)?.success().then_some(child)
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

fn answer(mut stdout: ChildStdout, deadline: Instant) -> Option<Vec<u8>> {
    let timeout = PollTimeout::try_from(deadline.saturating_duration_since(Instant::now()))
        .unwrap_or(PollTimeout::ZERO);
    let mut watched = [PollFd::new(stdout.as_fd(), PollFlags::POLLIN)];
    if nix::poll::poll(&mut watched, timeout).ok()? == 0 {
        return None;
    }
    let mut buffer = [0; 64];
    let read = stdout.read(&mut buffer).ok()?;
    Some(buffer[..read].to_vec())
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
