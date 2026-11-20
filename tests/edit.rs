use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime};

use harbour::handoff::{Listener, Reply, Verb};

#[test]
fn a_directory_is_refused() {
    let scratch = tempfile::tempdir().unwrap();
    fs::create_dir(scratch.path().join("acme-api")).unwrap();

    let output = edit(scratch.path(), "acme-api");

    assert_eq!(stderr(&output), "harbour: acme-api is a directory\n");
}

#[test]
fn a_directory_exits_one() {
    let scratch = tempfile::tempdir().unwrap();
    fs::create_dir(scratch.path().join("acme-api")).unwrap();

    assert_eq!(edit(scratch.path(), "acme-api").status.code(), Some(1));
}

#[test]
fn a_file_in_a_missing_folder_is_refused() {
    let scratch = tempfile::tempdir().unwrap();

    let output = edit(scratch.path(), "acme-api/src/main.rs");

    assert_eq!(
        stderr(&output),
        "harbour: acme-api/src/main.rs: No such file or directory\n"
    );
}

#[test]
fn a_file_in_a_missing_folder_exits_one() {
    let scratch = tempfile::tempdir().unwrap();

    let output = edit(scratch.path(), "acme-api/src/main.rs");

    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn a_path_longer_than_4096_bytes_is_refused() {
    let scratch = tempfile::tempdir().unwrap();
    let long = format!("{}/main.rs", "a".repeat(4096));

    let output = edit(scratch.path(), &long);

    assert_eq!(
        stderr(&output),
        "harbour: the path is longer than 4096 bytes\n"
    );
}

#[test]
fn with_no_harbour_running_a_file_is_refused() {
    let scratch = tempfile::tempdir().unwrap();
    fs::write(scratch.path().join("main.rs"), "").unwrap();

    let output = edit(scratch.path(), "main.rs");

    assert_eq!(stderr(&output), "harbour: no harbour is running\n");
}

#[test]
fn with_no_harbour_running_a_file_exits_one() {
    let scratch = tempfile::tempdir().unwrap();
    fs::write(scratch.path().join("main.rs"), "").unwrap();

    assert_eq!(edit(scratch.path(), "main.rs").status.code(), Some(1));
}

#[test]
fn with_no_harbour_running_nothing_is_printed_on_stdout() {
    let scratch = tempfile::tempdir().unwrap();
    fs::write(scratch.path().join("main.rs"), "").unwrap();

    assert!(edit(scratch.path(), "main.rs").stdout.is_empty());
}

#[test]
fn a_harbour_holding_the_file_is_asked_then_told_to_open_it() {
    let scratch = tempfile::tempdir().unwrap();
    let file = existing(scratch.path(), "main.rs");

    let (output, heard) = served(
        scratch.path(),
        "main.rs",
        vec![harbour(scratch.path(), |verb| match verb {
            Verb::Ask => Reply::Holds(SystemTime::now()),
            Verb::Open => Reply::Opened,
        })],
    );

    assert_eq!(
        heard[0],
        vec![(Verb::Ask, file.clone()), (Verb::Open, file)]
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stderr(&output), "");
    assert!(output.stdout.is_empty());
}

#[test]
fn a_file_that_does_not_exist_yet_is_sent_in_its_folder() {
    let scratch = tempfile::tempdir().unwrap();
    let folder = scratch.path().canonicalize().unwrap();

    let (_, heard) = served(
        scratch.path(),
        "notes.md",
        vec![harbour(scratch.path(), |_| Reply::None)],
    );

    assert_eq!(heard[0], vec![(Verb::Ask, folder.join("notes.md"))]);
}

#[test]
fn a_name_with_spaces_quotes_bars_percents_and_a_newline_arrives_whole() {
    let scratch = tempfile::tempdir().unwrap();
    let name = "a %b#c|d'e\"f\nname.rs";
    let file = existing(scratch.path(), name);

    let (_, heard) = served(
        scratch.path(),
        name,
        vec![harbour(scratch.path(), |_| Reply::None)],
    );

    assert_eq!(heard[0], vec![(Verb::Ask, file)]);
}

#[test]
fn a_harbour_holding_nothing_leaves_the_file_unopened() {
    let scratch = tempfile::tempdir().unwrap();
    let file = existing(scratch.path(), "main.rs");

    let (output, heard) = served(
        scratch.path(),
        "main.rs",
        vec![harbour(scratch.path(), |_| Reply::None)],
    );

    assert_eq!(heard[0], vec![(Verb::Ask, file.clone())]);
    assert_eq!(
        stderr(&output),
        format!("harbour: no harbour session holds {}\n", file.display())
    );
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn a_refusal_is_printed_and_exits_one() {
    let scratch = tempfile::tempdir().unwrap();
    existing(scratch.path(), "main.rs");

    let (output, _) = served(
        scratch.path(),
        "main.rs",
        vec![harbour(scratch.path(), |verb| match verb {
            Verb::Ask => Reply::Holds(SystemTime::now()),
            Verb::Open => Reply::Refused("acme refusal".to_string()),
        })],
    );

    assert_eq!(stderr(&output), "harbour: acme refusal\n");
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn an_open_that_is_never_answered_says_so() {
    let scratch = tempfile::tempdir().unwrap();
    existing(scratch.path(), "main.rs");

    let (output, _) = served(
        scratch.path(),
        "main.rs",
        vec![silent_on_open(scratch.path())],
    );

    assert_eq!(stderr(&output), "harbour: no answer from harbour\n");
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn of_two_harbours_the_one_whose_session_was_used_last_opens_the_file() {
    let scratch = tempfile::tempdir().unwrap();
    let file = existing(scratch.path(), "main.rs");
    let earlier = SystemTime::now() - Duration::from_secs(60);

    let (output, heard) = served(
        scratch.path(),
        "main.rs",
        vec![
            harbour(scratch.path(), move |verb| match verb {
                Verb::Ask => Reply::Holds(earlier),
                Verb::Open => Reply::Opened,
            }),
            harbour(scratch.path(), |verb| match verb {
                Verb::Ask => Reply::Holds(SystemTime::now()),
                Verb::Open => Reply::Opened,
            }),
        ],
    );

    assert_eq!(heard[0], vec![(Verb::Ask, file.clone())]);
    assert_eq!(
        heard[1],
        vec![(Verb::Ask, file.clone()), (Verb::Open, file)]
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn a_harbour_holding_nothing_does_not_hide_one_that_holds_the_file() {
    let scratch = tempfile::tempdir().unwrap();
    let file = existing(scratch.path(), "main.rs");

    let (_, heard) = served(
        scratch.path(),
        "main.rs",
        vec![
            harbour(scratch.path(), |_| Reply::None),
            harbour(scratch.path(), |verb| match verb {
                Verb::Ask => Reply::Holds(SystemTime::now()),
                Verb::Open => Reply::Opened,
            }),
        ],
    );

    assert_eq!(
        heard[1],
        vec![(Verb::Ask, file.clone()), (Verb::Open, file)]
    );
}

#[test]
fn the_socket_of_a_harbour_that_died_counts_as_no_harbour() {
    let scratch = tempfile::tempdir().unwrap();
    existing(scratch.path(), "main.rs");
    drop(UnixListener::bind(folder(scratch.path()).join("99999-0.sock")).unwrap());

    let output = edit(scratch.path(), "main.rs");

    assert_eq!(stderr(&output), "harbour: no harbour is running\n");
}

#[test]
fn a_socket_that_never_answers_counts_as_no_harbour_within_seconds() {
    let scratch = tempfile::tempdir().unwrap();
    existing(scratch.path(), "main.rs");
    let _silent = UnixListener::bind(folder(scratch.path()).join("99999-0.sock")).unwrap();
    let started = Instant::now();

    let output = edit(scratch.path(), "main.rs");

    assert_eq!(stderr(&output), "harbour: no harbour is running\n");
    assert!(
        started.elapsed() < Duration::from_secs(4),
        "{:?}",
        started.elapsed()
    );
}

#[test]
fn a_socket_not_named_by_harbour_is_never_asked() {
    let scratch = tempfile::tempdir().unwrap();
    existing(scratch.path(), "main.rs");
    let _other = UnixListener::bind(folder(scratch.path()).join("acme.sock")).unwrap();

    let output = edit(scratch.path(), "main.rs");

    assert_eq!(stderr(&output), "harbour: no harbour is running\n");
}

#[test]
fn a_folder_open_to_others_is_refused() {
    let scratch = tempfile::tempdir().unwrap();
    existing(scratch.path(), "main.rs");
    let folder = folder(scratch.path());
    fs::set_permissions(&folder, fs::Permissions::from_mode(0o755)).unwrap();

    let output = edit(scratch.path(), "main.rs");

    assert_eq!(
        stderr(&output),
        format!(
            "harbour: {} is not private; refusing to use it\n",
            folder.display()
        )
    );
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn a_folder_that_is_a_symlink_is_refused() {
    let scratch = tempfile::tempdir().unwrap();
    existing(scratch.path(), "main.rs");
    let elsewhere = scratch.path().join("elsewhere");
    fs::create_dir(&elsewhere).unwrap();
    fs::set_permissions(&elsewhere, fs::Permissions::from_mode(0o700)).unwrap();
    let folder = scratch.path().join("state/harbour/edit");
    fs::create_dir_all(folder.parent().unwrap()).unwrap();
    symlink(&elsewhere, &folder).unwrap();

    let output = edit(scratch.path(), "main.rs");

    assert_eq!(
        stderr(&output),
        format!(
            "harbour: {} is not private; refusing to use it\n",
            folder.display()
        )
    );
}

#[test]
fn a_symlink_to_a_harbour_elsewhere_is_never_asked() {
    let scratch = tempfile::tempdir().unwrap();
    existing(scratch.path(), "main.rs");
    let elsewhere = scratch.path().join("elsewhere");
    let away = listening_in(&elsewhere, Box::new(|_| Some(Reply::Opened)));
    let socket = fs::read_dir(&elsewhere)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    symlink(&socket, folder(scratch.path()).join("99999-0.sock")).unwrap();

    let (output, heard) = served(scratch.path(), "main.rs", vec![away]);

    assert!(heard[0].is_empty());
    assert_eq!(stderr(&output), "harbour: no harbour is running\n");
}

fn edit(scratch: &Path, file: &str) -> Output {
    command(scratch, file).output().unwrap()
}

fn command(scratch: &Path, file: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_harbour"));
    command
        .args(["edit", file])
        .env("HOME", scratch)
        .env("XDG_STATE_HOME", scratch.join("state"))
        .current_dir(scratch);
    command
}

type Answer = Box<dyn Fn(Verb) -> Option<Reply>>;

fn harbour(scratch: &Path, answer: impl Fn(Verb) -> Reply + 'static) -> (Listener, Answer) {
    listening(scratch, Box::new(move |verb| Some(answer(verb))))
}

fn silent_on_open(scratch: &Path) -> (Listener, Answer) {
    listening(
        scratch,
        Box::new(|verb| match verb {
            Verb::Ask => Some(Reply::Holds(SystemTime::now())),
            Verb::Open => None,
        }),
    )
}

fn listening(scratch: &Path, answer: Answer) -> (Listener, Answer) {
    listening_in(&scratch.join("state/harbour/edit"), answer)
}

fn listening_in(dir: &Path, answer: Answer) -> (Listener, Answer) {
    let mut listener = Listener::new(dir);
    assert!(listener.incoming().is_none());
    (listener, answer)
}

fn served(
    scratch: &Path,
    file: &str,
    mut harbours: Vec<(Listener, Answer)>,
) -> (Output, Vec<Vec<(Verb, PathBuf)>>) {
    let mut child = spawned(scratch, file);
    let mut heard = vec![Vec::new(); harbours.len()];
    let deadline = Instant::now() + Duration::from_secs(10);
    while child.try_wait().unwrap().is_none() && Instant::now() < deadline {
        for (index, (listener, answer)) in harbours.iter_mut().enumerate() {
            if let Some(request) = listener.incoming() {
                heard[index].push((request.verb, request.file.clone()));
                if let Some(reply) = answer(request.verb) {
                    request.answer(reply);
                }
            }
        }
        sleep(Duration::from_millis(5));
    }
    (child.wait_with_output().unwrap(), heard)
}

fn spawned(scratch: &Path, file: &str) -> Child {
    command(scratch, file)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

fn existing(scratch: &Path, name: &str) -> PathBuf {
    let file = scratch.join(name);
    fs::write(&file, "").unwrap();
    file.canonicalize().unwrap()
}

fn folder(scratch: &Path) -> PathBuf {
    let folder = scratch.join("state/harbour/edit");
    fs::create_dir_all(&folder).unwrap();
    fs::set_permissions(&folder, fs::Permissions::from_mode(0o700)).unwrap();
    folder
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
