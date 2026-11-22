use std::cell::RefCell;
use std::io;
use std::path::Path;
use std::rc::Rc;

use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use harbour::app::App;
use harbour::cursor::Shape;
use harbour::editor::Program;
use harbour::picker::Picker;
use ratatui::layout::Position;
use ratatui::style::Modifier;

use crate::{Screen, frames, home, pane_corner, reading_agent, script};

#[test]
fn the_terminal_cursor_sits_on_the_pane_cursor_cell() {
    let mut screen = Screen::new(shaped_agent("6"));
    let shown = focused_on_the_pane(&mut screen);

    let corner = pane_corner(&shown);

    assert_eq!(screen.cursor(), Some(Position::new(corner + 22, 3)));
}

#[test]
fn a_zoomed_session_keeps_its_cursor_cell_below_the_top_row_from_the_first_column() {
    let mut screen = Screen::new(shaped_agent("6"));
    focused_on_the_pane(&mut screen);

    screen.prefixed('z');
    screen.draw();

    assert_eq!(screen.cursor(), Some(Position::new(21, 3)));
}

#[test]
fn the_app_asks_for_the_shape_the_shown_program_set() {
    let mut screen = Screen::new(shaped_agent("6"));
    screen.press('n');
    screen.wait_for("acme agent shaped");

    assert_eq!(screen.app.cursor_shape(), 6);
}

#[test]
fn a_program_that_hid_its_cursor_shows_none() {
    let mut screen = Screen::new(script(
        "printf '\\033[?25lacme agent hidden'; exec sleep 600",
    ));
    focused_on_the_pane(&mut screen);
    screen.wait_for("acme agent hidden");

    assert_eq!(screen.cursor(), None);
}

#[test]
fn a_hidden_cursor_keeps_the_shape_the_program_set() {
    let mut screen = Screen::new(script(
        "printf '\\033[6 q\\033[?25lacme agent hidden'; exec sleep 600",
    ));
    focused_on_the_pane(&mut screen);
    screen.wait_for("acme agent hidden");

    assert_eq!(screen.cursor(), None);
    assert_eq!(screen.app.cursor_shape(), 6);
}

#[test]
fn with_no_session_the_terminal_shows_no_cursor() {
    let screen = Screen::new(shaped_agent("6"));

    assert_eq!(screen.cursor(), None);
    assert_eq!(screen.app.cursor_shape(), 0);
}

#[test]
fn the_help_view_shows_no_cursor_and_the_default_shape() {
    let mut screen = Screen::new(shaped_agent("6"));
    screen.press('n');
    screen.wait_for("acme agent shaped");

    screen.prefixed('?');

    screen.wait_for(" help ");
    assert_eq!(screen.cursor(), None);
    assert_eq!(screen.app.cursor_shape(), 0);
}

#[test]
fn a_question_asked_from_the_sidebar_shows_no_cursor_and_keeps_the_shape() {
    let mut screen = Screen::new(shaped_agent("6"));
    screen.press('n');
    screen.wait_for("acme agent shaped");

    screen.press('x');

    screen.wait_for("End? y/n");
    assert_eq!(screen.cursor(), None);
    assert_eq!(screen.app.cursor_shape(), 6);
}

#[test]
fn the_sidebar_focused_shows_no_cursor_in_the_pane_and_keeps_the_shape() {
    let mut screen = Screen::new(shaped_agent("6"));
    screen.press('n');

    screen.wait_for("acme agent shaped");

    assert_eq!(screen.cursor(), None);
    assert_eq!(screen.app.cursor_shape(), 6);
}

#[test]
fn the_cursor_comes_back_at_the_same_cell_once_the_pane_has_focus_again() {
    let mut screen = Screen::new(shaped_agent("6"));
    let shown = focused_on_the_pane(&mut screen);
    let corner = pane_corner(&shown);
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.wait_for("┏ sessions");
    let beside = screen.cursor();

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.wait_for("┌ sessions");
    assert_eq!(beside, None);
    assert_eq!(screen.cursor(), Some(Position::new(corner + 22, 3)));
    assert_eq!(screen.app.cursor_shape(), 6);
}

#[test]
fn the_editor_shows_no_cursor_while_the_sidebar_has_focus_and_its_cursor_again_after() {
    let mut screen = Screen::editing(reading_agent(), placed_editor());
    focused_on_the_pane(&mut screen);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor placed");
    let before = screen.cursor();
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 1, 1);
    screen.mouse(MouseEventKind::Up(MouseButton::Left), 1, 1);
    screen.wait_for("┏ sessions");
    let beside = screen.cursor();

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 70, 10);
    screen.mouse(MouseEventKind::Up(MouseButton::Left), 70, 10);

    screen.wait_for("┌ sessions");
    assert!(before.is_some());
    assert_eq!(beside, None);
    assert_eq!(screen.cursor(), before);
}

