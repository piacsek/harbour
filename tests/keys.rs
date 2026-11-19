use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind};
use harbour::keys::{encode, mouse};

#[test]
fn a_letter_is_sent_as_itself() {
    assert_eq!(encode(key(KeyCode::Char('a')), false), b"a");
}

#[test]
fn ctrl_and_a_letter_is_sent_as_its_control_byte() {
    assert_eq!(
        encode(with(KeyCode::Char('a'), KeyModifiers::CONTROL), false),
        [0x01]
    );
}

#[test]
fn alt_and_a_letter_is_sent_after_an_escape() {
    assert_eq!(
        encode(with(KeyCode::Char('b'), KeyModifiers::ALT), false),
        b"\x1bb"
    );
}

#[test]
fn enter_is_sent_as_a_carriage_return() {
    assert_eq!(encode(key(KeyCode::Enter), false), b"\r");
}

#[test]
fn shift_and_enter_is_sent_as_an_escaped_carriage_return() {
    assert_eq!(
        encode(with(KeyCode::Enter, KeyModifiers::SHIFT), false),
        b"\x1b\r"
    );
}

#[test]
fn shift_and_tab_is_sent_as_a_back_tab() {
    assert_eq!(
        encode(with(KeyCode::BackTab, KeyModifiers::SHIFT), false),
        b"\x1b[Z"
    );
}

#[test]
fn backspace_is_sent_as_delete() {
    assert_eq!(encode(key(KeyCode::Backspace), false), [0x7f]);
}

#[test]
fn ctrl_and_backspace_is_sent_as_a_backspace() {
    assert_eq!(
        encode(with(KeyCode::Backspace, KeyModifiers::CONTROL), false),
        [0x08]
    );
}

#[test]
fn escape_is_sent_as_itself() {
    assert_eq!(encode(key(KeyCode::Esc), false), b"\x1b");
}

#[test]
fn tab_is_sent_as_itself() {
    assert_eq!(encode(key(KeyCode::Tab), false), b"\t");
}

#[test]
fn an_arrow_is_sent_as_a_cursor_sequence() {
    assert_eq!(encode(key(KeyCode::Up), false), b"\x1b[A");
}

#[test]
fn an_arrow_in_application_cursor_mode_is_sent_as_an_ss3_sequence() {
    assert_eq!(encode(key(KeyCode::Left), true), b"\x1bOD");
}

#[test]
fn a_modified_arrow_carries_its_modifiers_in_the_sequence() {
    assert_eq!(
        encode(with(KeyCode::Right, KeyModifiers::ALT), true),
        b"\x1b[1;3C"
    );
}

#[test]
fn delete_is_sent_as_a_tilde_sequence() {
    assert_eq!(encode(key(KeyCode::Delete), false), b"\x1b[3~");
}

#[test]
fn a_modified_page_key_carries_its_modifiers_in_the_sequence() {
    assert_eq!(
        encode(with(KeyCode::PageUp, KeyModifiers::CONTROL), false),
        b"\x1b[5;5~"
    );
}

#[test]
fn f1_is_sent_as_an_ss3_sequence() {
    assert_eq!(encode(key(KeyCode::F(1)), false), b"\x1bOP");
}

#[test]
fn f5_is_sent_as_a_tilde_sequence() {
    assert_eq!(encode(key(KeyCode::F(5)), false), b"\x1b[15~");
}

#[test]
fn f12_is_sent_as_a_tilde_sequence() {
    assert_eq!(encode(key(KeyCode::F(12)), false), b"\x1b[24~");
}

#[test]
fn alt_and_a_key_without_a_sequence_sends_nothing() {
    assert_eq!(encode(with(KeyCode::F(20), KeyModifiers::ALT), false), b"");
}

#[test]
fn ctrl_and_a_bracket_is_sent_as_an_escape() {
    assert_eq!(
        encode(with(KeyCode::Char('['), KeyModifiers::CONTROL), false),
        [0x1b]
    );
}

#[test]
fn a_release_ends_with_a_small_m() {
    assert_eq!(
        mouse(MouseEventKind::Up(MouseButton::Left), 4, 2),
        Some(b"\x1b[<0;5;3m".to_vec())
    );
}

#[test]
fn the_right_button_is_button_two() {
    assert_eq!(
        mouse(MouseEventKind::Down(MouseButton::Right), 0, 0),
        Some(b"\x1b[<2;1;1M".to_vec())
    );
}

#[test]
fn the_middle_button_is_button_one() {
    assert_eq!(
        mouse(MouseEventKind::Down(MouseButton::Middle), 0, 0),
        Some(b"\x1b[<1;1;1M".to_vec())
    );
}

#[test]
fn a_drag_adds_thirty_two() {
    assert_eq!(
        mouse(MouseEventKind::Drag(MouseButton::Left), 0, 0),
        Some(b"\x1b[<32;1;1M".to_vec())
    );
}

#[test]
fn the_wheel_up_is_sixty_four() {
    assert_eq!(
        mouse(MouseEventKind::ScrollUp, 0, 0),
        Some(b"\x1b[<64;1;1M".to_vec())
    );
}

#[test]
fn the_wheel_down_is_sixty_five() {
    assert_eq!(
        mouse(MouseEventKind::ScrollDown, 0, 0),
        Some(b"\x1b[<65;1;1M".to_vec())
    );
}

#[test]
fn a_move_without_a_button_is_not_sent() {
    assert_eq!(mouse(MouseEventKind::Moved, 0, 0), None);
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn with(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}
