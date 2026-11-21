use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use harbour::tui::{EDITOR, SESSION_COMMAND};
use tempfile::TempDir;

use super::edit::{handed, source_file};
use super::{AGENT, Server, tools};

const CLEAN_NVIM: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/nvim/nvim");
const SIDEBAR_CLICK: &str = "\x1b[<0;3;3M";
const SIDEBAR_RELEASE: &str = "\x1b[<0;3;3m";
const WAITING: &str = "harbour: Neovim is waiting for a key; answer it and try again\n";

#[test]
#[ignore]
fn ctrl_h_in_normal_mode_in_the_leftmost_window_goes_to_claude_code() {
    let Some(neovim) = Neovim::start("nvim-leftmost") else {
        return;
    };

    neovim.server.send_keys(&["C-h"]);

    neovim.assert_in_claude_code();
}

#[test]
#[ignore]
fn a_tmux_command_run_from_neovim_targets_the_pane_of_harbour_and_ctrl_h_still_leaves() {
    let Some(neovim) = Neovim::start("nvim-tmux-pane") else {
        return;
    };
    let tmux = on_path("tmux").unwrap();
    let pane = neovim
        .server
        .tmux(&["display-message", "-p", "-t", "e2e", "#{pane_id}"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&pane.stderr);
    assert!(
        pane.status.success(),
        "display-message of harbour's pane failed: {stderr}"
    );
    let pane = String::from_utf8_lossy(&pane.stdout).trim_end().to_string();
    assert!(!pane.is_empty(), "display-message printed no pane id");
    let seen = format!("acme seen pane {pane}");
    let targeted = format!("acme targeted pane {pane}");

    neovim.server.send_keys(&[
        &format!(
            ":!echo acme seen pane $TMUX_PANE; {} display-message -p 'acme targeted pane \\#{{pane_id}}'",
            tmux.display()
        ),
        "Enter",
    ]);

    let screen = neovim.server.wait_for_screen_all(&[&seen, &targeted]);
    assert!(screen.contains(&seen), "{screen}");
    assert!(screen.contains(&targeted), "{screen}");
    neovim.server.send_keys(&["Enter"]);
    neovim.assert_in_neovim();
    neovim.server.send_keys(&["C-h"]);
    neovim.assert_in_claude_code();
}

#[test]
#[ignore]
fn x_with_a_modified_buffer_says_there_are_unsaved_changes() {
    let Some(neovim) = Neovim::start("nvim-modified") else {
        return;
    };
    neovim.type_text("acme");

    let screen = neovim.end_from_the_sidebar("End? y/n");

    assert!(screen.contains("Unsaved changes. End? y/n"), "{screen}");
}

#[test]
#[ignore]
fn harbour_edit_twice_in_a_row_opens_the_second_file_in_the_neovim_the_first_started() {
    let Some(neovim) = Neovim::beside_claude("nvim-twice", &[]) else {
        return;
    };
    let first = source_file(neovim.home.path(), "first.rs");
    let second = first.with_file_name("second.rs");
    std::fs::write(&second, "acme second line\n").unwrap();
    let state = neovim.home.path().join("xdg/state");

    let opened = handed(neovim.home.path(), Some(state.to_str().unwrap()), &first);
    let reopened = handed(neovim.home.path(), Some(state.to_str().unwrap()), &second);

    assert_eq!(String::from_utf8_lossy(&opened.stderr), "");
    assert_eq!(opened.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&reopened.stderr), "");
    assert_eq!(reopened.status.code(), Some(0));
    neovim.shows("acme second line");
}

