use std::fs;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, sleep};
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::editor::Program;
use harbour::handoff::{Reply, Verb};
use rmpv::Value;

use crate::directory::{cwd, following_with, repository, write_transcript};
use crate::handoff::{editing, file, harbour, harbour_with, soon, start_in};
use crate::script;

const WAITING: &str = "Neovim is waiting for a key; answer it and try again";
const BUSY: &str = "harbour was busy and did not get to it in time";
const CLOSED: &str = "Neovim did not start in time; harbour closed it";
const EXITED: &str = "Neovim exited before it took the file";

#[test]
fn an_open_that_starts_neovim_answers_once_it_has_the_file() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    let file = file(scratch.path(), "acme-api");
    let asked = answering(scratch.path());

    let reply = screen.app.hand(Verb::Open, &file, soon());

    assert_eq!(reply, Reply::Opened);
    assert_eq!(called(&asked).0, "nvim_get_mode");
    assert_eq!(called(&asked).0, "nvim_cmd");
}

#[test]
fn an_open_that_starts_a_neovim_that_never_answers_closes_it_and_leaves_the_file() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);

    let reply = screen.app.hand(
        Verb::Open,
        &file(scratch.path(), "acme-api"),
        Instant::now() + Duration::from_millis(600),
    );

    assert_eq!(reply, Reply::Untaken(CLOSED.to_string()));
    let shown = screen.draw();
    assert!(!shown.contains("acme nvim ready"), "{shown}");
}

#[test]
fn an_open_that_starts_a_neovim_which_exits_at_once_says_so_without_waiting() {
    let scratch = tempfile::tempdir().unwrap();
    let editor = Program::neovim("nvim", script("exit 1"));
    let mut screen = harbour_with(scratch.path(), editor);
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    let started = Instant::now();

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"), soon());

    assert_eq!(reply, Reply::Untaken(EXITED.to_string()));
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn a_neovim_harbour_closed_for_not_answering_is_started_anew_by_ctrl_l() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    screen.app.hand(
        Verb::Open,
        &file(scratch.path(), "acme-api"),
        Instant::now() + Duration::from_millis(600),
    );
    fs::remove_file(scratch.path().join("args")).unwrap();

    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme nvim ready");
    assert!(shown.contains("acme nvim ready"), "{shown}");
    assert!(scratch.path().join("args").exists());
}

#[test]
fn a_neovim_harbour_starts_for_an_open_has_until_the_deadline_to_answer() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    let _asked = answering_after(scratch.path(), Duration::from_millis(1500));

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"), soon());

    assert_eq!(reply, Reply::Opened);
}

#[test]
fn an_open_whose_deadline_runs_out_before_a_running_neovim_answers_says_harbour_was_busy() {
    let scratch = tempfile::tempdir().unwrap();
    let (mut screen, _socket) = editing(scratch.path());

    let reply = screen.app.hand(
        Verb::Open,
        &file(scratch.path(), "acme-api"),
        Instant::now() + Duration::from_millis(500),
    );

    assert_eq!(reply, Reply::Refused(BUSY.to_string()));
}

#[test]
fn an_open_that_starts_a_neovim_waiting_for_a_key_is_refused() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    let _asked = answering_in(scratch.path(), "n", true, Duration::ZERO, Edit::Done);

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"), soon());

    assert_eq!(reply, Reply::Refused(WAITING.to_string()));
}

#[test]
fn an_open_that_starts_a_neovim_which_stops_at_a_prompt_on_loading_the_file_is_refused() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    let _asked = answering_in(scratch.path(), "n", false, Duration::ZERO, Edit::Prompting);
    let started = Instant::now();

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"), soon());

    assert_eq!(reply, Reply::Refused(WAITING.to_string()));
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn an_open_that_raises_a_prompt_in_a_running_neovim_is_refused_at_once() {
    let scratch = tempfile::tempdir().unwrap();
    let (mut screen, socket) = editing(scratch.path());
    let _asked = neovim_in(&socket, "n", false, Edit::Asking);
    let started = Instant::now();

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"), soon());

    assert_eq!(reply, Reply::Refused(WAITING.to_string()));
    assert!(started.elapsed() < Duration::from_millis(500));
}

#[test]
fn open_while_neovim_is_on_its_command_line_is_refused() {
    let scratch = tempfile::tempdir().unwrap();
    let (mut screen, socket) = editing(scratch.path());
    let asked = neovim_in(&socket, "c", false, Edit::Done);

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"), soon());

    assert_eq!(reply, Reply::Refused(WAITING.to_string()));
    assert_eq!(called(&asked).0, "nvim_get_mode");
    assert!(asked.recv_timeout(Duration::from_millis(200)).is_err());
}

#[test]
fn open_while_neovim_shows_a_prompt_is_refused() {
    let scratch = tempfile::tempdir().unwrap();
    let (mut screen, socket) = editing(scratch.path());
    let asked = neovim_in(&socket, "r?", false, Edit::Done);

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"), soon());

    assert_eq!(reply, Reply::Refused(WAITING.to_string()));
    assert_eq!(called(&asked).0, "nvim_get_mode");
    assert!(asked.recv_timeout(Duration::from_millis(200)).is_err());
}

#[test]
fn open_while_neovim_waits_for_an_operator_motion_is_refused() {
    let scratch = tempfile::tempdir().unwrap();
    let (mut screen, socket) = editing(scratch.path());
    let asked = neovim_in(&socket, "no", false, Edit::Done);

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"), soon());

    assert_eq!(reply, Reply::Refused(WAITING.to_string()));
    assert_eq!(called(&asked).0, "nvim_get_mode");
    assert!(asked.recv_timeout(Duration::from_millis(200)).is_err());
}

