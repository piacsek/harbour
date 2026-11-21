use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use harbour::app::App;
use harbour::editor::Program;
use harbour::handoff::{Reply, Verb};
use harbour::input::ESCAPED;
use harbour::picker::Picker;

use crate::finding::FINDER;
use crate::handoff::soon;
use crate::keeping::titled;
use crate::remembering::{ID, OTHER_ID, Scratch, TITLE};
use crate::{Screen, home, pane_corner, script};

const GONE: &str = "That session is gone";
const NTH_FINDER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/nth-finder/fzf");
const OLD_FINDER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/old-finder/fzf");
const WAITING: &str = "harbour is waiting for an answer; try again after it";

#[test]
fn a_session_that_ended_while_the_list_was_open_is_selected_as_its_exited_card() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = stoppable_sessions(scratch.path(), 3);
    alt_s(&mut screen);
    screen.wait_for("acme fzf ready");

    stop(scratch.path(), 1);
    screen.type_text("home 1\r");

    let shown = screen.wait_until("the popup to close", |shown| !shown.contains("acme fzf"));
    assert!(selected_line(&shown).contains("acme-home 1"), "{shown}");
    assert!(!shown.contains(GONE), "{shown}");
}

#[test]
fn a_stopped_card_merged_while_the_list_was_open_jumps_to_the_session_that_holds_it() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, &titled(TITLE));
    let go = scratch.dir.path().join("go");
    let mut screen = Screen::of(
        scratch
            .app(registering_on(&go, &scratch.sessions()))
            .with_picker(finder()),
    );
    screen.wait_for("stopped");
    screen.press('n');
    screen.wait_for(LIVE);
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    alt_s(&mut screen);
    screen.wait_for("acme fzf ready");

    fs::write(&go, "").unwrap();
    screen.wait_until("the stopped card to merge", |shown| {
        !shown.contains("stopped")
    });
    screen.type_text(&format!("{TITLE}\r"));
    screen.wait_until("the popup to close", |shown| !shown.contains("acme fzf"));
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent got: hi");
    assert!(!shown.contains(GONE), "{shown}");
}

#[test]
fn a_remembered_session_that_exited_resumes_when_picked() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let count = scratch.dir.path().join("count");
    let command = script(&format!(
        "n=$(($(cat '{count}' 2>/dev/null || echo 0) + 1)); echo $n > '{count}'; \
         echo \"acme agent run $n $0 $1\"",
        count = count.display()
    ));
    let mut screen = Screen::of(scratch.app(command).with_picker(finder()));
    screen.wait_for(TITLE);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.wait_for("exited");

    pick(&mut screen, TITLE);

    let shown = screen.wait_for("acme agent run 2");
    assert!(
        shown.contains(&format!("acme agent run 2 --resume {ID}")),
        "{shown}"
    );
}

#[test]
fn a_word_at_the_end_of_a_long_title_is_found_in_a_narrow_popup() {
    let (_scratch, mut screen) = long_titled(
        "acme fix the flaky login test on the checkout page before zanzibar",
        59,
    );

    pick(&mut screen, "zanzibar");

    let shown = screen.wait_for(&format!("acme agent --resume {ID}"));
    assert!(
        shown.contains(&format!("acme agent --resume {ID}")),
        "{shown}"
    );
}

#[test]
fn a_word_at_the_end_of_a_long_cjk_title_is_found_in_a_narrow_popup() {
    let (_scratch, mut screen) = long_titled("港口の修正をする長い名前の最後にある灯台", 59);

    pick(&mut screen, "灯台");

    let shown = screen.wait_for(&format!("acme agent --resume {ID}"));
    assert!(
        shown.contains(&format!("acme agent --resume {ID}")),
        "{shown}"
    );
}

#[test]
fn a_line_fzf_prints_without_its_number_names_fzf_default_opts() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = stoppable_sessions(scratch.path(), 3);

    pick(&mut screen, "cut:home 2");

    let shown = screen.wait_until("the message", |shown| {
        sidebar_text(shown).contains("FZF_DEFAULT_OPTS")
    });
    assert!(selected_line(&shown).contains("acme-home 3"), "{shown}");
}

