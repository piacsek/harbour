use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use harbour::app::{App, UNFOCUSED_BORDER};
use harbour::editor::Program;
use ratatui::style::Modifier;

use crate::{Screen, in_order, reading_agent, script, shows_titled, sidebar_title, vertical};

#[test]
fn ctrl_l_in_claude_code_starts_the_editor_in_the_session_directory_and_shows_it() {
    let mut screen = Screen::editing(reading_agent(), reading_editor());
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor in acme-home");
    assert!(shown.contains("acme editor in acme-home"), "{shown}");
    assert!(!shown.contains("acme agent ready"), "{shown}");
}

#[test]
fn keys_typed_in_the_shown_editor_reach_it_and_not_claude_code() {
    let mut screen = Screen::editing(reading_agent(), reading_editor());
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor in");

    screen.type_text("hello\r");

    let shown = screen.wait_for("acme editor got: hello");
    assert!(shown.contains("acme editor got: hello"), "{shown}");
    assert!(!shown.contains("acme agent got"), "{shown}");
}

#[test]
fn ctrl_h_in_the_editor_shows_claude_code_again_and_keys_reach_it() {
    let mut screen = editor_open();

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent got: hi");
    assert!(shown.contains("acme agent got: hi"), "{shown}");
    assert!(!shown.contains("acme editor in"), "{shown}");
}

#[test]
fn ctrl_l_from_claude_code_shows_the_same_editor_again() {
    let mut screen = editor_open();
    screen.type_text("hello\r");
    screen.wait_for("acme editor got: hello");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.wait_for("acme agent ready");

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor got: hello");
    assert!(shown.contains("acme editor got: hello"), "{shown}");
    assert!(!shown.contains("acme agent ready"), "{shown}");
}

#[test]
fn enter_in_the_sidebar_goes_back_to_the_editor_the_pane_showed() {
    let mut screen = editor_open();
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 1, 1);
    screen.mouse(MouseEventKind::Up(MouseButton::Left), 1, 1);

    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.type_text("back\r");

    let shown = screen.wait_for("acme editor got: back");
    assert!(shown.contains("acme editor got: back"), "{shown}");
    assert!(!shown.contains("acme agent got"), "{shown}");
}

#[test]
fn ctrl_l_in_the_sidebar_goes_back_to_the_editor_the_pane_showed() {
    let mut screen = editor_open();
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 1, 1);
    screen.mouse(MouseEventKind::Up(MouseButton::Left), 1, 1);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.type_text("back\r");

    let shown = screen.wait_for("acme editor got: back");
    assert!(shown.contains("acme editor got: back"), "{shown}");
    assert!(!shown.contains("acme agent got"), "{shown}");
}

#[test]
fn the_pane_border_names_claude_code_and_the_editor_before_one_starts() {
    let mut screen = Screen::editing(reading_agent(), reading_editor());
    screen.press('n');

    let shown = screen.wait_for("acme agent ready");

    let top = shown.lines().next().unwrap();
    assert!(in_order(top, &[" claude ", " acme-edit "]), "{top}");
}

#[test]
fn the_pane_border_names_the_directory_of_the_editor_once_it_started() {
    let mut screen = editor_open();
    screen.resize(wide_enough(), 20);

    let shown = screen.draw();

    let top = shown.lines().next().unwrap();
    let editor = format!(" acme-edit: {} ", shown_directory());
    assert!(in_order(top, &[" claude ", &editor]), "{top}");
}

#[test]
fn the_pane_border_marks_claude_code_while_it_is_shown() {
    let mut screen = Screen::editing(reading_agent(), reading_editor());
    screen.press('n');
    let shown = screen.wait_for("acme agent ready");

    let (claude, editor) = tab_columns(&shown);

    assert!(screen.cell(claude, 0).modifier.contains(Modifier::BOLD));
    assert!(screen.cell(editor, 0).modifier.contains(Modifier::DIM));
}

