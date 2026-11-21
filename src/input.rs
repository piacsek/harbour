use std::io;
use std::os::fd::{AsRawFd, BorrowedFd};
use std::time::Duration;

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use nix::errno::Errno;
use nix::poll::{PollFd, PollFlags, PollTimeout};

const CHUNK: usize = 1024;
const PENDING_CAP: usize = 64;
const PASTE_START: &[u8] = b"\x1b[200~";

#[derive(Default)]
pub struct Input {
    buffer: Vec<u8>,
}

enum Step {
    Wait,
    Skip,
    Emit(Event),
}

impl Input {
    pub fn read(&mut self, fd: BorrowedFd, timeout: Duration) -> io::Result<Vec<Event>> {
        self.read_with(|chunk, wait| read_fd(fd, chunk, wait), timeout)
    }

    pub fn read_with(
        &mut self,
        mut read: impl FnMut(&mut [u8], Duration) -> io::Result<usize>,
        timeout: Duration,
    ) -> io::Result<Vec<Event>> {
        let mut chunk = [0; CHUNK];
        let mut events = Vec::new();
        let mut wait = timeout;
        loop {
            let read = read(&mut chunk, wait)?;
            events.extend(self.feed(&chunk[..read], read == CHUNK));
            if read == 0 || (read < CHUNK && self.buffer.is_empty()) {
                return Ok(events);
            }
            wait = Duration::ZERO;
        }
    }

    pub fn feed(&mut self, bytes: &[u8], full: bool) -> Vec<Event> {
        let mut events = Vec::new();
        for (index, byte) in bytes.iter().enumerate() {
            if !self.buffer.is_empty()
                && !self.buffer.starts_with(PASTE_START)
                && (*byte == 0x1b || self.buffer.len() >= PENDING_CAP)
            {
                self.buffer.clear();
            }
            self.buffer.push(*byte);
            match parse(&self.buffer, index + 1 < bytes.len() || full) {
                Step::Wait => {}
                Step::Skip => self.buffer.clear(),
                Step::Emit(event) => {
                    events.push(event);
                    self.buffer.clear();
                }
            }
        }
        events
    }
}

fn read_fd(fd: BorrowedFd, chunk: &mut [u8], timeout: Duration) -> io::Result<usize> {
    let timeout = PollTimeout::try_from(timeout).unwrap_or(PollTimeout::MAX);
    let mut watched = [PollFd::new(fd, PollFlags::POLLIN)];
    match nix::poll::poll(&mut watched, timeout) {
        Ok(0) | Err(Errno::EINTR) => return Ok(0),
        Ok(_) => {}
        Err(error) => return Err(error.into()),
    }
    match nix::unistd::read(fd.as_raw_fd(), chunk) {
        Ok(0) => Err(io::ErrorKind::UnexpectedEof.into()),
        Ok(read) => Ok(read),
        Err(Errno::EINTR | Errno::EAGAIN) => Ok(0),
        Err(error) => Err(error.into()),
    }
}

fn parse(buffer: &[u8], more: bool) -> Step {
    match buffer {
        [] => Step::Wait,
        [0x1b] if more => Step::Wait,
        [0x1b] => key(KeyCode::Esc),
        [0x1b, b'O'] => Step::Wait,
        [0x1b, b'O', last, ..] => match last {
            b'D' => key(KeyCode::Left),
            b'C' => key(KeyCode::Right),
            b'A' => key(KeyCode::Up),
            b'B' => key(KeyCode::Down),
            b'H' => key(KeyCode::Home),
            b'F' => key(KeyCode::End),
            b'P'..=b'S' => key(KeyCode::F(1 + last - b'P')),
            _ => Step::Skip,
        },
        [0x1b, b'[', ..] => csi(buffer),
        [0x1b, rest @ ..] => match parse(rest, more) {
            Step::Emit(Event::Key(mut alt)) => {
                alt.modifiers |= KeyModifiers::ALT;
                Step::Emit(Event::Key(alt))
            }
            step => step,
        },
        [b'\r', ..] => key(KeyCode::Enter),
        [b'\t', ..] => key(KeyCode::Tab),
        [0x7f, ..] => key(KeyCode::Backspace),
        [c @ 0x01..=0x1a, ..] => ctrl(char::from(c - 0x01 + b'a')),
        [0x1c, ..] => ctrl('\\'),
        [0x1d, ..] => ctrl(']'),
        [0x1e, ..] => ctrl('^'),
        [0x1f, ..] => ctrl('_'),
        [0, ..] => ctrl(' '),
        _ => utf8(buffer),
    }
}

