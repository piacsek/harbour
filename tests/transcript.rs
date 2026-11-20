use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use harbour::transcript::Transcript;

const ID: &str = "0f1e2d3c-4b5a-6978-8a9b-0c1d2e3f4a5b";

#[test]
fn no_transcript_yet_knows_nothing() {
    let projects = tempfile::tempdir().unwrap();
    let mut transcript = Transcript::new(projects.path(), ID);

    transcript.read();

    assert_eq!(transcript.facts().title, None);
    assert_eq!(transcript.facts().activity, None);
}

#[test]
fn the_title_is_the_last_ai_title() {
    let projects = tempfile::tempdir().unwrap();
    let file = transcript_file(projects.path(), "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Fix the login"}"#);
    append(
        &file,
        r#"{"type":"ai-title","aiTitle":"Fix the flaky login test"}"#,
    );
    let mut transcript = Transcript::new(projects.path(), ID);

    transcript.read();

    assert_eq!(
        transcript.facts().title.as_deref(),
        Some("Fix the flaky login test")
    );
}

#[test]
fn the_last_prompt_branch_directory_and_pull_request_are_read() {
    let projects = tempfile::tempdir().unwrap();
    let file = transcript_file(projects.path(), "-home-acme-api");
    append(
        &file,
        r#"{"type":"last-prompt","lastPrompt":"retry the build"}"#,
    );
    append(
        &file,
        r#"{"type":"user","gitBranch":"fix-login","cwd":"/home/acme/api","message":{}}"#,
    );
    append(
        &file,
        r#"{"type":"pr-link","prNumber":42,"prUrl":"https://example.com/acme/api/pull/42","prRepository":"acme/api"}"#,
    );
    let mut transcript = Transcript::new(projects.path(), ID);

    transcript.read();

    assert_eq!(
        transcript.facts().prompt.as_deref(),
        Some("retry the build")
    );
    assert_eq!(transcript.facts().branch.as_deref(), Some("fix-login"));
    assert_eq!(
        transcript.facts().directory,
        Some(PathBuf::from("/home/acme/api"))
    );
    assert_eq!(transcript.facts().pull_request, Some(42));
}

#[test]
fn a_tool_call_shows_its_description_first() {
    let projects = tempfile::tempdir().unwrap();
    let file = transcript_file(projects.path(), "-home-acme-api");
    append(
        &file,
        &tool(
            "Bash",
            r#"{"description":"Run the tests","command":"cargo test"}"#,
        ),
    );
    let mut transcript = Transcript::new(projects.path(), ID);

    transcript.read();

    assert_eq!(
        transcript.facts().activity.as_deref(),
        Some("Run the tests")
    );
}

