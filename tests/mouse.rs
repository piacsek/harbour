use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use harbour::mouse::{Gesture, route};
use ratatui::layout::Rect;
use vt100::{MouseProtocolEncoding, MouseProtocolMode};

const NONE: MouseProtocolMode = MouseProtocolMode::None;
const X10: MouseProtocolMode = MouseProtocolMode::Press;
const PRESS: MouseProtocolMode = MouseProtocolMode::PressRelease;
const BUTTON: MouseProtocolMode = MouseProtocolMode::ButtonMotion;
const ANY: MouseProtocolMode = MouseProtocolMode::AnyMotion;

const IDLE: Gesture = Gesture::Idle;
const IN_PANE: Gesture = Gesture::Pane;
const ELSEWHERE: Gesture = Gesture::Elsewhere;

const IN: (u16, u16) = (45, 3);
const OUT: (u16, u16) = (5, 5);

const PLAIN: KeyModifiers = KeyModifiers::NONE;
const SHIFT: KeyModifiers = KeyModifiers::SHIFT;
const ALT: KeyModifiers = KeyModifiers::ALT;
const CTRL: KeyModifiers = KeyModifiers::CONTROL;

const DOWN: MouseEventKind = MouseEventKind::Down(MouseButton::Left);
const RIGHT_DOWN: MouseEventKind = MouseEventKind::Down(MouseButton::Right);
const MIDDLE_DOWN: MouseEventKind = MouseEventKind::Down(MouseButton::Middle);
const UP: MouseEventKind = MouseEventKind::Up(MouseButton::Left);
const DRAG: MouseEventKind = MouseEventKind::Drag(MouseButton::Left);
const MOVE: MouseEventKind = MouseEventKind::Moved;
const WHEEL_UP: MouseEventKind = MouseEventKind::ScrollUp;
const WHEEL_DOWN: MouseEventKind = MouseEventKind::ScrollDown;

#[test]
fn each_event_reaches_the_session_as_its_mode_and_gesture_allow() {
    assert_eq!(sent(NONE, IDLE, DOWN, IN, PLAIN), None);
    assert_eq!(sent(NONE, IDLE, WHEEL_UP, IN, PLAIN), None);
    assert_eq!(sent(X10, IDLE, DOWN, IN, PLAIN), sgr("0;5;3M"));
    assert_eq!(sent(X10, IDLE, DOWN, IN, CTRL), sgr("0;5;3M"));
    assert_eq!(sent(X10, IN_PANE, UP, IN, PLAIN), None);
    assert_eq!(sent(X10, IN_PANE, DRAG, IN, PLAIN), None);
    assert_eq!(sent(X10, IDLE, WHEEL_UP, IN, PLAIN), sgr("64;5;3M"));
    assert_eq!(sent(PRESS, IDLE, DOWN, IN, PLAIN), sgr("0;5;3M"));
    assert_eq!(sent(PRESS, IDLE, RIGHT_DOWN, IN, PLAIN), sgr("2;5;3M"));
    assert_eq!(sent(PRESS, IDLE, MIDDLE_DOWN, IN, PLAIN), sgr("1;5;3M"));
    assert_eq!(sent(PRESS, IN_PANE, UP, IN, PLAIN), sgr("0;5;3m"));
    assert_eq!(sent(PRESS, IN_PANE, DRAG, IN, PLAIN), None);
    assert_eq!(sent(PRESS, IDLE, MOVE, IN, PLAIN), None);
    assert_eq!(sent(PRESS, IDLE, WHEEL_UP, IN, PLAIN), sgr("64;5;3M"));
    assert_eq!(sent(PRESS, IDLE, WHEEL_DOWN, IN, PLAIN), sgr("65;5;3M"));
    assert_eq!(sent(PRESS, IDLE, WHEEL_UP, OUT, PLAIN), None);
    assert_eq!(sent(BUTTON, IN_PANE, DRAG, IN, PLAIN), sgr("32;5;3M"));
    assert_eq!(sent(BUTTON, IDLE, MOVE, IN, PLAIN), None);
    assert_eq!(sent(ANY, IN_PANE, DRAG, IN, PLAIN), sgr("32;5;3M"));
    assert_eq!(sent(ANY, IDLE, MOVE, IN, PLAIN), sgr("35;5;3M"));
    assert_eq!(sent(ANY, IDLE, MOVE, OUT, PLAIN), None);
    assert_eq!(sent(BUTTON, IN_PANE, DRAG, OUT, PLAIN), sgr("32;1;5M"));
    assert_eq!(sent(BUTTON, IN_PANE, UP, OUT, PLAIN), sgr("0;1;5m"));
    assert_eq!(sent(BUTTON, IDLE, DOWN, OUT, PLAIN), None);
    assert_eq!(sent(BUTTON, ELSEWHERE, DRAG, IN, PLAIN), None);
    assert_eq!(sent(BUTTON, ELSEWHERE, UP, IN, PLAIN), None);
    assert_eq!(sent(BUTTON, IDLE, DRAG, IN, PLAIN), None);
    assert_eq!(sent(BUTTON, IDLE, UP, IN, PLAIN), None);
    assert_eq!(sent(PRESS, IDLE, DOWN, IN, SHIFT), sgr("4;5;3M"));
    assert_eq!(sent(PRESS, IDLE, DOWN, IN, ALT), sgr("8;5;3M"));
    assert_eq!(sent(PRESS, IDLE, DOWN, IN, CTRL), sgr("16;5;3M"));
    assert_eq!(sent(PRESS, IDLE, DOWN, IN, CTRL | SHIFT), sgr("20;5;3M"));
    assert_eq!(sent(PRESS, IN_PANE, UP, IN, CTRL), sgr("16;5;3m"));
    assert_eq!(sent(BUTTON, IN_PANE, DRAG, IN, ALT), sgr("40;5;3M"));
    assert_eq!(sent(PRESS, IDLE, WHEEL_UP, IN, CTRL), sgr("80;5;3M"));
}