#[test]
fn an_open_past_its_deadline_starts_no_editor() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);

    let reply = screen.app.hand(
        Verb::Open,
        &file(scratch.path(), "acme-api"),
        Instant::now(),
    );

    assert_eq!(reply, Reply::Refused(BUSY.to_string()));
    sleep(Duration::from_millis(200));
    assert!(!scratch.path().join("args").exists());
}

#[test]
fn open_goes_to_the_directory_the_transcript_names_now() {
    let scratch = tempfile::tempdir().unwrap();
    let two = repository(scratch.path(), "acme-two");
    let mut screen = following_with(scratch.path(), &[], locating_neovim(scratch.path()));
    let _asked = answering(scratch.path());
    write_transcript(scratch.path(), &[&cwd(&two)]);
    let file = two.canonicalize().unwrap().join("main.rs");

    let reply = screen.app.hand(Verb::Open, &file, soon());

    assert_eq!(reply, Reply::Opened);
    let shown = screen.wait_for("acme nvim at");
    assert!(shown.contains("acme nvim at acme-two"), "{shown}");
}

#[test]
fn ask_while_a_question_is_asked_is_answered_none() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    screen.press('x');

    let reply = screen
        .app
        .hand(Verb::Ask, &file(scratch.path(), "acme-api"), soon());

    assert_eq!(reply, Reply::None);
}

#[test]
fn ask_while_the_picker_is_open_is_answered_none() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    screen.press('n');

    let reply = screen
        .app
        .hand(Verb::Ask, &file(scratch.path(), "acme-api"), soon());

    assert_eq!(reply, Reply::None);
}

fn locating_neovim(scratch: &Path) -> Program {
    Program::neovim(
        "nvim",
        script(&format!(
            "{{ pwd -P; printf '%s\\n' \"$0\" \"$@\"; }} > '{}/args'; \
             echo \"acme nvim at $(basename \"$(pwd -P)\")\"; exec sleep 600",
            scratch.display()
        )),
    )
}

pub(crate) fn neovim_at(
    socket: &Path,
    blocking: bool,
    edit: Edit,
) -> Receiver<(String, Vec<Value>)> {
    neovim_in(socket, "n", blocking, edit)
}

fn neovim_in(
    socket: &Path,
    mode: &'static str,
    blocking: bool,
    edit: Edit,
) -> Receiver<(String, Vec<Value>)> {
    let listener = UnixListener::bind(socket).unwrap();
    let (asked, calls) = mpsc::channel();
    thread::spawn(move || {
        let mut held = Vec::new();
        while let Ok((mut stream, _)) = listener.accept() {
            let request = rmpv::decode::read_value(&mut stream).unwrap();
            let fields = request.as_array().unwrap().clone();
            let method = fields[2].as_str().unwrap().to_string();
            let (shown, blocked) = match edit {
                Edit::Asking if !held.is_empty() => ("r?", false),
                _ => (mode, blocking || !held.is_empty()),
            };
            let (error, result) = match (method.as_str(), edit) {
                ("nvim_get_mode", _) => (
                    Value::Nil,
                    Value::Map(vec![
                        (Value::from("mode"), Value::from(shown)),
                        (Value::from("blocking"), Value::from(blocked)),
                    ]),
                ),
                (_, Edit::Silent) => {
                    let _ = asked.send((method, fields[3].as_array().unwrap().clone()));
                    thread::sleep(Duration::from_secs(5));
                    continue;
                }
                (_, Edit::Prompting | Edit::Asking) => {
                    let _ = asked.send((method, fields[3].as_array().unwrap().clone()));
                    held.push(stream);
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
pub(crate) enum Edit {
    Done,
    Refused(&'static str),
    Silent,
    Prompting,
    Asking,
}

pub(crate) fn called(asked: &Receiver<(String, Vec<Value>)>) -> (String, Vec<Value>) {
    asked.recv_timeout(Duration::from_secs(5)).unwrap()
}

pub(crate) fn answering(scratch: &Path) -> Receiver<(String, Vec<Value>)> {
    answering_in(scratch, "n", false, Duration::ZERO, Edit::Done)
}

fn answering_after(scratch: &Path, delay: Duration) -> Receiver<(String, Vec<Value>)> {
    answering_in(scratch, "n", false, delay, Edit::Done)
}

fn answering_in(
    scratch: &Path,
    mode: &'static str,
    blocking: bool,
    delay: Duration,
    edit: Edit,
) -> Receiver<(String, Vec<Value>)> {
    let scratch = scratch.to_path_buf();
    let (forward, calls) = mpsc::channel();
    thread::spawn(move || {
        let mut bound = Vec::new();
        while scratch.exists() {
            let lines: Vec<String> = fs::read_to_string(scratch.join("args"))
                .unwrap_or_default()
                .lines()
                .map(String::from)
                .collect();
            if lines.len() >= 5 && !bound.contains(&lines[2]) {
                bound.push(lines[2].clone());
                sleep(delay);
                let asked = neovim_in(Path::new(&lines[2]), mode, blocking, edit);
                let forward = forward.clone();
                thread::spawn(move || {
                    for call in asked {
                        let _ = forward.send(call);
                    }
                });
            }
            sleep(Duration::from_millis(2));
        }
    });
    calls
}
