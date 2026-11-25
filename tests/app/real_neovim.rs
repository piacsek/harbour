use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use harbour::editor::{Changes, Editor, Program, Unopened, socket_base};
use harbour::nvim::{self, Mode};
use tempfile::TempDir;

use crate::deadline;
use crate::nvim_on_path::{real_nvim, real_nvim_in};

const CLEAN_NVIM: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/nvim/nvim");
const WAITING: &str = "refused: Neovim is waiting for a key; answer it and try again";
const PRESS_ENTER: &str = ":echo \"acme\\nprompt\"\r";
const INHERITED: [&str; 5] = ["NVIM", "NVIM_APPNAME", "VIMINIT", "TMUX", "TMUX_PANE"];
const ESCAPE: &str = "\x1b";
const CTRL_W: &str = "\x17";
const LEAVE_TERMINAL: &str = "\x1c\x0e";

#[test]
fn normal_mode_in_the_leftmost_window_lets_ctrl_h_go_to_claude_code() {
    let Some(neovim) = Neovim::start("leftmost") else {
        return;
    };

    let wants = neovim.editor.wants_ctrl_h();

    assert!(!wants);
}

#[test]
fn the_normal_mode_of_a_terminal_buffer_lets_ctrl_h_go_to_claude_code() {
    let Some(neovim) = Neovim::start("terminal") else {
        return;
    };
    neovim.types(":terminal sleep 30\r");
    neovim.holds("&buftype ==# 'terminal'");
    neovim.types("i");
    neovim.in_mode("t");
    neovim.types(LEAVE_TERMINAL);
    neovim.in_mode("nt");

    let wants = neovim.editor.wants_ctrl_h();

    assert!(!wants);
}

#[test]
fn a_window_on_the_left_makes_neovim_want_ctrl_h_and_its_mapping_moves_there() {
    let Some(neovim) = Neovim::start("split") else {
        return;
    };
    neovim.types(":nnoremap <C-h> <C-w>h\r");
    neovim.holds("maparg('<C-h>', 'n') !=# ''");
    neovim.types(":vsplit\r");
    neovim.holds("winnr('$') == 2");
    neovim.types(CTRL_W);
    neovim.types("l");
    neovim.holds("winnr() == 2");

    let wants = neovim.editor.wants_ctrl_h();
    neovim.editor.session.send(&ctrl_h());

    assert!(wants);
    neovim.holds("winnr() == 1");
}

#[test]
fn normal_mode_in_the_rightmost_window_lets_ctrl_l_go_to_claude_code() {
    let Some(neovim) = Neovim::start("rightmost") else {
        return;
    };

    let wants = neovim.editor.wants_ctrl_l();

    assert!(!wants);
}

#[test]
fn a_window_on_the_right_makes_neovim_want_ctrl_l() {
    let Some(neovim) = Neovim::start("split-right") else {
        return;
    };
    neovim.types(":vsplit\r");
    neovim.holds("winnr('$') == 2");
    neovim.holds("winnr() == 1");

    let wants = neovim.editor.wants_ctrl_l();

    assert!(wants);
}

#[test]
fn an_operator_pending_keeps_ctrl_h_in_neovim() {
    let Some(neovim) = Neovim::start("operator") else {
        return;
    };
    neovim.types("d");
    neovim.in_mode("no");

    let wants = neovim.editor.wants_ctrl_h();

    assert!(wants);
}

#[test]
fn a_pending_ctrl_w_keeps_ctrl_h_in_neovim() {
    let Some(neovim) = Neovim::start("ctrl-w") else {
        return;
    };
    neovim.types(CTRL_W);
    neovim.reaches("a blocking normal mode", |mode| {
        mode.mode == "n" && mode.blocking
    });

    let wants = neovim.editor.wants_ctrl_h();

    assert!(wants);
}

