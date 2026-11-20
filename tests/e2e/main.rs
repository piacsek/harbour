use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

use harbour::tui::{DIR_PICKER, SESSION_COMMAND};

const AGENT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/agent.sh");
const RAW_AGENT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/raw-agent.sh");
const PICKER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/picker.sh");
const FAKE_FZF: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/fzf");
const TOOLS: [&str; 7] = ["cat", "dd", "grep", "mkfifo", "sleep", "stty", "tr"];

#[test]
#[ignore]
fn the_binary_runs_in_a_scratch_tmux_server() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("version", 80, 10);

    server.run(home.path(), &[], &["--version"]);

    let screen = server.wait_for_screen("harbour 0.0.0");
    assert!(screen.contains("harbour 0.0.0"), "{screen}");
}

#[test]
#[ignore]
fn no_argument_on_a_terminal_opens_the_screen_with_the_hint() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("empty", 120, 20);

    server.run(home.path(), &[], &[]);

    let screen = server.wait_for_screen("No session yet");
    assert!(screen.contains("No session yet"), "{screen}");
}

#[test]
#[ignore]
fn q_leaves_the_screen_and_restores_the_terminal() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("quit", 120, 20);
    server.run(home.path(), &[], &[]);
    server.wait_for_screen("press n to start one");

    server.send_keys(&["q"]);

    let screen = server.wait_for_screen("harbour exited 0");
    assert!(screen.contains("harbour exited 0"), "{screen}");
    assert!(!screen.contains("press n to start one"), "{screen}");
}

#[test]
#[ignore]
fn n_starts_a_session_named_after_the_directory_and_shows_it() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("start", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen_all(&["acme agent in acme-api", "acme-api 1"]);
    assert!(screen.contains("acme agent in acme-api"), "{screen}");
    assert!(screen.contains("acme-api 1"), "{screen}");
}

#[test]
#[ignore]
fn n_runs_the_dir_picker_and_starts_the_session_in_the_directory_it_prints() {
    let home = tempfile::tempdir().unwrap();
    let web = home.path().join("acme-web");
    std::fs::create_dir(&web).unwrap();
    let server = Server::start("picker", 160, 20);
    server.run(
        home.path(),
        &[(SESSION_COMMAND, AGENT), (DIR_PICKER, PICKER)],
        &[],
    );
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme picker: type a directory");

    server.send_keys(&["../acme-web", "Enter"]);

    let screen = server.wait_for_screen_all(&["acme agent in acme-web", "acme-web 1"]);
    assert!(screen.contains("acme agent in acme-web"), "{screen}");
    assert!(screen.contains("acme-web 1"), "{screen}");
}

#[test]
#[ignore]
fn an_empty_dir_picker_setting_counts_as_unset() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("empty-picker", 160, 20);
    server.run(
        home.path(),
        &[(SESSION_COMMAND, AGENT), (DIR_PICKER, "")],
        &[],
    );
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen_all(&["acme agent in acme-api", "acme-api 1"]);
    assert!(screen.contains("acme-api 1"), "{screen}");
}

#[test]
#[ignore]
fn a_session_does_not_inherit_the_tmux_of_harbour() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("tmux-env", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen("tmux:[]");
    assert!(screen.contains("tmux:[]"), "{screen}");
}

#[test]
#[ignore]
fn keys_typed_in_the_focused_pane_reach_the_session_until_ctrl_h() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("focus", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent in acme-api");

    server.send_keys(&["Enter", "hello", "Enter", "C-h", "n"]);

    let screen = server.wait_for_screen("acme-api 2");
    assert!(screen.contains("acme-api 2"), "{screen}");
    server.send_keys(&["k"]);
    let screen = server.wait_for_screen("acme agent got: hello");
    assert!(screen.contains("acme agent got: hello"), "{screen}");
}

#[test]
#[ignore]
fn a_terminal_made_narrow_shows_the_sidebar_alone() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("narrow", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent in acme-api");

    server.resize(50, 20);

    let screen = server.wait_for_screen_showing_without("acme-api 1", "acme agent in");
    assert!(screen.contains("acme-api 1"), "{screen}");
    assert!(!screen.contains("acme agent in"), "{screen}");
}

#[test]
#[ignore]
fn a_paste_reaches_the_focused_session_bracketed() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("paste", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, RAW_AGENT)], &[]);
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent ready");
    server.send_keys(&["Enter"]);

    server.paste("hello");

    let screen = server.wait_for_screen("read:^[200~hello^[201~");
    assert!(screen.contains("read:^[200~hello^[201~"), "{screen}");
}

