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
const PRESS_ENTER: &str = ":echo \"acme\\nprompt\"";

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
fn ctrl_h_in_the_normal_mode_of_a_terminal_buffer_goes_to_claude_code() {
    let Some(neovim) = Neovim::start("nvim-terminal") else {
        return;
    };
    neovim.server.send_keys(&[":terminal sleep 600", "Enter"]);
    neovim.shows("term://");
    neovim.server.send_keys(&["i"]);
    neovim.shows("-- TERMINAL --");
    neovim.server.send_keys(&["C-\\", "C-n"]);
    neovim
        .server
        .send_keys(&[":echo 'acme-mode-' . mode(1)", "Enter"]);
    let screen = neovim.server.wait_for_screen("acme-mode-nt");
    assert!(screen.contains("acme-mode-nt"), "{screen}");

    neovim.server.send_keys(&["C-h"]);

    neovim.assert_in_claude_code();
}

#[test]
#[ignore]
fn ctrl_h_with_a_window_on_the_left_reaches_the_mapping_that_moves_to_it() {
    let Some(neovim) = Neovim::start("nvim-split") else {
        return;
    };
    neovim
        .server
        .send_keys(&[":nnoremap <C-h> <C-w>h", "Enter"]);
    neovim.settled();
    neovim.server.send_keys(&[":vsplit", "Enter"]);
    neovim.settled();
    neovim.server.send_keys(&["C-w", "l"]);
    neovim.settled();

    neovim.server.send_keys(&["C-h"]);

    neovim
        .server
        .send_keys(&[":echo 'acme-window-' . winnr()", "Enter"]);
    let screen = neovim.server.wait_for_screen("acme-window-1");
    assert!(screen.contains("acme-window-1"), "{screen}");
}

#[test]
#[ignore]
fn ctrl_h_with_an_operator_pending_stays_in_neovim() {
    let Some(neovim) = Neovim::start("nvim-operator") else {
        return;
    };
    neovim.server.send_keys(&["d"]);
    neovim.pending("d");

    neovim.server.send_keys(&["C-h"]);

    neovim.assert_in_neovim();
}

#[test]
#[ignore]
fn ctrl_h_after_ctrl_w_stays_in_neovim() {
    let Some(neovim) = Neovim::start("nvim-ctrl-w") else {
        return;
    };
    neovim.server.send_keys(&["C-w"]);
    neovim.pending("^W");

    neovim.server.send_keys(&["C-h"]);

    neovim.assert_in_neovim();
}

#[test]
#[ignore]
fn ctrl_h_at_a_press_enter_prompt_stays_in_neovim() {
    let Some(neovim) = Neovim::start("nvim-press-enter") else {
        return;
    };
    neovim.press_enter_prompt();

    neovim.server.send_keys(&["C-h"]);

    neovim.assert_in_neovim();
}

