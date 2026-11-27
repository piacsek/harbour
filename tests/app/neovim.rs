use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use harbour::editor::Program;
use rmpv::Value;

use crate::{Screen, needs_a_terminal, pane_corner, reading_agent, script};

use crate::{deadline, scaled};

const PATIENCE: Duration = Duration::from_millis(100);
const LEFTMOST: &str = "winnr('h') == winnr()";
const RIGHTMOST: &str = "winnr('l') == winnr()";
const UNSAVED: &str = "len(filter(getbufinfo({'bufmodified': 1}), \
                       {_, b -> index(['', 'acwrite'], getbufvar(b.bufnr, '&buftype')) >= 0}))";

#[test]
fn neovim_is_told_to_listen_on_a_socket() {
    let scratch = tempfile::tempdir().unwrap();

    open(scratch.path(), "exec sleep 600");

    let flag = fs::read_to_string(scratch.path().join("flag")).unwrap();
    assert_eq!(flag.trim_end(), "--listen");
}

#[test]
fn the_socket_lives_in_a_directory_only_its_user_can_enter() {
    let scratch = tempfile::tempdir().unwrap();

    let _screen = open(scratch.path(), "exec sleep 600");

    let socket = socket(scratch.path());
    let mode = fs::metadata(socket.parent().unwrap())
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o700);
}

#[test]
fn ctrl_h_asks_neovim_its_mode_then_whether_its_window_is_leftmost() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = open(scratch.path(), "exec sleep 600");
    let asked = ready(&socket(scratch.path()), Some(1));

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    let first = asked.recv_timeout(5 * deadline()).unwrap();
    let second = asked.recv_timeout(5 * deadline()).unwrap();
    assert_eq!(first, "nvim_get_mode");
    assert_eq!(second, format!("nvim_eval {LEFTMOST}"));
}

#[test]
fn ctrl_h_in_the_leftmost_window_in_normal_mode_shows_claude_code() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = open(scratch.path(), "exec sleep 600");
    let _asked = ready(&socket(scratch.path()), Some(1));

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme agent ready");
    assert!(shown.contains("acme agent ready"), "{shown}");
    assert!(!shown.contains("acme nvim read"), "{shown}");
}

#[test]
fn ctrl_h_anywhere_else_in_neovim_reaches_neovim() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = open(scratch.path(), "exec sleep 600");
    let _asked = ready(&socket(scratch.path()), Some(0));

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme nvim read:08");
    assert!(shown.contains("acme nvim read:08"), "{shown}");
}

#[test]
fn ctrl_h_shows_claude_code_when_nothing_listens_on_the_socket() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = open(scratch.path(), "exec sleep 600");

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme agent ready");
    assert!(shown.contains("acme agent ready"), "{shown}");
}

#[test]
fn ctrl_h_shows_claude_code_soon_when_neovim_never_answers() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen =
        open_with(fake_neovim(scratch.path(), "exec sleep 600").with_patience(deadline()));
    let _asked = answering(&socket(scratch.path()), None, None);

    let before = Instant::now();
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    let waited = before.elapsed();

    let shown = screen.wait_for("acme agent ready");
    assert!(shown.contains("acme agent ready"), "{shown}");
    assert!(waited >= deadline(), "{waited:?}");
    assert!(waited < 2 * deadline(), "{waited:?}");
}

#[test]
fn ctrl_h_on_a_silent_neovim_waits_out_one_mode_query_by_default() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = open_with(fake_neovim(scratch.path(), "exec sleep 600"));
    let asked = answering(&socket(scratch.path()), None, None);

    let before = Instant::now();
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    let waited = before.elapsed();

    let shown = screen.wait_for("acme agent ready");
    assert!(shown.contains("acme agent ready"), "{shown}");
    assert_eq!(asked.recv_timeout(5 * deadline()).unwrap(), "nvim_get_mode");
    assert!(asked.recv_timeout(scaled(PATIENCE)).is_err());
    assert!(waited < scaled(10 * PATIENCE), "{waited:?}");
}

