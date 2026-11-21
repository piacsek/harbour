use std::thread::sleep;
use std::time::{Duration, Instant};

use harbour::keys::Keyboard;
use harbour::session::Session;

use crate::{ROWS, replied, start, wait_for};

#[test]
fn a_session_whose_program_asked_for_nothing_gets_legacy_keys() {
    let session = start("echo ready; sleep 5");
    wait_for(&session, "ready");
    assert_eq!(session.keyboard(), Keyboard::Legacy);
}

#[test]
fn a_session_whose_program_asked_for_modify_other_keys_gets_the_xterm_form() {
    let session = start("printf '\\033[>4;2m'; echo ready; sleep 5");
    assert_eq!(keyboard_once(&session, "ready"), Keyboard::Xterm);
}

#[test]
fn a_session_whose_program_asked_for_modify_other_keys_level_one_gets_the_xterm_form() {
    let session = start("printf '\\033[>4;1m'; echo ready; sleep 5");
    assert_eq!(keyboard_once(&session, "ready"), Keyboard::Xterm);
}

#[test]
fn a_session_whose_program_reset_modify_other_keys_gets_legacy_keys() {
    let session = start("printf '\\033[>4;2m\\033[>4m'; echo ready; sleep 5");
    assert_eq!(keyboard_once(&session, "ready"), Keyboard::Legacy);
}

#[test]
fn a_session_whose_program_disabled_modify_other_keys_gets_legacy_keys() {
    let session = start("printf '\\033[>4;2m\\033[>4n'; echo ready; sleep 5");
    assert_eq!(keyboard_once(&session, "ready"), Keyboard::Legacy);
}

#[test]
fn a_session_whose_program_disabled_modify_function_keys_keeps_the_xterm_form() {
    let session = start("printf '\\033[>4;2m\\033[>n\\033[>2n'; echo ready; sleep 5");
    assert_eq!(keyboard_once(&session, "ready"), Keyboard::Xterm);
}

#[test]
fn a_session_whose_program_set_modify_other_keys_to_zero_gets_legacy_keys() {
    let session = start("printf '\\033[>4;2m\\033[>4;0m'; echo ready; sleep 5");
    assert_eq!(keyboard_once(&session, "ready"), Keyboard::Legacy);
}

#[test]
fn a_session_whose_program_pushed_kitty_disambiguation_gets_the_kitty_form() {
    let session = start("printf '\\033[>1u'; echo ready; sleep 5");
    assert_eq!(keyboard_once(&session, "ready"), Keyboard::Kitty);
}

#[test]
fn a_session_whose_program_pushed_neovims_kitty_flags_gets_the_kitty_form() {
    let session = start("printf '\\033[>3u'; echo ready; sleep 5");
    assert_eq!(keyboard_once(&session, "ready"), Keyboard::Kitty);
}

#[test]
fn a_session_whose_program_pushed_kitty_flags_without_disambiguation_gets_legacy_keys() {
    let session = start("printf '\\033[>8u'; echo ready; sleep 5");
    assert_eq!(keyboard_once(&session, "ready"), Keyboard::Legacy);
}

#[test]
fn a_session_whose_program_popped_its_kitty_flags_gets_legacy_keys() {
    let session = start("printf '\\033[>1u\\033[<u'; echo ready; sleep 5");
    assert_eq!(keyboard_once(&session, "ready"), Keyboard::Legacy);
}

#[test]
fn a_session_whose_program_popped_one_of_two_kitty_pushes_keeps_the_kitty_form() {
    let session = start("printf '\\033[>1u\\033[>1u\\033[<1u'; echo ready; sleep 5");
    assert_eq!(keyboard_once(&session, "ready"), Keyboard::Kitty);
}

#[test]
fn a_session_whose_program_set_kitty_disambiguation_gets_the_kitty_form() {
    let session = start("printf '\\033[=1;1u'; echo ready; sleep 5");
    assert_eq!(keyboard_once(&session, "ready"), Keyboard::Kitty);
}

