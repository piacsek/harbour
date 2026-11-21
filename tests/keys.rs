use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use harbour::keys::{Keyboard, encode, encode_as};

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
fn ctrl_and_escape_is_sent_as_escape_to_a_session_that_did_not_ask() {
    assert_eq!(legacy(with(KeyCode::Esc, KeyModifiers::CONTROL)), b"\x1b");
}

#[test]
fn ctrl_and_escape_is_sent_in_the_xterm_form_to_a_session_that_asked_for_it() {
    assert_eq!(
        xterm(with(KeyCode::Esc, KeyModifiers::CONTROL)),
        b"\x1b[27;5;27~"
    );
}

#[test]
fn ctrl_and_escape_is_sent_in_the_kitty_form_to_a_session_that_asked_for_it() {
    assert_eq!(
        kitty(with(KeyCode::Esc, KeyModifiers::CONTROL)),
        b"\x1b[27;5u"
    );
}

#[test]
fn ctrl_and_enter_is_sent_as_an_escaped_carriage_return_to_a_session_that_did_not_ask() {
    assert_eq!(
        legacy(with(KeyCode::Enter, KeyModifiers::CONTROL)),
        b"\x1b\r"
    );
}

#[test]
fn ctrl_and_enter_is_sent_in_the_xterm_form_to_a_session_that_asked_for_it() {
    assert_eq!(
        xterm(with(KeyCode::Enter, KeyModifiers::CONTROL)),
        b"\x1b[27;5;13~"
    );
}

#[test]
fn shift_and_enter_is_sent_in_the_kitty_form_to_a_session_that_asked_for_it() {
    assert_eq!(
        kitty(with(KeyCode::Enter, KeyModifiers::SHIFT)),
        b"\x1b[13;2u"
    );
}

#[test]
fn ctrl_and_tab_is_sent_as_a_tab_to_a_session_that_did_not_ask() {
    assert_eq!(legacy(with(KeyCode::Tab, KeyModifiers::CONTROL)), b"\t");
}

#[test]
fn ctrl_and_tab_is_sent_in_the_xterm_form_to_a_session_that_asked_for_it() {
    assert_eq!(
        xterm(with(KeyCode::Tab, KeyModifiers::CONTROL)),
        b"\x1b[27;5;9~"
    );
}

#[test]
fn ctrl_and_tab_is_sent_in_the_kitty_form_to_a_session_that_asked_for_it() {
    assert_eq!(
        kitty(with(KeyCode::Tab, KeyModifiers::CONTROL)),
        b"\x1b[9;5u"
    );
}

#[test]
fn ctrl_and_one_is_sent_as_the_digit_to_a_session_that_did_not_ask() {
    assert_eq!(
        legacy(with(KeyCode::Char('1'), KeyModifiers::CONTROL)),
        b"1"
    );
}

#[test]
fn ctrl_and_one_is_sent_in_the_xterm_form_to_a_session_that_asked_for_it() {
    assert_eq!(
        xterm(with(KeyCode::Char('1'), KeyModifiers::CONTROL)),
        b"\x1b[27;5;49~"
    );
}

#[test]
fn ctrl_and_zero_is_sent_in_the_kitty_form_to_a_session_that_asked_for_it() {
    assert_eq!(
        kitty(with(KeyCode::Char('0'), KeyModifiers::CONTROL)),
        b"\x1b[48;5u"
    );
}

#[test]
fn ctrl_and_a_semicolon_is_sent_as_the_character_to_a_session_that_did_not_ask() {
    assert_eq!(
        legacy(with(KeyCode::Char(';'), KeyModifiers::CONTROL)),
        b";"
    );
}

#[test]
fn ctrl_and_a_semicolon_is_sent_in_the_xterm_form_to_a_session_that_asked_for_it() {
    assert_eq!(
        xterm(with(KeyCode::Char(';'), KeyModifiers::CONTROL)),
        b"\x1b[27;5;59~"
    );
}

#[test]
fn ctrl_and_a_quote_is_sent_in_the_kitty_form_to_a_session_that_asked_for_it() {
    assert_eq!(
        kitty(with(KeyCode::Char('\''), KeyModifiers::CONTROL)),
        b"\x1b[39;5u"
    );
}

#[test]
fn ctrl_and_three_is_sent_as_an_escape_to_a_session_that_did_not_ask() {
    assert_eq!(
        legacy(with(KeyCode::Char('3'), KeyModifiers::CONTROL)),
        [0x1b]
    );
}

#[test]
fn ctrl_and_three_is_sent_in_the_xterm_form_to_a_session_that_asked_for_it() {
    assert_eq!(
        xterm(with(KeyCode::Char('3'), KeyModifiers::CONTROL)),
        b"\x1b[27;5;51~"
    );
}

