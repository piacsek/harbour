use std::fs;
use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::editor::Program;
use harbour::picker::Picker;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

use crate::{Screen, reading_agent, script};

#[test]
fn the_editor_opens_at_the_root_of_the_repository_claude_code_started_in() {
    let scratch = tempfile::tempdir().unwrap();
    let start = folder(scratch.path(), "acme-api/src");
    fs::create_dir(scratch.path().join("acme-api/.git")).unwrap();
    let mut screen = started_in(&start, reading_agent());

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor at");
    assert!(shown.contains("acme editor at acme-api"), "{shown}");
}

#[test]
fn a_worktree_root_is_found_by_its_git_file() {
    let scratch = tempfile::tempdir().unwrap();
    let start = folder(scratch.path(), "acme-wt/lib/deep");
    fs::write(scratch.path().join("acme-wt/.git"), "gitdir: /acme/.git\n").unwrap();
    let mut screen = started_in(&start, reading_agent());

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor at");
    assert!(shown.contains("acme editor at acme-wt"), "{shown}");
}

#[test]
fn outside_a_repository_the_editor_opens_in_claude_codes_directory() {
    let scratch = tempfile::tempdir().unwrap();
    let start = folder(scratch.path(), "acme-plain/notes");
    let mut screen = started_in(&start, reading_agent());

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor at");
    assert!(shown.contains("acme editor at notes"), "{shown}");
}

#[test]
fn claude_code_is_given_a_session_id_when_harbour_reads_its_transcripts() {
    let scratch = tempfile::tempdir().unwrap();

    let _screen = following(scratch.path(), &[]);

    let args = fs::read_to_string(scratch.path().join("args")).unwrap();
    let (flag, id) = args.split_once(' ').unwrap();
    assert_eq!(flag, "--session-id");
    assert_eq!(id.len(), 36);
    assert_eq!(id.chars().nth(14), Some('4'));
}

#[test]
fn two_sessions_get_different_session_ids() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = following(scratch.path(), &[]);
    let first = fs::read_to_string(scratch.path().join("args")).unwrap();
    fs::remove_file(scratch.path().join("args")).unwrap();
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    screen.press('n');
    screen.wait_for_all(&["acme-start 2", "acme agent ready"]);

    let second = fs::read_to_string(scratch.path().join("args")).unwrap();
    assert_ne!(first, second);
}

#[test]
fn claude_code_gets_no_session_id_when_harbour_does_not_read_its_transcripts() {
    let scratch = tempfile::tempdir().unwrap();
    let start = folder(scratch.path(), "acme-api");

    let _screen = started_in(&start, recording_agent(scratch.path()));

    let args = fs::read_to_string(scratch.path().join("args")).unwrap();
    assert!(!args.contains("--session-id"), "{args}");
}

#[test]
fn the_editor_opens_at_the_root_of_the_directory_the_transcript_names_last() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let two = repository(scratch.path(), "acme-two");
    let mut screen = following(
        scratch.path(),
        &[
            &cwd(&one),
            &cwd(&folder(&two, "src")),
            "{\"type\":\"queue-operation\"}",
        ],
    );

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor at");
    assert!(shown.contains("acme editor at acme-two"), "{shown}");
}

#[test]
fn a_line_that_is_not_json_is_skipped() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let mut screen = following(scratch.path(), &[&cwd(&one), "{\"cwd\": \"/acme/half"]);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor at");
    assert!(shown.contains("acme editor at acme-one"), "{shown}");
}

#[test]
fn without_a_transcript_the_editor_opens_at_the_root_of_the_start_directory() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = following(scratch.path(), &[]);
    fs::remove_dir_all(scratch.path().join("projects")).unwrap();

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor at");
    assert!(shown.contains("acme editor at acme-start"), "{shown}");
}

