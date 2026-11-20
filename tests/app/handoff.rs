use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, sleep};
use std::time::{Duration, SystemTime};

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::editor::Program;
use harbour::handoff::{Reply, Verb};
use harbour::picker::Picker;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use rmpv::Value;

use crate::{Screen, numbered_agent, script};

#[test]
fn with_no_session_nothing_holds_a_file() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());

    let reply = screen
        .app
        .hand(Verb::Ask, &file(scratch.path(), "acme-api"));

    assert_eq!(reply, Reply::None);
}

#[test]
fn a_session_holding_the_file_answers_with_its_last_use() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    let before = SystemTime::now();
    start_in(&mut screen, scratch.path(), "acme-api", 1);

    let used = last_use(&mut screen, scratch.path());

    assert!(used >= before);
}

#[test]
fn a_file_outside_every_session_is_held_by_none() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);

    let reply = screen
        .app
        .hand(Verb::Ask, &file(scratch.path(), "acme-web"));

    assert_eq!(reply, Reply::None);
}

#[test]
fn a_session_started_through_a_symlink_holds_the_files_behind_it() {
    let scratch = tempfile::tempdir().unwrap();
    fs::create_dir_all(scratch.path().join("acme-api/src")).unwrap();
    let link = scratch.path().join("acme-link");
    std::os::unix::fs::symlink(scratch.path().join("acme-api"), &link).unwrap();
    let mut screen = at_home(App::new(numbered_agent(scratch.path())).with_home(link));
    screen.press('n');
    screen.wait_for("acme agent 1");

    let reply = screen
        .app
        .hand(Verb::Ask, &file(scratch.path(), "acme-api"));

    assert_ne!(reply, Reply::None);
}

#[test]
fn a_key_moves_the_last_use_of_the_selected_session_forward() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    let earlier = last_use(&mut screen, scratch.path());
    sleep(Duration::from_millis(5));

    screen.press('k');

    assert!(last_use(&mut screen, scratch.path()) > earlier);
}

#[test]
fn open_with_no_editor_running_starts_neovim_at_the_sessions_root() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"));

    assert_eq!(reply, Reply::Opened);
    let shown = screen.wait_for("acme nvim ready");
    assert!(shown.contains("acme nvim ready"), "{shown}");
    let root = scratch.path().canonicalize().unwrap().join("acme-api");
    assert_eq!(recorded(scratch.path())[0], root.display().to_string());
}

#[test]
fn the_opened_editor_takes_the_keys() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"));
    screen.wait_for("acme nvim ready");

    screen.press('x');

    let shown = screen.wait_for("acme nvim read:78");
    assert!(shown.contains("acme nvim read:78"), "{shown}");
}

#[test]
fn open_with_no_editor_running_gives_neovim_the_file_as_its_one_argument() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    let file = file(scratch.path(), "acme-api");

    screen.app.hand(Verb::Open, &file);

    screen.wait_for("acme nvim ready");
    let recorded = recorded(scratch.path());
    assert_eq!(recorded[1], "--listen");
    assert_eq!(recorded[3], "--");
    assert_eq!(recorded[4], file.display().to_string());
    assert_eq!(recorded.len(), 5);
}

#[test]
fn open_to_a_running_neovim_starts_no_other() {
    let scratch = tempfile::tempdir().unwrap();
    let (mut screen, socket) = editing(scratch.path());
    let _asked = neovim_at(&socket, false, Edit::Done);
    fs::remove_file(scratch.path().join("args")).unwrap();

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"));

    assert_eq!(reply, Reply::Opened);
    let shown = screen.wait_for("acme nvim ready");
    assert!(shown.contains("acme nvim ready"), "{shown}");
    sleep(Duration::from_millis(200));
    assert!(!scratch.path().join("args").exists());
}

#[test]
fn open_to_a_running_neovim_edits_the_path_as_it_is() {
    let scratch = tempfile::tempdir().unwrap();
    let (mut screen, socket) = editing(scratch.path());
    let asked = neovim_at(&socket, false, Edit::Done);
    let file = file(scratch.path(), "acme-api").with_file_name("a %b#c|d e.rs");

    screen.app.hand(Verb::Open, &file);

    assert_eq!(called(&asked).0, "nvim_get_mode");
    let (method, arguments) = called(&asked);
    assert_eq!(method, "nvim_cmd");
    assert_eq!(field(&arguments[0], "cmd"), Value::from("edit"));
    assert_eq!(
        field(&arguments[0], "args"),
        Value::Array(vec![Value::Binary(file.as_os_str().as_bytes().to_vec())])
    );
    assert_eq!(
        field(&field(&arguments[0], "magic"), "file"),
        Value::from(false)
    );
    assert_eq!(
        field(&field(&arguments[0], "magic"), "bar"),
        Value::from(false)
    );
    assert_eq!(arguments[1], Value::Map(Vec::new()));
}