#[test]
#[ignore]
fn a_click_in_the_pane_reaches_the_session() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("mouse", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, RAW_AGENT)], &[]);
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent ready");

    server.send_bytes("\x1b[<0;60;5M");

    let screen = server.wait_for_screen("read:^[<0;6;4M");
    assert!(screen.contains("read:^[<0;6;4M"), "{screen}");
}

#[test]
#[ignore]
fn a_command_that_cannot_start_is_named_in_the_pane() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("missing", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, "acme-agent-missing")], &[]);
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen("Could not start acme-agent-missing");
    assert!(
        screen.contains("Could not start acme-agent-missing"),
        "{screen}"
    );
    assert!(!screen.contains("acme-api 1"), "{screen}");
}

#[test]
#[ignore]
fn q_with_a_running_session_asks_and_y_leaves() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("quit-asks", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent in acme-api");

    server.send_keys(&["q"]);
    let screen = server.wait_for_screen("Quit? y/n");
    assert!(screen.contains("Quit? y/n"), "{screen}");
    server.send_keys(&["y"]);

    let screen = server.wait_for_screen("harbour exited 0");
    assert!(screen.contains("harbour exited 0"), "{screen}");
}

#[test]
#[ignore]
fn the_demo_shows_made_up_sessions_one_of_them_exited() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("demo", 160, 20);

    server.run(home.path(), &[], &["--demo"]);

    let screen =
        server.wait_for_screen_all(&["● acme-api 1", "○ acme-api 2 exited 1", "● acme-api 3"]);
    assert!(screen.contains("● acme-api 1"), "{screen}");
    assert!(screen.contains("○ acme-api 2 exited 1"), "{screen}");
    assert!(screen.contains("● acme-api 3"), "{screen}");
    let screen = server.wait_for_screen("fix the flaky retry test in acme/api");
    assert!(
        screen.contains("fix the flaky retry test in acme/api"),
        "{screen}"
    );
}

#[test]
#[ignore]
fn n_in_the_demo_without_fzf_starts_a_session_at_once() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("demo-no-fzf", 160, 20);
    server.run(home.path(), &[], &["--demo"]);
    server.wait_for_screen("● acme-api 3");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen("acme-api 4");
    assert!(screen.contains("acme-api 4"), "{screen}");
    assert!(!screen.contains("Could not start"), "{screen}");
}

#[test]
#[ignore]
fn n_in_a_deleted_directory_says_the_directory_cannot_be_read() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("deleted", 160, 20);
    server.launch(
        home.path(),
        "rmdir \"$PWD\" &&",
        &[(SESSION_COMMAND, AGENT)],
        &[],
    );
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen("Could not start");
    assert!(screen.contains("could not read the working"), "{screen}");
    assert!(screen.contains("No session yet"), "{screen}");
}

#[test]
#[ignore]
fn n_in_the_demo_lists_made_up_directories_in_fzf() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("demo-picker", 160, 20);
    let path = format!("{FAKE_FZF}:{}", tools(home.path()));
    server.run(home.path(), &[("PATH", &path)], &["--demo"]);
    server.wait_for_screen("● acme-api 3");
    server.send_keys(&["n"]);

    let screen = server.wait_for_screen_all(&["acme fzf lists:", "/home/acme/web"]);
    assert!(screen.contains("acme fzf lists:"), "{screen}");
    assert!(screen.contains("/home/acme/web"), "{screen}");
    let scratch = home
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    assert!(!screen.contains(&scratch), "{screen}");
}

struct Server {
    socket: String,
}

impl Server {
    fn start(name: &str, width: u16, height: u16) -> Self {
        let server = Self {
            socket: format!("harbour-e2e-{name}-{}", std::process::id()),
        };
        let status = server
            .tmux(&[
                "new-session",
                "-d",
                "-s",
                "e2e",
                "-x",
                &width.to_string(),
                "-y",
                &height.to_string(),
                "sleep 600",
            ])
            .status()
            .unwrap_or_else(|err| {
                panic!("the e2e tests drive a scratch tmux server; install tmux (brew install tmux) and rerun `cargo test -- --include-ignored`: {err}")
            });
        assert!(status.success(), "could not start the tmux e2e server");
        server.option(&["default-shell", "/bin/sh"]);
        server.option(&["status", "off"]);
        server
    }

    fn paste(&self, text: &str) {
        let status = self.tmux(&["set-buffer", text]).status().unwrap();
        assert!(status.success(), "could not set the paste buffer");
        let status = self
            .tmux(&["paste-buffer", "-p", "-t", "e2e"])
            .status()
            .unwrap();
        assert!(status.success(), "could not paste");
    }

