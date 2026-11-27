use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};

use harbour::picker::Picker;

use crate::{Screen, directory, home, needs_a_terminal, reading_agent, script, starts_focused};

use crate::deadline;

#[test]
fn a_directory_the_picker_prints_starts_a_session_there_named_after_it() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let web = directory_in(scratch.path(), "acme-web");
    let mut screen = Screen::picking(placed_agent(), Picker::Command(typed_picker()));
    screen.press('n');
    screen.wait_for("acme picker");

    screen.type_text(&format!("{}\r", web.display()));

    let shown = screen.wait_for_all(&["acme-web 1", "acme agent in acme-web"]);
    assert!(shown.contains("acme-web 1"), "{shown}");
    assert!(shown.contains("acme agent in acme-web"), "{shown}");
}

#[test]
fn esc_in_the_picker_starts_nothing() {
    let mut screen = Screen::picking(placed_agent(), Picker::Command(escapable_picker()));
    screen.press('n');
    screen.wait_for("acme picker");

    screen.key(KeyCode::Esc, KeyModifiers::NONE);

    let shown = closed(&mut screen);
    assert!(shown.contains("No session yet"), "{shown}");
    assert!(!shown.contains(" directory "), "{shown}");
}

#[test]
fn a_picker_that_prints_nothing_starts_nothing() {
    needs_a_terminal();
    let picker = script("echo 'acme picker' >&2; read -r line");
    let mut screen = Screen::picking(placed_agent(), Picker::Command(picker));
    screen.press('n');
    screen.wait_for("acme picker");

    screen.type_text("\r");

    let shown = closed(&mut screen);
    assert!(shown.contains("No session yet"), "{shown}");
    assert!(!shown.contains(" directory "), "{shown}");
}

#[test]
fn a_picker_that_exits_with_an_error_starts_nothing() {
    needs_a_terminal();
    let picker = script("echo 'acme picker' >&2; read -r line; pwd; exit 1");
    let mut screen = Screen::picking(placed_agent(), Picker::Command(picker));
    screen.press('n');
    screen.wait_for("acme picker");

    screen.type_text("\r");

    let shown = closed(&mut screen);
    assert!(shown.contains("No session yet"), "{shown}");
    assert!(!shown.contains(" directory "), "{shown}");
}

#[test]
fn a_path_that_is_not_a_directory_is_named_and_starts_nothing() {
    needs_a_terminal();
    let picker = script("echo 'acme picker' >&2; read -r line; echo /acme/gone");
    let mut screen = Screen::picking(placed_agent(), Picker::Command(picker));
    screen.press('n');
    screen.wait_for("acme picker");

    screen.type_text("\r");

    let shown = screen.wait_for_all(&["Not a directory: /acme/gone", "No session yet"]);
    assert!(shown.contains("Not a directory: /acme/gone"), "{shown}");
    assert!(shown.contains("No session yet"), "{shown}");
    assert!(!shown.contains(" directory "), "{shown}");
}

#[test]
fn a_relative_path_is_read_from_the_directory_of_harbour() {
    needs_a_terminal();
    std::fs::create_dir_all(home().join("src")).unwrap();
    let picker = script("echo 'acme picker' >&2; read -r line; echo src");
    let mut screen = Screen::picking(placed_agent(), Picker::Command(picker));
    screen.press('n');
    screen.wait_for("acme picker");

    screen.type_text("\r");

    let shown = screen.wait_for_all(&["src 1", "acme agent in src"]);
    assert!(shown.contains("src 1"), "{shown}");
    assert!(shown.contains("acme agent in src"), "{shown}");
}

#[test]
fn a_picker_that_cannot_start_is_named_and_starts_nothing() {
    let picker = vec!["acme-nopicker".to_string()];
    let mut screen = Screen::picking(placed_agent(), Picker::Command(picker));

    screen.press('n');

    let shown = screen.wait_for_all(&["Could not start acme-nopicker", "No session yet"]);
    assert!(shown.contains("Could not start acme-nopicker"), "{shown}");
    assert!(shown.contains("No session yet"), "{shown}");
    assert!(!shown.contains(" directory "), "{shown}");
}