#[test]
fn the_socket_directory_is_removed_once_neovim_exits() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = open(scratch.path(), "exit 0");
    let socket = socket(scratch.path());
    screen.press('q');

    screen.wait_for("acme agent ready");

    assert!(!socket.parent().unwrap().exists());
}

#[test]
fn x_asks_neovim_whether_it_has_unsaved_changes() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = in_the_sidebar(scratch.path());
    let asked = ready(&socket(scratch.path()), Some(1));

    screen.press('x');

    let first = asked.recv_timeout(5 * deadline()).unwrap();
    let second = asked.recv_timeout(5 * deadline()).unwrap();
    assert_eq!(first, "nvim_get_mode");
    assert_eq!(second, format!("nvim_eval {UNSAVED}"));
}

#[test]
fn x_names_unsaved_changes_in_neovim() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = in_the_sidebar(scratch.path());
    let _asked = ready(&socket(scratch.path()), Some(2));

    screen.press('x');

    let shown = screen.draw();
    assert!(shown.contains("Unsaved changes. End? y/n"), "{shown}");
}

#[test]
fn x_asks_the_plain_question_when_neovim_has_nothing_unsaved() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = in_the_sidebar(scratch.path());
    let _asked = ready(&socket(scratch.path()), Some(0));

    screen.press('x');

    let shown = screen.draw();
    assert!(shown.contains("End? y/n"), "{shown}");
    assert!(!shown.contains("Unsaved"), "{shown}");
}

#[test]
fn x_says_neovim_may_have_unsaved_changes_when_nothing_listens_on_the_socket() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = in_the_sidebar(scratch.path());

    screen.press('x');

    let shown = screen.draw();
    assert!(shown.contains("May have unsaved"), "{shown}");
}

#[test]
fn x_says_neovim_may_have_unsaved_changes_when_it_never_answers() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = in_the_sidebar_of(fake_neovim(scratch.path(), "exec sleep 600"));
    let _asked = answering(&socket(scratch.path()), None, None);

    screen.press('x');

    let shown = screen.draw();
    assert!(shown.contains("May have unsaved"), "{shown}");
}

#[test]
fn x_says_neovim_may_have_unsaved_changes_while_it_is_blocking() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = in_the_sidebar(scratch.path());
    let asked = answering(&socket(scratch.path()), Some(("n", true)), Some(0));

    screen.press('x');

    let shown = screen.draw();
    assert!(shown.contains("May have unsaved"), "{shown}");
    assert_eq!(asked.try_iter().count(), 1);
}

#[test]
fn x_says_neovim_may_have_unsaved_changes_when_it_tells_its_mode_but_not_its_buffers() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = in_the_sidebar_of(fake_neovim(scratch.path(), "exec sleep 600"));
    let _asked = answering(&socket(scratch.path()), Some(("n", false)), None);

    screen.press('x');

    let shown = screen.draw();
    assert!(shown.contains("May have unsaved"), "{shown}");
}

#[test]
fn ctrl_h_while_neovim_is_blocking_reaches_neovim() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = open(scratch.path(), "exec sleep 600");
    let asked = answering(&socket(scratch.path()), Some(("n", true)), Some(1));

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme nvim read:08");
    assert!(shown.contains("acme nvim read:08"), "{shown}");
    assert_eq!(asked.try_iter().count(), 1);
}

#[test]
fn ctrl_h_with_an_operator_pending_in_the_leftmost_window_reaches_neovim() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = open(scratch.path(), "exec sleep 600");
    let _asked = answering(&socket(scratch.path()), Some(("no", false)), Some(1));

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme nvim read:08");
    assert!(shown.contains("acme nvim read:08"), "{shown}");
}

#[test]
fn ctrl_h_after_ctrl_o_in_insert_mode_reaches_neovim() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = open(scratch.path(), "exec sleep 600");
    let _asked = answering(&socket(scratch.path()), Some(("niI", false)), Some(1));

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme nvim read:08");
    assert!(shown.contains("acme nvim read:08"), "{shown}");
}