#[test]
fn a_tool_call_without_a_description_shows_its_command_then_its_file_then_its_name() {
    let projects = tempfile::tempdir().unwrap();
    let file = transcript_file(projects.path(), "-home-acme-api");
    let mut transcript = Transcript::new(projects.path(), ID);

    append(&file, &tool("Bash", r#"{"command":"cargo test"}"#));
    transcript.read();
    let command = transcript.facts().activity.clone();
    append(
        &file,
        &tool("Edit", r#"{"file_path":"/home/acme/api/src/login.rs"}"#),
    );
    transcript.read();
    let path = transcript.facts().activity.clone();
    append(&file, &tool("ToolSearch", r#"{"query":"select:Read"}"#));
    transcript.read();
    let name = transcript.facts().activity.clone();

    assert_eq!(command.as_deref(), Some("cargo test"));
    assert_eq!(path.as_deref(), Some("/home/acme/api/src/login.rs"));
    assert_eq!(name.as_deref(), Some("ToolSearch"));
}

#[test]
fn only_what_was_appended_is_read_again() {
    let projects = tempfile::tempdir().unwrap();
    let file = transcript_file(projects.path(), "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"First"}"#);
    let mut transcript = Transcript::new(projects.path(), ID);
    transcript.read();

    append(&file, r#"{"type":"last-prompt","lastPrompt":"and then"}"#);
    transcript.read();

    assert_eq!(transcript.facts().title.as_deref(), Some("First"));
    assert_eq!(transcript.facts().prompt.as_deref(), Some("and then"));
}

#[test]
fn a_line_without_its_newline_yet_waits_for_it() {
    let projects = tempfile::tempdir().unwrap();
    let file = transcript_file(projects.path(), "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Old"}"#);
    write_raw(&file, r#"{"type":"ai-title","#);
    let mut transcript = Transcript::new(projects.path(), ID);
    transcript.read();
    let before = transcript.facts().title.clone();

    write_raw(&file, "\"aiTitle\":\"New\"}\n");
    transcript.read();

    assert_eq!(before.as_deref(), Some("Old"));
    assert_eq!(transcript.facts().title.as_deref(), Some("New"));
}

#[test]
fn a_bad_line_or_a_field_of_the_wrong_type_keeps_the_last_good_value() {
    let projects = tempfile::tempdir().unwrap();
    let file = transcript_file(projects.path(), "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Good"}"#);
    append(&file, r#"{"type":"pr-link","prNumber":7}"#);
    append(&file, "not json at all");
    append(&file, r#"{"type":"ai-title","aiTitle":12}"#);
    append(&file, r#"{"type":"pr-link","prNumber":"seven"}"#);
    append(
        &file,
        r#"{"type":"assistant","message":{"content":"text"}}"#,
    );
    append(
        &file,
        r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash","input":[]}]}}"#,
    );
    let mut transcript = Transcript::new(projects.path(), ID);

    transcript.read();

    assert_eq!(transcript.facts().title.as_deref(), Some("Good"));
    assert_eq!(transcript.facts().pull_request, Some(7));
    assert_eq!(transcript.facts().activity.as_deref(), Some("Bash"));
}

#[test]
fn a_huge_transcript_is_read_from_its_last_mebibyte() {
    let projects = tempfile::tempdir().unwrap();
    let file = transcript_file(projects.path(), "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Too far back"}"#);
    let filler = format!(r#"{{"type":"progress","data":"{}"}}"#, "x".repeat(1000));
    for _ in 0..1100 {
        append(&file, &filler);
    }
    append(
        &file,
        r#"{"type":"last-prompt","lastPrompt":"near the end"}"#,
    );
    let mut transcript = Transcript::new(projects.path(), ID);

    transcript.read();

    assert_eq!(transcript.facts().title, None);
    assert_eq!(transcript.facts().prompt.as_deref(), Some("near the end"));
}

#[test]
fn a_line_longer_than_a_mebibyte_is_skipped() {
    let projects = tempfile::tempdir().unwrap();
    let file = transcript_file(projects.path(), "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Kept"}"#);
    let mut transcript = Transcript::new(projects.path(), ID);
    transcript.read();

    write_raw(&file, r#"{"type":"ai-title","aiTitle":""#);
    write_raw(&file, &"y".repeat(600 * 1024));
    transcript.read();
    write_raw(&file, &"y".repeat(600 * 1024));
    transcript.read();
    write_raw(&file, "\"}\n");
    append(&file, r#"{"type":"last-prompt","lastPrompt":"after it"}"#);
    transcript.read();

    assert_eq!(transcript.facts().title.as_deref(), Some("Kept"));
    assert_eq!(transcript.facts().prompt.as_deref(), Some("after it"));
}

#[test]
fn a_transcript_that_shrinks_is_read_again_from_its_tail() {
    let projects = tempfile::tempdir().unwrap();
    let file = transcript_file(projects.path(), "-home-acme-api");
    append(
        &file,
        r#"{"type":"ai-title","aiTitle":"Before the rewrite, a long title"}"#,
    );
    let mut transcript = Transcript::new(projects.path(), ID);
    transcript.read();

    fs::write(&file, "{\"type\":\"ai-title\",\"aiTitle\":\"After\"}\n").unwrap();
    transcript.read();

    assert_eq!(transcript.facts().title.as_deref(), Some("After"));
}

#[test]
fn another_sessions_transcript_is_never_read() {
    let projects = tempfile::tempdir().unwrap();
    let folder = projects.path().join("-home-acme-api");
    fs::create_dir_all(&folder).unwrap();
    append(
        &folder.join("9a8b7c6d-5e4f-4a3b-2c1d-0e9f8a7b6c5d.jsonl"),
        r#"{"type":"ai-title","aiTitle":"Someone else"}"#,
    );
    let mut transcript = Transcript::new(projects.path(), ID);

    transcript.read();

    assert_eq!(transcript.facts().title, None);
}

#[test]
fn the_subagents_still_running_are_counted() {
    let projects = tempfile::tempdir().unwrap();
    let file = transcript_file(projects.path(), "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many hands"}"#);
    subagent(&file, "agent-a1", r#"{"agentType":"general-purpose"}"#);
    subagent(
        &file,
        "agent-a2",
        r#"{"agentType":"Explore","finished":false}"#,
    );
    subagent(
        &file,
        "agent-a3",
        r#"{"agentType":"Explore","finished":true}"#,
    );
    let mut transcript = Transcript::new(projects.path(), ID);

    transcript.read();

    assert_eq!(transcript.facts().subagents, 2);
}

#[test]
fn a_subagent_that_finishes_is_no_longer_counted() {
    let projects = tempfile::tempdir().unwrap();
    let file = transcript_file(projects.path(), "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many hands"}"#);
    subagent(&file, "agent-a1", r#"{"agentType":"general-purpose"}"#);
    let mut transcript = Transcript::new(projects.path(), ID);
    transcript.read();
    let before = transcript.facts().subagents;

    subagent(
        &file,
        "agent-a1",
        r#"{"agentType":"general-purpose","finished":true}"#,
    );
    transcript.read();

    assert_eq!(before, 1);
    assert_eq!(transcript.facts().subagents, 0);
}

#[test]
fn a_subagent_file_older_than_the_session_or_not_an_object_is_not_counted() {
    let projects = tempfile::tempdir().unwrap();
    let file = transcript_file(projects.path(), "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many hands"}"#);
    subagent(&file, "agent-a1", r#"{"agentType":"general-purpose"}"#);
    subagent(&file, "agent-a2", "[1, 2]");
    subagent(&file, "agent-a3", "not json");
    let old = subagent(&file, "agent-a4", r#"{"agentType":"Explore"}"#);
    fs::File::options()
        .write(true)
        .open(&old)
        .unwrap()
        .set_modified(SystemTime::now() - Duration::from_secs(3600))
        .unwrap();
    let mut transcript =
        Transcript::new(projects.path(), ID).since(SystemTime::now() - Duration::from_secs(60));

    transcript.read();

    assert_eq!(transcript.facts().subagents, 1);
}

fn transcript_file(projects: &Path, folder: &str) -> PathBuf {
    let folder = projects.join(folder);
    fs::create_dir_all(&folder).unwrap();
    folder.join(format!("{ID}.jsonl"))
}

fn append(file: &Path, line: &str) {
    write_raw(file, &format!("{line}\n"));
}

fn write_raw(file: &Path, text: &str) {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)
        .unwrap()
        .write_all(text.as_bytes())
        .unwrap();
}

fn tool(name: &str, input: &str) -> String {
    format!(
        r#"{{"type":"assistant","message":{{"content":[{{"type":"tool_use","name":"{name}","input":{input}}}]}}}}"#
    )
}

fn subagent(transcript: &Path, name: &str, meta: &str) -> PathBuf {
    let folder = transcript.with_extension("").join("subagents");
    fs::create_dir_all(&folder).unwrap();
    let file = folder.join(format!("{name}.meta.json"));
    fs::write(&file, meta).unwrap();
    file
}