fn csi(buffer: &[u8]) -> Step {
    let Some(&third) = buffer.get(2) else {
        return Step::Wait;
    };
    let last = buffer[buffer.len() - 1];
    match third {
        b'[' => match buffer.get(3) {
            None => Step::Wait,
            Some(&f @ b'A'..=b'E') => key(KeyCode::F(1 + f - b'A')),
            Some(_) => Step::Skip,
        },
        b'D' => key(KeyCode::Left),
        b'C' => key(KeyCode::Right),
        b'A' => key(KeyCode::Up),
        b'B' => key(KeyCode::Down),
        b'H' => key(KeyCode::Home),
        b'F' => key(KeyCode::End),
        b'Z' => Step::Emit(Event::Key(KeyEvent::new(
            KeyCode::BackTab,
            KeyModifiers::SHIFT,
        ))),
        b'M' => normal_mouse(buffer),
        b'<' => sgr_mouse(buffer),
        b'I' | b'O' => Step::Skip,
        b';' => modified(buffer),
        b'P' => key(KeyCode::F(1)),
        b'Q' => key(KeyCode::F(2)),
        b'S' => key(KeyCode::F(4)),
        b'?' => match last {
            b'u' if buffer.len() < 5 => Step::Wait,
            0x40..=0x7e => Step::Skip,
            _ => Step::Wait,
        },
        b'0'..=b'9' if buffer.len() == 3 || !(64..=126).contains(&last) => Step::Wait,
        b'0'..=b'9' if buffer.starts_with(PASTE_START) => paste(buffer),
        b'0'..=b'9' => match last {
            b'M' => rxvt_mouse(buffer),
            b'~' => special(buffer),
            b'u' => csi_u(buffer),
            b'R' => Step::Skip,
            _ => modified(buffer),
        },
        _ => Step::Skip,
    }
}

fn modified(buffer: &[u8]) -> Step {
    let Some(mut fields) = fields(buffer, 2) else {
        return Step::Skip;
    };
    fields.next();
    let (modifiers, kind) = match modifier_and_kind(&mut fields) {
        Some((mask, kind)) => (held(mask), kind),
        None if buffer.len() > 3 => match buffer[buffer.len() - 2] {
            digit @ b'0'..=b'9' => (held(digit - b'0'), KeyEventKind::Press),
            _ => return Step::Skip,
        },
        None => (KeyModifiers::NONE, KeyEventKind::Press),
    };
    let code = match buffer[buffer.len() - 1] {
        b'A' => KeyCode::Up,
        b'B' => KeyCode::Down,
        b'C' => KeyCode::Right,
        b'D' => KeyCode::Left,
        b'F' => KeyCode::End,
        b'H' => KeyCode::Home,
        b'P' => KeyCode::F(1),
        b'Q' => KeyCode::F(2),
        b'R' => KeyCode::F(3),
        b'S' => KeyCode::F(4),
        _ => return Step::Skip,
    };
    Step::Emit(Event::Key(KeyEvent::new_with_kind(code, modifiers, kind)))
}

fn special(buffer: &[u8]) -> Step {
    let Some(mut fields) = fields(buffer, 2) else {
        return Step::Skip;
    };
    let Some(first) = fields.next().and_then(|field| field.parse::<u8>().ok()) else {
        return Step::Skip;
    };
    let (mask, kind) = modifier_and_kind(&mut fields).unwrap_or((1, KeyEventKind::Press));
    let code = match first {
        27 => {
            let Some(code) = fields
                .next()
                .and_then(|field| field.parse::<u32>().ok())
                .and_then(|codepoint| extended(codepoint, held(mask)))
            else {
                return Step::Skip;
            };
            code
        }
        1 | 7 => KeyCode::Home,
        2 => KeyCode::Insert,
        3 => KeyCode::Delete,
        4 | 8 => KeyCode::End,
        5 => KeyCode::PageUp,
        6 => KeyCode::PageDown,
        v @ 11..=15 => KeyCode::F(v - 10),
        v @ 17..=21 => KeyCode::F(v - 11),
        v @ 23..=26 => KeyCode::F(v - 12),
        v @ 28..=29 => KeyCode::F(v - 13),
        v @ 31..=34 => KeyCode::F(v - 14),
        _ => return Step::Skip,
    };
    Step::Emit(Event::Key(KeyEvent::new_with_kind_and_state(
        code,
        held(mask),
        kind,
        state(mask),
    )))
}

