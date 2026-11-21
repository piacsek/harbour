use std::path::Path;

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::editor::Program;
use harbour::picker::Picker;
use ratatui::style::Color;

use crate::finding::FINDER;
use crate::side::{agent, editing, editor, side};
use crate::{Screen, pane_corner, region, script};

#[test]
fn alt_backslash_with_the_editor_already_running_puts_that_editor_beside_with_claude_code_focused()
{
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = editing(scratch.path());
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor ready 18 78");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    side(&mut screen);

    let shown = screen.wait_for("acme editor size: 18 58");
    assert!(
        region(&shown, 60, 60).contains("acme editor ready 18 78"),
        "{shown}"
    );
    assert_eq!(pane_corner(&shown), 60, "{shown}");
    assert!(screen.cursor().unwrap().x < 60, "{shown}");
}

#[test]
fn alt_backslash_from_the_editor_half_reaches_the_editor_and_keeps_the_mode() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = raw_editor_beside(scratch.path(), "");

    side(&mut screen);

    let shown = screen.wait_for("acme editor read:");
    assert!(shown.contains("acme editor read: 1b 5c"), "{shown}");
    assert_eq!(pane_corner(&shown), 60, "{shown}");
    assert!(screen.cursor().unwrap().x > 60, "{shown}");
}

#[test]
fn alt_backslash_reaches_an_editor_that_asked_for_extended_keys_in_its_encoding() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = raw_editor_beside(scratch.path(), "printf '\\033[>4;2m'; ");

    side(&mut screen);

    let shown = screen.wait_for("acme editor read:");
    assert!(shown.contains("acme editor read: 1b 5b"), "{shown}");
    assert_eq!(pane_corner(&shown), 60, "{shown}");
}

#[test]
fn the_width_keys_from_the_editor_half_reach_the_editor() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = raw_editor_beside(scratch.path(), "");

    screen.key(KeyCode::Char('='), KeyModifiers::ALT);

    let shown = screen.wait_for("acme editor read:");
    assert!(shown.contains("acme editor read: 1b 3d"), "{shown}");
    assert_eq!(pane_corner(&shown), 60, "{shown}");
}

#[test]
fn u_from_the_claude_code_half_reaches_claude_code_and_opens_no_usage() {
    let scratch = tempfile::tempdir().unwrap();
    let app = App::new(agent(scratch.path()))
        .with_editor(Program::new("acme-edit", editor()))
        .with_usage_place(Some(scratch.path().join("usage")))
        .with_usage(script("echo 'acme usage open'; exec sleep 600"));
    let mut screen = split_of(app);

    screen.type_text("u\r");

    let shown = screen.wait_for("acme agent 1 got: u");
    assert!(!shown.contains("acme usage open"), "{shown}");
    assert_eq!(pane_corner(&shown), 60, "{shown}");
}

#[test]
fn esc_in_the_session_finder_leaves_side_by_side_as_it_was() {
    let scratch = tempfile::tempdir().unwrap();
    let app = App::new(agent(scratch.path()))
        .with_editor(Program::new("acme-edit", editor()))
        .with_picker(Picker::Command(vec![FINDER.into()]));
    let mut screen = split_of(app);
    screen.key(KeyCode::Char('s'), KeyModifiers::ALT);
    screen.wait_for("acme fzf ready");

    screen.key(KeyCode::Esc, KeyModifiers::NONE);
    screen.type_text("\r");

    let shown = screen.wait_until("the finder closed", |shown| !shown.contains("acme fzf"));
    assert!(region(&shown, 0, 60).contains("acme agent 1"), "{shown}");
    assert!(
        region(&shown, 60, 60).contains("acme editor ready"),
        "{shown}"
    );
    assert_eq!(pane_corner(&shown), 60, "{shown}");
    assert!(screen.cursor().unwrap().x < 60, "{shown}");
}

#[test]
fn the_configured_sidebar_width_is_ignored_side_by_side_and_comes_back_after() {
    let scratch = tempfile::tempdir().unwrap();
    let app = App::new(agent(scratch.path()))
        .with_editor(Program::new("acme-edit", editor()))
        .with_sidebar_width(30);
    let mut screen = Screen::of(app);
    screen.press('n');
    let before = screen.wait_for("acme agent 1");
    side(&mut screen);
    let split = screen.wait_for("acme editor ready");

    side(&mut screen);

    let after = screen.draw();
    assert_eq!(pane_corner(&before), 30, "{before}");
    assert_eq!(pane_corner(&split), 60, "{split}");
    assert_eq!(pane_corner(&after), 30, "{after}");
}

#[test]
fn the_focused_half_takes_the_accent() {
    let scratch = tempfile::tempdir().unwrap();
    let app = App::new(agent(scratch.path()))
        .with_editor(Program::new("acme-edit", editor()))
        .with_accent(Color::Magenta);
    let mut screen = split_of(app);
    let left = screen.cell(0, 0);
    let right = screen.cell(60, 0);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let moved_left = screen.cell(0, 0);
    let moved_right = screen.cell(60, 0);
    assert_eq!(left.fg, Color::Magenta);
    assert_ne!(right.fg, Color::Magenta);
    assert_ne!(moved_left.fg, Color::Magenta);
    assert_eq!(moved_right.fg, Color::Magenta);
}

#[test]
fn a_jump_to_an_exited_session_ends_the_mode_and_brings_the_sidebar_back() {
    let scratch = tempfile::tempdir().unwrap();
    let count = scratch.path().join("count");
    let agent = script(&format!(
        "n=$(($(cat '{count}' 2>/dev/null || echo 0) + 1)); echo $n > '{count}'; \
         echo \"acme agent $n\"; [ $n = 2 ] && exit 3; exec sleep 600",
        count = count.display()
    ));
    let app = App::new(agent).with_editor(Program::new("acme-edit", editor()));
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");
    screen.key(KeyCode::Char('1'), KeyModifiers::CONTROL);
    side(&mut screen);
    screen.wait_for("acme editor ready");

    screen.key(KeyCode::Char('2'), KeyModifiers::CONTROL);

    let shown = screen.wait_until("the sidebar back", |shown| {
        !shown.contains("acme editor ready")
    });
    assert!(shown.contains("acme agent 2"), "{shown}");
    assert!(region(&shown, 0, 40).contains("exited 3"), "{shown}");
    assert_eq!(pane_corner(&shown), 40, "{shown}");
}

fn split_of(app: App) -> Screen {
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("acme agent 1");
    side(&mut screen);
    screen.wait_until("side by side", |shown| {
        region(shown, 0, 60).contains("acme agent 1")
            && region(shown, 60, 60).contains("acme editor ready")
    });
    screen
}

fn raw_editor_beside(scratch: &Path, asking: &str) -> Screen {
    let editor = Program::new(
        "acme-edit",
        script(&format!(
            "stty raw -echo; {asking}echo 'acme editor ready'; \
             dd bs=1 count=2 2>/dev/null | od -An -tx1 | tr -s ' \\n' '  ' | \
             sed 's/^/acme editor read:/'; exec sleep 600"
        )),
    );
    let mut screen = split_of(App::new(agent(scratch)).with_editor(editor));
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen
}