#[test]
fn a_session_whose_program_cleared_kitty_disambiguation_gets_legacy_keys() {
    let session = start("printf '\\033[=1u\\033[=1;3u'; echo ready; sleep 5");
    assert_eq!(keyboard_once(&session, "ready"), Keyboard::Legacy);
}

#[test]
fn a_session_whose_program_asked_for_both_forms_gets_the_kitty_form() {
    let session = start("printf '\\033[>4;2m\\033[>1u'; echo ready; sleep 5");
    assert_eq!(keyboard_once(&session, "ready"), Keyboard::Kitty);
}

#[test]
fn a_kitty_keyboard_query_is_answered_with_no_flags_before_a_push() {
    let session = start(&replied("\\033[?u", 5));
    let expected = "reply:[?0u";
    let screen = wait_for(&session, expected);
    assert!(screen.contains(expected), "{screen}");
}

#[test]
fn a_kitty_keyboard_query_is_answered_with_the_disambiguation_it_pushed() {
    let session = start(&replied("\\033[>3u\\033[?u", 5));
    let expected = "reply:[?1u";
    let screen = wait_for(&session, expected);
    assert!(screen.contains(expected), "{screen}");
}

#[test]
fn a_terminal_reset_forgets_what_the_program_asked_for() {
    let session = start("printf '\\033[>4;2m\\033[>1u'; printf '\\033c'; echo ready; sleep 5");
    assert_eq!(keyboard_once(&session, "ready"), Keyboard::Legacy);
}

#[test]
fn a_request_after_a_terminal_reset_is_kept() {
    let session = start("printf '\\033c\\033[>4;2m'; echo ready; sleep 5");
    assert_eq!(keyboard_once(&session, "ready"), Keyboard::Xterm);
}

#[test]
fn a_session_whose_program_exited_forgets_what_it_asked_for() {
    let mut session = start("printf '\\033[>4;2m'; echo ready; sleep 0.3");
    wait_for(&session, "ready");
    let deadline = Instant::now() + Duration::from_secs(5);
    while session.exit().is_none() && Instant::now() < deadline {
        sleep(Duration::from_millis(20));
    }
    assert_eq!(settled(&session, Keyboard::Legacy), Keyboard::Legacy);
}

#[test]
fn a_session_whose_screen_was_reset_keeps_the_keys_its_program_asked_for() {
    let mut session =
        Session::start(&asking_wide_at_the_edge(), &std::env::temp_dir(), ROWS, 160).unwrap();
    wait_for(&session, "ready");
    session.resize(ROWS, 159);
    session.send(b"x");
    sleep(Duration::from_millis(300));

    session.resize(ROWS, 159);

    assert!(session.take_panics() > 0);
    assert_eq!(session.keyboard(), Keyboard::Xterm);
}

fn keyboard_once(session: &Session, text: &str) -> Keyboard {
    wait_for(session, text);
    session.keyboard()
}

fn settled(session: &Session, expected: Keyboard) -> Keyboard {
    let deadline = Instant::now() + Duration::from_secs(5);
    while session.keyboard() != expected && Instant::now() < deadline {
        sleep(Duration::from_millis(20));
    }
    session.keyboard()
}

fn asking_wide_at_the_edge() -> [String; 3] {
    let script = "printf '\\033[>4;2m'; \
                  trap 'printf \"\\033[1;1H%*s\" \"$(stty size | cut -d\" \" -f2)\" \"\"' WINCH; \
                  cols=$(stty size | cut -d' ' -f2); \
                  printf '\\033[1;%dH\\343\\201\\202' $((cols-1)); \
                  printf '\\033[3;1Hready\\033[5;1H'; \
                  stty raw -echo; \
                  while :; do dd bs=1 count=1 2>/dev/null; done";
    ["/bin/sh".to_string(), "-c".into(), script.into()]
}
