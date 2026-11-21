use crossterm::event::{KeyCode, KeyModifiers};
use harbour::editor::Program;
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
    assert!(right.starts_with("┌ acme-edit: …/"), "{right}");
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

fn side_by_side(title: &str) -> Screen {
    let claude = Claude::new(title);
    let editor = Program::new("acme-edit", script("exec sleep 600"));
    let mut screen = Screen::of(claude.app().with_editor(editor));
    screen.resize(120, 20);
    started(&mut screen);
    screen.key(KeyCode::Char('\\'), KeyModifiers::ALT);
    screen.draw();
    screen
}