fn csi_u(buffer: &[u8]) -> Step {
    let Some(mut fields) = fields(buffer, 2) else {
        return Step::Skip;
    };
    let mut codepoints = fields.next().unwrap_or_default().split(':');
    let Some(codepoint) = codepoints
        .next()
        .and_then(|field| field.parse::<u32>().ok())
    else {
        return Step::Skip;
    };
    let (mask, kind) = modifier_and_kind(&mut fields).unwrap_or((1, KeyEventKind::Press));
    let mut modifiers = held(mask);
    let Some(mut code) = extended(codepoint, modifiers) else {
        return Step::Skip;
    };
    if modifiers.contains(KeyModifiers::SHIFT)
        && let Some(shifted) = codepoints
            .next()
            .and_then(|field| field.parse::<u32>().ok())
            .and_then(char::from_u32)
    {
        code = KeyCode::Char(shifted);
        modifiers.remove(KeyModifiers::SHIFT);
    }
    Step::Emit(Event::Key(KeyEvent::new_with_kind_and_state(
        code,
        modifiers,
        kind,
        state(mask),
    )))
}

fn extended(codepoint: u32, modifiers: KeyModifiers) -> Option<KeyCode> {
    Some(match char::from_u32(codepoint)? {
        '\x1b' => KeyCode::Esc,
        '\r' => KeyCode::Enter,
        '\t' if modifiers.contains(KeyModifiers::SHIFT) => KeyCode::BackTab,
        '\t' => KeyCode::Tab,
        '\x7f' => KeyCode::Backspace,
        c => KeyCode::Char(c),
    })
}

fn fields(buffer: &[u8], from: usize) -> Option<std::str::Split<'_, char>> {
    std::str::from_utf8(&buffer[from..buffer.len() - 1])
        .ok()
        .map(|text| text.split(';'))
}

fn modifier_and_kind<'a>(fields: &mut impl Iterator<Item = &'a str>) -> Option<(u8, KeyEventKind)> {
    let mut parts = fields.next()?.split(':');
    let mask = parts.next()?.parse::<u8>().ok()?;
    let kind = match parts.next().and_then(|part| part.parse::<u8>().ok()) {
        Some(2) => KeyEventKind::Repeat,
        Some(3) => KeyEventKind::Release,
        _ => KeyEventKind::Press,
    };
    Some((mask, kind))
}

fn held(mask: u8) -> KeyModifiers {
    let bits = mask.saturating_sub(1);
    [
        (1, KeyModifiers::SHIFT),
        (2, KeyModifiers::ALT),
        (4, KeyModifiers::CONTROL),
        (8, KeyModifiers::SUPER),
        (16, KeyModifiers::HYPER),
        (32, KeyModifiers::META),
    ]
    .into_iter()
    .filter(|(bit, _)| bits & bit != 0)
    .fold(KeyModifiers::NONE, |all, (_, modifier)| all | modifier)
}

fn state(mask: u8) -> KeyEventState {
    let bits = mask.saturating_sub(1);
    let mut state = KeyEventState::NONE;
    if bits & 64 != 0 {
        state |= KeyEventState::CAPS_LOCK;
    }
    if bits & 128 != 0 {
        state |= KeyEventState::NUM_LOCK;
    }
    state
}

fn paste(buffer: &[u8]) -> Step {
    if buffer.ends_with(b"\x1b[201~") {
        let text = String::from_utf8_lossy(&buffer[6..buffer.len() - 6]).into_owned();
        Step::Emit(Event::Paste(text))
    } else {
        Step::Wait
    }
}

fn rxvt_mouse(buffer: &[u8]) -> Step {
    let Some(mut fields) = fields(buffer, 2) else {
        return Step::Skip;
    };
    let mut number = || fields.next().and_then(|field| field.parse::<u16>().ok());
    let (Some(cb), Some(column), Some(row)) = (number(), number(), number()) else {
        return Step::Skip;
    };
    match u8::try_from(cb).ok().and_then(|cb| cb.checked_sub(32)) {
        Some(cb) => mouse(cb, column, row),
        None => Step::Skip,
    }
}