#[test]
fn ctrl_and_eight_is_sent_in_the_kitty_form_to_a_session_that_asked_for_it() {
    assert_eq!(
        kitty(with(KeyCode::Char('8'), KeyModifiers::CONTROL)),
        b"\x1b[56;5u"
    );
}

#[test]
fn ctrl_shift_and_a_letter_is_sent_as_ctrl_and_the_letter_to_a_session_that_did_not_ask() {
    assert_eq!(legacy(with(KeyCode::Char('p'), ctrl_shift())), [0x10]);
}

#[test]
fn ctrl_shift_and_a_letter_is_sent_in_the_xterm_form_to_a_session_that_asked_for_it() {
    assert_eq!(
        xterm(with(KeyCode::Char('p'), ctrl_shift())),
        b"\x1b[27;6;112~"
    );
}

#[test]
fn ctrl_shift_and_an_upper_case_letter_is_sent_in_the_kitty_form_with_the_lower_case_code() {
    assert_eq!(
        kitty(with(KeyCode::Char('P'), ctrl_shift())),
        b"\x1b[112;6u"
    );
}

#[test]
fn f13_is_sent_as_nothing_to_a_session_that_did_not_ask() {
    assert_eq!(legacy(key(KeyCode::F(13))), b"");
}

#[test]
fn f13_is_sent_as_shift_and_f1_to_a_session_that_asked_for_the_xterm_form() {
    assert_eq!(xterm(key(KeyCode::F(13))), b"\x1b[1;2P");
}

#[test]
fn f13_is_sent_in_the_kitty_form_to_a_session_that_asked_for_it() {
    assert_eq!(kitty(key(KeyCode::F(13))), b"\x1b[57376u");
}

#[test]
fn ctrl_and_f13_is_sent_as_ctrl_shift_and_f1_to_a_session_that_asked_for_the_xterm_form() {
    assert_eq!(
        xterm(with(KeyCode::F(13), KeyModifiers::CONTROL)),
        b"\x1b[1;6P"
    );
}

#[test]
fn alt_and_f17_is_sent_as_alt_shift_and_f5_to_a_session_that_asked_for_the_xterm_form() {
    assert_eq!(
        xterm(with(KeyCode::F(17), KeyModifiers::ALT)),
        b"\x1b[15;4~"
    );
}

#[test]
fn shift_and_f13_is_sent_as_nothing_to_a_session_that_asked_for_the_xterm_form() {
    assert_eq!(xterm(with(KeyCode::F(13), KeyModifiers::SHIFT)), b"");
}

#[test]
fn f25_is_sent_as_ctrl_and_f1_to_a_session_that_asked_for_the_xterm_form() {
    assert_eq!(xterm(key(KeyCode::F(25))), b"\x1b[1;5P");
}

#[test]
fn f30_is_sent_as_ctrl_and_f6_to_a_session_that_asked_for_the_xterm_form() {
    assert_eq!(xterm(key(KeyCode::F(30))), b"\x1b[17;5~");
}

#[test]
fn ctrl_and_f25_is_sent_as_nothing_to_a_session_that_asked_for_the_xterm_form() {
    assert_eq!(xterm(with(KeyCode::F(25), KeyModifiers::CONTROL)), b"");
}

#[test]
fn ctrl_and_i_is_sent_as_a_tab_to_a_session_that_did_not_ask() {
    assert_eq!(
        legacy(with(KeyCode::Char('i'), KeyModifiers::CONTROL)),
        b"\t"
    );
}

#[test]
fn ctrl_and_i_is_sent_in_the_xterm_form_to_a_session_that_asked_for_it() {
    assert_eq!(
        xterm(with(KeyCode::Char('i'), KeyModifiers::CONTROL)),
        b"\x1b[27;5;105~"
    );
}

#[test]
fn ctrl_and_i_is_sent_in_the_kitty_form_to_a_session_that_asked_for_it() {
    assert_eq!(
        kitty(with(KeyCode::Char('i'), KeyModifiers::CONTROL)),
        b"\x1b[105;5u"
    );
}

#[test]
fn ctrl_and_m_is_sent_as_a_carriage_return_to_a_session_that_did_not_ask() {
    assert_eq!(
        legacy(with(KeyCode::Char('m'), KeyModifiers::CONTROL)),
        b"\r"
    );
}

#[test]
fn ctrl_and_m_is_sent_in_the_xterm_form_to_a_session_that_asked_for_it() {
    assert_eq!(
        xterm(with(KeyCode::Char('m'), KeyModifiers::CONTROL)),
        b"\x1b[27;5;109~"
    );
}