#[test]
fn ctrl_h_in_the_leftmost_window_in_terminal_normal_mode_shows_claude_code() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = open(scratch.path(), "exec sleep 600");
    let _asked = answering(&socket(scratch.path()), Some(("nt", false)), Some(1));

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme agent ready");
    assert!(shown.contains("acme agent ready"), "{shown}");
}

#[test]
fn ctrl_h_in_terminal_normal_mode_with_a_window_to_the_left_reaches_neovim() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = open(scratch.path(), "exec sleep 600");
    let _asked = answering(&socket(scratch.path()), Some(("nt", false)), Some(0));

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme nvim read:08");
    assert!(shown.contains("acme nvim read:08"), "{shown}");
}

#[test]
fn a_notification_before_the_answer_does_not_hide_it() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = in_the_sidebar(scratch.path());
    let _asked = serving(&socket(scratch.path()), Some(("n", false)), Some(1), true);

    screen.press('x');

    let shown = screen.draw();
    assert!(shown.contains("Unsaved changes. End? y/n"), "{shown}");
    assert!(!shown.contains("May have"), "{shown}");
}

#[test]
fn q_names_unsaved_changes_in_neovim() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = in_the_sidebar(scratch.path());
    let _asked = ready(&socket(scratch.path()), Some(1));

    screen.prefixed('q');

    let shown = screen.draw();
    assert!(shown.contains("Unsaved changes. Quit? y/n"), "{shown}");
}

#[test]
fn ctrl_c_to_the_quit_question_with_unsaved_changes_in_neovim_keeps_harbour_running() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = in_the_sidebar(scratch.path());
    let _asked = ready(&socket(scratch.path()), Some(1));
    screen.prefixed('q');
    let asked = screen.draw();
    assert!(asked.contains("Unsaved changes. Quit? y/n"), "{asked}");

    screen.key(KeyCode::Char('c'), KeyModifiers::CONTROL);

    let shown = screen.draw();
    assert!(!shown.contains("y/n"), "{shown}");
    assert!(screen.app.running());
}

#[test]
fn ctrl_h_kept_by_a_neovim_that_asked_for_the_kitty_protocol_reaches_it_in_the_kitty_form() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = open_with(kitty_neovim(scratch.path()));
    let _asked = ready(&socket(scratch.path()), Some(0));

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme nvim read:1b5b3130343b3575");
    assert!(shown.contains("acme nvim read:1b5b3130343b3575"), "{shown}");
}

#[test]
fn swapped_ctrl_l_asks_neovim_its_mode_then_whether_its_window_is_rightmost() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = swapped(scratch.path());
    let asked = ready(&socket(scratch.path()), Some(1));

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let first = asked.recv_timeout(5 * deadline()).unwrap();
    let second = asked.recv_timeout(5 * deadline()).unwrap();
    assert_eq!(first, "nvim_get_mode");
    assert_eq!(second, format!("nvim_eval {RIGHTMOST}"));
}

#[test]
fn swapped_ctrl_l_with_a_window_to_the_right_reaches_neovim() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = swapped(scratch.path());
    let _asked = ready(&socket(scratch.path()), Some(0));

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme nvim read:0c");
    assert!(shown.contains("acme nvim read:0c"), "{shown}");
}

#[test]
fn swapped_ctrl_l_in_the_rightmost_window_focuses_claude_code() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = swapped(scratch.path());
    let _asked = ready(&socket(scratch.path()), Some(1));

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent got: hi");
    assert!(!shown.contains("acme nvim read:"), "{shown}");
}

#[test]
fn swapped_ctrl_h_kept_by_neovim_reaches_neovim_and_the_sidebar_stays_hidden() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = swapped(scratch.path());
    let _asked = ready(&socket(scratch.path()), Some(0));

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme nvim read:08");
    assert!(shown.contains("acme nvim read:08"), "{shown}");
    assert_eq!(pane_corner(&shown), 60, "{shown}");
}

fn in_the_sidebar(scratch: &Path) -> Screen {
    in_the_sidebar_of(fake_neovim(scratch, "exec sleep 600").with_patience(5 * deadline()))
}

