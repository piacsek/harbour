use std::io::Write;
use std::os::fd::AsFd;
use std::time::Duration;

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use harbour::input::Input;

#[test]
fn legacy_keys_read_as_before() {
    assert_eq!(parsed(b"a"), [key(KeyCode::Char('a'))]);
    assert_eq!(
        parsed(b"A"),
        [with(KeyCode::Char('A'), KeyModifiers::SHIFT)]
    );
    assert_eq!(parsed("é".as_bytes()), [key(KeyCode::Char('é'))]);
    assert_eq!(parsed(b"\r"), [key(KeyCode::Enter)]);
    assert_eq!(parsed(b"\t"), [key(KeyCode::Tab)]);
    assert_eq!(parsed(b"\x7f"), [key(KeyCode::Backspace)]);
    assert_eq!(parsed(b"\x1b"), [key(KeyCode::Esc)]);
    assert_eq!(parsed(b"\x1b\x1b"), [key(KeyCode::Esc)]);
    assert_eq!(
        parsed(b"\x1b[Z"),
        [with(KeyCode::BackTab, KeyModifiers::SHIFT)]
    );
    assert_eq!(parsed(b"\x1b[A"), [key(KeyCode::Up)]);
    assert_eq!(parsed(b"\x1bOP"), [key(KeyCode::F(1))]);
    assert_eq!(parsed(b"\x1b[3~"), [key(KeyCode::Delete)]);
    assert_eq!(parsed(b"\x1b[15~"), [key(KeyCode::F(5))]);
}

#[test]
fn function_keys_thirteen_to_twenty_read_as_their_numbers() {
    assert_eq!(parsed(b"\x1b[25~"), [key(KeyCode::F(13))]);
    assert_eq!(parsed(b"\x1b[26~"), [key(KeyCode::F(14))]);
    assert_eq!(parsed(b"\x1b[28~"), [key(KeyCode::F(15))]);
    assert_eq!(parsed(b"\x1b[29~"), [key(KeyCode::F(16))]);
    assert_eq!(parsed(b"\x1b[31~"), [key(KeyCode::F(17))]);
    assert_eq!(parsed(b"\x1b[32~"), [key(KeyCode::F(18))]);
    assert_eq!(parsed(b"\x1b[33~"), [key(KeyCode::F(19))]);
    assert_eq!(parsed(b"\x1b[34~"), [key(KeyCode::F(20))]);
}

#[test]
fn legacy_control_keys_read_as_before() {
    assert_eq!(parsed(b"\x01"), [ctrl('a')]);
    assert_eq!(parsed(b"\x08"), [ctrl('h')]);
    assert_eq!(parsed(b"\x0c"), [ctrl('l')]);
    assert_eq!(parsed(b"\n"), [ctrl('j')]);
    assert_eq!(parsed(b"\x1c"), [ctrl('\\')]);
    assert_eq!(parsed(b"\x1d"), [ctrl(']')]);
    assert_eq!(parsed(b"\x1e"), [ctrl('^')]);
    assert_eq!(parsed(b"\x1f"), [ctrl('_')]);
    assert_eq!(parsed(b"\0"), [ctrl(' ')]);
}

#[test]
fn legacy_modified_keys_read_as_before() {
    assert_eq!(
        parsed(b"\x1ba"),
        [with(KeyCode::Char('a'), KeyModifiers::ALT)]
    );
    assert_eq!(parsed(b"\x1b\r"), [with(KeyCode::Enter, KeyModifiers::ALT)]);
    assert_eq!(parsed(b"\x1b\x01"), [with(KeyCode::Char('a'), ALT_CTRL)]);
    assert_eq!(
        parsed(b"\x1b[1;5A"),
        [with(KeyCode::Up, KeyModifiers::CONTROL)]
    );
    assert_eq!(
        parsed(b"\x1b[1;2C"),
        [with(KeyCode::Right, KeyModifiers::SHIFT)]
    );
    assert_eq!(
        parsed(b"\x1b[5;5~"),
        [with(KeyCode::PageUp, KeyModifiers::CONTROL)]
    );
}

