use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::editor::Program;
use ratatui::style::Modifier;

use crate::status::showing;
use crate::{Screen, below, column_of, script};

const ID: &str = "0f1e2d3c-4b5a-6978-8a9b-0c1d2e3f4a5b";
const OTHER_ID: &str = "9a8b7c6d-5e4f-4a3b-2c1d-0e9f8a7b6c5d";

#[test]
fn an_unselected_working_session_shows_its_last_prompt_dimmed_not_its_last_tool_call() {
    let claude = Claude::new();
    claude.transcript(
        ID,
        r#"{"type":"ai-title","aiTitle":"Fix the flaky login test"}"#,
    );
    claude.transcript(
        ID,
        r#"{"type":"last-prompt","lastPrompt":"fix the login test"}"#,
    );
    claude.transcript(ID, &tool(r#"{"description":"Run the login tests"}"#));
    let mut screen = claude.screen("busy", ID);
    screen.press('n');
    screen.press('n');

    let shown = screen.wait_for("fix the login test");
    let prompt = column_of(&shown, 3, "fix the login test");

    assert_eq!(
        below(&shown, "Fix the flaky login test"),
        "fix the login test"
    );
    assert!(!shown.contains("Run the login tests"), "{shown}");
    assert!(screen.cell(prompt, 3).modifier.contains(Modifier::DIM));
}

#[test]
fn the_selected_working_session_shows_no_second_line() {
    let claude = Claude::new();
    claude.transcript(
        ID,
        r#"{"type":"ai-title","aiTitle":"Fix the flaky login test"}"#,
    );
    claude.transcript(
        ID,
        r#"{"type":"last-prompt","lastPrompt":"fix the login test"}"#,
    );
    let mut screen = claude.screen("busy", ID);
    screen.press('n');

    let shown = screen.wait_for("Fix the flaky login test");

    assert_eq!(below(&shown, "Fix the flaky login test"), "");
}

#[test]
fn an_unselected_idle_session_shows_its_last_prompt_dimmed() {
    let claude = Claude::new();
    claude.transcript(
        ID,
        r#"{"type":"ai-title","aiTitle":"Tidy the billing module"}"#,
    );
    claude.transcript(ID, &tool(r#"{"description":"Run the billing tests"}"#));
    claude.transcript(
        ID,
        r#"{"type":"last-prompt","lastPrompt":"split the invoice\nhelper"}"#,
    );
    let mut screen = claude.screen("idle", ID);
    screen.press('n');
    screen.press('n');

    let shown = screen.wait_for("split the invoice helper");
    let prompt = column_of(&shown, 3, "split the invoice helper");

    assert_eq!(
        below(&shown, "Tidy the billing module"),
        "split the invoice helper"
    );
    assert!(screen.cell(prompt, 3).modifier.contains(Modifier::DIM));
}

#[test]
fn a_blocked_session_shows_what_it_waits_for_not_its_last_tool_call() {
    let claude = Claude::new();
    claude.transcript(ID, r#"{"type":"ai-title","aiTitle":"Deploy acme-api"}"#);
    claude.transcript(ID, &tool(r#"{"command":"make deploy"}"#));
    let mut screen = claude.screen_with(&format!(
        r#"{{"status":"waiting","waitingFor":"permission prompt","sessionId":"{ID}"}}"#
    ));
    screen.press('n');

    let shown = screen.wait_for("Deploy acme-api");

    assert_eq!(below(&shown, "Deploy acme-api"), "permission prompt");
}

#[test]
fn the_selected_idle_session_shows_no_second_line() {
    let claude = Claude::new();
    claude.transcript(
        ID,
        r#"{"type":"ai-title","aiTitle":"Tidy the billing module"}"#,
    );
    claude.transcript(
        ID,
        r#"{"type":"last-prompt","lastPrompt":"split the invoice helper"}"#,
    );
    let mut screen = claude.screen("idle", ID);
    screen.press('n');

    let shown = screen.wait_for("Tidy the billing module");

    assert_eq!(below(&shown, "Tidy the billing module"), "");
}

#[test]
fn the_narrow_layout_shows_what_the_selected_blocked_session_waits_for() {
    let claude = Claude::new();
    claude.transcript(ID, r#"{"type":"ai-title","aiTitle":"Deploy acme-api"}"#);
    let mut screen = claude.screen_with(&blocked());
    screen.press('n');
    screen.wait_for("permission prompt");

    screen.resize(50, 20);
    let shown = screen.draw();

    assert_eq!(below(&shown, "Deploy acme-api"), "permission prompt");
}

#[test]
fn the_narrow_layout_shows_no_second_line_on_the_selected_working_session() {
    let claude = Claude::new();
    claude.transcript(
        ID,
        r#"{"type":"ai-title","aiTitle":"Fix the flaky login test"}"#,
    );
    claude.transcript(
        ID,
        r#"{"type":"last-prompt","lastPrompt":"fix the login test"}"#,
    );
    let mut screen = claude.screen("busy", ID);
    screen.press('n');
    screen.wait_for("Fix the flaky login test");

    screen.resize(50, 20);
    let shown = screen.draw();

    assert_eq!(below(&shown, "Fix the flaky login test"), "");
}

#[test]
fn the_narrow_layout_shows_no_second_line_on_the_selected_idle_session() {
    let claude = Claude::new();
    claude.transcript(
        ID,
        r#"{"type":"ai-title","aiTitle":"Tidy the billing module"}"#,
    );
    claude.transcript(
        ID,
        r#"{"type":"last-prompt","lastPrompt":"split the invoice helper"}"#,
    );
    let mut screen = claude.screen("idle", ID);
    screen.press('n');
    screen.wait_for("Tidy the billing module");

    screen.resize(50, 20);
    let shown = screen.draw();

    assert_eq!(below(&shown, "Tidy the billing module"), "");
}

#[test]
fn the_selected_blocked_session_shows_what_it_waits_for_beside_its_editor() {
    let claude = Claude::new();
    claude.transcript(ID, r#"{"type":"ai-title","aiTitle":"Deploy acme-api"}"#);
    let mut screen = claude.editing(&blocked());

    let shown = screen.wait_for_all(&["acme editor ready", "permission prompt"]);

    assert_eq!(below(&shown, "Deploy acme-api"), "permission prompt");
}

#[test]
fn the_selected_working_session_shows_no_second_line_beside_its_editor() {
    let claude = Claude::new();
    claude.transcript(
        ID,
        r#"{"type":"ai-title","aiTitle":"Fix the flaky login test"}"#,
    );
    claude.transcript(
        ID,
        r#"{"type":"last-prompt","lastPrompt":"fix the login test"}"#,
    );
    let mut screen = claude.editing(&format!(r#"{{"status":"busy","sessionId":"{ID}"}}"#));

    let shown = screen.wait_for_all(&["acme editor ready", "Fix the flaky login test"]);

    assert_eq!(below(&shown, "Fix the flaky login test"), "");
}

#[test]
fn the_third_line_takes_the_transcripts_directory_branch_and_pull_request() {
    let claude = Claude::new();
    let repository = claude.repository("acme-web", "main");
    claude.transcript(ID, r#"{"type":"ai-title","aiTitle":"Ship the web login"}"#);
    claude.transcript(
        ID,
        &format!(
            r#"{{"type":"user","gitBranch":"fix-login","cwd":"{}","message":{{}}}}"#,
            repository.display()
        ),
    );
    claude.transcript(ID, r#"{"type":"pr-link","prNumber":42}"#);
    let mut screen = claude.screen("busy", ID);
    screen.press('n');

    let shown = screen.wait_for("acme-web · fix-login · #42");

    assert!(shown.contains("acme-web · fix-login · #42"), "{shown}");
}

#[test]
fn a_new_session_id_forgets_the_old_transcript() {
    let claude = Claude::new();
    claude.transcript(
        ID,
        r#"{"type":"ai-title","aiTitle":"The first conversation"}"#,
    );
    claude.transcript(
        OTHER_ID,
        r#"{"type":"ai-title","aiTitle":"The second one"}"#,
    );
    let mut screen = claude.screen_with(&format!(r#"{{"status":"idle","sessionId":"{ID}"}}"#));
    screen.press('n');
    let before = screen.wait_for("The first conversation");
    assert!(before.contains("The first conversation"), "{before}");

    claude.register(&format!(r#"{{"status":"idle","sessionId":"{OTHER_ID}"}}"#));

    let shown = screen.wait_for("The second one");
    assert!(!shown.contains("The first conversation"), "{shown}");
}

#[test]
fn a_title_appended_later_replaces_the_one_shown() {
    let claude = Claude::new();
    claude.transcript(ID, r#"{"type":"ai-title","aiTitle":"A first guess"}"#);
    let mut screen = claude.screen("busy", ID);
    screen.press('n');
    let before = screen.wait_for("A first guess");
    assert!(before.contains("A first guess"), "{before}");

    claude.transcript(ID, r#"{"type":"ai-title","aiTitle":"A better title"}"#);

    let shown = screen.wait_for("A better title");
    assert!(!shown.contains("A first guess"), "{shown}");
}

#[test]
fn an_exited_session_keeps_its_title() {
    let claude = Claude::new();
    claude.transcript(ID, r#"{"type":"ai-title","aiTitle":"Short lived"}"#);
    let mut screen = claude.screen_running(&format!(
        "printf '%s' '{{\"status\":\"busy\",\"sessionId\":\"{ID}\"}}' > '{}'/$$.json; sleep 1; exit 3",
        claude.sessions().display()
    ));
    screen.press('n');
    let before = screen.wait_for("Short lived");
    assert!(before.contains("Short lived"), "{before}");

    let shown = screen.wait_for("exited 3");
    sleep(Duration::from_millis(600));
    let again = screen.draw();

    assert!(shown.contains("Short lived"), "{shown}");
    assert!(again.contains("Short lived"), "{again}");
}

#[test]
fn the_third_line_counts_the_subagents_still_running() {
    let claude = Claude::new();
    let repository = claude.repository("acme-api", "main");
    claude.transcript(
        ID,
        &format!(
            r#"{{"type":"user","cwd":"{}","message":{{}}}}"#,
            repository.display()
        ),
    );
    claude.subagent(ID, "agent-a1", r#"{"agentType":"Explore"}"#);
    let mut screen = claude.screen("busy", ID);
    screen.press('n');
    let before = screen.wait_for("acme-api · main · 1 subagent");
    assert!(before.contains("acme-api · main · 1 subagent"), "{before}");

    claude.subagent(ID, "agent-a2", r#"{"agentType":"Plan"}"#);

    let shown = screen.wait_for("acme-api · main · 2 subagents");
    assert!(shown.contains("acme-api · main · 2 subagents"), "{shown}");
}

#[test]
fn an_exited_session_counts_no_subagent() {
    let claude = Claude::new();
    claude.transcript(ID, r#"{"type":"ai-title","aiTitle":"Short lived"}"#);
    claude.subagent(ID, "agent-a1", r#"{"agentType":"Explore"}"#);
    let mut screen = claude.screen_running(&format!(
        "printf '%s' '{{\"status\":\"busy\",\"sessionId\":\"{ID}\"}}' > '{}'/$$.json; sleep 1; exit 3",
        claude.sessions().display()
    ));
    screen.press('n');
    let before = screen.wait_for("1 subagent");
    assert!(before.contains("1 subagent"), "{before}");

    let shown = screen.wait_for("exited 3");

    assert!(!shown.contains("subagent"), "{shown}");
}

struct Claude {
    config: tempfile::TempDir,
    pid_file: PathBuf,
}

impl Claude {
    fn new() -> Claude {
        let config = tempfile::tempdir().unwrap();
        fs::create_dir_all(config.path().join("sessions")).unwrap();
        fs::create_dir_all(config.path().join("projects/-home-acme")).unwrap();
        let pid_file = config.path().join("pid");
        Claude { config, pid_file }
    }

    fn sessions(&self) -> PathBuf {
        self.config.path().join("sessions")
    }

    fn transcript(&self, id: &str, line: &str) {
        let file = self
            .config
            .path()
            .join(format!("projects/-home-acme/{id}.jsonl"));
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(file)
            .unwrap()
            .write_all(format!("{line}\n").as_bytes())
            .unwrap();
    }

    fn subagent(&self, id: &str, name: &str, meta: &str) {
        let folder = self
            .config
            .path()
            .join(format!("projects/-home-acme/{id}/subagents"));
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join(format!("{name}.meta.json")), meta).unwrap();
    }

    fn repository(&self, name: &str, branch: &str) -> PathBuf {
        let repository = self.config.path().join(name);
        fs::create_dir_all(repository.join(".git")).unwrap();
        fs::write(
            repository.join(".git/HEAD"),
            format!("ref: refs/heads/{branch}\n"),
        )
        .unwrap();
        repository
    }

    fn screen(&self, status: &str, id: &str) -> Screen {
        self.screen_with(&format!(r#"{{"status":"{status}","sessionId":"{id}"}}"#))
    }

    fn screen_with(&self, json: &str) -> Screen {
        self.screen_running(&self.registering(json))
    }

    fn registering(&self, json: &str) -> String {
        format!(
            "echo $$ > '{}'; printf '%s' '{json}' > '{}'/$$.json; echo 'acme agent'; exec sleep 600",
            self.pid_file.display(),
            self.sessions().display()
        )
    }

    fn screen_running(&self, text: &str) -> Screen {
        showing(self.app(text))
    }

    fn editing(&self, json: &str) -> Screen {
        let editor = Program::new(
            "acme-edit",
            script("echo 'acme editor ready'; exec sleep 600"),
        );
        let mut screen = showing(self.app(&self.registering(json)).with_editor(editor));
        screen.press('n');
        screen.wait_for("acme agent");
        screen.key(KeyCode::Enter, KeyModifiers::NONE);
        screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
        screen
    }

    fn app(&self, text: &str) -> App {
        App::new(script(text))
            .with_registry(self.sessions())
            .with_transcripts(self.config.path().join("projects"))
    }

    fn register(&self, json: &str) {
        let pid = fs::read_to_string(&self.pid_file).unwrap();
        fs::write(self.sessions().join(format!("{}.json", pid.trim())), json).unwrap();
    }
}

fn tool(input: &str) -> String {
    format!(
        r#"{{"type":"assistant","message":{{"content":[{{"type":"tool_use","name":"Bash","input":{input}}}]}}}}"#
    )
}

fn blocked() -> String {
    format!(r#"{{"status":"waiting","waitingFor":"permission prompt","sessionId":"{ID}"}}"#)
}
