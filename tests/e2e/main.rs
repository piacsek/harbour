use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

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

    let screen = server.wait_for_screen("press n to start one");
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
        let mut words = vec![
            "env".to_string(),
            "-u".to_string(),
            "XDG_CONFIG_HOME".to_string(),
            "-u".to_string(),
            "XDG_STATE_HOME".to_string(),
            quoted(&format!("HOME={}", home.display())),
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
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut screen = self.screen();
        while !screen.contains(needle) && Instant::now() < deadline {
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