fn normal_mouse(buffer: &[u8]) -> Step {
    match buffer {
        [_, _, _, cb, column, row, ..] => match cb.checked_sub(32) {
            Some(cb) => mouse(
                cb,
                u16::from(column.saturating_sub(32)),
                u16::from(row.saturating_sub(32)),
            ),
            None => Step::Skip,
        },
        _ => Step::Wait,
    }
}

fn sgr_mouse(buffer: &[u8]) -> Step {
    let released = match buffer[buffer.len() - 1] {
        b'm' => true,
        b'M' => false,
        _ => return Step::Wait,
    };
    let Some(mut fields) = fields(buffer, 3) else {
        return Step::Skip;
    };
    let mut number = || fields.next().and_then(|field| field.parse::<u16>().ok());
    let (Some(cb), Some(column), Some(row)) = (number(), number(), number()) else {
        return Step::Skip;
    };
    let Ok(cb) = u8::try_from(cb) else {
        return Step::Skip;
    };
    match mouse(cb, column, row) {
        Step::Emit(Event::Mouse(mut event)) if released => {
            if let MouseEventKind::Down(button) = event.kind {
                event.kind = MouseEventKind::Up(button);
            }
            Step::Emit(Event::Mouse(event))
        }
        step => step,
    }
}

fn mouse(cb: u8, column: u16, row: u16) -> Step {
    let (Some(column), Some(row)) = (column.checked_sub(1), row.checked_sub(1)) else {
        return Step::Skip;
    };
    let button = (cb & 0b0000_0011) | ((cb & 0b1100_0000) >> 4);
    let dragging = cb & 0b0010_0000 != 0;
    let kind = match (button, dragging) {
        (0, false) => MouseEventKind::Down(MouseButton::Left),
        (1, false) => MouseEventKind::Down(MouseButton::Middle),
        (2, false) => MouseEventKind::Down(MouseButton::Right),
        (0, true) => MouseEventKind::Drag(MouseButton::Left),
        (1, true) => MouseEventKind::Drag(MouseButton::Middle),
        (2, true) => MouseEventKind::Drag(MouseButton::Right),
        (3, false) => MouseEventKind::Up(MouseButton::Left),
        (3..=5, true) => MouseEventKind::Moved,
        (4, false) => MouseEventKind::ScrollUp,
        (5, false) => MouseEventKind::ScrollDown,
        (6, false) => MouseEventKind::ScrollLeft,
        (7, false) => MouseEventKind::ScrollRight,
        _ => return Step::Skip,
    };
    let mut modifiers = KeyModifiers::NONE;
    if cb & 0b0000_0100 != 0 {
        modifiers |= KeyModifiers::SHIFT;
    }
    if cb & 0b0000_1000 != 0 {
        modifiers |= KeyModifiers::ALT;
    }
    if cb & 0b0001_0000 != 0 {
        modifiers |= KeyModifiers::CONTROL;
    }
    Step::Emit(Event::Mouse(MouseEvent {
        kind,
        column,
        row,
        modifiers,
    }))
}

fn utf8(buffer: &[u8]) -> Step {
    match std::str::from_utf8(buffer) {
        Ok(text) => match text.chars().next() {
            Some(c) if c.is_uppercase() => Step::Emit(Event::Key(KeyEvent::new(
                KeyCode::Char(c),
                KeyModifiers::SHIFT,
            ))),
            Some(c) => key(KeyCode::Char(c)),
            None => Step::Skip,
        },
        Err(_) => {
            let needed = match buffer[0] {
                0x00..=0x7f => 1,
                0xc0..=0xdf => 2,
                0xe0..=0xef => 3,
                0xf0..=0xf7 => 4,
                _ => return Step::Skip,
            };
            if buffer[1..]
                .iter()
                .any(|byte| byte & 0b1100_0000 != 0b1000_0000)
            {
                Step::Skip
            } else if buffer.len() < needed {
                Step::Wait
            } else {
                Step::Skip
            }
        }
    }
}

fn key(code: KeyCode) -> Step {
    Step::Emit(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)))
}

fn ctrl(c: char) -> Step {
    Step::Emit(Event::Key(KeyEvent::new(
        KeyCode::Char(c),
        KeyModifiers::CONTROL,
    )))
}