#[test]
fn an_accept_nth_in_the_users_options_still_jumps_to_the_pick() {
    let scratch = tempfile::tempdir().unwrap();
    let app = App::new(stoppable_agent(scratch.path()))
        .with_picker(Picker::Command(vec![NTH_FINDER.into()]));
    let mut screen = started(app, 3);

    pick(&mut screen, "home 1");

    let shown = screen.wait_until("the popup to close", |shown| !shown.contains("acme fzf"));
    assert!(selected_line(&shown).contains("acme-home 1"), "{shown}");
}

#[test]
fn an_fzf_without_accept_nth_still_opens_and_jumps_to_the_pick() {
    let scratch = tempfile::tempdir().unwrap();
    let app = App::new(stoppable_agent(scratch.path()))
        .with_picker(Picker::Command(vec![OLD_FINDER.into()]));
    let mut screen = started(app, 3);

    pick(&mut screen, "home 1");

    let shown = screen.wait_until("the popup to close", |shown| !shown.contains("acme fzf"));
    assert!(selected_line(&shown).contains("acme-home 1"), "{shown}");
    assert!(!shown.contains("Could not start"), "{shown}");
}

#[test]
fn a_session_that_had_exited_before_the_list_opened_is_selected() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = stoppable_sessions(scratch.path(), 3);
    stop(scratch.path(), 1);
    screen.wait_for("exited");
    alt_s(&mut screen);
    screen.wait_for("acme fzf ready");

    screen.type_text("home 1\r");

    let shown = screen.wait_until("the popup to close", |shown| !shown.contains("acme fzf"));
    assert!(selected_line(&shown).contains("acme-home 1"), "{shown}");
    assert!(!shown.contains(GONE), "{shown}");
}

#[test]
fn a_stopped_card_picked_resumes() {
    let scratch = Scratch::new();
    scratch.remember(ID, "The first one", 0);
    scratch.remember(OTHER_ID, TITLE, 1);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    scratch.transcript(OTHER_ID, r#"{"type":"user","message":{}}"#);
    let mut screen = Screen::of(scratch.app(resuming_agent()).with_picker(finder()));
    screen.wait_for(TITLE);
    alt_s(&mut screen);
    screen.wait_for("acme fzf ready");

    screen.type_text(&format!("{TITLE}\r"));

    let shown = screen.wait_for(&format!("acme agent --resume {OTHER_ID}"));
    assert!(
        shown.contains(&format!("acme agent --resume {OTHER_ID}")),
        "{shown}"
    );
}

#[test]
fn a_stopped_card_whose_directory_is_gone_says_why_and_leaves_the_sidebar_focused() {
    let (scratch, mut screen) = live_beside_stopped(120);
    fs::remove_dir_all(scratch.checkout()).unwrap();

    pick(&mut screen, "The first one");

    let shown = screen.wait_for("Not a directory");
    assert_sidebar_focused(&mut screen, &shown);
}

#[test]
fn a_stopped_card_whose_directory_is_gone_leaves_the_sidebar_focused_when_narrow() {
    let (scratch, mut screen) = live_beside_stopped(59);
    fs::remove_dir_all(scratch.checkout()).unwrap();

    pick(&mut screen, "The first one");

    let shown = screen.wait_for("Not a directory");
    assert_sidebar_focused(&mut screen, &shown);
}

#[test]
fn a_stopped_card_open_elsewhere_says_why_and_leaves_the_sidebar_focused() {
    let (scratch, mut screen) = live_beside_stopped(120);
    hold(&scratch);

    pick(&mut screen, "The first one");

    let shown = screen.wait_for("is open elsewhere");
    assert_sidebar_focused(&mut screen, &shown);
}

#[test]
fn a_picker_that_is_not_fzf_says_the_finder_needs_fzf() {
    let mut screen = Screen::of(
        App::new(script("echo 'acme agent ready'; exec sleep 600"))
            .with_picker(Picker::Command(script("echo ."))),
    );
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    alt_s(&mut screen);

    let shown = screen.wait_for("needs fzf");
    assert!(shown.contains("needs fzf, not /bin/sh"), "{shown}");
}

#[test]
fn a_missing_fzf_says_it_could_not_start() {
    let mut screen = Screen::of(
        App::new(script("echo 'acme agent ready'; exec sleep 600")).with_picker(Picker::Fuzzy {
            fzf: vec!["/acme-missing/fzf".to_string()],
            list: vec!["true".to_string()],
            here_first: true,
            required: false,
        }),
    );
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    alt_s(&mut screen);

    let shown = screen.wait_for("Could not start");
    assert!(
        shown.contains("Could not start /acme-missing/fzf"),
        "{shown}"
    );
}

#[test]
fn fzf_exiting_with_2_says_it_could_not_start() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = stoppable_sessions(scratch.path(), 1);
    alt_s(&mut screen);
    screen.wait_for("acme fzf ready");

    screen.type_text("broken\r");

    let shown = screen.wait_for("Could not start");
    assert!(sidebar_text(&shown).contains("it exited with 2"), "{shown}");
}

