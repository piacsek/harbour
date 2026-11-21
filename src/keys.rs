use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Keyboard {
    #[default]
    Legacy,
    Xterm,
    Kitty,
}

pub fn encode_as(key: KeyEvent, application_cursor: bool, keyboard: Keyboard) -> Vec<u8> {
    let modifier = modifier_parameter(key.modifiers);
    match (keyboard, key.code, unexpressed(key)) {
        (Keyboard::Legacy, ..) => encode(key, application_cursor),
        (Keyboard::Xterm, _, Some((code, modifier))) => {
            format!("\x1b[27;{modifier};{code}~").into_bytes()
        }
        (Keyboard::Kitty, _, Some((code, modifier))) => {
            format!("\x1b[{code};{modifier}u").into_bytes()
        }
        (Keyboard::Xterm, KeyCode::F(number @ 13..=24), None)
            if !key.modifiers.contains(KeyModifiers::SHIFT) =>
        {
            encode(
                KeyEvent::new(KeyCode::F(number - 12), key.modifiers | KeyModifiers::SHIFT),
                application_cursor,
            )
        }
        (Keyboard::Xterm, KeyCode::F(number @ 25..=35), None) if modifier == 1 => encode(
            KeyEvent::new(KeyCode::F(number - 24), KeyModifiers::CONTROL),
            application_cursor,
        ),
        (Keyboard::Kitty, KeyCode::F(number @ 13..=35), None) => {
            let code = 57363 + u32::from(number);
            if modifier > 1 {
                format!("\x1b[{code};{modifier}u").into_bytes()
            } else {
                format!("\x1b[{code}u").into_bytes()
            }
        }
        _ => encode(key, application_cursor),
    }
}

fn unexpressed(key: KeyEvent) -> Option<(u32, u8)> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let modifier = modifier_parameter(key.modifiers);
    match key.code {
        KeyCode::Esc if ctrl || shift => Some((27, modifier)),
        KeyCode::Enter if ctrl || shift => Some((13, modifier)),
        KeyCode::Tab if ctrl || shift => Some((9, modifier)),
        KeyCode::BackTab if ctrl => {
            Some((9, modifier_parameter(key.modifiers | KeyModifiers::SHIFT)))
        }
        KeyCode::Backspace if shift => Some((127, modifier)),
        KeyCode::Char(c) if ctrl && c.is_ascii_uppercase() => Some((
            u32::from(c.to_ascii_lowercase()),
            modifier_parameter(key.modifiers | KeyModifiers::SHIFT),
        )),
        KeyCode::Char(c) if ctrl && shift && c.is_ascii_lowercase() => {
            Some((u32::from(c), modifier))
        }
        KeyCode::Char(c @ ('i' | 'm' | '[')) if ctrl => Some((u32::from(c), modifier)),
        KeyCode::Char(c) if ctrl && !has_control_byte(c) => Some((u32::from(c), modifier)),
        _ => None,
    }
}

fn has_control_byte(c: char) -> bool {
    c.is_ascii_alphabetic() || matches!(c, ' ' | '@' | '[' | '\\' | ']' | '^' | '_')
}

pub fn encode(key: KeyEvent, application_cursor: bool) -> Vec<u8> {
    let modifier = modifier_parameter(key.modifiers);
    let cursor = |letter: char| {
        if modifier > 1 {
            format!("\x1b[1;{modifier}{letter}").into_bytes()
        } else if application_cursor {
            format!("\x1bO{letter}").into_bytes()
        } else {
            format!("\x1b[{letter}").into_bytes()
        }
    };
    let tilde = |number: u8| {
        if modifier > 1 {
            format!("\x1b[{number};{modifier}~").into_bytes()
        } else {
            format!("\x1b[{number}~").into_bytes()
        }
    };
    match key.code {
        KeyCode::Up => cursor('A'),
        KeyCode::Down => cursor('B'),
        KeyCode::Right => cursor('C'),
        KeyCode::Left => cursor('D'),
        KeyCode::Home => cursor('H'),
        KeyCode::End => cursor('F'),
        KeyCode::Insert => tilde(2),
        KeyCode::Delete => tilde(3),
        KeyCode::PageUp => tilde(5),
        KeyCode::PageDown => tilde(6),
        KeyCode::F(number @ 1..=4) if modifier > 1 => {
            format!("\x1b[1;{modifier}{}", char::from(b'O' + number)).into_bytes()
        }
        KeyCode::F(number @ 1..=4) => format!("\x1bO{}", char::from(b'O' + number)).into_bytes(),
        KeyCode::F(number @ 5..=12) => {
            tilde([15, 17, 18, 19, 20, 21, 23, 24][usize::from(number - 5)])
        }
        KeyCode::BackTab => b"\x1b[Z".to_vec(),
        _ => plain(key),
    }
}