#[test]
fn ctrl_and_m_is_sent_in_the_kitty_form_to_a_session_that_asked_for_it() {
    assert_eq!(
        kitty(with(KeyCode::Char('m'), KeyModifiers::CONTROL)),
        b"\x1b[109;5u"
    );
}

#[test]
fn ctrl_and_a_bracket_is_sent_in_the_xterm_form_to_a_session_that_asked_for_it() {
    assert_eq!(
        xterm(with(KeyCode::Char('['), KeyModifiers::CONTROL)),
        b"\x1b[27;5;91~"
    );
}

#[test]
fn ctrl_and_a_bracket_is_sent_in_the_kitty_form_to_a_session_that_asked_for_it() {
    assert_eq!(
        kitty(with(KeyCode::Char('['), KeyModifiers::CONTROL)),
        b"\x1b[91;5u"
    );
}

#[test]
fn tab_and_enter_stay_themselves_for_a_session_that_asked() {
    assert_eq!(kitty(key(KeyCode::Tab)), b"\t");
    assert_eq!(kitty(key(KeyCode::Enter)), b"\r");
    assert_eq!(xterm(key(KeyCode::Tab)), b"\t");
    assert_eq!(xterm(key(KeyCode::Enter)), b"\r");
}

#[test]
fn ctrl_and_a_backslash_stays_its_control_byte_for_a_session_that_asked() {
    assert_eq!(
        kitty(with(KeyCode::Char('\\'), KeyModifiers::CONTROL)),
        [0x1c]
    );
    assert_eq!(
        xterm(with(KeyCode::Char('\\'), KeyModifiers::CONTROL)),
        [0x1c]
    );
}

#[test]
fn ctrl_and_a_closing_bracket_stays_its_control_byte_for_a_session_that_asked() {
    assert_eq!(
        kitty(with(KeyCode::Char(']'), KeyModifiers::CONTROL)),
        [0x1d]
    );
    assert_eq!(
        xterm(with(KeyCode::Char(']'), KeyModifiers::CONTROL)),
        [0x1d]
    );
}

#[test]
fn ctrl_and_a_caret_stays_its_control_byte_for_a_session_that_asked() {
    assert_eq!(
        kitty(with(KeyCode::Char('^'), KeyModifiers::CONTROL)),
        [0x1e]
    );
    assert_eq!(
        xterm(with(KeyCode::Char('^'), KeyModifiers::CONTROL)),
        [0x1e]
    );
}

#[test]
fn ctrl_and_an_underscore_stays_its_control_byte_for_a_session_that_asked() {
    assert_eq!(
        kitty(with(KeyCode::Char('_'), KeyModifiers::CONTROL)),
        [0x1f]
    );
    assert_eq!(
        xterm(with(KeyCode::Char('_'), KeyModifiers::CONTROL)),
        [0x1f]
    );
}

#[test]
fn ctrl_and_a_space_stays_its_control_byte_for_a_session_that_asked() {
    assert_eq!(
        kitty(with(KeyCode::Char(' '), KeyModifiers::CONTROL)),
        [0x00]
    );
    assert_eq!(
        xterm(with(KeyCode::Char(' '), KeyModifiers::CONTROL)),
        [0x00]
    );
}

#[test]
fn ctrl_and_a_letter_stays_its_control_byte_for_a_session_that_asked() {
    assert_eq!(
        xterm(with(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        [0x03]
    );
    assert_eq!(
        kitty(with(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        [0x03]
    );
}

#[test]
fn escape_stays_itself_for_a_session_that_asked() {
    assert_eq!(xterm(key(KeyCode::Esc)), b"\x1b");
    assert_eq!(kitty(key(KeyCode::Esc)), b"\x1b");
}

#[test]
fn alt_and_a_letter_stays_escaped_for_a_session_that_asked() {
    assert_eq!(xterm(with(KeyCode::Char('b'), KeyModifiers::ALT)), b"\x1bb");
    assert_eq!(kitty(with(KeyCode::Char('b'), KeyModifiers::ALT)), b"\x1bb");
}

#[test]
fn an_arrow_in_application_cursor_mode_stays_an_ss3_sequence_for_a_session_that_asked() {
    assert_eq!(
        encode_as(key(KeyCode::Left), true, Keyboard::Kitty),
        b"\x1bOD"
    );
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn with(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

fn ctrl_shift() -> KeyModifiers {
    KeyModifiers::CONTROL | KeyModifiers::SHIFT
}

fn legacy(key: KeyEvent) -> Vec<u8> {
    encode_as(key, false, Keyboard::Legacy)
}

fn xterm(key: KeyEvent) -> Vec<u8> {
    encode_as(key, false, Keyboard::Xterm)
}

fn kitty(key: KeyEvent) -> Vec<u8> {
    encode_as(key, false, Keyboard::Kitty)
}