#[test]
fn a_directory_named_further_back_than_the_last_four_mebibytes_is_not_read() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let filler = format!("{{\"message\":\"{}\"}}", "a".repeat(1024));
    let mut lines = vec![cwd(&one)];
    lines.extend(std::iter::repeat_n(filler, 4 * 1024));
    let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
    let mut screen = following(scratch.path(), &lines);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor at");
    assert!(shown.contains("acme editor at acme-start"), "{shown}");
}

#[test]
fn claude_code_in_another_repository_gets_a_second_editor_there() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let two = repository(scratch.path(), "acme-two");
    let mut screen = following(scratch.path(), &[&cwd(&one)]);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor at acme-one");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    write_transcript(scratch.path(), &[&cwd(&two)]);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor at");
    assert!(shown.contains("acme editor at acme-two"), "{shown}");
}

#[test]
fn moving_back_shows_the_first_editor_still_running() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let two = repository(scratch.path(), "acme-two");
    let mut screen = following(scratch.path(), &[&cwd(&one)]);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor at acme-one");
    screen.type_text("kept\r");
    screen.wait_for("acme editor got: kept");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    write_transcript(scratch.path(), &[&cwd(&two)]);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor at acme-two");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    write_transcript(scratch.path(), &[&cwd(&one)]);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor got: kept");
    assert!(shown.contains("acme editor got: kept"), "{shown}");
}

#[test]
fn a_directory_that_is_gone_is_named_and_nothing_walks_up_to_its_repository() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let gone = one.join("gone");
    let mut screen = following(scratch.path(), &[&cwd(&gone)]);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("Not a directory");
    assert!(shown.contains("Not a directory"), "{shown}");
    assert!(shown.contains("acme agent ready"), "{shown}");
    assert!(!shown.contains("acme editor at"), "{shown}");
}

fn started_in(directory: &Path, agent: Vec<String>) -> Screen {
    let mut screen = Screen {
        app: App::new(agent)
            .with_editor(locating_editor())
            .with_picker(choosing(directory)),
        terminal: Terminal::new(TestBackend::new(200, 20)).unwrap(),
    };
    screen.draw();
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen
}

fn choosing(directory: &Path) -> Picker {
    Picker::Command(script(&format!("printf '%s\\n' '{}'", directory.display())))
}

fn locating_editor() -> Program {
    Program::new(
        "acme-edit",
        script(
            "echo \"acme editor at $(basename \"$(pwd -P)\")\"; \
             while read -r line; do echo \"acme editor got: $line\"; done",
        ),
    )
}

fn folder(scratch: &Path, path: &str) -> PathBuf {
    let folder = scratch.join(path);
    fs::create_dir_all(&folder).unwrap();
    folder
}

fn following(scratch: &Path, lines: &[&str]) -> Screen {
    let start = repository(scratch, "acme-start");
    let mut screen = Screen {
        app: App::new(recording_agent(scratch))
            .with_editor(locating_editor())
            .with_picker(choosing(&start))
            .with_transcripts(scratch.join("projects")),
        terminal: Terminal::new(TestBackend::new(200, 20)).unwrap(),
    };
    screen.draw();
    screen.press('n');
    screen.wait_for("acme agent ready");
    write_transcript(scratch, lines);
    screen
}

fn recording_agent(scratch: &Path) -> Vec<String> {
    script(&format!(
        "printf '%s %s' \"$0\" \"$1\" > '{}/args'; echo 'acme agent ready'; exec sleep 600",
        scratch.display()
    ))
}

fn write_transcript(scratch: &Path, lines: &[&str]) {
    let args = fs::read_to_string(scratch.join("args")).unwrap();
    let id = args.split_once(' ').unwrap().1;
    let project = folder(scratch, "projects/-acme-start");
    let mut text = lines.join("\n");
    text.push('\n');
    fs::write(project.join(format!("{id}.jsonl")), text).unwrap();
}

fn cwd(directory: &Path) -> String {
    format!("{{\"type\":\"user\",\"cwd\":\"{}\"}}", directory.display())
}

fn repository(scratch: &Path, name: &str) -> PathBuf {
    let repository = folder(scratch, name);
    fs::create_dir_all(repository.join(".git")).unwrap();
    repository
}
