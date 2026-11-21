use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use harbour::editor::Program;

use crate::editor_input::raw_editor;
use crate::{Screen, raw_agent, reading_agent};

const CLICKS: &str = "\\033[?1000h\\033[?1006h";
const DRAGS: &str = "\\033[?1002h\\033[?1006h";

#[test]
fn a_click_in_the_editor_half_focuses_it_and_reaches_the_editor() {
    let mut screen = side_by_side(reading_agent(), raw_editor(CLICKS, 9));

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 65, 3);

    screen.wait_for("acme editor read:[<0;5;3M");
    assert_eq!(screen.cell(60, 0).symbol(), "┏");
    assert_eq!(screen.cell(0, 0).symbol(), "┌");
}

#[test]
fn a_click_in_the_claude_code_half_focuses_it_and_reaches_claude_code() {
    let mut screen = side_by_side(raw_agent(CLICKS, 9), raw_editor(CLICKS, 9));
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 5, 3);

    screen.wait_for("acme agent read:[<0;5;3M");
    assert_eq!(screen.cell(0, 0).symbol(), "┏");
    assert_eq!(screen.cell(60, 0).symbol(), "┌");
}

#[test]
fn a_click_on_the_border_between_the_halves_reaches_nothing_and_keeps_the_focus() {
    let mut screen = side_by_side(reading_agent(), raw_editor(CLICKS, 9));
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 60, 3);
    screen.mouse(MouseEventKind::Up(MouseButton::Left), 60, 3);
    screen.draw();
    assert_eq!(screen.cell(0, 0).symbol(), "┏");

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 66, 4);

    let shown = screen.wait_for("acme editor read:");
    assert!(shown.contains("acme editor read:[<0;6;4M"), "{shown}");
}

#[test]
fn a_drag_from_the_editor_half_across_the_middle_stays_with_the_editor() {
    let mut screen = side_by_side(reading_agent(), raw_editor(DRAGS, 19));

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 65, 3);
    screen.mouse(MouseEventKind::Drag(MouseButton::Left), 10, 3);

    let shown = screen.wait_for("acme editor read:");
    assert!(
        shown.contains("acme editor read:[<0;5;3M[<32;1;3M"),
        "{shown}"
    );
}

#[test]
fn a_wheel_over_the_unfocused_editor_half_reaches_it_and_keeps_the_focus() {
    let mut screen = side_by_side(reading_agent(), raw_editor(CLICKS, 10));

    screen.mouse(MouseEventKind::ScrollUp, 65, 3);

    screen.wait_for("acme editor read:[<64;5;3M");
    assert_eq!(screen.cell(0, 0).symbol(), "┏");
}

#[test]
fn swapped_a_click_in_the_left_half_focuses_the_editor_and_reaches_it() {
    let mut screen = side_by_side(reading_agent(), raw_editor(CLICKS, 9));
    swap(&mut screen);

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 5, 3);

    screen.wait_for("acme editor read:[<0;5;3M");
    assert_eq!(screen.cell(0, 0).symbol(), "┏");
    assert_eq!(screen.cell(60, 0).symbol(), "┌");
}

#[test]
fn swapped_a_drag_from_the_right_half_across_the_middle_stays_with_claude_code() {
    let mut screen = side_by_side(raw_agent(DRAGS, 19), raw_editor(DRAGS, 19));
    swap(&mut screen);

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 65, 3);
    screen.mouse(MouseEventKind::Drag(MouseButton::Left), 10, 3);

    let shown = screen.wait_for("acme agent read:[<0;5;3M[<32;1;3M");
    assert!(
        shown.contains("acme agent read:[<0;5;3M[<32;1;3M"),
        "{shown}"
    );
}

fn side_by_side(agent: Vec<String>, editor: Program) -> Screen {
    let mut screen = Screen::editing(agent, editor);
    screen.press('n');
    screen.wait_for("acme agent");
    screen.key(KeyCode::Char('\\'), KeyModifiers::ALT);
    screen.wait_for("acme editor ready");
    screen
}

fn swap(screen: &mut Screen) {
    screen.key(KeyCode::Char('}'), KeyModifiers::ALT);
}