#[test]
fn a_press_enter_prompt_keeps_ctrl_h_in_neovim() {
    let Some(neovim) = Neovim::start("press-enter") else {
        return;
    };
    neovim.types(PRESS_ENTER);
    neovim.in_mode("r");

    let wants = neovim.editor.wants_ctrl_h();

    assert!(wants);
}

#[test]
fn insert_mode_keeps_ctrl_h_in_neovim() {
    let Some(neovim) = Neovim::start("insert") else {
        return;
    };
    neovim.types("i");
    neovim.in_mode("i");

    let wants = neovim.editor.wants_ctrl_h();

    assert!(wants);
}

#[test]
fn a_modified_buffer_counts_as_unsaved_changes() {
    let Some(neovim) = Neovim::start("modified") else {
        return;
    };
    neovim.type_text("acme");

    let changes = neovim.editor.changes();

    assert_eq!(changes, Changes::Unsaved);
}

#[test]
fn a_modified_acwrite_buffer_counts_as_unsaved_changes() {
    let Some(neovim) = Neovim::start("acwrite") else {
        return;
    };
    neovim.types(":file acme.txt\r");
    neovim.holds("expand('%:t') ==# 'acme.txt'");
    neovim.types(":setlocal buftype=acwrite\r");
    neovim.holds("&buftype ==# 'acwrite'");
    neovim.types(":autocmd BufWriteCmd <buffer> set nomodified\r");
    neovim.holds("exists('#BufWriteCmd#<buffer>')");
    neovim.type_text("acme");

    let changes = neovim.editor.changes();

    assert_eq!(changes, Changes::Unsaved);
}

#[test]
fn a_press_enter_prompt_leaves_the_changes_unknown() {
    let Some(neovim) = Neovim::start("x-press-enter") else {
        return;
    };
    neovim.types(PRESS_ENTER);
    neovim.in_mode("r");

    let changes = neovim.editor.changes();

    assert_eq!(changes, Changes::Unknown);
}

#[test]
fn nothing_modified_counts_as_saved() {
    let Some(neovim) = Neovim::start("saved") else {
        return;
    };

    let changes = neovim.editor.changes();

    assert_eq!(changes, Changes::Saved);
}

#[test]
fn a_name_full_of_ex_specials_opens_as_it_is_named() {
    let Some(mut neovim) = Neovim::start("specials") else {
        return;
    };
    let file = neovim.scratch.file("acme %b#c|d e.rs");

    let opened = neovim.open(&file);

    assert_eq!(opened, "opened");
    neovim.holds("expand('%:t') ==# 'acme %b#c|d e.rs'");
    neovim.holds("getline(1) ==# 'acme handed line'");
}

#[test]
fn an_open_while_neovim_is_on_its_command_line_is_refused() {
    let Some(mut neovim) = Neovim::start("cmdline") else {
        return;
    };
    let file = neovim.scratch.file("main.rs");
    neovim.types(":acme");
    neovim.in_mode("c");

    let opened = neovim.open(&file);

    assert_eq!(opened, WAITING);
}

#[test]
fn an_open_while_an_operator_waits_for_its_motion_is_refused() {
    let Some(mut neovim) = Neovim::start("edit-operator") else {
        return;
    };
    let file = neovim.scratch.file("main.rs");
    neovim.types("d");
    neovim.in_mode("no");

    let opened = neovim.open(&file);

    assert_eq!(opened, WAITING);
}

#[test]
fn an_open_into_a_neovim_just_started_at_a_stale_swap_prompt_is_refused() {
    let Some(scratch) = Scratch::new("starting-swap") else {
        return;
    };
    let file = scratch.file("main.rs");
    scratch.left_unsaved(&file);
    let mut neovim = scratch.neovim(Some(&file));

    let opened = neovim.open(&file);

    assert_eq!(opened, WAITING);
}