#[test]
fn enter_in_fzf_starts_the_session_where_harbour_was_opened() {
    needs_a_terminal();
    let mut screen = Screen::picking(
        placed_agent(),
        fuzzy(first_line_fzf(), listing("/acme/web")),
    );
    screen.press('n');
    screen.wait_for("acme picker");

    screen.type_text("\r");

    let row = format!("{} 1", directory());
    let shown = screen.wait_for_all(&[&row, &format!("acme agent in {}", directory())]);
    assert!(shown.contains(&row), "{shown}");
    assert!(!shown.contains(" directory "), "{shown}");
}

#[test]
fn fzf_lists_the_directory_of_harbour_first_then_zoxide_without_it() {
    let scratch = tempfile::tempdir().unwrap();
    let here = here();
    let ranked = format!("/acme/api\n{here}\n/acme/web\n");
    let fzf = recording_fzf(scratch.path());
    let mut screen = Screen::picking(placed_agent(), fuzzy(fzf, listing(&ranked)));

    screen.press('n');

    closed(&mut screen);
    let listed = std::fs::read_to_string(scratch.path().join("listed")).unwrap();
    assert_eq!(listed, format!("{here}\n/acme/api\n/acme/web\n"));
}

#[test]
fn without_zoxide_fzf_lists_only_the_directory_of_harbour() {
    let scratch = tempfile::tempdir().unwrap();
    let fzf = recording_fzf(scratch.path());
    let mut screen = Screen::picking(placed_agent(), fuzzy(fzf, vec!["acme-nozoxide".into()]));

    screen.press('n');

    closed(&mut screen);
    let listed = std::fs::read_to_string(scratch.path().join("listed")).unwrap();
    assert_eq!(listed, format!("{}\n", here()));
}

#[test]
fn without_fzf_n_starts_the_session_where_harbour_was_opened_at_once() {
    let picker = fuzzy(vec!["acme-nofzf".into()], listing("/acme/web"));
    let mut screen = Screen::picking(placed_agent(), picker);

    screen.press('n');

    let row = format!("{} 1", directory());
    let shown = screen.wait_for_all(&[&row, &format!("acme agent in {}", directory())]);
    assert!(shown.contains(&row), "{shown}");
    assert!(!shown.contains(" directory "), "{shown}");
}

#[test]
fn a_confirmed_directory_gives_its_session_the_focus() {
    needs_a_terminal();
    let mut screen = Screen::picking(reading_agent(), Picker::Command(typed_picker()));
    screen.press('n');
    screen.wait_for("acme picker");
    screen.type_text(&format!("{}\r", here()));
    screen.wait_for("acme agent ready");

    screen.type_text("hello\r");

    let shown = screen.wait_for("acme agent got: hello");
    assert!(shown.contains("acme agent got: hello"), "{shown}");
}

#[test]
fn a_cancelled_picker_leaves_the_focus_on_the_sidebar() {
    needs_a_terminal();
    let mut screen = Screen::picking(reading_agent(), Picker::Command(typed_picker()));
    screen.press('n');
    screen.wait_for("acme picker");
    screen.type_text(&format!("{}\r", here()));
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.press('n');
    screen.wait_for("acme picker");

    screen.type_text("\r");

    let shown = popup_closed(&mut screen, 5 * deadline());
    assert!(starts_focused(&shown), "{shown}");
}

#[test]
fn a_paste_while_the_popup_is_open_reaches_the_picker() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let web = directory_in(scratch.path(), "acme-web");
    let mut screen = Screen::picking(placed_agent(), Picker::Command(typed_picker()));
    screen.press('n');
    screen.wait_for("acme picker");

    screen.app.paste(&web.display().to_string());
    screen.type_text("\r");

    let shown = screen.wait_for("acme-web 1");
    assert!(shown.contains("acme-web 1"), "{shown}");
}

#[test]
fn a_click_in_the_popup_reaches_a_picker_that_asked_for_the_mouse() {
    let mut screen = Screen::picking(placed_agent(), Picker::Command(clickable_picker()));
    screen.press('n');
    screen.wait_for("acme picker");

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 20, 6);

    let shown = screen.wait_for("acme picker got E[<0;");
    assert!(shown.contains("acme picker got E[<0;"), "{shown}");
}