#[test]
#[ignore]
fn ctrl_h_in_insert_mode_stays_in_neovim() {
    let Some(neovim) = Neovim::start("nvim-insert") else {
        return;
    };
    neovim.server.send_keys(&["i"]);
    neovim.shows("-- INSERT --");

    neovim.server.send_keys(&["C-h"]);

    neovim.assert_in_neovim();
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
fn x_with_a_modified_acwrite_buffer_says_there_are_unsaved_changes() {
    let Some(neovim) = Neovim::start("nvim-acwrite") else {
        return;
    };
    neovim.server.send_keys(&[":file acme.txt", "Enter"]);
    neovim.settled();
    neovim
        .server
        .send_keys(&[":setlocal buftype=acwrite", "Enter"]);
    neovim.settled();
    neovim
        .server
        .send_keys(&[":autocmd BufWriteCmd <buffer> set nomodified", "Enter"]);
    neovim.settled();
    neovim.type_text("acme");

    let screen = neovim.end_from_the_sidebar("End? y/n");

    assert!(screen.contains("Unsaved changes. End? y/n"), "{screen}");
}

#[test]
#[ignore]
fn x_at_a_press_enter_prompt_says_there_may_be_unsaved_changes() {
    let Some(neovim) = Neovim::start("nvim-x-press-enter") else {
        return;
    };
    neovim.press_enter_prompt();

    let screen = neovim.end_from_the_sidebar("End? y/n");

    assert!(
        screen.contains("May have unsaved changes. End? y/n"),
        "{screen}"
    );
}

#[test]
#[ignore]
fn x_with_nothing_modified_asks_the_plain_question() {
    let Some(neovim) = Neovim::start("nvim-saved") else {
        return;
    };

    let screen = neovim.end_from_the_sidebar("End? y/n");

    assert!(screen.contains("End? y/n"), "{screen}");
    assert!(!screen.contains("nsaved changes"), "{screen}");
}

#[test]
#[ignore]
fn harbour_edit_opens_a_name_full_of_ex_specials_in_the_running_neovim() {
    let Some(neovim) = Neovim::start("nvim-edit") else {
        return;
    };
    let file = source_file(neovim.home.path(), "acme %b#c|d e.rs");
    let state = neovim.home.path().join("xdg/state");

    let output = handed(neovim.home.path(), Some(state.to_str().unwrap()), &file);

    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    assert_eq!(output.status.code(), Some(0));
    neovim.shows("acme handed line");
    neovim
        .server
        .send_keys(&[":echo 'acme-name-' . expand('%:t')", "Enter"]);
    neovim.shows("acme-name-acme %b#c|d e.rs");
}

#[test]
#[ignore]
fn harbour_edit_twice_in_a_row_opens_the_second_file_in_the_neovim_the_first_started() {
    let Some(neovim) = Neovim::beside_claude("nvim-twice") else {
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
fn harbour_edit_while_neovim_is_on_its_command_line_is_refused() {
    let Some(neovim) = Neovim::start("nvim-edit-cmdline") else {
        return;
    };
    neovim.server.send_keys(&[":acme"]);
    neovim.shows(":acme");
    let file = source_file(neovim.home.path(), "main.rs");
    let state = neovim.home.path().join("xdg/state");

    let output = handed(neovim.home.path(), Some(state.to_str().unwrap()), &file);

    assert_eq!(String::from_utf8_lossy(&output.stderr), WAITING);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
#[ignore]
fn harbour_edit_while_an_operator_waits_for_its_motion_is_refused() {
    let Some(neovim) = Neovim::start("nvim-edit-operator") else {
        return;
    };
    neovim.server.send_keys(&["d"]);
    neovim.pending("d");
    let file = source_file(neovim.home.path(), "main.rs");
    let state = neovim.home.path().join("xdg/state");

    let output = handed(neovim.home.path(), Some(state.to_str().unwrap()), &file);

    assert_eq!(String::from_utf8_lossy(&output.stderr), WAITING);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
#[ignore]
fn harbour_edit_starting_a_neovim_stopped_at_a_stale_swap_prompt_is_refused() {
    let Some(neovim) = Neovim::beside_claude("nvim-edit-swap") else {
        return;
    };
    let file = source_file(neovim.home.path(), "main.rs");
    let state = neovim.home.path().join("xdg/state");
    left_unsaved(&neovim, &file);

    let output = handed(neovim.home.path(), Some(state.to_str().unwrap()), &file);

    assert_eq!(String::from_utf8_lossy(&output.stderr), WAITING);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
#[ignore]
fn harbour_edit_while_neovim_shows_a_swap_prompt_is_refused_and_leaves_it_up() {
    let Some(neovim) = Neovim::start_with_rows("nvim-edit-prompt", 50) else {
        return;
    };
    let first = source_file(neovim.home.path(), "main.rs");
    let second = source_file(neovim.home.path(), "lib.rs");
    let state = neovim.home.path().join("xdg/state");
    left_unsaved(&neovim, &first);
    let stalled = handed(neovim.home.path(), Some(state.to_str().unwrap()), &first);
    assert_eq!(String::from_utf8_lossy(&stalled.stderr), WAITING);
    assert_eq!(stalled.status.code(), Some(2));
    neovim.shows("Found a swap file");

    let output = handed(neovim.home.path(), Some(state.to_str().unwrap()), &second);

    assert_eq!(String::from_utf8_lossy(&output.stderr), WAITING);
    assert_eq!(output.status.code(), Some(2));
    neovim.shows("Found a swap file");
}

struct Neovim {
    server: Server,
    home: TempDir,
}

impl Neovim {
    fn start(name: &str) -> Option<Neovim> {
        Neovim::start_with_rows(name, 24)
    }

    fn start_with_rows(name: &str, rows: u16) -> Option<Neovim> {
        let neovim = Neovim::beside_claude_with_rows(name, rows)?;
        neovim.server.send_keys(&["C-l"]);
        let editing = neovim.server.wait_for_screen("claude | nvim: ");
        assert!(editing.contains("claude | nvim: "), "{editing}");
        neovim.assert_in_neovim();
        Some(neovim)
    }

    fn beside_claude(name: &str) -> Option<Neovim> {
        Neovim::beside_claude_with_rows(name, 24)
    }

    fn beside_claude_with_rows(name: &str, rows: u16) -> Option<Neovim> {
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
        let server = Server::start(name, 200, rows);
        server.run(
            home.path(),
            &[
                (SESSION_COMMAND, AGENT),
                (EDITOR, CLEAN_NVIM),
                ("XDG_CONFIG_HOME", &config),
                ("XDG_DATA_HOME", &data),
                ("XDG_STATE_HOME", &state),
                ("XDG_CACHE_HOME", &cache),
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
        let cleared = self.server.wait_until(|screen| {
            screen.contains("claude | nvim: ")
                && !screen.contains("acme-settled")
                && !screen.contains(":echo")
        });
        assert!(!cleared.contains("acme-settled"), "{cleared}");
        assert!(!cleared.contains(":echo"), "{cleared}");
    }

    fn shows(&self, text: &str) {
        let screen = self.server.wait_for_screen(text);
        assert!(screen.contains(text), "{screen}");
    }

    fn pending(&self, keys: &str) {
        let screen = self
            .server
            .wait_until(|screen| screen.lines().any(|line| showing(line, keys)));
        assert!(screen.lines().any(|line| showing(line, keys)), "{screen}");
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

    fn press_enter_prompt(&self) {
        self.server.send_keys(&[PRESS_ENTER, "Enter"]);
        let screen = self.server.wait_for_screen("Press ENTER");
        assert!(screen.contains("Press ENTER"), "{screen}");
    }

    fn end_from_the_sidebar(&self, question: &str) -> String {
        self.server.send_bytes(SIDEBAR_CLICK);
        self.server.send_bytes(SIDEBAR_RELEASE);
        let focused = self.server.wait_for_screen("┏ sessions");
        assert!(focused.contains("┏ sessions"), "{focused}");
        self.server.send_keys(&["x"]);
        self.server.wait_for_screen(question)
    }
}

fn nvim_on_path() -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|directory| directory.join("nvim"))
        .find(|candidate| is_executable(candidate))
}

fn showing(line: &str, keys: &str) -> bool {
    line.trim_end()
        .trim_end_matches('┃')
        .trim_end()
        .ends_with(&format!(" {keys}"))
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