#[test]
fn the_pane_border_marks_the_editor_while_it_is_shown() {
    let mut screen = editor_open();
    let shown = screen.draw();

    let (claude, editor) = tab_columns(&shown);

    assert!(screen.cell(claude, 0).modifier.contains(Modifier::DIM));
    assert!(screen.cell(editor, 0).modifier.contains(Modifier::BOLD));
}

#[test]
fn opening_an_editor_does_not_skip_a_number_in_the_next_session_name() {
    let mut screen = Screen::editing(reading_agent(), reading_editor());
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor in");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    screen.press('n');

    let shown = screen.wait_for("acme-home 2");
    assert!(shown.contains("acme-home 2"), "{shown}");
}

#[test]
fn e_in_the_sidebar_does_nothing_while_the_session_has_no_editor() {
    let mut screen = Screen::editing(reading_agent(), reading_editor());
    screen.press('n');
    screen.wait_for("acme agent ready");

    screen.prefixed('e');
    screen.prefixed('?');

    let shown = screen.draw();
    assert!(shows_titled(&shown, "help"), "{shown}");
    assert!(!shown.contains("acme editor in"), "{shown}");
}

#[test]
fn e_in_the_sidebar_shows_the_editor_and_focuses_it() {
    let mut screen = editor_open();
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    screen.prefixed('e');
    screen.type_text("hello\r");

    let shown = screen.wait_for("acme editor got: hello");
    assert!(shown.contains("acme editor got: hello"), "{shown}");
}

pub(crate) fn reading_editor() -> Program {
    Program::new("acme-edit", reading_editor_script())
}

#[test]
fn a_session_with_an_editor_keeps_its_card_and_counts_once() {
    let mut screen = editor_open();

    let shown = screen.draw();

    assert_eq!(sidebar_title(&shown), " sessions · 1 ", "{shown}");
    assert_eq!(shown.matches("acme-home 1").count(), 1, "{shown}");
    assert_eq!(card_text(&shown), "acme-home", "{shown}");
}

#[test]
fn without_icons_the_pane_border_is_drawn_as_before() {
    let mut screen = Screen::editing(reading_agent(), reading_editor());
    screen.press('n');

    let shown = screen.wait_for("acme agent ready");

    let top = shown.lines().next().unwrap();
    let border: String = top.chars().skip(SIDEBAR_COLUMNS).collect();
    let set = UNFOCUSED_BORDER.to_border_set();
    let expected = format!(
        "{} claude | acme-edit {}{}",
        set.top_left,
        set.horizontal_top.repeat(BORDER_FILL),
        set.top_right
    );
    assert_eq!(border, expected);
}

#[test]
fn with_icons_the_pane_border_puts_a_glyph_before_claude_and_neovim() {
    let editor = Program::neovim("nvim", reading_editor_script());
    let mut screen = Screen::of(
        App::new(reading_agent())
            .with_editor(editor)
            .with_nerd_font(),
    );
    screen.press('n');

    let shown = screen.wait_for("acme agent ready");

    let top = shown.lines().next().unwrap();
    let claude = format!(" {SPARKLE} claude ");
    let editor = format!(" {NEOVIM} nvim ");
    assert!(in_order(top, &[&claude, &editor]), "{top}");
}

#[test]
fn with_icons_an_editor_that_is_not_neovim_gets_no_glyph() {
    let mut screen = Screen::of(
        App::new(reading_agent())
            .with_editor(reading_editor())
            .with_nerd_font(),
    );
    screen.press('n');

    let shown = screen.wait_for("acme agent ready");

    let top = shown.lines().next().unwrap();
    let claude = format!(" {SPARKLE} claude ");
    assert!(in_order(top, &[&claude, " acme-edit "]), "{top}");
    assert!(!top.contains(NEOVIM), "{top}");
}

#[test]
fn with_icons_a_card_ends_with_the_neovim_glyph_while_its_editor_is_open() {
    let editor = Program::neovim("nvim", reading_editor_script());
    let mut screen = editor_open_in(
        App::new(reading_agent())
            .with_editor(editor)
            .with_nerd_font(),
    );

    let shown = screen.draw();

    let card = format!("{REPOSITORY} acme-home {NEOVIM}");
    assert_eq!(card_text(&shown), card, "{shown}");
}