#[test]
#[ignore]
fn harbour_edit_starting_a_neovim_stopped_at_a_stale_swap_prompt_is_refused() {
    let Some(neovim) = Neovim::beside_claude("nvim-edit-swap", &[]) else {
        return;
    };
    let file = source_file(neovim.home.path(), "main.rs");
    let state = neovim.home.path().join("xdg/state");
    left_unsaved(&neovim, &file);

    let output = handed(neovim.home.path(), Some(state.to_str().unwrap()), &file);

    neovim.end_every_neovim();
    assert_eq!(String::from_utf8_lossy(&output.stderr), WAITING);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
#[ignore]
fn shift_enter_in_insert_mode_with_extended_keys_on_starts_a_line_and_stays_in_insert_mode() {
    let Some(neovim) = Neovim::start_with("nvim-shift-enter", &[&["extended-keys", "on"]]) else {
        return;
    };

    neovim
        .server
        .send_keys(&["i", "acme-one", "S-Enter", "acme-two"]);

    neovim.shows("┃acme-two");
    neovim.shows("-- INSERT --");
}

#[test]
#[ignore]
fn a_ctrl_escape_mapping_in_neovim_fires_with_extended_keys_on() {
    let Some(neovim) = Neovim::start_with("nvim-ctrl-esc", &[&["extended-keys", "on"]]) else {
        return;
    };
    neovim.mapped("<C-Esc>", "fired");

    neovim.server.send_keys(&["C-Escape"]);

    neovim.shows("acme-fired");
}

#[test]
#[ignore]
fn ctrl_shift_p_and_ctrl_p_are_two_keys_in_neovim_with_extended_keys_on() {
    let Some(neovim) = Neovim::start_with("nvim-ctrl-shift-p", &[&["extended-keys", "on"]]) else {
        return;
    };
    neovim.mapped("<C-S-p>", "shifted");

    neovim.server.send_keys(&["C-S-p"]);

    neovim.shows("acme-shifted");
}

#[test]
#[ignore]
fn ctrl_zero_in_neovims_insert_mode_stays_in_insert_mode_with_extended_keys_on() {
    let Some(neovim) = Neovim::start_with("nvim-ctrl-0", &[&["extended-keys", "on"]]) else {
        return;
    };

    neovim
        .server
        .send_keys(&["i", "acme-one", "C-0", "acme-two"]);

    neovim.shows("acme-one0acme-two");
    neovim.shows("-- INSERT --");
}

#[test]
#[ignore]
fn ctrl_backslash_ctrl_n_leaves_neovims_terminal_mode_with_extended_keys_off() {
    let Some(neovim) = Neovim::start_with("nvim-ctrl-backslash", &[&["extended-keys", "off"]])
    else {
        return;
    };
    neovim.server.send_keys(&[":terminal", "Enter", "i"]);
    neovim.shows("-- TERMINAL --");

    neovim.server.send_keys(&["C-\\", "C-n"]);

    let left = neovim.server.wait_until("terminal mode left", |screen| {
        !screen.contains("-- TERMINAL --")
    });
    assert!(!left.contains("-- TERMINAL --"), "{left}");
}

#[test]
#[ignore]
fn a_ctrl_closing_bracket_mapping_in_neovim_fires_with_extended_keys_off() {
    let Some(neovim) = Neovim::start_with("nvim-ctrl-bracket", &[&["extended-keys", "off"]]) else {
        return;
    };
    neovim.mapped("<C-]>", "jumped");

    neovim.server.send_keys(&["C-]"]);

    neovim.shows("acme-jumped");
}

#[test]
#[ignore]
fn ctrl_opening_bracket_leaves_neovims_insert_mode_with_extended_keys_on() {
    let Some(neovim) = Neovim::start_with("nvim-ctrl-escape", &[&["extended-keys", "on"]]) else {
        return;
    };
    neovim.server.send_keys(&["i", "acme-one"]);
    neovim.shows("-- INSERT --");

    neovim.server.send_keys(&["C-["]);

    let left = neovim.server.wait_until("insert mode left", |screen| {
        !screen.contains("-- INSERT --")
    });
    assert!(!left.contains("-- INSERT --"), "{left}");
}

#[test]
#[ignore]
fn ctrl_i_and_tab_are_two_keys_in_neovim_with_extended_keys_on() {
    let Some(neovim) = Neovim::start_with("nvim-ctrl-i", &[&["extended-keys", "on"]]) else {
        return;
    };
    neovim.mapped("<Tab>", "tabbed");
    neovim.mapped("<C-i>", "ctrl-i");

    neovim.server.send_keys(&["C-i"]);
    neovim.shows("acme-ctrl-i");
    neovim.server.send_keys(&["Tab"]);

    neovim.shows("acme-tabbed");
}

struct Neovim {
    server: Server,
    home: TempDir,
}

impl Neovim {
    fn start(name: &str) -> Option<Neovim> {
        Neovim::start_with(name, &[])
    }

    fn start_with(name: &str, options: &[&[&str]]) -> Option<Neovim> {
        let neovim = Neovim::beside_claude(name, options)?;
        neovim.server.send_keys(&["C-l"]);
        let editing = neovim.server.wait_for_screen("claude | nvim: ");
        assert!(editing.contains("claude | nvim: "), "{editing}");
        neovim.assert_in_neovim();
        Some(neovim)
    }

    fn beside_claude(name: &str, options: &[&[&str]]) -> Option<Neovim> {
        let Some(real) = nvim_on_path() else {
            eprintln!(
                "skipping {name}: no nvim on PATH, so the real-Neovim test has nothing to run"
            );
            return None;
        };
        let home = tempfile::tempdir().unwrap();
        let bin = PathBuf::from(tools(home.path()));
        std::os::unix::fs::symlink(real, bin.join("real-nvim")).unwrap();
        let xdg = |name: &str| home.path().join("xdg").join(name).display().to_string();
        let (config, data, state, cache) = (xdg("config"), xdg("data"), xdg("state"), xdg("cache"));
        let sockets = home.path().join("tmux");
        std::fs::create_dir(&sockets).unwrap();
        let sockets = sockets.display().to_string();
        let server = Server::start(name, 200, 24);
        if let Some(option) = options.iter().find(|option| !server.server_option(option)) {
            eprintln!("skipping {name}: this tmux cannot set {option:?}");
            return None;
        }
        server.run(
            home.path(),
            &[
                (SESSION_COMMAND, AGENT),
                (EDITOR, CLEAN_NVIM),
                ("XDG_CONFIG_HOME", &config),
                ("XDG_DATA_HOME", &data),
                ("XDG_STATE_HOME", &state),
                ("XDG_CACHE_HOME", &cache),
                ("TMUX_TMPDIR", &sockets),
            ],
            &[],
        );
        server.wait_for_screen("press n to start one");
        server.send_keys(&["n"]);
        server.wait_for_screen("acme agent in acme-api");
        server.send_keys(&["Enter"]);
        let focused = server.wait_for_screen("┏ claude");
        assert!(focused.contains("┏ claude"), "{focused}");
        Some(Neovim { server, home })
    }

    fn assert_in_neovim(&self) {
        self.server.send_keys(&["Escape"]);
        self.settled();
    }

    fn settled(&self) {
        self.server
            .send_keys(&[":echo 'acme-' . 'settled'", "Enter"]);
        let screen = self.server.wait_for_screen("acme-settled");
        assert!(screen.contains("acme-settled"), "{screen}");
        self.server.send_keys(&[":echo ''", "Enter"]);
        let cleared = self.server.wait_until("the echo cleared", |screen| {
            screen.contains("claude | nvim: ")
                && !screen.contains("acme-settled")
                && !screen.contains(":echo")
        });
        assert!(!cleared.contains("acme-settled"), "{cleared}");
        assert!(!cleared.contains(":echo"), "{cleared}");
    }

    fn mapped(&self, key: &str, word: &str) {
        self.server.send_keys(&[
            &format!(":nnoremap {key} <Cmd>echo 'acme-' . '{word}'<CR>"),
            "Enter",
        ]);
        self.settled();
    }

    fn shows(&self, text: &str) {
        let screen = self.server.wait_for_screen(text);
        assert!(screen.contains(text), "{screen}");
    }

    fn type_text(&self, text: &str) {
        self.server.send_keys(&["i"]);
        self.shows("-- INSERT --");
        self.server.send_keys(&[text]);
        let line = format!("┃{text}");
        self.shows(&line);
        self.server.send_keys(&["Escape"]);
        self.settled();
    }

    fn assert_in_claude_code(&self) {
        let screen = self
            .server
            .wait_for_screen_showing_without("acme agent in acme-api", "acme-settled");
        assert!(screen.contains("acme agent in acme-api"), "{screen}");
        self.server.send_keys(&["acme-ping", "Enter"]);
        let screen = self.server.wait_for_screen("acme agent got: acme-ping");
        assert!(screen.contains("acme agent got: acme-ping"), "{screen}");
    }

    fn end_from_the_sidebar(&self, question: &str) -> String {
        self.server.send_bytes(SIDEBAR_CLICK);
        self.server.send_bytes(SIDEBAR_RELEASE);
        let focused = self.server.wait_for_screen("┏ sessions");
        assert!(focused.contains("┏ sessions"), "{focused}");
        self.server.send_keys(&["x"]);
        self.server.wait_for_screen(question)
    }

    fn end_every_neovim(&self) {
        for pid in lingering(self.home.path(), Duration::ZERO) {
            let _ = Command::new("kill").args(["-KILL", &pid]).status();
        }
    }
}

impl Drop for Neovim {
    fn drop(&mut self) {
        let _ = self.server.tmux(&["kill-server"]).status();
        let left = lingering(self.home.path(), Duration::from_secs(3));
        self.end_every_neovim();
        if !std::thread::panicking() {
            assert!(left.is_empty(), "Neovim left running: {left:?}");
        }
    }
}

fn nvim_on_path() -> Option<PathBuf> {
    on_path("nvim")
}

fn on_path(program: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|directory| directory.join(program))
        .find(|candidate| is_executable(candidate))
}