#[test]
fn alt_s_from_an_editor_reaches_the_editor_and_opens_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let editor = Program::new("acme-edit", asking_editor());
    let mut screen = Screen::of(
        App::new(stoppable_agent(scratch.path()))
            .with_editor(editor)
            .with_picker(finder()),
    );
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor ready");

    alt_s(&mut screen);

    let shown = screen.wait_for("acme editor read:");
    assert!(shown.contains("acme editor read: 1b"), "{shown}");
    assert!(!shown.contains("acme fzf"), "{shown}");
}

#[test]
fn alt_s_read_after_an_escape_reaches_the_session_and_opens_nothing() {
    let mut screen = Screen::of(App::new(raw_reader(2)).with_picker(finder()));
    screen.press('n');
    screen.wait_for("acme agent ready");

    screen.app.key(KeyEvent::new_with_kind_and_state(
        KeyCode::Char('s'),
        KeyModifiers::ALT,
        KeyEventKind::Press,
        ESCAPED,
    ));

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 1b 73"), "{shown}");
    assert!(!shown.contains("acme fzf"), "{shown}");
}

#[test]
fn alt_s_with_no_card_does_nothing() {
    let mut screen = Screen::of(
        App::new(script("echo 'acme agent ready'; exec sleep 600")).with_picker(finder()),
    );

    alt_s(&mut screen);
    screen.press('n');

    let shown = screen.wait_for("acme agent ready");
    assert!(!shown.contains("acme fzf"), "{shown}");
}

#[test]
fn alt_s_with_caps_lock_opens_the_list() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = stoppable_sessions(scratch.path(), 1);

    screen.key(KeyCode::Char('S'), KeyModifiers::ALT);

    let shown = screen.wait_for("acme fzf ready");
    assert!(shown.contains("acme fzf ready"), "{shown}");
}