#[test]
fn a_frame_with_the_sidebar_focused_never_shows_the_cursor() {
    let mut screen = Screen::new(shaped_agent("6"));
    screen.press('n');
    screen.wait_for("acme agent shaped");
    let written = Written::default();
    let mut frames = frames();
    let mut shape = Shape::default();

    frames
        .draw(&mut screen.app, &mut shape, &mut written.clone())
        .unwrap();

    let text = written.text();
    assert!(text.starts_with("\x1b[?2026h\x1b[?25l"), "{text:?}");
    assert!(!text.contains("\x1b[?25h"), "{text:?}");
}

#[test]
fn the_picker_popup_shows_its_own_cursor_and_shape() {
    let mut screen = Screen::picking(shaped_agent("6"), Picker::Command(typed_picker()));
    started_through_the_picker(&mut screen, &home());

    screen.press('n');

    screen.wait_for("acme picker");
    assert_eq!(screen.cursor(), Some(Position::new(13, 4)));
    assert_eq!(screen.app.cursor_shape(), 0);
}

#[test]
fn a_session_started_through_the_picker_shows_its_cursor_at_once() {
    let mut screen = Screen::picking(shaped_agent("6"), Picker::Command(typed_picker()));
    screen.press('n');
    screen.wait_for("acme picker");

    screen.type_text(&format!("{}\r", home().display()));

    let shown = screen.wait_for_all(&["acme agent shaped", "┌ sessions"]);
    let corner = pane_corner(&shown);
    assert_eq!(screen.cursor(), Some(Position::new(corner + 22, 3)));
    assert_eq!(screen.app.cursor_shape(), 6);
}

#[test]
fn another_session_shown_takes_its_own_shape() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(first_shaped_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");
    let second = screen.app.cursor_shape();

    screen.press('k');

    screen.wait_for("acme agent 1");
    assert_eq!(second, 0);
    assert_eq!(screen.app.cursor_shape(), 6);
}

#[test]
fn switching_sessions_hides_the_cursor_before_changing_its_shape() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(bar_then_block_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");
    let written = Written::default();
    let mut frames = frames();
    let mut shape = Shape::default();
    frames
        .draw(&mut screen.app, &mut shape, &mut written.clone())
        .unwrap();
    screen.press('k');
    screen.wait_for("acme agent 1");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.wait_for("┌ sessions");
    written.clear();

    frames
        .draw(&mut screen.app, &mut shape, &mut written.clone())
        .unwrap();

    let text = written.text();
    let hidden = text.find("\x1b[?25l").unwrap_or(usize::MAX);
    let shaped = text.find("\x1b[6 q").unwrap();
    let shown = text.rfind("\x1b[?25h").unwrap();
    assert!(hidden < shaped, "{text:?}");
    assert!(shaped < shown, "{text:?}");
}

#[test]
fn a_frame_hides_the_cursor_before_its_first_cell_and_moves_then_shows_it_in_one_write_last() {
    let mut screen = Screen::new(script(
        "printf '\\033[3;5Hacme agent plain'; exec sleep 600",
    ));
    screen.press('n');
    screen.wait_for("acme agent plain");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    let shown = screen.wait_for("┌ sessions");
    let corner = pane_corner(&shown);
    let written = Written::default();
    let mut frames = frames();
    let mut shape = Shape::default();

    frames
        .draw(&mut screen.app, &mut shape, &mut written.clone())
        .unwrap();

    let text = written.text();
    let last = format!("\x1b[4;{}H\x1b[?25h\x1b[?2026l{FLUSH}", corner + 22);
    assert!(text.starts_with("\x1b[?2026h\x1b[?25l"), "{text:?}");
    assert!(text.ends_with(&last), "{text:?}");
    assert_eq!(text.matches("\x1b[?25h").count(), 1, "{text:?}");
}

#[test]
fn an_exited_session_shows_no_cursor() {
    let mut screen = Screen::new(script(
        "printf '\\033[3;5Hacme agent done'; read line; exit 0",
    ));
    focused_on_the_pane(&mut screen);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.wait_for_all(&["exited", "acme agent done", "┌ sessions"]);

    assert_eq!(screen.cursor(), None);
}

#[test]
fn the_narrowest_sidebar_keeps_the_cursor_in_the_pane() {
    let mut screen = Screen::at(App::new(shaped_agent("6")), home(), 60);
    let shown = focused_on_the_pane(&mut screen);

    let corner = pane_corner(&shown);

    assert_eq!(corner, 20);
    assert_eq!(screen.cursor(), Some(Position::new(corner + 22, 3)));
}

