use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::editor::Program;
use harbour::icons::ClaudeIcon;
use ratatui::style::Modifier;

use crate::zoom_border::{Claude, started, top};
use crate::{Screen, script};

#[test]
fn the_claude_code_half_names_the_session_and_claude() {
    let mut screen = side_by_side("Fix the login");

    let top = top(&screen.draw());

    let left: String = top.chars().take(60).collect();
    assert!(
        left.starts_with("┏ ● Fix the login  acme-api · main  3m  claude ━"),
        "{left}"
    );
    assert!(left.ends_with("━┓"), "{left}");
}

#[test]
fn the_editor_half_names_the_editor_its_directory_and_the_counts() {
    let mut screen = side_by_side("Fix the login");

    let top = top(&screen.draw());

    let right: String = top.chars().skip(60).collect();
    assert!(right.starts_with("┌ acme-edit: "), "{right}");
    assert!(right.contains("/acme-api ─"), "{right}");
    assert!(right.ends_with("─ sessions ● 1 ┐"), "{right}");
}

#[test]
fn the_focused_half_names_its_program_bold_and_the_other_dim() {
    let mut screen = side_by_side("Fix the login");
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let top = top(&screen.draw());

    let claude = top
        .find("claude")
        .map(|at| u16::try_from(top[..at].chars().count()).unwrap())
        .unwrap();
    let editor = top
        .find("acme-edit")
        .map(|at| u16::try_from(top[..at].chars().count()).unwrap())
        .unwrap();
    assert!(screen.cell(claude, 0).modifier.contains(Modifier::DIM));
    assert!(screen.cell(editor, 0).modifier.contains(Modifier::BOLD));
    assert_eq!(screen.cell(60, 0).symbol(), "┏");
}

#[test]
fn a_cjk_title_on_the_claude_code_half_is_cut_by_display_width() {
    let mut screen = side_by_side("修正ログイン修正ログイン修正ログイン修正ログイン修正ログイン");

    let top = top(&screen.draw());

    assert_eq!(screen.cell(2, 0).symbol(), "●");
    assert_eq!(screen.cell(4, 0).symbol(), "修");
    assert!(top.contains("…  3m  claude ━"), "{top}");
    assert!(top.contains("claude"), "{top}");
    assert_eq!(screen.cell(59, 0).symbol(), "┓");
    assert_eq!(screen.cell(60, 0).symbol(), "┌");
}

#[test]
fn swapped_the_left_half_names_the_session_and_the_editor() {
    let mut screen = swapped("Fix the login");

    let top = top(&screen.draw());

    let left: String = top.chars().take(60).collect();
    assert!(
        left.starts_with("┌ ● Fix the login  acme-api · main  3m  acme-edit"),
        "{left}"
    );
    assert_eq!(screen.cell(59, 0).symbol(), "┐");
}

#[test]
fn swapped_the_right_half_names_claude_and_the_counts() {
    let mut screen = swapped("Fix the login");

    let top = top(&screen.draw());

    let right: String = top.chars().skip(60).collect();
    assert!(right.starts_with("┏ claude ━"), "{right}");
    assert!(right.ends_with("━ sessions ● 1 ┓"), "{right}");
}

#[test]
fn swapped_a_cjk_title_on_the_left_half_is_cut_by_display_width() {
    let mut screen = swapped("修正ログイン修正ログイン修正ログイン修正ログイン修正ログイン");

    let top = top(&screen.draw());

    assert_eq!(screen.cell(2, 0).symbol(), "●");
    assert_eq!(screen.cell(4, 0).symbol(), "修");
    assert!(top.contains("…  3m  "), "{top}");
    assert_eq!(screen.cell(59, 0).symbol(), "┐");
    assert_eq!(screen.cell(60, 0).symbol(), "┏");
}

#[test]
fn swapped_a_cjk_title_keeps_the_headline_it_has_unswapped() {
    let title = "修正ログイン修正ログイン修正ログイン修正ログイン修正ログイン";
    let unswapped = top(&side_by_side(title).draw());
    let headline: String = unswapped
        .chars()
        .skip(1)
        .take_while(|c| *c != 'c')
        .collect();

    let swapped = top(&swapped(title).draw());

    let left: String = swapped.chars().skip(1).collect();
    assert!(left.starts_with(&headline), "{headline}\n{swapped}");
}

#[test]
fn with_the_clawd_icon_the_claude_code_half_names_claude_after_its_glyph() {
    let mut screen = shown_side_by_side(
        Claude::new("Fix the login")
            .app()
            .with_nerd_font()
            .with_claude_icon(ClaudeIcon::Clawd),
    );

    let top = top(&screen.draw());

    let left: String = top.chars().take(60).collect();
    assert!(left.contains("  \u{100000}  claude ━"), "{left}");
}

#[test]
fn swapped_with_the_clawd_icon_the_right_half_names_claude_after_its_glyph() {
    let mut screen = shown_side_by_side(
        Claude::new("Fix the login")
            .app()
            .with_nerd_font()
            .with_claude_icon(ClaudeIcon::Clawd),
    );
    screen.prefixed('}');

    let top = top(&screen.draw());

    let right: String = top.chars().skip(60).collect();
    assert!(right.starts_with("┏ \u{100000}  claude ━"), "{right}");
}

#[test]
fn without_the_nerd_font_the_clawd_icon_draws_no_glyph() {
    let mut screen = shown_side_by_side(
        Claude::new("Fix the login")
            .app()
            .with_claude_icon(ClaudeIcon::Clawd),
    );

    let top = top(&screen.draw());

    let left: String = top.chars().take(60).collect();
    assert!(left.contains("  3m  claude ━"), "{left}");
}

#[test]
fn side_by_side_under_120_columns_keeps_the_focused_half_boxed() {
    let mut screen = side_by_side("Fix the login");

    screen.resize(100, 20);

    let top = top(&screen.draw());
    assert!(
        top.starts_with("┏ ● Fix the login  acme-api · main  3m  claude | acme-edit"),
        "{top}"
    );
    assert!(top.ends_with(" sessions ● 1 ┓"), "{top}");
}

fn side_by_side(title: &str) -> Screen {
    shown_side_by_side(Claude::new(title).app())
}

fn shown_side_by_side(app: App) -> Screen {
    let editor = Program::new("acme-edit", script("exec sleep 600"));
    let mut screen = Screen::of(app.with_editor(editor));
    screen.resize(120, 20);
    started(&mut screen);
    screen.prefixed('\\');
    screen.draw();
    screen
}

fn swapped(title: &str) -> Screen {
    let mut screen = side_by_side(title);
    screen.prefixed('}');
    screen
}