#[test]
fn a_slow_list_holds_up_neither_harbour_nor_the_picker() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let stall = (10 * deadline()).as_secs();
    let listed = scratch.path().join("listed");
    let slow = script(&format!(
        "sleep {stall}; touch '{}'; printf '/acme/web\\n'",
        listed.display()
    ));
    let mut screen = Screen::picking(placed_agent(), fuzzy(first_line_fzf(), slow));

    screen.press('n');

    screen.wait_for("acme picker");
    screen.type_text("x\r");
    screen.wait_for("acme picker read x");
    assert!(!listed.exists());
    let row = format!("{} 1", directory());
    let shown = screen.wait_for(&row);
    assert!(shown.contains(&row), "{shown}");
}

#[test]
fn a_confirmed_directory_that_cannot_start_leaves_the_focus_on_the_sidebar() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let locked = directory_in(scratch.path(), "acme-locked");
    let mut screen = Screen::picking(reading_agent(), Picker::Command(typed_picker()));
    screen.press('n');
    screen.wait_for("acme picker");
    screen.type_text(&format!("{}\r", here()));
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();
    screen.press('n');
    screen.wait_for("acme picker");

    screen.type_text(&format!("{}\r", locked.display()));

    let shown = screen.wait_for("Could not start");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(shown.contains("Could not start"), "{shown}");
    assert!(starts_focused(&shown), "{shown}");
}

#[test]
fn a_picker_whose_interpreter_is_missing_is_named_and_starts_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let broken = scratch.path().join("acme-broken-picker");
    std::fs::write(&broken, "#!/acme/no-interpreter\n").unwrap();
    std::fs::set_permissions(&broken, std::fs::Permissions::from_mode(0o755)).unwrap();
    let picker = Picker::Command(vec![broken.display().to_string()]);
    let mut screen = Screen::picking(placed_agent(), picker);

    screen.press('n');

    let shown = screen.wait_for("Could not start");
    assert!(shown.contains("Could not start"), "{shown}");
    assert!(shown.contains("acme-broken-picker"), "{shown}");
    assert!(shown.contains("No session yet"), "{shown}");
}

#[test]
fn bytes_after_the_first_line_do_not_matter() {
    let scratch = tempfile::tempdir().unwrap();
    let web = directory_in(scratch.path(), "acme-web");
    let printing = script(&format!("printf '%s\\n\\377\\n' '{}'", web.display()));
    let mut screen = Screen::picking(placed_agent(), Picker::Command(printing));

    screen.press('n');

    let shown = screen.wait_for("acme-web 1");
    assert!(shown.contains("acme-web 1"), "{shown}");
}

#[test]
fn picking_the_root_names_the_session_after_the_path() {
    let mut screen = Screen::picking(placed_agent(), Picker::Command(script("echo /")));

    screen.press('n');

    let shown = screen.wait_for("/ 1");
    assert!(shown.contains(" / 1"), "{shown}");
}

#[test]
fn a_directory_name_that_is_not_utf8_reaches_fzf_byte_for_byte() {
    let scratch = tempfile::tempdir().unwrap();
    let from = scratch.path().join(OsStr::from_bytes(b"acme-\xff"));
    let picker = fuzzy(recording_fzf(scratch.path()), listing(""));

    let command = picker.command(&from, &scratch.path().join("picked"));

    assert!(
        command
            .iter()
            .any(|argument| argument.as_bytes() == from.as_os_str().as_bytes())
    );
}

#[test]
fn fzf_is_given_the_options_that_fix_what_it_prints() {
    let scratch = tempfile::tempdir().unwrap();
    let picker = fuzzy(vec!["fzf".to_string()], listing(""));

    let command = picker.command(scratch.path(), &scratch.path().join("picked"));

    let fzf = command
        .iter()
        .position(|argument| argument == "fzf")
        .unwrap();
    assert_eq!(command[fzf + 1], "--no-expect");
    assert_eq!(command[fzf + 2], "--no-print-query");
    assert_eq!(command[fzf + 3], "--no-read0");
    assert_eq!(command[fzf + 4], "--no-print0");
    assert_eq!(command[fzf - 1], "5");
}

#[test]
fn fzf_exiting_with_an_error_is_named_and_starts_nothing() {
    let picker = fuzzy(script("echo 'acme fzf' >&2; exit 2"), listing(""));
    let mut screen = Screen::picking(placed_agent(), picker);

    screen.press('n');

    let shown = screen.wait_for_all(&[
        "Could not start /bin/sh: it exited with 2",
        "No session yet",
    ]);
    assert!(
        shown.contains("Could not start /bin/sh: it exited with 2"),
        "{shown}"
    );
    assert!(shown.contains("No session yet"), "{shown}");
}