#[test]
fn the_keys_harbour_binds_go_to_fzf_while_the_list_is_open() {
    let scratch = tempfile::tempdir().unwrap();
    let app = App::new(stoppable_agent(scratch.path()))
        .with_picker(finder())
        .with_usage_place(Some(scratch.path().join("usage")))
        .with_usage(script("echo 'acme usage open'; exec sleep 600"));
    let mut screen = started(app, 3);
    alt_s(&mut screen);
    screen.wait_for("acme fzf ready");

    screen.key(KeyCode::Char('2'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('z'), KeyModifiers::ALT);
    screen.press('u');
    screen.type_text("\r");

    let shown = screen.wait_until("the popup to close", |shown| !shown.contains("acme fzf"));
    assert!(selected_line(&shown).contains("acme-home 3"), "{shown}");
    assert_eq!(corners(&shown), 2, "{shown}");
    assert!(!shown.contains("acme usage open"), "{shown}");
}

#[test]
fn the_width_keys_do_nothing_while_the_list_is_open() {
    let scratch = tempfile::tempdir().unwrap();
    let app = App::new(stoppable_agent(scratch.path()))
        .with_picker(finder())
        .with_sidebar_width(30);
    let mut screen = started(app, 2);
    alt_s(&mut screen);
    screen.wait_for("acme fzf ready");

    screen.key(KeyCode::Char('='), KeyModifiers::ALT);
    screen.key(KeyCode::Char('='), KeyModifiers::ALT);
    screen.key(KeyCode::Char('-'), KeyModifiers::ALT);
    screen.type_text("\r");

    let shown = screen.wait_until("the popup to close", |shown| !shown.contains("acme fzf"));
    assert_eq!(pane_corner(&shown), 30, "{shown}");
}

#[test]
fn a_paste_goes_to_fzf_while_the_list_is_open() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = stoppable_sessions(scratch.path(), 3);
    alt_s(&mut screen);
    screen.wait_for("acme fzf ready");

    screen.app.paste("home 2");
    screen.type_text("\r");
    screen.wait_until("the popup to close", |shown| !shown.contains("acme fzf"));
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 2 got:");
    assert!(shown.contains("acme agent 2 got: hi"), "{shown}");
}

#[test]
fn harbour_edit_is_refused_while_the_list_is_open() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = stoppable_sessions(scratch.path(), 1);
    alt_s(&mut screen);
    screen.wait_for("acme fzf ready");

    let reply = screen.app.hand(Verb::Open, &notes(), soon());

    assert_eq!(reply, Reply::Refused(WAITING.to_string()));
}

#[test]
fn a_pick_while_zoomed_stays_zoomed() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = stoppable_sessions(scratch.path(), 3);
    screen.key(KeyCode::Char('z'), KeyModifiers::ALT);
    alt_s(&mut screen);
    screen.wait_for("acme fzf ready");

    screen.type_text("home 2\r");
    screen.wait_until("the popup to close", |shown| !shown.contains("acme fzf"));
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 2 got:");
    assert_eq!(corners(&shown), 1, "{shown}");
}

fn alt_s(screen: &mut Screen) {
    screen.key(KeyCode::Char('s'), KeyModifiers::ALT);
}

fn pick(screen: &mut Screen, text: &str) {
    alt_s(screen);
    screen.wait_for("acme fzf ready");
    screen.type_text(&format!("{text}\r"));
}

fn notes() -> PathBuf {
    let notes = home().canonicalize().unwrap().join("notes.md");
    fs::write(&notes, "acme notes").unwrap();
    notes
}

fn finder() -> Picker {
    Picker::Command(vec![FINDER.into()])
}

fn stoppable_sessions(scratch: &Path, count: usize) -> Screen {
    started(
        App::new(stoppable_agent(scratch)).with_picker(finder()),
        count,
    )
}

fn started(app: App, count: usize) -> Screen {
    let mut screen = Screen::of(app);
    for n in 1..=count {
        screen.press('n');
        screen.wait_for(&format!("acme agent {n}"));
        screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    }
    screen
}

fn stoppable_agent(scratch: &Path) -> Vec<String> {
    let dir = scratch.display();
    script(&format!(
        "n=$(($(cat '{dir}/count' 2>/dev/null || echo 0) + 1)); echo $n > '{dir}/count'; \
         echo $$ > '{dir}/pid'$n; echo \"acme agent $n\"; \
         (while [ ! -e '{dir}/stop'$n ]; do sleep 0.05; done; kill $$) & \
         while read -r line; do echo \"acme agent $n got: $line\"; done; exec sleep 600"
    ))
}

fn stop(scratch: &Path, n: usize) {
    fs::write(scratch.join(format!("stop{n}")), "").unwrap();
    let pid = fs::read_to_string(scratch.join(format!("pid{n}"))).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !ended(pid.trim()) {
        assert!(Instant::now() < deadline, "agent {n} did not end");
        sleep(Duration::from_millis(20));
    }
}