fn plain(key: KeyEvent) -> Vec<u8> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let mut escaped = key.modifiers.contains(KeyModifiers::ALT);
    let bytes = match key.code {
        KeyCode::Char(c) if ctrl => {
            control(c).map_or_else(|| c.to_string().into_bytes(), |byte| vec![byte])
        }
        KeyCode::Char(c) => c.to_string().into_bytes(),
        KeyCode::Enter => {
            escaped |= shift || ctrl;
            b"\r".to_vec()
        }
        KeyCode::Esc => b"\x1b".to_vec(),
        KeyCode::Tab => b"\t".to_vec(),
        KeyCode::Backspace if ctrl => vec![0x08],
        KeyCode::Backspace => vec![0x7f],
        _ => Vec::new(),
    };
    if escaped && !bytes.is_empty() {
        [b"\x1b".as_slice(), &bytes].concat()
    } else {
        bytes
    }
}

fn control(c: char) -> Option<u8> {
    match c.to_ascii_lowercase() {
        letter @ 'a'..='z' => Some(letter as u8 & 0x1f),
        ' ' | '@' | '2' => Some(0),
        '[' | '3' => Some(0x1b),
        '\\' | '4' => Some(0x1c),
        ']' | '5' => Some(0x1d),
        '^' | '6' => Some(0x1e),
        '_' | '-' | '7' | '/' => Some(0x1f),
        '8' | '?' => Some(0x7f),
        _ => None,
    }
}

fn modifier_parameter(modifiers: KeyModifiers) -> u8 {
    1 + u8::from(modifiers.contains(KeyModifiers::SHIFT))
        + 2 * u8::from(modifiers.contains(KeyModifiers::ALT))
        + 4 * u8::from(modifiers.contains(KeyModifiers::CONTROL))
}

const KITTY_DEPTH: usize = 16;

#[derive(Clone, Debug, Default)]
pub struct Asked {
    level: u16,
    kitty: u16,
    pushed: Vec<u16>,
}

impl Asked {
    pub fn keyboard(&self) -> Keyboard {
        if self.kitty & 1 == 1 {
            Keyboard::Kitty
        } else if self.level > 0 {
            Keyboard::Xterm
        } else {
            Keyboard::Legacy
        }
    }

    pub fn csi(&mut self, first: Option<u8>, parameters: &[&[u16]], last: char) -> Option<Vec<u8>> {
        let value = |index: usize| {
            parameters
                .get(index)
                .and_then(|values| values.first())
                .copied()
        };
        match (first, last) {
            (Some(b'>'), 'm') if matches!(value(0), None | Some(4)) => {
                self.level = value(1).unwrap_or(0);
            }
            (Some(b'>'), 'n') if value(0) == Some(4) => self.level = 0,
            (Some(b'>'), 'u') => {
                if self.pushed.len() == KITTY_DEPTH {
                    self.pushed.remove(0);
                }
                self.pushed.push(self.kitty);
                self.kitty = value(0).unwrap_or(0) & 1;
            }
            (Some(b'<'), 'u') => {
                for _ in 0..value(0).unwrap_or(1).max(1) {
                    self.kitty = self.pushed.pop().unwrap_or(0);
                }
            }
            (Some(b'='), 'u') => {
                let flags = value(0).unwrap_or(0) & 1;
                self.kitty = match value(1).unwrap_or(1) {
                    2 => self.kitty | flags,
                    3 => self.kitty & !flags,
                    _ => flags,
                };
            }
            (Some(b'?'), 'u') => return Some(format!("\x1b[?{}u", self.kitty).into_bytes()),
            _ => {}
        }
        None
    }
}