fn is_executable(candidate: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    candidate
        .metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

fn appeared(file: &Path) -> bool {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if file.exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

fn left_unsaved(neovim: &Neovim, file: &Path) {
    let xdg = neovim.home.path().join("xdg");
    let preserved = neovim.home.path().join(format!(
        "preserved-{}",
        file.file_name().unwrap().to_string_lossy()
    ));
    let mut other = Command::new(nvim_on_path().unwrap())
        .args(["--headless", "--clean", "-c", "normal! oacme unsaved"])
        .args(["-c", "preserve", "-c"])
        .arg(format!("call writefile([], '{}')", preserved.display()))
        .arg(file)
        .env("HOME", neovim.home.path())
        .env("XDG_CONFIG_HOME", xdg.join("config"))
        .env("XDG_DATA_HOME", xdg.join("data"))
        .env("XDG_STATE_HOME", xdg.join("state"))
        .env("XDG_CACHE_HOME", xdg.join("cache"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let swapped = appeared(&preserved);
    other.kill().unwrap();
    other.wait().unwrap();
    assert!(swapped, "the swap file was never preserved");
}

fn lingering(home: &Path, patience: Duration) -> Vec<String> {
    let mark = home.file_name().unwrap().to_string_lossy().into_owned();
    let deadline = Instant::now() + patience;
    loop {
        let listed = Command::new("ps")
            .args(["-axww", "-o", "pid=,command="])
            .output()
            .unwrap();
        let alive: Vec<String> = String::from_utf8_lossy(&listed.stdout)
            .lines()
            .filter(|line| line.contains("real-nvim") && line.contains(&mark))
            .filter_map(|line| line.split_whitespace().next().map(String::from))
            .collect();
        if alive.is_empty() || Instant::now() >= deadline {
            return alive;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