#[test]
fn an_open_while_a_swap_prompt_is_up_is_refused_and_leaves_it_up() {
    let Some(scratch) = Scratch::new("swap-prompt") else {
        return;
    };
    let first = scratch.file("main.rs");
    let second = scratch.file("lib.rs");
    scratch.left_unsaved(&first);
    let mut neovim = scratch.neovim(Some(&first));
    neovim.in_mode("r?");

    let opened = neovim.open(&second);
    let after = nvim::mode(&neovim.socket(), Instant::now() + 5 * deadline());

    assert_eq!(opened, WAITING);
    assert_eq!(after.map(|mode| mode.mode).as_deref(), Some("r?"));
}

#[test]
fn an_open_that_raises_a_swap_prompt_in_a_running_neovim_is_refused_and_leaves_it_up() {
    let Some(mut neovim) = Neovim::start("raised-swap") else {
        return;
    };
    let file = neovim.scratch.file("main.rs");
    neovim.scratch.left_unsaved(&file);

    let opened = neovim.open(&file);
    let after = nvim::mode(&neovim.socket(), Instant::now() + 5 * deadline());

    assert_eq!(opened, WAITING);
    assert_eq!(after.map(|mode| mode.mode).as_deref(), Some("r?"));
}

#[test]
#[should_panic(expected = "no nvim on PATH in CI")]
fn a_missing_neovim_fails_the_real_neovim_tests_in_ci() {
    let empty = tempfile::tempdir().unwrap();

    real_nvim_in(
        "acme",
        Some(empty.path().into()),
        Some(OsString::from("true")),
    );
}

#[test]
fn a_missing_neovim_skips_the_real_neovim_tests_outside_ci() {
    let empty = tempfile::tempdir().unwrap();

    let found = real_nvim_in("acme", Some(empty.path().into()), None);

    assert_eq!(found, None);
}

struct Scratch {
    home: TempDir,
}

impl Scratch {
    fn new(name: &str) -> Option<Scratch> {
        let real = real_nvim(name)?;
        let home = tempfile::tempdir().unwrap();
        for folder in ["bin", "tmp", "work"] {
            std::fs::create_dir_all(home.path().join(folder)).unwrap();
        }
        std::os::unix::fs::symlink(real, home.path().join("bin/real-nvim")).unwrap();
        let tmp = home.path().join("tmp");
        assert_eq!(
            socket_base(&tmp),
            tmp,
            "the scratch TMPDIR {} is {} bytes, too long for the editor's socket, which then goes under /tmp",
            tmp.display(),
            tmp.as_os_str().len()
        );
        Some(Scratch { home })
    }

    fn path(&self) -> &Path {
        self.home.path()
    }

    fn file(&self, name: &str) -> PathBuf {
        let file = self.path().join("work").canonicalize().unwrap().join(name);
        std::fs::write(&file, "acme handed line\n").unwrap();
        file
    }

    fn environment(&self) -> Vec<(String, String)> {
        let at = |below: &str| self.path().join(below).display().to_string();
        let path = format!("{}:/usr/bin:/bin", at("bin"));
        vec![
            ("HOME".into(), at("")),
            ("TMPDIR".into(), at("tmp")),
            ("XDG_CONFIG_HOME".into(), at("xdg/config")),
            ("XDG_DATA_HOME".into(), at("xdg/data")),
            ("XDG_STATE_HOME".into(), at("xdg/state")),
            ("XDG_CACHE_HOME".into(), at("xdg/cache")),
            ("PATH".into(), path),
        ]
    }

