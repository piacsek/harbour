use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
            escaped |= shift;
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
