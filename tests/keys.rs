use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use harbour::keys::{Keyboard, encode, encode_as};

const SHIFT: KeyModifiers = KeyModifiers::SHIFT;
const CONTROL: KeyModifiers = KeyModifiers::CONTROL;
const ALT: KeyModifiers = KeyModifiers::ALT;

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
fn ctrl_and_enter_is_sent_as_an_escaped_carriage_return() {
    assert_eq!(
        encode(with(KeyCode::Enter, KeyModifiers::CONTROL), false),
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
fn a_session_that_asked_for_nothing_gets_the_legacy_bytes() {
    assert_eq!(legacy(plain('a')), b"a");
    assert_eq!(legacy(shift('A')), b"A");
    assert_eq!(legacy(ctrl('a')), b"\x01");
    assert_eq!(legacy(ctrl_shift('a')), b"\x01");
    assert_eq!(legacy(ctrl('A')), b"\x01");
    assert_eq!(legacy(alt('a')), b"\x1ba");
    assert_eq!(legacy(ctrl_alt('a')), b"\x1b\x01");
    assert_eq!(legacy(ctrl('i')), b"\t");
    assert_eq!(legacy(ctrl('m')), b"\r");
    assert_eq!(legacy(ctrl('[')), b"\x1b");
    assert_eq!(legacy(ctrl('\\')), b"\x1c");
    assert_eq!(legacy(ctrl(']')), b"\x1d");
    assert_eq!(legacy(ctrl(' ')), b"\x00");
    assert_eq!(legacy(ctrl('/')), b"\x1f");
    assert_eq!(legacy(ctrl('2')), b"\x00");
    assert_eq!(legacy(ctrl('3')), b"\x1b");
    assert_eq!(legacy(ctrl('8')), b"\x7f");
    assert_eq!(legacy(ctrl('0')), b"0");
    assert_eq!(legacy(ctrl('9')), b"9");
    assert_eq!(legacy(ctrl(',')), b",");
    assert_eq!(legacy(ctrl('.')), b".");
    assert_eq!(legacy(ctrl(';')), b";");
    assert_eq!(legacy(ctrl('?')), b"\x7f");
    assert_eq!(legacy(key(KeyCode::Enter)), b"\r");
    assert_eq!(legacy(with(KeyCode::Enter, SHIFT)), b"\x1b\r");
    assert_eq!(legacy(with(KeyCode::Enter, CONTROL)), b"\x1b\r");
    assert_eq!(legacy(with(KeyCode::Enter, ALT)), b"\x1b\r");
    assert_eq!(legacy(key(KeyCode::Tab)), b"\t");
    assert_eq!(legacy(with(KeyCode::Tab, SHIFT)), b"\t");
    assert_eq!(legacy(with(KeyCode::Tab, CONTROL)), b"\t");
    assert_eq!(legacy(with(KeyCode::Tab, ALT)), b"\x1b\t");
    assert_eq!(legacy(with(KeyCode::BackTab, SHIFT)), b"\x1b[Z");
    assert_eq!(legacy(with(KeyCode::BackTab, CONTROL | SHIFT)), b"\x1b[Z");
    assert_eq!(legacy(key(KeyCode::Esc)), b"\x1b");
    assert_eq!(legacy(with(KeyCode::Esc, CONTROL)), b"\x1b");
    assert_eq!(legacy(with(KeyCode::Esc, SHIFT)), b"\x1b");
    assert_eq!(legacy(with(KeyCode::Esc, ALT)), b"\x1b\x1b");
    assert_eq!(legacy(key(KeyCode::Backspace)), b"\x7f");
    assert_eq!(legacy(with(KeyCode::Backspace, SHIFT)), b"\x7f");
    assert_eq!(legacy(with(KeyCode::Backspace, ALT)), b"\x1b\x7f");
    assert_eq!(legacy(key(KeyCode::Up)), b"\x1b[A");
    assert_eq!(legacy(with(KeyCode::Up, CONTROL)), b"\x1b[1;5A");
    assert_eq!(legacy(key(KeyCode::F(1))), b"\x1bOP");
    assert_eq!(legacy(with(KeyCode::F(1), CONTROL)), b"\x1b[1;5P");
    assert_eq!(legacy(key(KeyCode::F(3))), b"\x1bOR");
    assert_eq!(legacy(with(KeyCode::F(3), SHIFT)), b"\x1b[1;2R");
    assert_eq!(legacy(key(KeyCode::F(5))), b"\x1b[15~");
    assert_eq!(legacy(with(KeyCode::F(5), CONTROL)), b"\x1b[15;5~");
    assert_eq!(legacy(key(KeyCode::F(13))), b"");
    assert_eq!(legacy(key(KeyCode::F(25))), b"");
    assert_eq!(cursor(Keyboard::Legacy), b"\x1bOA");
}

#[test]
fn a_program_that_asked_for_modify_other_keys_level_one_gets_xterms_level_one_bytes() {
    assert_eq!(xterm_one(plain('a')), b"a");
    assert_eq!(xterm_one(shift('A')), b"A");
    assert_eq!(xterm_one(ctrl('a')), b"\x01");
    assert_eq!(xterm_one(ctrl_shift('a')), b"\x01");
    assert_eq!(xterm_one(ctrl('A')), b"\x01");
    assert_eq!(xterm_one(alt('a')), b"\x1ba");
    assert_eq!(xterm_one(ctrl_alt('a')), b"\x1b\x01");
    assert_eq!(xterm_one(ctrl('i')), b"\t");
    assert_eq!(xterm_one(ctrl('m')), b"\r");
    assert_eq!(xterm_one(ctrl('[')), b"\x1b");
    assert_eq!(xterm_one(ctrl('\\')), b"\x1c");
    assert_eq!(xterm_one(ctrl(']')), b"\x1d");
    assert_eq!(xterm_one(ctrl(' ')), b"\x00");
    assert_eq!(xterm_one(ctrl('/')), b"\x1f");
    assert_eq!(xterm_one(ctrl('2')), b"\x00");
    assert_eq!(xterm_one(ctrl('3')), b"\x1b");
    assert_eq!(xterm_one(ctrl('8')), b"\x7f");
    assert_eq!(xterm_one(ctrl('0')), b"\x1b[27;5;48~");
    assert_eq!(xterm_one(ctrl('9')), b"\x1b[27;5;57~");
    assert_eq!(xterm_one(ctrl(',')), b"\x1b[27;5;44~");
    assert_eq!(xterm_one(ctrl('.')), b"\x1b[27;5;46~");
    assert_eq!(xterm_one(ctrl(';')), b"\x1b[27;5;59~");
    assert_eq!(xterm_one(ctrl('?')), b"\x1b[27;5;63~");
    assert_eq!(xterm_one(key(KeyCode::Enter)), b"\r");
    assert_eq!(xterm_one(with(KeyCode::Enter, SHIFT)), b"\x1b[27;2;13~");
    assert_eq!(xterm_one(with(KeyCode::Enter, CONTROL)), b"\x1b[27;5;13~");
    assert_eq!(xterm_one(with(KeyCode::Enter, ALT)), b"\x1b\r");
    assert_eq!(xterm_one(key(KeyCode::Tab)), b"\t");
    assert_eq!(xterm_one(with(KeyCode::Tab, SHIFT)), b"\x1b[27;2;9~");
    assert_eq!(xterm_one(with(KeyCode::Tab, CONTROL)), b"\x1b[27;5;9~");
    assert_eq!(xterm_one(with(KeyCode::Tab, ALT)), b"\x1b\t");
    assert_eq!(xterm_one(with(KeyCode::BackTab, SHIFT)), b"\x1b[Z");
    assert_eq!(
        xterm_one(with(KeyCode::BackTab, CONTROL | SHIFT)),
        b"\x1b[27;6;9~"
    );
    assert_eq!(xterm_one(key(KeyCode::Esc)), b"\x1b");
    assert_eq!(xterm_one(with(KeyCode::Esc, CONTROL)), b"\x1b[27;5;27~");
    assert_eq!(xterm_one(with(KeyCode::Esc, SHIFT)), b"\x1b[27;2;27~");
    assert_eq!(xterm_one(with(KeyCode::Esc, ALT)), b"\x1b\x1b");
    assert_eq!(xterm_one(key(KeyCode::Backspace)), b"\x7f");
    assert_eq!(
        xterm_one(with(KeyCode::Backspace, SHIFT)),
        b"\x1b[27;2;127~"
    );
    assert_eq!(xterm_one(with(KeyCode::Backspace, ALT)), b"\x1b\x7f");
    assert_eq!(xterm_one(key(KeyCode::Up)), b"\x1b[A");
    assert_eq!(xterm_one(with(KeyCode::Up, CONTROL)), b"\x1b[1;5A");
    assert_eq!(xterm_one(key(KeyCode::F(1))), b"\x1bOP");
    assert_eq!(xterm_one(with(KeyCode::F(1), CONTROL)), b"\x1b[1;5P");
    assert_eq!(xterm_one(key(KeyCode::F(3))), b"\x1bOR");
    assert_eq!(xterm_one(with(KeyCode::F(3), SHIFT)), b"\x1b[1;2R");
    assert_eq!(xterm_one(key(KeyCode::F(5))), b"\x1b[15~");
    assert_eq!(xterm_one(with(KeyCode::F(5), CONTROL)), b"\x1b[15;5~");
    assert_eq!(xterm_one(key(KeyCode::F(13))), b"\x1b[1;2P");
    assert_eq!(xterm_one(key(KeyCode::F(25))), b"\x1b[1;5P");
    assert_eq!(cursor(Keyboard::XtermOne), b"\x1bOA");
}

#[test]
fn a_program_that_asked_for_modify_other_keys_level_two_gets_xterms_level_two_bytes() {
    assert_eq!(xterm_two(plain('a')), b"a");
    assert_eq!(xterm_two(shift('A')), b"A");
    assert_eq!(xterm_two(ctrl('a')), b"\x1b[27;5;97~");
    assert_eq!(xterm_two(ctrl_shift('a')), b"\x1b[27;6;97~");
    assert_eq!(xterm_two(ctrl('A')), b"\x1b[27;5;65~");
    assert_eq!(xterm_two(alt('a')), b"\x1b[27;3;97~");
    assert_eq!(xterm_two(ctrl_alt('a')), b"\x1b[27;7;97~");
    assert_eq!(xterm_two(ctrl('i')), b"\x1b[27;5;105~");
    assert_eq!(xterm_two(ctrl('m')), b"\x1b[27;5;109~");
    assert_eq!(xterm_two(ctrl('[')), b"\x1b[27;5;91~");
    assert_eq!(xterm_two(ctrl('\\')), b"\x1b[27;5;92~");
    assert_eq!(xterm_two(ctrl(']')), b"\x1b[27;5;93~");
    assert_eq!(xterm_two(ctrl(' ')), b"\x1b[27;5;32~");
    assert_eq!(xterm_two(ctrl('/')), b"\x1b[27;5;47~");
    assert_eq!(xterm_two(ctrl('2')), b"\x1b[27;5;50~");
    assert_eq!(xterm_two(ctrl('3')), b"\x1b[27;5;51~");
    assert_eq!(xterm_two(ctrl('8')), b"\x1b[27;5;56~");
    assert_eq!(xterm_two(ctrl('0')), b"\x1b[27;5;48~");
    assert_eq!(xterm_two(ctrl('9')), b"\x1b[27;5;57~");
    assert_eq!(xterm_two(ctrl(',')), b"\x1b[27;5;44~");
    assert_eq!(xterm_two(ctrl('.')), b"\x1b[27;5;46~");
    assert_eq!(xterm_two(ctrl(';')), b"\x1b[27;5;59~");
    assert_eq!(xterm_two(ctrl('?')), b"\x1b[27;5;63~");
    assert_eq!(xterm_two(key(KeyCode::Enter)), b"\r");
    assert_eq!(xterm_two(with(KeyCode::Enter, SHIFT)), b"\x1b[27;2;13~");
    assert_eq!(xterm_two(with(KeyCode::Enter, CONTROL)), b"\x1b[27;5;13~");
    assert_eq!(xterm_two(with(KeyCode::Enter, ALT)), b"\x1b[27;3;13~");
    assert_eq!(xterm_two(key(KeyCode::Tab)), b"\t");
    assert_eq!(xterm_two(with(KeyCode::Tab, SHIFT)), b"\x1b[27;2;9~");
    assert_eq!(xterm_two(with(KeyCode::Tab, CONTROL)), b"\x1b[27;5;9~");
    assert_eq!(xterm_two(with(KeyCode::Tab, ALT)), b"\x1b[27;3;9~");
    assert_eq!(xterm_two(with(KeyCode::BackTab, SHIFT)), b"\x1b[27;2;9~");
    assert_eq!(
        xterm_two(with(KeyCode::BackTab, CONTROL | SHIFT)),
        b"\x1b[27;6;9~"
    );
    assert_eq!(xterm_two(key(KeyCode::Esc)), b"\x1b");
    assert_eq!(xterm_two(with(KeyCode::Esc, CONTROL)), b"\x1b[27;5;27~");
    assert_eq!(xterm_two(with(KeyCode::Esc, SHIFT)), b"\x1b[27;2;27~");
    assert_eq!(xterm_two(with(KeyCode::Esc, ALT)), b"\x1b[27;3;27~");
    assert_eq!(xterm_two(key(KeyCode::Backspace)), b"\x7f");
    assert_eq!(
        xterm_two(with(KeyCode::Backspace, SHIFT)),
        b"\x1b[27;2;127~"
    );
    assert_eq!(xterm_two(with(KeyCode::Backspace, ALT)), b"\x1b[27;3;127~");
    assert_eq!(xterm_two(key(KeyCode::Up)), b"\x1b[A");
    assert_eq!(xterm_two(with(KeyCode::Up, CONTROL)), b"\x1b[1;5A");
    assert_eq!(xterm_two(key(KeyCode::F(1))), b"\x1bOP");
    assert_eq!(xterm_two(with(KeyCode::F(1), CONTROL)), b"\x1b[1;5P");
    assert_eq!(xterm_two(key(KeyCode::F(3))), b"\x1bOR");
    assert_eq!(xterm_two(with(KeyCode::F(3), SHIFT)), b"\x1b[1;2R");
    assert_eq!(xterm_two(key(KeyCode::F(5))), b"\x1b[15~");
    assert_eq!(xterm_two(with(KeyCode::F(5), CONTROL)), b"\x1b[15;5~");
    assert_eq!(xterm_two(key(KeyCode::F(13))), b"\x1b[1;2P");
    assert_eq!(xterm_two(key(KeyCode::F(25))), b"\x1b[1;5P");
    assert_eq!(cursor(Keyboard::XtermTwo), b"\x1bOA");
}

#[test]
fn a_program_that_asked_for_kitty_disambiguation_gets_the_kitty_bytes() {
    assert_eq!(kitty(plain('a')), b"a");
    assert_eq!(kitty(shift('A')), b"A");
    assert_eq!(kitty(ctrl('a')), b"\x1b[97;5u");
    assert_eq!(kitty(ctrl_shift('a')), b"\x1b[97;6u");
    assert_eq!(kitty(ctrl('A')), b"\x1b[97;6u");
    assert_eq!(kitty(alt('a')), b"\x1b[97;3u");
    assert_eq!(kitty(ctrl_alt('a')), b"\x1b[97;7u");
    assert_eq!(kitty(ctrl('i')), b"\x1b[105;5u");
    assert_eq!(kitty(ctrl('m')), b"\x1b[109;5u");
    assert_eq!(kitty(ctrl('[')), b"\x1b[91;5u");
    assert_eq!(kitty(ctrl('\\')), b"\x1b[92;5u");
    assert_eq!(kitty(ctrl(']')), b"\x1b[93;5u");
    assert_eq!(kitty(ctrl(' ')), b"\x1b[32;5u");
    assert_eq!(kitty(ctrl('/')), b"\x1b[47;5u");
    assert_eq!(kitty(ctrl('2')), b"\x1b[50;5u");
    assert_eq!(kitty(ctrl('3')), b"\x1b[51;5u");
    assert_eq!(kitty(ctrl('8')), b"\x1b[56;5u");
    assert_eq!(kitty(ctrl('0')), b"\x1b[48;5u");
    assert_eq!(kitty(ctrl('9')), b"\x1b[57;5u");
    assert_eq!(kitty(ctrl(',')), b"\x1b[44;5u");
    assert_eq!(kitty(ctrl('.')), b"\x1b[46;5u");
    assert_eq!(kitty(ctrl(';')), b"\x1b[59;5u");
    assert_eq!(kitty(ctrl('?')), b"\x1b[63;5u");
    assert_eq!(kitty(key(KeyCode::Enter)), b"\r");
    assert_eq!(kitty(with(KeyCode::Enter, SHIFT)), b"\x1b[13;2u");
    assert_eq!(kitty(with(KeyCode::Enter, CONTROL)), b"\x1b[13;5u");
    assert_eq!(kitty(with(KeyCode::Enter, ALT)), b"\x1b[13;3u");
    assert_eq!(kitty(key(KeyCode::Tab)), b"\t");
    assert_eq!(kitty(with(KeyCode::Tab, SHIFT)), b"\x1b[9;2u");
    assert_eq!(kitty(with(KeyCode::Tab, CONTROL)), b"\x1b[9;5u");
    assert_eq!(kitty(with(KeyCode::Tab, ALT)), b"\x1b[9;3u");
    assert_eq!(kitty(with(KeyCode::BackTab, SHIFT)), b"\x1b[9;2u");
    assert_eq!(kitty(with(KeyCode::BackTab, CONTROL | SHIFT)), b"\x1b[9;6u");
    assert_eq!(kitty(key(KeyCode::Esc)), b"\x1b[27u");
    assert_eq!(kitty(with(KeyCode::Esc, CONTROL)), b"\x1b[27;5u");
    assert_eq!(kitty(with(KeyCode::Esc, SHIFT)), b"\x1b[27;2u");
    assert_eq!(kitty(with(KeyCode::Esc, ALT)), b"\x1b[27;3u");
    assert_eq!(kitty(key(KeyCode::Backspace)), b"\x7f");
    assert_eq!(kitty(with(KeyCode::Backspace, SHIFT)), b"\x1b[127;2u");
    assert_eq!(kitty(with(KeyCode::Backspace, ALT)), b"\x1b[127;3u");
    assert_eq!(kitty(key(KeyCode::Up)), b"\x1b[A");
    assert_eq!(kitty(with(KeyCode::Up, CONTROL)), b"\x1b[1;5A");
    assert_eq!(kitty(key(KeyCode::F(1))), b"\x1bOP");
    assert_eq!(kitty(with(KeyCode::F(1), CONTROL)), b"\x1b[1;5P");
    assert_eq!(kitty(key(KeyCode::F(3))), b"\x1b[13~");
    assert_eq!(kitty(with(KeyCode::F(3), SHIFT)), b"\x1b[13;2~");
    assert_eq!(kitty(key(KeyCode::F(5))), b"\x1b[15~");
    assert_eq!(kitty(with(KeyCode::F(5), CONTROL)), b"\x1b[15;5~");
    assert_eq!(kitty(key(KeyCode::F(13))), b"\x1b[57376u");
    assert_eq!(kitty(key(KeyCode::F(25))), b"\x1b[57388u");
    assert_eq!(cursor(Keyboard::Kitty), b"\x1bOA");
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn with(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

fn plain(c: char) -> KeyEvent {
    key(KeyCode::Char(c))
}

fn shift(c: char) -> KeyEvent {
    with(KeyCode::Char(c), SHIFT)
}

fn ctrl(c: char) -> KeyEvent {
    with(KeyCode::Char(c), CONTROL)
}

fn ctrl_shift(c: char) -> KeyEvent {
    with(KeyCode::Char(c), CONTROL | SHIFT)
}

fn alt(c: char) -> KeyEvent {
    with(KeyCode::Char(c), ALT)
}

fn ctrl_alt(c: char) -> KeyEvent {
    with(KeyCode::Char(c), CONTROL | ALT)
}

fn cursor(keyboard: Keyboard) -> Vec<u8> {
    encode_as(key(KeyCode::Up), true, keyboard)
}

fn legacy(key: KeyEvent) -> Vec<u8> {
    encode_as(key, false, Keyboard::Legacy)
}

fn xterm_one(key: KeyEvent) -> Vec<u8> {
    encode_as(key, false, Keyboard::XtermOne)
}

fn xterm_two(key: KeyEvent) -> Vec<u8> {
    encode_as(key, false, Keyboard::XtermTwo)
}

fn kitty(key: KeyEvent) -> Vec<u8> {
    encode_as(key, false, Keyboard::Kitty)
}