#[test]
fn alt_z_read_after_an_escape_is_marked_and_the_extended_forms_are_not() {
    assert_eq!(
        parsed(b"\x1bz"),
        [with(KeyCode::Char('z'), KeyModifiers::ALT)]
    );
    assert_eq!(
        parsed(b"\x1b[27;3;122~"),
        [with(KeyCode::Char('z'), KeyModifiers::ALT)]
    );
    assert_eq!(
        parsed(b"\x1b[122;3u"),
        [with(KeyCode::Char('z'), KeyModifiers::ALT)]
    );
}

#[test]
fn alt_z_with_caps_lock_on_reads_as_alt_with_a_capital_z() {
    assert_eq!(
        parsed(b"\x1b[27;3;90~"),
        [with(KeyCode::Char('Z'), KeyModifiers::ALT)]
    );
}

#[test]
fn the_xterm_form_reads_as_the_modified_key() {
    assert_eq!(
        parsed(b"\x1b[27;2;13~"),
        [with(KeyCode::Enter, KeyModifiers::SHIFT)]
    );
    assert_eq!(
        parsed(b"\x1b[27;5;13~"),
        [with(KeyCode::Enter, KeyModifiers::CONTROL)]
    );
    assert_eq!(parsed(b"\x1b[27;6;13~"), [with(KeyCode::Enter, CTRL_SHIFT)]);
    assert_eq!(
        parsed(b"\x1b[27;3;13~"),
        [with(KeyCode::Enter, KeyModifiers::ALT)]
    );
    assert_eq!(
        parsed(b"\x1b[27;5;9~"),
        [with(KeyCode::Tab, KeyModifiers::CONTROL)]
    );
    assert_eq!(
        parsed(b"\x1b[27;2;9~"),
        [with(KeyCode::BackTab, KeyModifiers::SHIFT)]
    );
    assert_eq!(
        parsed(b"\x1b[27;5;27~"),
        [with(KeyCode::Esc, KeyModifiers::CONTROL)]
    );
    assert_eq!(
        parsed(b"\x1b[27;5;127~"),
        [with(KeyCode::Backspace, KeyModifiers::CONTROL)]
    );
    assert_eq!(
        parsed(b"\x1b[27;5;49~"),
        [with(KeyCode::Char('1'), KeyModifiers::CONTROL)]
    );
    assert_eq!(
        parsed(b"\x1b[27;5;104~"),
        [with(KeyCode::Char('h'), KeyModifiers::CONTROL)]
    );
    assert_eq!(
        parsed(b"\x1b[27;3;97~"),
        [with(KeyCode::Char('a'), KeyModifiers::ALT)]
    );
}

#[test]
fn the_csi_u_form_reads_as_the_modified_key() {
    assert_eq!(
        parsed(b"\x1b[13;2u"),
        [with(KeyCode::Enter, KeyModifiers::SHIFT)]
    );
    assert_eq!(
        parsed(b"\x1b[13;5u"),
        [with(KeyCode::Enter, KeyModifiers::CONTROL)]
    );
    assert_eq!(parsed(b"\x1b[13;6u"), [with(KeyCode::Enter, CTRL_SHIFT)]);
    assert_eq!(
        parsed(b"\x1b[9;5u"),
        [with(KeyCode::Tab, KeyModifiers::CONTROL)]
    );
    assert_eq!(
        parsed(b"\x1b[9;2u"),
        [with(KeyCode::BackTab, KeyModifiers::SHIFT)]
    );
    assert_eq!(
        parsed(b"\x1b[27;5u"),
        [with(KeyCode::Esc, KeyModifiers::CONTROL)]
    );
    assert_eq!(
        parsed(b"\x1b[127;5u"),
        [with(KeyCode::Backspace, KeyModifiers::CONTROL)]
    );
    assert_eq!(
        parsed(b"\x1b[49;5u"),
        [with(KeyCode::Char('1'), KeyModifiers::CONTROL)]
    );
    assert_eq!(
        parsed(b"\x1b[104;5u"),
        [with(KeyCode::Char('h'), KeyModifiers::CONTROL)]
    );
}