#[test]
fn a_narrow_terminal_showing_only_the_sidebar_shows_no_cursor() {
    let mut screen = Screen::at(App::new(shaped_agent("6")), home(), 50);
    screen.press('n');
    screen.wait_for("acme-home 1");

    assert_eq!(screen.cursor(), None);
}

#[test]
fn a_message_over_the_bottom_of_a_narrow_pane_hides_the_cursor_under_it() {
    let missing = Program::new("acme-edit", vec!["acme-editor-missing".to_string()]);
    let mut screen = Screen::editing(
        script("printf '\\033[14;1Hacme agent low'; exec sleep 600"),
        missing,
    );
    screen.resize(50, 16);
    screen.press('n');
    screen.wait_for("acme-home 1");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.wait_for("acme agent low");
    let before = screen.cursor();

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    screen.wait_for("Could not start acme-edit");
    assert_eq!(before, Some(Position::new(15, 14)));
    assert_eq!(screen.cursor(), None);
}

#[test]
fn a_message_over_the_bottom_of_a_narrow_pane_keeps_the_cursor_above_it() {
    let missing = Program::new("acme-edit", vec!["acme-editor-missing".to_string()]);
    let mut screen = Screen::editing(
        script("printf '\\033[2;1Hacme agent high'; exec sleep 600"),
        missing,
    );
    screen.resize(50, 16);
    screen.press('n');
    screen.wait_for("acme-home 1");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.wait_for("acme agent high");

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    screen.wait_for("Could not start acme-edit");
    assert_eq!(screen.cursor(), Some(Position::new(16, 2)));
}

#[test]
fn a_wide_character_under_the_cursor_is_drawn_whole_and_unstyled() {
    let mut screen = Screen::new(script("printf 'acme 東京\\033[2D'; exec sleep 600"));
    screen.press('n');
    screen.wait_for("京");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    let shown = screen.wait_for("┌ sessions");
    let corner = pane_corner(&shown);

    let cell = screen.cell(corner + 8, 1);

    assert_eq!(screen.cursor(), Some(Position::new(corner + 8, 1)));
    assert_eq!(cell.symbol(), "京");
    assert!(!cell.modifier.contains(Modifier::REVERSED));
}

fn focused_on_the_pane(screen: &mut Screen) -> String {
    screen.press('n');
    screen.wait_for("acme agent");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.wait_for("┌ sessions")
}

fn placed_editor() -> Program {
    Program::new(
        "acme-edit",
        script("printf '\\033[3;5Hacme editor placed'; exec sleep 600"),
    )
}

fn shaped_agent(shape: &str) -> Vec<String> {
    script(&format!(
        "printf '\\033[{shape} q\\033[3;5Hacme agent shaped'; exec sleep 600"
    ))
}

fn first_shaped_agent(scratch: &Path) -> Vec<String> {
    let count = scratch.join("count");
    script(&format!(
        "n=$(($(cat '{count}' 2>/dev/null || echo 0) + 1)); echo $n > '{count}'; \
         if [ $n = 1 ]; then printf '\\033[6 q'; fi; \
         echo \"acme agent $n\"; exec sleep 600",
        count = count.display()
    ))
}

fn bar_then_block_agent(scratch: &Path) -> Vec<String> {
    let count = scratch.join("count");
    script(&format!(
        "n=$(($(cat '{count}' 2>/dev/null || echo 0) + 1)); echo $n > '{count}'; \
         if [ $n = 1 ]; then printf '\\033[6 q'; else printf '\\033[2 q'; fi; \
         echo \"acme agent $n\"; exec sleep 600",
        count = count.display()
    ))
}

#[derive(Clone, Default)]
struct Written(Rc<RefCell<Vec<u8>>>);

const FLUSH: &str = "<flush>";

impl Written {
    fn clear(&self) {
        self.0.borrow_mut().clear();
    }

    fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.borrow()).into_owned()
    }
}

impl io::Write for Written {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.borrow_mut().extend_from_slice(FLUSH.as_bytes());
        Ok(())
    }
}

fn typed_picker() -> Vec<String> {
    script("echo 'acme picker' >&2; read -r line; printf '%s\\n' \"$line\"")
}

fn started_through_the_picker(screen: &mut Screen, directory: &Path) {
    screen.press('n');
    screen.wait_for("acme picker");
    screen.type_text(&format!("{}\r", directory.display()));
    screen.wait_for("acme agent shaped");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.wait_for("┏ sessions");
}
