use std::fs;
use std::io::Write;
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
    let git_dir = folder(scratch.path(), "acme/.git/worktrees/acme-wt");
    fs::write(
        scratch.path().join("acme-wt/.git"),
        format!("gitdir: {}\n", git_dir.display()),
    )
    .unwrap();
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
fn claude_code_gets_no_session_id_from_a_harbour_that_reads_transcripts_but_does_not_name_sessions()
{
    let scratch = tempfile::tempdir().unwrap();
    let start = folder(scratch.path(), "acme-api");
    let mut screen = Screen {
        app: App::new(recording_agent(scratch.path()))
            .with_picker(choosing(&start))
            .with_transcripts(scratch.path().join("projects")),
        terminal: Terminal::new(TestBackend::new(200, 20)).unwrap(),
    };
    screen.draw();

    screen.press('n');

    screen.wait_for("acme agent ready");
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
fn a_new_session_id_without_a_transcript_keeps_the_directory_claude_code_was_in() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let start = repository(scratch.path(), "acme-start");
    let mut screen = following(scratch.path(), &[&cwd(&one)]);
    write_registry(scratch.path(), "acme-cleared", &start);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor at");
    assert!(shown.contains("acme editor at acme-one"), "{shown}");
}

#[test]
fn a_second_new_session_id_without_a_transcript_keeps_the_directory_found_last() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let two = repository(scratch.path(), "acme-two");
    let start = repository(scratch.path(), "acme-start");
    let mut screen = following(scratch.path(), &[&cwd(&one)]);
    write_registry(scratch.path(), "acme-first", &start);
    write_named_transcript(scratch.path(), "acme-first", &[&cwd(&two)]);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor at acme-two");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    write_registry(scratch.path(), "acme-second", &start);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor at");
    assert!(shown.contains("acme editor at acme-two"), "{shown}");
    assert!(!shown.contains("acme editor at acme-one"), "{shown}");
    assert!(!shown.contains("acme editor at acme-start"), "{shown}");
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

#[test]
fn after_claude_code_changes_its_session_the_editor_follows_the_transcript_the_registry_names() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let two = repository(scratch.path(), "acme-two");
    let mut screen = following(scratch.path(), &[&cwd(&one)]);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor at acme-one");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    write_registry(scratch.path(), "acme-cleared", &one);
    write_named_transcript(scratch.path(), "acme-cleared", &[&cwd(&two)]);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor at acme-two");
    assert!(shown.contains("acme editor at acme-two"), "{shown}");
}

#[test]
fn the_registry_names_the_directory_while_its_session_has_no_transcript() {
    let scratch = tempfile::tempdir().unwrap();
    let two = repository(scratch.path(), "acme-two");
    let mut screen = following(scratch.path(), &[]);
    write_registry(scratch.path(), "acme-fresh", &folder(&two, "src"));

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor at");
    assert!(shown.contains("acme editor at acme-two"), "{shown}");
}

#[test]
fn a_session_id_that_is_not_a_plain_name_is_not_looked_up() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let two = repository(scratch.path(), "acme-two");
    let mut screen = following(scratch.path(), &[]);
    write_registry(scratch.path(), "../acme-elsewhere", &one);
    write_named_transcript(scratch.path(), "../acme-elsewhere", &[&cwd(&two)]);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor at");
    assert!(shown.contains("acme editor at acme-one"), "{shown}");
}

#[test]
fn after_claude_code_exits_the_editor_stays_at_the_directory_last_seen() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let two = repository(scratch.path(), "acme-two");
    let start = repository(scratch.path(), "acme-start");
    let mut screen = following(scratch.path(), &[&cwd(&one)]);
    write_registry(scratch.path(), "acme-resumed", &start);
    write_named_transcript(scratch.path(), "acme-resumed", &[&cwd(&two)]);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor at acme-two");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    end_agent(scratch.path());
    screen.wait_for("exited");

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor at");
    assert!(shown.contains("acme editor at acme-two"), "{shown}");
    assert!(!shown.contains("acme editor at acme-one"), "{shown}");
}

#[test]
fn a_transcript_that_disappears_leaves_the_directory_last_seen() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let two = repository(scratch.path(), "acme-two");
    let start = repository(scratch.path(), "acme-start");
    let mut screen = following(scratch.path(), &[&cwd(&one)]);
    write_registry(scratch.path(), "acme-resumed", &start);
    write_named_transcript(scratch.path(), "acme-resumed", &[&cwd(&two)]);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor at acme-two");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    fs::remove_file(
        scratch
            .path()
            .join("projects/-acme-start/acme-resumed.jsonl"),
    )
    .unwrap();

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor at");
    assert!(shown.contains("acme editor at acme-two"), "{shown}");
    assert!(!shown.contains("acme editor at acme-one"), "{shown}");
    assert!(!shown.contains("acme editor at acme-start"), "{shown}");
}