#[test]
fn open_while_neovim_waits_for_a_key_is_refused_and_edits_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let (mut screen, socket) = editing(scratch.path());
    let asked = neovim_at(&socket, true, Edit::Done);

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"));

    let refusal = "Neovim is waiting for a key; answer it and try again";
    assert_eq!(reply, Reply::Refused(refusal.to_string()));
    assert_eq!(called(&asked).0, "nvim_get_mode");
    assert!(asked.recv_timeout(Duration::from_millis(200)).is_err());
}

#[test]
fn open_that_neovim_refuses_names_its_error_and_keeps_the_session_shown() {
    let scratch = tempfile::tempdir().unwrap();
    let (mut screen, socket) = editing(scratch.path());
    let _asked = neovim_at(
        &socket,
        false,
        Edit::Refused("Vim:E37: No write since last change"),
    );

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"));

    let refusal = "Neovim refused: Vim:E37: No write since last change";
    assert_eq!(reply, Reply::Refused(refusal.to_string()));
    let shown = screen.draw();
    assert!(!shown.contains("acme nvim ready"), "{shown}");
}

#[test]
fn open_to_a_neovim_that_never_answers_the_edit_says_so() {
    let scratch = tempfile::tempdir().unwrap();
    let (mut screen, socket) = editing(scratch.path());
    let asked = neovim_at(&socket, false, Edit::Silent);

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"));

    assert_eq!(reply, Reply::Refused("Neovim did not answer".to_string()));
    assert_eq!(called(&asked).0, "nvim_get_mode");
    assert_eq!(called(&asked).0, "nvim_cmd");
}

#[test]
fn open_to_a_neovim_that_never_answers_says_so() {
    let scratch = tempfile::tempdir().unwrap();
    let (mut screen, _socket) = editing(scratch.path());

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"));

    assert_eq!(reply, Reply::Refused("Neovim did not answer".to_string()));
}

#[test]
fn open_goes_to_the_selected_holder_over_one_used_later() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    start_in(&mut screen, scratch.path(), "acme-api", 2);
    screen.press('k');
    screen.press('j');

    screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"));

    assert_eq!(opened_beside(&mut screen), "acme agent 2");
}

#[test]
fn without_a_selected_holder_open_goes_to_the_one_used_last() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    start_in(&mut screen, scratch.path(), "acme-api", 2);
    screen.press('k');
    start_in(&mut screen, scratch.path(), "acme-web", 3);

    screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"));

    assert_eq!(opened_beside(&mut screen), "acme agent 1");
}

#[test]
fn open_with_an_editor_other_than_neovim_is_refused() {
    let scratch = tempfile::tempdir().unwrap();
    let editor = Program::new("acme-edit", script("exec sleep 600"));
    let mut screen = harbour_with(scratch.path(), editor);
    start_in(&mut screen, scratch.path(), "acme-api", 1);

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"));

    let refusal = "a file can only be handed to Neovim, and HARBOUR_EDITOR is acme-edit";
    assert_eq!(reply, Reply::Refused(refusal.to_string()));
}

#[test]
fn open_for_a_file_no_session_holds_is_refused() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    let file = file(scratch.path(), "acme-web");

    let reply = screen.app.hand(Verb::Open, &file);

    let refusal = format!("no harbour session holds {}", file.display());
    assert_eq!(reply, Reply::Refused(refusal));
}

#[test]
fn open_while_a_question_is_asked_is_refused_and_the_question_stays() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    screen.press('x');

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"));

    assert_eq!(reply, Reply::Refused(WAITING.to_string()));
    let shown = screen.draw();
    assert!(shown.contains("End? y/n"), "{shown}");
}

#[test]
fn open_while_the_picker_is_open_is_refused() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    screen.press('n');

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"));

    assert_eq!(reply, Reply::Refused(WAITING.to_string()));
}

#[test]
fn open_while_the_help_is_shown_closes_it_and_opens_the_file() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    screen.press('?');

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"));

    assert_eq!(reply, Reply::Opened);
    let shown = screen.wait_for("acme nvim ready");
    assert!(shown.contains("acme nvim ready"), "{shown}");
}

#[test]
fn open_with_a_neovim_that_cannot_start_names_why() {
    let scratch = tempfile::tempdir().unwrap();
    let missing = scratch.path().join("acme-missing").display().to_string();
    let mut screen = harbour_with(scratch.path(), Program::neovim("nvim", vec![missing]));
    start_in(&mut screen, scratch.path(), "acme-api", 1);

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"));

    let Reply::Refused(message) = reply else {
        panic!("not refused: {reply:?}");
    };
    assert!(message.starts_with("Could not start nvim: "), "{message}");
    let shown = screen.draw();
    assert!(shown.contains("Could not start nvim"), "{shown}");
}

const WAITING: &str = "harbour is waiting for an answer; try again after it";

fn harbour(scratch: &Path) -> Screen {
    harbour_with(scratch, recording_neovim(scratch))
}

fn harbour_with(scratch: &Path, editor: Program) -> Screen {
    fs::create_dir_all(scratch.join("acme-api/src")).unwrap();
    fs::create_dir_all(scratch.join("acme-web")).unwrap();
    let next = scratch.join("next").display().to_string();
    let picker = script(&format!(
        "d=$(cat '{next}' 2>/dev/null); rm -f '{next}'; \
         if [ -n \"$d\" ]; then printf '%s\\n' \"$d\"; else exec sleep 600; fi"
    ));
    Screen::of(
        App::new(numbered_agent(scratch))
            .with_picker(Picker::Command(picker))
            .with_editor(editor),
    )
}