fn ended(pid: &str) -> bool {
    let out = Command::new("ps")
        .args(["-o", "stat=", "-p", pid])
        .output()
        .unwrap();
    let stat = String::from_utf8_lossy(&out.stdout);
    stat.trim().is_empty() || stat.trim().starts_with('Z')
}

fn resuming_agent() -> Vec<String> {
    script(
        "echo \"acme agent $0 $1 in $(basename \"$PWD\")\"; \
         while :; do read -r line || continue; echo \"acme agent got: $line\"; done",
    )
}

fn asking_editor() -> Vec<String> {
    script(
        "stty raw -echo; printf '\\033[>4;2m' >&2; echo 'acme editor ready' >&2; \
         dd bs=1 count=1 2>/dev/null | od -An -tx1 | tr -s ' \\n' '  ' | \
         sed 's/^/acme editor read:/' >&2; exec sleep 600",
    )
}

fn raw_reader(length: usize) -> Vec<String> {
    script(&format!(
        "stty raw -echo; echo 'acme agent ready' >&2; \
         dd bs=1 count={length} 2>/dev/null | od -An -tx1 | tr -s ' \\n' '  ' | \
         sed 's/^/acme agent read:/' >&2; exec sleep 600"
    ))
}

const LIVE: &str = "acme agent live";

fn live_beside_stopped(width: u16) -> (Scratch, Screen) {
    let scratch = Scratch::new();
    scratch.remember(ID, "The first one", 0);
    scratch.remember(OTHER_ID, TITLE, 1);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    scratch.transcript(OTHER_ID, r#"{"type":"user","message":{}}"#);
    let command = script(&format!("[ \"$1\" ] || echo '{LIVE}'; exec sleep 600"));
    let mut screen = Screen::of(scratch.app(command).with_picker(finder()));
    screen.resize(width, 20);
    screen.wait_for(TITLE);
    screen.press('n');
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.wait_for(LIVE);
    (scratch, screen)
}

fn registering_on(go: &Path, sessions: &Path) -> Vec<String> {
    script(&format!(
        "echo '{LIVE}'; while [ ! -e '{go}' ]; do sleep 0.05; done; \
         printf '%s' '{{\"status\":\"busy\",\"sessionId\":\"{ID}\"}}' > '{sessions}'/$$.json; \
         while read -r line; do echo \"acme agent got: $line\"; done; exec sleep 600",
        go = go.display(),
        sessions = sessions.display()
    ))
}

fn long_titled(title: &str, width: u16) -> (Scratch, Screen) {
    let scratch = Scratch::new();
    scratch.remember(ID, title, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let mut screen = Screen::of(scratch.app(resuming_agent()).with_picker(finder()));
    screen.resize(width, 20);
    screen.wait_for("stopped");
    (scratch, screen)
}

fn hold(scratch: &Scratch) {
    fs::write(
        scratch
            .sessions()
            .join(format!("{}.json", std::process::id())),
        format!(r#"{{"status":"idle","sessionId":"{ID}"}}"#),
    )
    .unwrap();
}

fn assert_sidebar_focused(screen: &mut Screen, shown: &str) {
    assert!(
        shown.lines().next().unwrap().starts_with("┏ sessions"),
        "{shown}"
    );
    let before = selected_line(shown);
    screen.press('j');
    let moved = screen.draw();
    assert_ne!(selected_line(&moved), before, "{moved}");
}

fn selected_line(shown: &str) -> String {
    shown
        .lines()
        .find(|line| line.contains('▌'))
        .unwrap_or_default()
        .to_string()
}

fn corners(shown: &str) -> usize {
    shown
        .lines()
        .next()
        .unwrap_or_default()
        .chars()
        .filter(|c| matches!(c, '┌' | '┏'))
        .count()
}

fn sidebar_text(shown: &str) -> String {
    shown
        .lines()
        .filter_map(|line| line.split('┃').nth(1))
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join(" ")
}