#[test]
fn the_registry_directory_is_not_used_once_a_transcript_named_one() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let two = repository(scratch.path(), "acme-two");
    let mut screen = following(scratch.path(), &[&cwd(&one)]);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor at acme-one");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    write_registry(scratch.path(), "acme-fresh", &two);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor at");
    assert!(shown.contains("acme editor at acme-one"), "{shown}");
    assert!(!shown.contains("acme editor at acme-two"), "{shown}");
}

#[test]
fn e_in_the_sidebar_shows_the_editor_of_a_directory_claude_code_left() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let two = repository(scratch.path(), "acme-two");
    let start = repository(scratch.path(), "acme-start");
    let mut screen = following(scratch.path(), &[&cwd(&one)]);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor at acme-one");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    write_registry(scratch.path(), "acme-resumed", &start);
    write_named_transcript(scratch.path(), "acme-resumed", &[&cwd(&two)]);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor at acme-two");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    screen.press('e');

    let shown = screen.wait_for("acme editor at acme-one");
    assert!(shown.contains("acme editor at acme-one"), "{shown}");
    assert!(!shown.contains("acme editor at acme-two"), "{shown}");
}

#[test]
fn e_in_the_sidebar_comes_back_round_to_the_first_editor() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let two = repository(scratch.path(), "acme-two");
    let start = repository(scratch.path(), "acme-start");
    let mut screen = following(scratch.path(), &[&cwd(&one)]);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor at acme-one");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    write_registry(scratch.path(), "acme-resumed", &start);
    write_named_transcript(scratch.path(), "acme-resumed", &[&cwd(&two)]);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor at acme-two");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.press('e');
    screen.wait_for("acme editor at acme-one");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    screen.press('e');

    let shown = screen.wait_for("acme editor at acme-two");
    assert!(shown.contains("acme editor at acme-two"), "{shown}");
    assert!(!shown.contains("acme editor at acme-one"), "{shown}");
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

pub(crate) fn folder(scratch: &Path, path: &str) -> PathBuf {
    let folder = scratch.join(path);
    fs::create_dir_all(&folder).unwrap();
    folder
}

pub(crate) fn following(scratch: &Path, lines: &[&str]) -> Screen {
    let start = repository(scratch, "acme-start");
    let mut screen = Screen {
        app: App::new(recording_agent(scratch))
            .with_editor(locating_editor())
            .with_picker(choosing(&start))
            .with_transcripts(scratch.join("projects"))
            .naming_sessions()
            .with_registry(folder(scratch, "sessions")),
        terminal: Terminal::new(TestBackend::new(200, 20)).unwrap(),
    };
    screen.draw();
    screen.press('n');
    let shown = screen.wait_for("acme agent ready");
    assert!(shown.contains("acme agent ready"), "{shown}");
    write_transcript(scratch, lines);
    screen
}

fn recording_agent(scratch: &Path) -> Vec<String> {
    script(&format!(
        "printf '%s %s' \"$0\" \"$1\" > '{scratch}/args'; echo $$ > '{scratch}/pid'; \
         echo 'acme agent ready'; exec sleep 600",
        scratch = scratch.display()
    ))
}

pub(crate) fn write_transcript(scratch: &Path, lines: &[&str]) {
    let args = fs::read_to_string(scratch.join("args")).unwrap();
    let id = args.split_once(' ').unwrap().1;
    write_named_transcript(scratch, id, lines);
}

pub(crate) fn write_named_transcript(scratch: &Path, id: &str, lines: &[&str]) {
    let project = folder(scratch, "projects/-acme-start");
    let mut text = lines.join("\n");
    text.push('\n');
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(project.join(format!("{id}.jsonl")))
        .unwrap()
        .write_all(text.as_bytes())
        .unwrap();
}

pub(crate) fn cwd(directory: &Path) -> String {
    format!("{{\"type\":\"user\",\"cwd\":\"{}\"}}", directory.display())
}

pub(crate) fn repository(scratch: &Path, name: &str) -> PathBuf {
    let repository = folder(scratch, name);
    fs::create_dir_all(repository.join(".git")).unwrap();
    repository
}

pub(crate) fn write_registry(scratch: &Path, id: &str, directory: &Path) {
    let pid = fs::read_to_string(scratch.join("pid")).unwrap();
    let record = format!(
        "{{\"pid\":{},\"sessionId\":\"{id}\",\"cwd\":\"{}\",\"status\":\"idle\"}}",
        pid.trim(),
        directory.display()
    );
    fs::write(
        scratch.join(format!("sessions/{}.json", pid.trim())),
        record,
    )
    .unwrap();
}

pub(crate) fn end_agent(scratch: &Path) {
    let pid = fs::read_to_string(scratch.join("pid")).unwrap();
    let status = std::process::Command::new("kill")
        .arg(pid.trim())
        .status()
        .unwrap();
    assert!(status.success());
}
