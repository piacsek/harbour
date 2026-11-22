use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::os::fd::AsFd;
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
}

impl Tmux {
    pub fn from_env() -> Option<Tmux> {
        Tmux::from_vars(std::env::var_os("TMUX"), std::env::var_os("TMUX_PANE"))
    }

    pub fn from_vars(tmux: Option<OsString>, pane: Option<OsString>) -> Option<Tmux> {
        tmux.filter(|tmux| !tmux.is_empty())?;
        let pane = pane.filter(|pane| !pane.is_empty())?;
        Some(Tmux::with_program(vec!["tmux".into()], pane))
    }

    pub fn with_program(program: Vec<OsString>, pane: OsString) -> Tmux {
        Tmux { program, pane }
    }

    pub fn select(&self, direction: Direction) -> bool {
        let deadline = Instant::now() + DEADLINE;
        let (key, edge, flag) = match direction {
            Direction::Below => (
                "C-j",
                "#{pane_active}#{window_active}#{pane_at_bottom}",
                "-D",
            ),
            Direction::Above => ("C-k", "#{pane_active}#{window_active}#{pane_at_top}", "-U"),
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
        answer.is_some_and(|answer| answer.trim_ascii() == b"110")
            && self
                .run(
                    &["select-pane".as_ref(), flag.as_ref(), "-t".as_ref(), pane],
                    Stdio::null(),
                    deadline,
                )
                .is_some()
    }

    fn run(&self, args: &[&OsStr], stdout: Stdio, deadline: Instant) -> Option<Child> {
        let (program, before) = self.program.split_first()?;
        let mut child = Command::new(program)
            .args(before)
            .args(args)
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        waited(&mut child, deadline)?.success().then_some(child)
    }
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
