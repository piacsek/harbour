use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};

use crate::{Screen, numbered_agent, raw_agent, reading_agent};

#[test]
fn a_paste_reaches_the_focused_session() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.app.paste("from the clipboard\r");

    let shown = screen.wait_for("acme agent got: from the clipboard");
    assert!(
        shown.contains("acme agent got: from the clipboard"),
        "{shown}"
    );
}

#[test]
fn a_paste_is_bracketed_when_the_session_asked_for_it() {
    let mut screen = Screen::new(raw_agent("\\033[?2004h", 14));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.app.paste("hi");

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read:[200~hi[201~"), "{shown}");
}

#[test]
fn a_paste_while_the_sidebar_has_focus_does_not_reach_the_session() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");

    screen.app.paste("zz\r");
    screen.type_text("\r\r");

    let shown = screen.wait_for("acme agent got:");
    assert!(shown.contains("acme agent got:"), "{shown}");
    assert!(!shown.contains("got: zz"), "{shown}");
}

#[test]
fn a_click_in_the_pane_reaches_a_session_that_asked_for_the_mouse() {
    let mut screen = Screen::new(raw_agent("\\033[?1000h\\033[?1006h", 9));
    screen.press('n');
    screen.wait_for("acme agent ready");

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 45, 3);

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read:[<0;5;3M"), "{shown}");
}

#[test]
fn a_click_on_the_sidebar_gives_it_the_focus() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 5, 5);
    screen.press('n');

    let shown = screen.wait_for("acme agent 2");
    assert!(shown.contains("acme agent 2"), "{shown}");
}

#[test]
fn a_click_in_the_pane_gives_it_the_focus_and_is_not_sent_unasked() {
    let mut screen = Screen::new(raw_agent("", 1));
    screen.press('n');
    screen.wait_for("acme agent ready");

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 45, 3);
    screen.press('z');

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read:z"), "{shown}");
}