    fn send_bytes(&self, bytes: &str) {
        let hex: Vec<String> = bytes.bytes().map(|byte| format!("{byte:02x}")).collect();
        let args: Vec<&str> = ["send-keys", "-t", "e2e", "-H"]
            .into_iter()
            .chain(hex.iter().map(String::as_str))
            .collect();
        let status = self.tmux(&args).status().unwrap();
        assert!(status.success(), "could not send {bytes:?}");
    }

    fn resize(&self, width: u16, height: u16) {
        let status = self
            .tmux(&[
                "resize-window",
                "-t",
                "e2e",
                "-x",
                &width.to_string(),
                "-y",
                &height.to_string(),
            ])
            .status()
            .unwrap();
        assert!(status.success(), "could not resize the window");
    }

    fn option(&self, args: &[&str]) {
        let status = self
            .tmux(&[&["set-option", "-g"], args].concat())
            .status()
            .unwrap();
        assert!(status.success(), "could not set {args:?}");
    }

    fn tmux(&self, args: &[&str]) -> Command {
        let mut command = Command::new("tmux");
        command
            .arg("-L")
            .arg(&self.socket)
            .arg("-f")
            .arg("/dev/null")
            .args(args);
        command
    }

    fn run(&self, home: &Path, env: &[(&str, &str)], args: &[&str]) {
        self.launch(home, "", env, args);
    }

    fn launch(&self, home: &Path, before: &str, env: &[(&str, &str)], args: &[&str]) {
        let directory = home.join("acme-api");
        std::fs::create_dir_all(&directory).unwrap();
        let mut words = vec![
            "cd".to_string(),
            quoted(&directory.display().to_string()),
            "&&".to_string(),
            before.to_string(),
            "env".to_string(),
            "-u".to_string(),
            "XDG_CONFIG_HOME".to_string(),
            "-u".to_string(),
            "XDG_STATE_HOME".to_string(),
            "-u".to_string(),
            DIR_PICKER.to_string(),
            "-u".to_string(),
            SESSION_COMMAND.to_string(),
            quoted(&format!("HOME={}", home.display())),
            quoted(&format!("PATH={}", tools(home))),
        ];
        words.extend(
            env.iter()
                .map(|(key, value)| quoted(&format!("{key}={value}"))),
        );
        words.push(quoted(env!("CARGO_BIN_EXE_harbour")));
        words.extend(args.iter().map(|arg| quoted(arg)));
        let command = format!(
            "{}; echo \"harbour exited $?\"; exec sleep 600",
            words.join(" ")
        );
        let status = self
            .tmux(&["respawn-pane", "-k", "-t", "e2e", &command])
            .status()
            .unwrap();
        assert!(status.success(), "could not start harbour in the pane");
    }

    fn send_keys(&self, keys: &[&str]) {
        for key in keys {
            let status = self
                .tmux(&["send-keys", "-t", "e2e", key])
                .status()
                .unwrap();
            assert!(status.success(), "could not send {key}");
            sleep(Duration::from_millis(100));
        }
    }

    fn screen(&self) -> String {
        let output = self
            .tmux(&["capture-pane", "-p", "-t", "e2e"])
            .output()
            .unwrap();
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    fn wait_for_screen(&self, needle: &str) -> String {
        self.wait_for_screen_all(&[needle])
    }

    fn wait_for_screen_all(&self, needles: &[&str]) -> String {
        self.wait_until(|screen| needles.iter().all(|needle| screen.contains(needle)))
    }

    fn wait_for_screen_showing_without(&self, shown: &str, gone: &str) -> String {
        self.wait_until(|screen| screen.contains(shown) && !screen.contains(gone))
    }

    fn wait_until(&self, done: impl Fn(&str) -> bool) -> String {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut screen = self.screen();
        while !done(&screen) && Instant::now() < deadline {
            sleep(Duration::from_millis(50));
            screen = self.screen();
        }
        screen
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.tmux(&["kill-server"]).status();
    }
}

fn quoted(word: &str) -> String {
    format!("'{}'", word.replace('\'', r"'\''"))
}

fn tools(home: &Path) -> String {
    let bin = home.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    for tool in TOOLS {
        let found = ["/bin", "/usr/bin"]
            .iter()
            .map(|directory| Path::new(directory).join(tool))
            .find(|path| path.exists())
            .unwrap();
        let link = bin.join(tool);
        if !link.exists() {
            std::os::unix::fs::symlink(found, link).unwrap();
        }
    }
    bin.display().to_string()
}
