use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use harbour::keymap::{Action, After, Keymap};

#[test]
fn a_bare_key_after_the_prefix_runs_its_action() {
    assert_eq!(after('s', KeyModifiers::NONE), After::Act(Action::Find));
}

#[test]
fn a_key_with_ctrl_held_after_the_prefix_runs_the_same_action() {
    assert_eq!(after('s', KeyModifiers::CONTROL), After::Act(Action::Find));
}

#[test]
fn a_digit_with_ctrl_held_after_the_prefix_jumps() {
    assert_eq!(after('2', KeyModifiers::CONTROL), After::Jump(2));
}

#[test]
fn ctrl_and_shift_held_after_the_prefix_act_as_shift() {
    assert_eq!(
        after('X', KeyModifiers::CONTROL | KeyModifiers::SHIFT),
        After::Act(Action::Close)
    );
}

#[test]
fn ctrl_and_shift_held_on_a_lower_case_letter_act_as_its_capital() {
    assert_eq!(
        after('x', KeyModifiers::CONTROL | KeyModifiers::SHIFT),
        After::Act(Action::Close)
    );
}

#[test]
fn shift_held_on_a_lower_case_letter_acts_as_its_capital() {
    assert_eq!(after('x', KeyModifiers::SHIFT), After::Act(Action::Close));
}

#[test]
fn ctrl_c_after_the_prefix_shows_the_changelog() {
    assert_eq!(
        after('c', KeyModifiers::CONTROL),
        After::Act(Action::Changelog)
    );
}

#[test]
fn a_configured_key_with_ctrl_held_runs_its_action() {
    let keymap = Keymap::default().bind(Action::Zoom, Some('f'));

    let after = keymap.after(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL));

    assert_eq!(after, After::Act(Action::Zoom));
}

#[test]
fn an_unbound_key_with_ctrl_held_does_nothing() {
    assert_eq!(after('w', KeyModifiers::CONTROL), After::Unknown);
}

#[test]
fn the_prefix_again_is_still_the_prefix() {
    assert_eq!(after('/', KeyModifiers::CONTROL), After::Prefix);
}

#[test]
fn a_ctrl_semicolon_prefix_again_is_still_the_prefix() {
    let keymap = Keymap::default().with_prefix(';');

    let after = keymap.after(KeyEvent::new(KeyCode::Char(';'), KeyModifiers::CONTROL));

    assert_eq!(after, After::Prefix);
}

#[test]
fn alt_held_after_the_prefix_does_nothing() {
    assert_eq!(after('s', KeyModifiers::ALT), After::Unknown);
}

#[test]
fn ctrl_and_alt_held_after_the_prefix_do_nothing() {
    assert_eq!(
        after('s', KeyModifiers::CONTROL | KeyModifiers::ALT),
        After::Unknown
    );
}

#[test]
fn escape_after_the_prefix_cancels_it() {
    let after = Keymap::default().after(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

    assert_eq!(after, After::Cancel);
}

fn after(c: char, modifiers: KeyModifiers) -> After {
    Keymap::default().after(KeyEvent::new(KeyCode::Char(c), modifiers))
}