#[test]
fn a_press_decides_where_the_gesture_belongs() {
    assert_eq!(next(IDLE, DOWN, IN), IN_PANE);
    assert_eq!(next(IDLE, DOWN, OUT), ELSEWHERE);
    assert_eq!(next(IN_PANE, DRAG, OUT), IN_PANE);
    assert_eq!(next(ELSEWHERE, DRAG, IN), ELSEWHERE);
    assert_eq!(next(IN_PANE, UP, OUT), IDLE);
    assert_eq!(next(ELSEWHERE, UP, IN), IDLE);
}

#[test]
fn an_empty_pane_forwards_nothing_and_ends_the_gesture() {
    let (gesture, bytes) = route(
        IN_PANE,
        event(DRAG, IN, PLAIN),
        Rect::default(),
        BUTTON,
        SGR,
    );
    assert_eq!(gesture, IDLE);
    assert_eq!(bytes, None);
}

#[test]
fn the_report_follows_the_encoding_the_session_asked_for() {
    assert_eq!(
        encoded(DOWN, IN, PLAIN, LEGACY),
        Some(b"\x1b[M %#".to_vec())
    );
    assert_eq!(encoded(UP, IN, PLAIN, LEGACY), Some(b"\x1b[M#%#".to_vec()));
    assert_eq!(encoded(UP, IN, CTRL, LEGACY), Some(b"\x1b[M3%#".to_vec()));
    assert_eq!(encoded(DOWN, IN, PLAIN, UTF8), Some(b"\x1b[M %#".to_vec()));
    assert_eq!(encoded(DOWN, FAR, PLAIN, LEGACY), None);
    assert_eq!(
        encoded(DOWN, FAR, PLAIN, UTF8),
        Some("\x1b[M \u{12d}#".as_bytes().to_vec())
    );
    assert_eq!(
        encoded(DOWN, FAR, PLAIN, SGR),
        Some(b"\x1b[<0;269;3M".to_vec())
    );
}

const SGR: MouseProtocolEncoding = MouseProtocolEncoding::Sgr;
const LEGACY: MouseProtocolEncoding = MouseProtocolEncoding::Default;
const UTF8: MouseProtocolEncoding = MouseProtocolEncoding::Utf8;
const FAR: (u16, u16) = (309, 3);

fn pane() -> Rect {
    Rect::new(41, 1, 300, 18)
}

fn event(kind: MouseEventKind, (column, row): (u16, u16), modifiers: KeyModifiers) -> MouseEvent {
    MouseEvent {
        kind,
        column,
        row,
        modifiers,
    }
}

fn sent(
    mode: MouseProtocolMode,
    gesture: Gesture,
    kind: MouseEventKind,
    at: (u16, u16),
    modifiers: KeyModifiers,
) -> Option<String> {
    route(gesture, event(kind, at, modifiers), pane(), mode, SGR)
        .1
        .map(|bytes| String::from_utf8(bytes).unwrap())
}

fn next(gesture: Gesture, kind: MouseEventKind, at: (u16, u16)) -> Gesture {
    route(gesture, event(kind, at, PLAIN), pane(), BUTTON, SGR).0
}

fn encoded(
    kind: MouseEventKind,
    at: (u16, u16),
    modifiers: KeyModifiers,
    encoding: MouseProtocolEncoding,
) -> Option<Vec<u8>> {
    route(
        IN_PANE,
        event(kind, at, modifiers),
        pane(),
        BUTTON,
        encoding,
    )
    .1
}

fn sgr(report: &str) -> Option<String> {
    Some(format!("\x1b[<{report}"))
}