#[test]
fn an_unknown_sequence_is_dropped_and_the_next_key_still_reads() {
    assert_eq!(parsed(b"\x1b[99~a"), [key(KeyCode::Char('a'))]);
    assert_eq!(parsed(b"\x1b[27;5;1114112~a"), [key(KeyCode::Char('a'))]);
}

#[test]
fn several_keys_in_one_read_each_read() {
    assert_eq!(
        parsed(b"j\x1b[27;2;13~k"),
        [
            key(KeyCode::Char('j')),
            with(KeyCode::Enter, KeyModifiers::SHIFT),
            key(KeyCode::Char('k')),
        ]
    );
}

#[test]
fn a_sequence_split_across_reads_reads_once_complete() {
    let mut input = Input::default();

    let first = input.feed(b"\x1b[27;2", false);
    let second = input.feed(b";13~", false);

    assert_eq!(first, []);
    assert_eq!(second, [with(KeyCode::Enter, KeyModifiers::SHIFT)]);
}

#[test]
fn an_escape_at_the_end_of_a_full_read_waits_for_the_rest() {
    let mut input = Input::default();

    let first = input.feed(b"\x1b", true);
    let second = input.feed(b"[A", false);

    assert_eq!(first, []);
    assert_eq!(second, [key(KeyCode::Up)]);
}

#[test]
fn a_bracketed_paste_reads_as_one_paste() {
    assert_eq!(
        parsed(b"\x1b[200~acme\r\x1b[27;2;13~\x1b[201~"),
        [Event::Paste("acme\r\x1b[27;2;13~".into())]
    );
}

#[test]
fn a_large_paste_in_full_pieces_reads_as_one_paste_from_one_read() {
    let text = "acme ".repeat(60 * 1024);

    let events = read_in_pieces(&pasted(&text), 1024);

    assert_eq!(events, [Event::Paste(text)]);
}

#[test]
fn a_large_paste_in_short_pieces_reads_as_one_paste_from_one_read() {
    let text = "acme ".repeat(60 * 1024);

    let events = read_in_pieces(&pasted(&text), 500);

    assert_eq!(events, [Event::Paste(text)]);
}

#[test]
fn a_sequence_that_never_ends_does_not_swallow_the_keys_after_it() {
    assert_eq!(parsed(b"\x1b[?2004;1$yx"), [key(KeyCode::Char('x'))]);
    assert_eq!(parsed(b"\x1b[1;\x1b[A"), [key(KeyCode::Up)]);
    let overlong = [b"\x1b[".as_slice(), &[b'1'; 62], b"x"].concat();
    assert_eq!(parsed(&overlong), [key(KeyCode::Char('x'))]);
}

#[test]
fn mouse_reports_read_as_before() {
    assert_eq!(
        parsed(b"\x1b[<0;5;3M"),
        [mouse(
            MouseEventKind::Down(MouseButton::Left),
            4,
            2,
            KeyModifiers::NONE
        )]
    );
    assert_eq!(
        parsed(b"\x1b[<0;5;3m"),
        [mouse(
            MouseEventKind::Up(MouseButton::Left),
            4,
            2,
            KeyModifiers::NONE
        )]
    );
    assert_eq!(
        parsed(b"\x1b[<16;1;1M"),
        [mouse(
            MouseEventKind::Down(MouseButton::Left),
            0,
            0,
            KeyModifiers::CONTROL
        )]
    );
    assert_eq!(
        parsed(b"\x1b[<32;7;2M"),
        [mouse(
            MouseEventKind::Drag(MouseButton::Left),
            6,
            1,
            KeyModifiers::NONE
        )]
    );
    assert_eq!(
        parsed(b"\x1b[<65;2;2M"),
        [mouse(MouseEventKind::ScrollDown, 1, 1, KeyModifiers::NONE)]
    );
    assert_eq!(
        parsed(b"\x1b[M #$"),
        [mouse(
            MouseEventKind::Down(MouseButton::Left),
            2,
            3,
            KeyModifiers::NONE
        )]
    );
    assert_eq!(
        parsed(b"\x1b[32;3;4M"),
        [mouse(
            MouseEventKind::Down(MouseButton::Left),
            2,
            3,
            KeyModifiers::NONE
        )]
    );
}