fn in_the_sidebar_of(neovim: Program) -> Screen {
    let mut screen = open_with(neovim);
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 1, 1);
    screen.mouse(MouseEventKind::Up(MouseButton::Left), 1, 1);
    screen
}

fn open(scratch: &Path, then: &str) -> Screen {
    open_with(fake_neovim(scratch, then).with_patience(5 * deadline()))
}

fn open_with(neovim: Program) -> Screen {
    let mut screen = Screen::editing(reading_agent(), neovim);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme nvim ready");
    screen
}

fn fake_neovim(scratch: &Path, then: &str) -> Program {
    let scratch = scratch.display();
    Program::neovim(
        "nvim",
        script(&format!(
            "printf '%s\\n' \"$0\" > '{scratch}/flag'; printf '%s\\n' \"$1\" > '{scratch}/socket'; \
             stty raw -echo; echo 'acme nvim ready'; \
             r=$(dd bs=1 count=1 2>/dev/null | od -An -tx1 | tr -d ' \\n'); \
             printf 'acme nvim read:%s' \"$r\"; {then}"
        )),
    )
}

fn kitty_neovim(scratch: &Path) -> Program {
    let scratch = scratch.display();
    Program::neovim(
        "nvim",
        script(&format!(
            "printf '%s\\n' \"$1\" > '{scratch}/socket'; \
             stty raw -echo; printf '\\033[>1u'; echo 'acme nvim ready'; \
             r=$(dd bs=1 count=8 2>/dev/null | od -An -tx1 | tr -d ' \\n'); \
             printf 'acme nvim read:%s' \"$r\"; exec sleep 600"
        )),
    )
    .with_patience(5 * deadline())
}

fn socket(scratch: &Path) -> PathBuf {
    PathBuf::from(
        fs::read_to_string(scratch.join("socket"))
            .unwrap()
            .trim_end(),
    )
}

fn ready(socket: &Path, eval: Option<i64>) -> Receiver<String> {
    answering(socket, Some(("n", false)), eval)
}

fn answering(
    socket: &Path,
    mode: Option<(&'static str, bool)>,
    eval: Option<i64>,
) -> Receiver<String> {
    serving(socket, mode, eval, false)
}

fn serving(
    socket: &Path,
    mode: Option<(&'static str, bool)>,
    eval: Option<i64>,
    notify_first: bool,
) -> Receiver<String> {
    let listener = UnixListener::bind(socket).unwrap();
    let (asked, requests) = mpsc::channel();
    thread::spawn(move || {
        let mut held = Vec::new();
        while let Ok((mut stream, _)) = listener.accept() {
            let request = rmpv::decode::read_value(&mut stream).unwrap();
            let fields = request.as_array().unwrap().clone();
            let method = fields[2].as_str().unwrap().to_string();
            let answer = if method == "nvim_get_mode" {
                let _ = asked.send(method);
                mode.map(|(mode, blocking)| {
                    Value::Map(vec![
                        (Value::from("mode"), Value::from(mode)),
                        (Value::from("blocking"), Value::from(blocking)),
                    ])
                })
            } else {
                let expression = fields[3][0].as_str().unwrap();
                let _ = asked.send(format!("{method} {expression}"));
                eval.map(Value::from)
            };
            match answer {
                Some(answer) => {
                    if notify_first {
                        let notification = Value::Array(vec![
                            Value::from(2),
                            Value::from("acme_event"),
                            Value::Array(Vec::new()),
                        ]);
                        rmpv::encode::write_value(&mut stream, &notification).unwrap();
                    }
                    let reply =
                        Value::Array(vec![Value::from(1), fields[1].clone(), Value::Nil, answer]);
                    rmpv::encode::write_value(&mut stream, &reply).unwrap();
                }
                None => held.push(stream),
            }
        }
    });
    requests
}

fn swapped(scratch: &Path) -> Screen {
    let neovim = fake_neovim(scratch, "exec sleep 600").with_patience(5 * deadline());
    let mut screen = Screen::editing(reading_agent(), neovim);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.prefixed('\\');
    screen.wait_for("acme nvim ready");
    screen.prefixed('}');
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen
}