    fn left_unsaved(&self, file: &Path) {
        let preserved = self.path().join(format!(
            "preserved-{}",
            file.file_name().unwrap().to_string_lossy()
        ));
        let mut command = Command::new(self.path().join("bin/real-nvim"));
        for name in INHERITED {
            command.env_remove(name);
        }
        let mut other = command
            .args(["--headless", "--clean", "-c", "normal! oacme unsaved"])
            .args(["-c", "preserve", "-c"])
            .arg(format!("call writefile([], '{}')", preserved.display()))
            .arg(file)
            .envs(self.environment())
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

    fn neovim(self, file: Option<&Path>) -> Neovim {
        let mut command = vec!["/usr/bin/env".to_string()];
        for unset in INHERITED {
            command.push("-u".into());
            command.push(unset.into());
        }
        for (name, value) in self.environment() {
            command.push(format!("{name}={value}"));
        }
        command.push(CLEAN_NVIM.into());
        let program = Program::neovim("nvim", command).with_patience(5 * deadline());
        let editor = Editor::start(
            &program,
            &self.path().join("work"),
            &self.path().join("tmp"),
            1,
            50,
            120,
            file,
        )
        .unwrap();
        Neovim {
            editor,
            scratch: self,
        }
    }
}

struct Neovim {
    editor: Editor,
    scratch: Scratch,
}

impl Neovim {
    fn start(name: &str) -> Option<Neovim> {
        let neovim = Scratch::new(name)?.neovim(None);
        neovim.in_mode("n");
        Some(neovim)
    }

    fn socket(&self) -> PathBuf {
        let listed = std::fs::read_dir(self.scratch.path().join("tmp")).unwrap();
        let private = listed
            .map(|entry| entry.unwrap().path())
            .find(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("harbour-nvim")
            })
            .expect("the editor made no socket directory");
        private.join("socket")
    }

    fn types(&self, keys: &str) {
        self.editor.session.send(keys.as_bytes());
    }

    fn type_text(&self, text: &str) {
        self.types("i");
        self.in_mode("i");
        self.types(text);
        self.holds(&format!("getline(1) ==# '{text}'"));
        self.types(ESCAPE);
        self.in_mode("n");
    }

    fn in_mode(&self, wanted: &str) {
        self.reaches(&format!("mode {wanted:?}"), |mode| mode.mode == wanted);
    }

    fn reaches(&self, what: &str, done: impl Fn(&Mode) -> bool) {
        reached(&self.socket(), what, 5 * deadline(), done);
    }

    fn holds(&self, expression: &str) {
        held(&self.socket(), expression, 5 * deadline());
    }

    fn open(&mut self, file: &Path) -> String {
        match self.editor.open(file, Instant::now() + 5 * deadline()) {
            Ok(()) => "opened".to_string(),
            Err(Unopened::Refused(refusal)) => format!("refused: {refusal}"),
            Err(Unopened::Silent) => "silent".to_string(),
            Err(Unopened::Exited) => "exited".to_string(),
        }
    }
}

impl Drop for Neovim {
    fn drop(&mut self) {
        self.editor.session.end();
        let left = lingering(self.scratch.path(), 5 * deadline());
        for pid in &left {
            let _ = Command::new("kill").args(["-KILL", pid]).status();
        }
        if !std::thread::panicking() {
            assert!(left.is_empty(), "Neovim left running: {left:?}");
        }
    }
}

pub(super) fn reached(socket: &Path, what: &str, patience: Duration, done: impl Fn(&Mode) -> bool) {
    let until = Instant::now() + patience;
    loop {
        let mode = nvim::mode(socket, Instant::now() + deadline());
        if mode.as_ref().is_some_and(&done) {
            return;
        }
        let last = mode.map_or("none".to_string(), |mode| {
            format!("{:?} blocking {}", mode.mode, mode.blocking)
        });
        assert!(
            Instant::now() < until,
            "waited {patience:?} for {what}, last mode: {last}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

pub(super) fn held(socket: &Path, expression: &str, patience: Duration) {
    let until = Instant::now() + patience;
    loop {
        let value = nvim::eval(socket, expression, Instant::now() + deadline());
        if value == Some(1) {
            return;
        }
        assert!(
            Instant::now() < until,
            "waited {patience:?} for {expression}, last value: {value:?}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn ctrl_h() -> Vec<u8> {
    harbour::keys::encode(
        KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL),
        false,
    )
}

fn appeared(file: &Path) -> bool {
    let until = Instant::now() + 5 * deadline();
    while Instant::now() < until {
        if file.exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
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