#[test]
fn replies_from_the_terminal_are_not_keys() {
    assert_eq!(parsed(b"\x1b[12;40Rq"), [key(KeyCode::Char('q'))]);
    assert_eq!(parsed(b"\x1b[Iq"), [key(KeyCode::Char('q'))]);
    assert_eq!(parsed(b"\x1b[?1uq"), [key(KeyCode::Char('q'))]);
}

#[test]
fn a_read_returns_what_arrived_on_the_descriptor() {
    let (reader, mut writer) = std::io::pipe().unwrap();
    writer.write_all(b"\x1b[13;5u").unwrap();
    let mut input = Input::default();

    let events = input.read(reader.as_fd(), Duration::from_secs(5)).unwrap();

    assert_eq!(events, [with(KeyCode::Enter, KeyModifiers::CONTROL)]);
}

#[test]
fn a_read_with_nothing_arriving_returns_nothing_after_the_timeout() {
    let (reader, _writer) = std::io::pipe().unwrap();
    let mut input = Input::default();

    let events = input
        .read(reader.as_fd(), Duration::from_millis(10))
        .unwrap();

    assert_eq!(events, []);
}

#[test]
fn a_read_after_the_terminal_hung_up_ends_the_input() {
    let (reader, writer) = std::io::pipe().unwrap();
    drop(writer);
    let mut input = Input::default();

    let error = input
        .read(reader.as_fd(), Duration::from_secs(5))
        .unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
}

#[test]
fn keys_that_arrived_before_the_hangup_are_read_first() {
    let (reader, mut writer) = std::io::pipe().unwrap();
    writer.write_all(b"q").unwrap();
    drop(writer);
    let mut input = Input::default();

    let events = input.read(reader.as_fd(), Duration::from_secs(5)).unwrap();

    assert_eq!(events, [key(KeyCode::Char('q'))]);
}

const CTRL_SHIFT: KeyModifiers = KeyModifiers::CONTROL.union(KeyModifiers::SHIFT);
const ALT_CTRL: KeyModifiers = KeyModifiers::ALT.union(KeyModifiers::CONTROL);

fn parsed(bytes: &[u8]) -> Vec<Event> {
    Input::default().feed(bytes, false)
}

fn key(code: KeyCode) -> Event {
    with(code, KeyModifiers::NONE)
}

fn ctrl(c: char) -> Event {
    with(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn with(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}

fn mouse(kind: MouseEventKind, column: u16, row: u16, modifiers: KeyModifiers) -> Event {
    Event::Mouse(MouseEvent {
        kind,
        column,
        row,
        modifiers,
    })
}

fn pasted(text: &str) -> Vec<u8> {
    [b"\x1b[200~".as_slice(), text.as_bytes(), b"\x1b[201~"].concat()
}

fn read_in_pieces(bytes: &[u8], size: usize) -> Vec<Event> {
    let mut pieces = bytes.chunks(size);
    Input::default()
        .read_with(
            |chunk, _| {
                Ok(pieces.next().map_or(0, |piece| {
                    chunk[..piece.len()].copy_from_slice(piece);
                    piece.len()
                }))
            },
            Duration::ZERO,
        )
        .unwrap()
}