#[test]
fn fzf_with_no_match_starts_nothing_and_says_nothing() {
    needs_a_terminal();
    let picker = fuzzy(
        script("echo 'acme fzf' >&2; read -r key </dev/tty; exit 1"),
        listing(""),
    );
    let mut screen = Screen::picking(placed_agent(), picker);
    screen.press('n');
    screen.wait_for("acme fzf");

    screen.type_text("\r");

    let shown = closed(&mut screen);
    assert!(shown.contains("No session yet"), "{shown}");
    assert!(!shown.contains("Could not start"), "{shown}");
}

#[test]
fn fzf_escaped_starts_nothing_and_says_nothing() {
    needs_a_terminal();
    let picker = fuzzy(
        script("echo 'acme fzf' >&2; read -r key </dev/tty; exit 130"),
        listing(""),
    );
    let mut screen = Screen::picking(placed_agent(), picker);
    screen.press('n');
    screen.wait_for("acme fzf");

    screen.type_text("\r");

    let shown = closed(&mut screen);
    assert!(shown.contains("No session yet"), "{shown}");
    assert!(!shown.contains("Could not start"), "{shown}");
}

#[test]
fn a_picked_path_through_dot_dot_starts_in_the_directory_it_names() {
    let scratch = tempfile::tempdir().unwrap();
    let api = directory_in(scratch.path(), "acme-api");
    directory_in(&api, "acme-web");
    let printing = script(&format!("echo '{}/acme-web/..'", api.display()));
    let mut screen = Screen::picking(placed_agent(), Picker::Command(printing));

    screen.press('n');

    let shown = screen.wait_for_all(&["acme-api 1", "acme agent in acme-api"]);
    assert!(shown.contains(" acme-api 1"), "{shown}");
    assert!(shown.contains("acme agent in acme-api"), "{shown}");
}

fn placed_agent() -> Vec<String> {
    script("echo \"acme agent in ${PWD##*/}\"; exec sleep 600")
}

fn typed_picker() -> Vec<String> {
    script("echo 'acme picker' >&2; read -r line; printf '%s\\n' \"$line\"")
}

fn escapable_picker() -> Vec<String> {
    script("stty raw -echo; echo 'acme picker' >&2; dd bs=1 count=1 >/dev/null 2>&1; exit 130")
}

fn clickable_picker() -> Vec<String> {
    script(
        "stty raw -echo; printf '\\033[?1000h\\033[?1006h' >&2; echo 'acme picker' >&2; \
         got=$(dd bs=1 count=5 2>/dev/null | tr '\\033' E); \
         printf 'acme picker got %s\\r\\n' \"$got\" >&2; exec sleep 600",
    )
}

fn fuzzy(fzf: Vec<String>, list: Vec<String>) -> Picker {
    Picker::Fuzzy {
        fzf,
        list,
        here_first: true,
        required: false,
    }
}

fn listing(lines: &str) -> Vec<String> {
    script(&format!("printf '{lines}'"))
}

fn first_line_fzf() -> Vec<String> {
    script(
        "echo 'acme picker' >&2; read -r key </dev/tty; echo \"acme picker read $key\" >&2; head -n 1",
    )
}

fn recording_fzf(scratch: &Path) -> Vec<String> {
    let listed = scratch.join("listed");
    script(&format!("cat > '{}'; exit 1", listed.display()))
}

fn here() -> String {
    home().display().to_string()
}

fn closed(screen: &mut Screen) -> String {
    popup_closed(screen, 5 * deadline());
    screen.wait_for("No session yet")
}

fn popup_closed(screen: &mut Screen, patience: Duration) -> String {
    let until = Instant::now() + patience;
    loop {
        let shown = screen.draw();
        if !shown.contains(" directory ") {
            return shown;
        }
        assert!(
            Instant::now() < until,
            "waited {patience:?} for the picker to close, last screen:\n{shown}"
        );
        sleep(Duration::from_millis(20));
    }
}

fn directory_in(scratch: &Path, name: &str) -> PathBuf {
    let directory = scratch.join(name);
    std::fs::create_dir(&directory).unwrap();
    directory
}
