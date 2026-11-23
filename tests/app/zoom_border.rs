use std::fs;
use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::editor::Program;

use crate::{Screen, in_order, script};

const ID: &str = "4c3b2a19-0f8e-4d7c-9b6a-5f4e3d2c1b0a";

#[test]
fn the_zoomed_top_row_shows_the_session_and_the_counts() {
    let claude = Claude::new("Fix the login");
    let mut screen = Screen::of(claude.app());
    started(&mut screen);

    zoom(&mut screen);

    let top = top(&screen.draw());
    assert!(
        top.starts_with(" ● Fix the login  acme-api · main  3m   "),
        "{top}"
    );
    assert!(top.ends_with("   sessions ● 1 "), "{top}");
}

#[test]
fn the_zoomed_session_fills_every_row_below_the_top_one_edge_to_edge() {
    let claude = Claude::new("Fix the login");
    let mut screen = Screen::of(claude.app());
    started(&mut screen);

    zoom(&mut screen);

    let shown = screen.draw();
    let second = shown.lines().nth(1).unwrap_or_default();
    assert!(second.starts_with("acme agent"), "{shown}");
}

#[test]
fn the_zoomed_top_row_keeps_the_names_after_the_session() {
    let claude = Claude::new("Fix the login");
    let editor = Program::new("acme-edit", script("exec sleep 600"));
    let mut screen = Screen::of(claude.app().with_editor(editor));
    started(&mut screen);

    zoom(&mut screen);

    let top = top(&screen.draw());
    assert!(
        top.starts_with(" ● Fix the login  acme-api · main  3m  claude "),
        "{top}"
    );
    assert!(in_order(&top, &[" claude ", " acme-edit   "]), "{top}");
    assert!(top.ends_with("   sessions ● 1 "), "{top}");
}

#[test]
fn a_long_editor_directory_leaves_the_session_its_dot_and_title() {
    let claude = Claude::in_repository("Fix the login", "acme-workspace/acme-services/acme-api");
    let editor = Program::new("acme-edit", script("exec sleep 600"));
    let mut screen = Screen::of(claude.app().with_editor(editor));
    screen.resize(78, 20);
    started(&mut screen);
    zoom(&mut screen);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let top = top(&screen.wait_for("acme-edit: "));
    assert!(
        top.starts_with(" ● Fix the login  acme-api…  3m  claude "),
        "{top}"
    );
    assert!(in_order(&top, &[" claude ", " acme-edit: …"]), "{top}");
    assert!(top.ends_with("  sessions ● 1 "), "{top}");
}

#[test]
fn a_narrow_zoomed_top_row_cuts_the_session_and_keeps_the_counts() {
    let claude = Claude::new("Fix the login");
    let mut screen = Screen::of(claude.app());
    started(&mut screen);
    zoom(&mut screen);

    screen.resize(33, 20);

    let top = top(&screen.draw());
    assert_eq!(top, " ● Fix the lo…  3m  sessions ● 1 ");
}

#[test]
fn the_narrowest_zoomed_top_row_keeps_the_counts_and_the_dot_with_a_column_of_title() {
    let claude = Claude::new("Fix the login");
    let mut screen = Screen::of(claude.app());
    started(&mut screen);
    zoom(&mut screen);

    screen.resize(9, 20);

    let top = top(&screen.draw());
    assert_eq!(top, " ● …  ●1 ");
}

#[test]
fn the_waiting_prefix_on_a_narrow_zoomed_top_row_makes_room_for_itself() {
    let claude = Claude::new("Fix the login");
    let mut screen = Screen::of(claude.app());
    started(&mut screen);
    zoom(&mut screen);
    screen.resize(16, 20);

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);

    let top = top(&screen.draw());
    assert!(top.starts_with(" ●"), "{top}");
    assert!(top.ends_with(" C-/ ●1 "), "{top}");
}

#[test]
fn a_cjk_title_on_the_zoomed_top_row_is_cut_by_display_width() {
    let claude = Claude::new("修正ログイン");
    let mut screen = Screen::of(claude.app());
    started(&mut screen);
    zoom(&mut screen);

    screen.resize(28, 20);

    let top = top(&screen.draw());
    assert!(top.starts_with(" ● 修"), "{top}");
    assert!(top.contains("…  3m   sessions ● 1 "), "{top}");
}

#[test]
fn a_one_row_terminal_shows_the_zoomed_top_row_alone() {
    let claude = Claude::new("Fix the login");
    let mut screen = Screen::of(claude.app());
    started(&mut screen);
    zoom(&mut screen);

    screen.resize(60, 1);

    let shown = screen.draw();
    assert!(
        shown.starts_with(" ● Fix the login  acme-api · main  3m "),
        "{shown}"
    );
    assert!(!shown.contains("acme agent"), "{shown}");
}

#[test]
fn leaving_zoom_takes_the_session_off_the_top_row() {
    let claude = Claude::new("Fix the login");
    let mut screen = Screen::of(claude.app());
    started(&mut screen);
    zoom(&mut screen);

    zoom(&mut screen);

    let top = top(&screen.draw());
    assert!(!top.contains("acme-api · main"), "{top}");
}

fn zoom(screen: &mut Screen) {
    screen.prefixed('z');
}

pub(crate) fn started(screen: &mut Screen) {
    screen.press('n');
    screen.wait_for_all(&["acme-api", "main", "3m"]);
}

pub(crate) fn top(shown: &str) -> String {
    shown.lines().next().unwrap_or_default().to_string()
}

pub(crate) struct Claude {
    config: tempfile::TempDir,
}

impl Claude {
    pub(crate) fn new(title: &str) -> Claude {
        Claude::in_repository(title, "acme-api")
    }

    fn in_repository(title: &str, path: &str) -> Claude {
        let config = tempfile::tempdir().unwrap();
        let projects = config.path().join("projects/-home-acme");
        fs::create_dir_all(config.path().join("sessions")).unwrap();
        fs::create_dir_all(&projects).unwrap();
        let repository = repository(config.path(), path, "main");
        fs::write(
            projects.join(format!("{ID}.jsonl")),
            format!(
                "{{\"type\":\"ai-title\",\"aiTitle\":\"{title}\"}}\n\
                 {{\"type\":\"user\",\"cwd\":\"{}\",\"message\":{{}}}}\n",
                repository.display()
            ),
        )
        .unwrap();
        Claude { config }
    }

    fn sessions(&self) -> PathBuf {
        self.config.path().join("sessions")
    }

    pub(crate) fn app(&self) -> App {
        App::new(script(&format!(
            "printf '%s' '{{\"status\":\"busy\",\"sessionId\":\"{ID}\",\"statusUpdatedAt\":'$(( ($(date +%s) - 180) * 1000 ))'}}' > '{}'/$$.json; \
             echo 'acme agent'; exec sleep 600",
            self.sessions().display()
        )))
        .with_registry(self.sessions())
        .with_transcripts(self.config.path().join("projects"))
    }
}

fn repository(root: &Path, name: &str, branch: &str) -> PathBuf {
    let repository = root.join(name);
    fs::create_dir_all(repository.join(".git")).unwrap();
    fs::write(
        repository.join(".git/HEAD"),
        format!("ref: refs/heads/{branch}\n"),
    )
    .unwrap();
    repository
}
