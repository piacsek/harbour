use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Position, Rect};
use vt100::{MouseProtocolEncoding, MouseProtocolMode};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Gesture {
    #[default]
    Idle,
    Pane(usize),
    Elsewhere,
}

#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub session: usize,
    pub mode: MouseProtocolMode,
    pub encoding: MouseProtocolEncoding,
}

pub fn route(
    gesture: Gesture,
    event: MouseEvent,
    pane: Rect,
    target: Option<Target>,
) -> (Gesture, Option<Vec<u8>>) {
    let Some(Target {
        session,
        mode,
        encoding,
    }) = target.filter(|_| !pane.is_empty())
    else {
        return (Gesture::Idle, None);
    };
    let gesture = match gesture {
        Gesture::Pane(owner) if owner != session => Gesture::Idle,
        gesture => gesture,
    };
    let inside = if pane.contains(Position::new(event.column, event.row)) {
        Gesture::Pane(session)
    } else {
        Gesture::Elsewhere
    };
    let (next, owner) = match event.kind {
        MouseEventKind::Down(_) => (inside, inside),
        MouseEventKind::Up(_) => (Gesture::Idle, gesture),
        MouseEventKind::Drag(_) => (gesture, gesture),
        _ => (gesture, inside),
    };
    if owner != Gesture::Pane(session) {
        return (next, None);
    }
    let column = event.column.clamp(pane.x, pane.right() - 1) - pane.x + 1;
    let row = event.row.clamp(pane.y, pane.bottom() - 1) - pane.y + 1;
    let bytes = report(event.kind, mode).and_then(|(code, release)| {
        let code = code + modifier_bits(event.modifiers, mode);
        encode(encoding, code, release, column, row)
    });
    (next, bytes)
}

fn report(kind: MouseEventKind, mode: MouseProtocolMode) -> Option<(u8, bool)> {
    let wanted = match kind {
        MouseEventKind::Down(_)
        | MouseEventKind::ScrollUp
        | MouseEventKind::ScrollDown
        | MouseEventKind::ScrollLeft
        | MouseEventKind::ScrollRight => mode != MouseProtocolMode::None,
        MouseEventKind::Up(_) => {
            !matches!(mode, MouseProtocolMode::None | MouseProtocolMode::Press)
        }
        MouseEventKind::Drag(_) => matches!(
            mode,
            MouseProtocolMode::ButtonMotion | MouseProtocolMode::AnyMotion
        ),
        MouseEventKind::Moved => mode == MouseProtocolMode::AnyMotion,
    };
    if !wanted {
        return None;
    }
    Some(match kind {
        MouseEventKind::Down(button) => (button_code(button), false),
        MouseEventKind::Up(button) => (button_code(button), true),
        MouseEventKind::Drag(button) => (button_code(button) + 32, false),
        MouseEventKind::Moved => (35, false),
        MouseEventKind::ScrollUp => (64, false),
        MouseEventKind::ScrollDown => (65, false),
        MouseEventKind::ScrollLeft => (66, false),
        MouseEventKind::ScrollRight => (67, false),
    })
}

fn button_code(button: MouseButton) -> u8 {
    match button {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
    }
}

fn modifier_bits(modifiers: KeyModifiers, mode: MouseProtocolMode) -> u8 {
    if mode == MouseProtocolMode::Press {
        return 0;
    }
    4 * u8::from(modifiers.contains(KeyModifiers::SHIFT))
        + 8 * u8::from(modifiers.contains(KeyModifiers::ALT))
        + 16 * u8::from(modifiers.contains(KeyModifiers::CONTROL))
}

fn encode(
    encoding: MouseProtocolEncoding,
    code: u8,
    release: bool,
    column: u16,
    row: u16,
) -> Option<Vec<u8>> {
    if encoding == MouseProtocolEncoding::Sgr {
        let last = if release { 'm' } else { 'M' };
        return Some(format!("\x1b[<{code};{column};{row}{last}").into_bytes());
    }
    let code = if release { 3 + (code & 0b1_1100) } else { code };
    let mut bytes = b"\x1b[M".to_vec();
    for value in [u32::from(code), u32::from(column), u32::from(row)] {
        let value = value + 32;
        match encoding {
            MouseProtocolEncoding::Utf8 => {
                let c = char::from_u32(value).filter(|_| value < 0x800)?;
                bytes.extend(c.to_string().as_bytes());
            }
            _ => bytes.push(u8::try_from(value).ok()?),
        }
    }
    Some(bytes)
}