fn at_home(app: App) -> Screen {
    let mut screen = Screen {
        app,
        terminal: Terminal::new(TestBackend::new(120, 20)).unwrap(),
    };
    screen.draw();
    screen
}

fn start_in(screen: &mut Screen, scratch: &Path, folder: &str, number: usize) {
    fs::write(
        scratch.join("next"),
        scratch.join(folder).display().to_string(),
    )
    .unwrap();
    screen.press('n');
    let row = format!("{folder} {number}");
    let agent = format!("acme agent {number}");
    let shown = screen.wait_for_all(&[&row, &agent]);
    assert!(shown.contains(&row), "{shown}");
    assert!(shown.contains(&agent), "{shown}");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
}

fn editing(scratch: &Path) -> (Screen, PathBuf) {
    let mut screen = harbour(scratch);
    start_in(&mut screen, scratch, "acme-api", 1);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    let shown = screen.wait_for("acme nvim ready");
    assert!(shown.contains("acme nvim ready"), "{shown}");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    let shown = screen.wait_for_all(&["acme agent 1"]);
    assert!(!shown.contains("acme nvim ready"), "{shown}");
    let socket = PathBuf::from(&recorded(scratch)[2]);
    (screen, socket)
}

fn neovim_at(socket: &Path, blocking: bool, edit: Edit) -> Receiver<(String, Vec<Value>)> {
    let listener = UnixListener::bind(socket).unwrap();
    let (asked, calls) = mpsc::channel();
    thread::spawn(move || {
        while let Ok((mut stream, _)) = listener.accept() {
            let request = rmpv::decode::read_value(&mut stream).unwrap();
            let fields = request.as_array().unwrap().clone();
            let method = fields[2].as_str().unwrap().to_string();
            let (error, result) = match (method.as_str(), edit) {
                ("nvim_get_mode", _) => (
                    Value::Nil,
                    Value::Map(vec![
                        (Value::from("mode"), Value::from("n")),
                        (Value::from("blocking"), Value::from(blocking)),
                    ]),
                ),
                (_, Edit::Silent) => {
                    let _ = asked.send((method, fields[3].as_array().unwrap().clone()));
                    thread::sleep(Duration::from_secs(5));
                    continue;
                }
                (_, Edit::Refused(refusal)) => (
                    Value::Array(vec![Value::from(0), Value::from(refusal)]),
                    Value::Nil,
                ),
                _ => (Value::Nil, Value::Map(Vec::new())),
            };
            let _ = asked.send((method, fields[3].as_array().unwrap().clone()));
            let reply = Value::Array(vec![Value::from(1), fields[1].clone(), error, result]);
            rmpv::encode::write_value(&mut stream, &reply).unwrap();
        }
    });
    calls
}

#[derive(Clone, Copy)]
enum Edit {
    Done,
    Refused(&'static str),
    Silent,
}

fn called(asked: &Receiver<(String, Vec<Value>)>) -> (String, Vec<Value>) {
    asked.recv_timeout(Duration::from_secs(5)).unwrap()
}

fn field(map: &Value, name: &str) -> Value {
    map.as_map()
        .unwrap()
        .iter()
        .find(|(key, _)| key.as_str() == Some(name))
        .map(|(_, value)| value.clone())
        .unwrap()
}

fn opened_beside(screen: &mut Screen) -> String {
    let shown = screen.wait_for("acme nvim ready");
    assert!(shown.contains("acme nvim ready"), "{shown}");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    let shown = screen.wait_for("acme agent");
    let at = shown.find("acme agent ").unwrap();
    shown[at..at + "acme agent 1".len()].to_string()
}

fn last_use(screen: &mut Screen, scratch: &Path) -> SystemTime {
    let reply = screen.app.hand(Verb::Ask, &file(scratch, "acme-api"));
    let Reply::Holds(used) = reply else {
        panic!("not held: {reply:?}");
    };
    used
}

fn recording_neovim(scratch: &Path) -> Program {
    let scratch = scratch.display();
    Program::neovim(
        "nvim",
        script(&format!(
            "{{ pwd -P; printf '%s\\n' \"$0\" \"$@\"; }} > '{scratch}/args'; \
             stty raw -echo; echo 'acme nvim ready'; \
             r=$(dd bs=1 count=1 2>/dev/null | od -An -tx1 | tr -d ' \\n'); \
             printf 'acme nvim read:%s' \"$r\"; exec sleep 600"
        )),
    )
}

fn recorded(scratch: &Path) -> Vec<String> {
    fs::read_to_string(scratch.join("args"))
        .unwrap()
        .lines()
        .map(String::from)
        .collect()
}

fn file(scratch: &Path, folder: &str) -> PathBuf {
    scratch
        .canonicalize()
        .unwrap()
        .join(folder)
        .join("src/main.rs")
}