#[test]
fn with_icons_a_card_shows_no_glyph_for_an_editor_that_is_not_neovim() {
    let mut screen = editor_open_in(
        App::new(reading_agent())
            .with_editor(reading_editor())
            .with_nerd_font(),
    );

    let shown = screen.draw();

    let card = format!("{REPOSITORY} acme-home");
    assert_eq!(card_text(&shown), card, "{shown}");
    assert!(!shown.contains(NEOVIM), "{shown}");
}

#[test]
fn with_icons_a_card_that_never_opened_neovim_shows_no_editor_glyph() {
    let editor = Program::neovim("nvim", reading_editor_script());
    let mut screen = Screen::of(
        App::new(reading_agent())
            .with_editor(editor)
            .with_nerd_font(),
    );
    screen.press('n');

    let shown = screen.wait_for("acme agent ready");

    let card = card_line(&shown);
    assert!(card.contains(REPOSITORY), "{card}");
    assert!(!card.contains(NEOVIM), "{card}");
}

#[test]
fn with_icons_a_card_whose_last_neovim_closed_shows_no_editor_glyph() {
    let editor = Program::neovim(
        "nvim",
        script("echo 'acme editor in place'; read -r line; exit 0"),
    );
    let mut screen = editor_open_in(
        App::new(reading_agent())
            .with_editor(editor)
            .with_nerd_font(),
    );
    screen.type_text("done\r");

    let shown = screen.wait_for("acme agent ready");

    let card = card_line(&shown);
    assert!(card.contains(REPOSITORY), "{card}");
    assert!(!card.contains(NEOVIM), "{card}");
}

#[test]
fn without_icons_a_card_with_neovim_open_is_drawn_as_before() {
    let editor = Program::neovim("nvim", reading_editor_script());
    let mut screen = editor_open_in(App::new(reading_agent()).with_editor(editor));

    let shown = screen.draw();

    assert_eq!(card_text(&shown), "acme-home", "{shown}");
}

pub(crate) fn editor_open() -> Screen {
    editor_open_in(App::new(reading_agent()).with_editor(reading_editor()))
}

fn editor_open_in(app: App) -> Screen {
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor in");
    screen
}

fn card_line(shown: &str) -> String {
    shown
        .lines()
        .nth(3)
        .unwrap()
        .chars()
        .take(SIDEBAR_COLUMNS)
        .collect()
}

fn card_text(shown: &str) -> String {
    card_line(shown)
        .trim_matches(|c: char| vertical(c) || c == '▌' || c == ' ')
        .to_string()
}

fn reading_editor_script() -> Vec<String> {
    script(
        "echo \"acme editor in ${PWD##*/}\"; \
         while :; do read -r line || continue; echo \"acme editor got: $line\"; done",
    )
}

fn shown_directory() -> String {
    let directory = crate::home();
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
    match home.and_then(|home| directory.strip_prefix(home).ok().map(ToOwned::to_owned)) {
        Some(below) => format!("~/{}", below.display()),
        None => directory.display().to_string(),
    }
}

fn wide_enough() -> u16 {
    u16::try_from(3 * (crate::home().as_os_str().len() + 40)).unwrap()
}

fn tab_columns(shown: &str) -> (u16, u16) {
    let text = shown.lines().next().unwrap();
    let claude = text.find(" claude ").unwrap();
    let editor = claude + text[claude..].find(" acme-edit").unwrap();
    let claude = text[..claude].chars().count() + 1;
    let editor = text[..editor].chars().count() + 1;
    (
        u16::try_from(claude).unwrap(),
        u16::try_from(editor).unwrap(),
    )
}

const SIDEBAR_COLUMNS: usize = 40;
const BORDER_FILL: usize = 58;
const SPARKLE: &str = "\u{F0674}";
const NEOVIM: &str = "\u{E6AE}";
const REPOSITORY: &str = "\u{F401}";
