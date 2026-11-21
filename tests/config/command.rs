use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::{Command, Output};

use crate::{config_path, write_config};

#[test]
fn harbour_config_names_the_file_and_every_default() {
    let home = tempfile::tempdir().unwrap();

    let output = config_command(home.path());

    assert_eq!(
        stdout(&output),
        format!(
            "file: {} (none there)\neditor = [\"nvim\"]  (default)\npicker = [\"fzf\"]  (default)\nsession_command = [\"claude\"]  (default)\nsidebar_width = a third  (default)\naccent = \"cyan\"  (default)\n",
            config_path(home.path()).display()
        )
    );
    assert!(output.status.success());
}

#[test]
fn harbour_config_shows_what_the_file_sets() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(
        home.path(),
        "editor = [\"nvim\", \"--clean\"]\nsidebar_width = 40\naccent = \"#d7875f\"\n",
    );

    let output = config_command(home.path());

    assert_eq!(
        stdout(&output),
        format!(
            "file: {}\neditor = [\"nvim\", \"--clean\"]  (file)\npicker = [\"fzf\"]  (default)\nsession_command = [\"claude\"]  (default)\nsidebar_width = 40  (file)\naccent = \"#d7875f\"  (file)\n",
            path.display()
        )
    );
}

#[test]
fn harbour_config_names_a_light_accent_as_the_file_may_write_it() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "accent = \"LightRed\"\n");

    let output = config_command(home.path());

    assert!(
        stdout(&output).contains("\naccent = \"lightred\"  (file)\n"),
        "{}",
        stdout(&output)
    );
}

#[test]
fn harbour_config_names_the_variable_a_setting_comes_from() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "editor = [\"nvim\", \"--clean\"]\n");

    let output = Command::new(env!("CARGO_BIN_EXE_harbour"))
        .arg("config")
        .env("HOME", home.path())
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("HARBOUR_EDITOR")
        .env_remove("HARBOUR_DIR_PICKER")
        .env_remove("HARBOUR_SESSION_COMMAND")
        .env("HARBOUR_EDITOR", "acme-vim")
        .output()
        .unwrap();

    assert!(
        stdout(&output).contains("\neditor = [\"acme-vim\"]  (variable HARBOUR_EDITOR)\n"),
        "{}",
        stdout(&output)
    );
}

#[test]
fn harbour_config_without_a_home_says_there_is_no_file() {
    let output = Command::new(env!("CARGO_BIN_EXE_harbour"))
        .arg("config")
        .env_remove("HOME")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("HARBOUR_EDITOR")
        .env_remove("HARBOUR_DIR_PICKER")
        .env_remove("HARBOUR_SESSION_COMMAND")
        .output()
        .unwrap();

    assert!(
        stdout(&output).starts_with("file: none (HOME is not set)\n"),
        "{}",
        stdout(&output)
    );
}

#[test]
fn harbour_config_refuses_a_syntax_error_in_one_line() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "editor = [\"nvim\"\n");

    let output = config_command(home.path());

    assert_eq!(
        stderr(&output),
        format!(
            "harbour: {}:1: editor: unclosed array, expected `]`\n",
            path.display()
        )
    );
    assert_eq!(stdout(&output), "");
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn harbour_config_refuses_a_wrong_type_in_one_line() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "sidebar_width = \"40\"\n");

    let output = config_command(home.path());

    assert_eq!(
        stderr(&output),
        format!(
            "harbour: {}:1: sidebar_width: expected a whole number of columns above 0\n",
            path.display()
        )
    );
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn harbour_config_refuses_an_unknown_key_in_one_line() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "retention_days = 7\n");

    let output = config_command(home.path());

    assert_eq!(
        stderr(&output),
        format!(
            "harbour: {}:1: retention_days: unknown key\n",
            path.display()
        )
    );
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn harbour_config_shows_a_control_character_escaped() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "acme\u{7}bell = 1\n");

    let output = config_command(home.path());

    assert!(
        stderr(&output).starts_with(&format!(
            "harbour: {}:1: acme\\u{{7}}bell: ",
            path.display()
        )),
        "{}",
        stderr(&output)
    );
}

#[test]
fn harbour_config_shows_each_setting_on_one_line_with_its_words_escaped() {
    let home = tempfile::tempdir().unwrap();
    write_config(
        home.path(),
        "session_command = [\"acme-agent\", \"two\\nlines\", \"say \\\"hi\\\"\", \"back\\\\slash\"]\n",
    );

    let output = config_command(home.path());

    assert!(
        stdout(&output).contains(
            "\nsession_command = [\"acme-agent\", \"two\\nlines\", \"say \\\"hi\\\"\", \"back\\\\slash\"]  (file)\n"
        ),
        "{}",
        stdout(&output)
    );
    assert_eq!(stdout(&output).lines().count(), 6);
}

#[test]
fn harbour_config_says_a_variable_that_is_not_utf8_was_ignored() {
    let home = tempfile::tempdir().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_harbour"))
        .arg("config")
        .env("HOME", home.path())
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("HARBOUR_EDITOR")
        .env_remove("HARBOUR_DIR_PICKER")
        .env_remove("HARBOUR_SESSION_COMMAND")
        .env("HARBOUR_DIR_PICKER", OsStr::from_bytes(b"acme-\xffpicker"))
        .output()
        .unwrap();

    assert!(
        stdout(&output).contains(
            "\npicker = [\"fzf\"]  (default; variable HARBOUR_DIR_PICKER ignored: not UTF-8)\n"
        ),
        "{}",
        stdout(&output)
    );
}

#[test]
fn harbour_config_shows_a_file_path_with_a_newline_escaped_on_one_line() {
    let home = tempfile::tempdir().unwrap();
    let folder = home.path().join("acme\nconfig");

    let output = Command::new(env!("CARGO_BIN_EXE_harbour"))
        .arg("config")
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", &folder)
        .env_remove("HARBOUR_EDITOR")
        .env_remove("HARBOUR_DIR_PICKER")
        .env_remove("HARBOUR_SESSION_COMMAND")
        .output()
        .unwrap();

    assert!(
        stdout(&output).starts_with(&format!(
            "file: {}/acme\\nconfig/harbour/config.toml (none there)\n",
            home.path().display()
        )),
        "{}",
        stdout(&output)
    );
    assert_eq!(stdout(&output).lines().count(), 6);
}

fn config_command(home: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_harbour"))
        .arg("config")
        .env("HOME", home)
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("HARBOUR_EDITOR")
        .env_remove("HARBOUR_DIR_PICKER")
        .env_remove("HARBOUR_SESSION_COMMAND")
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
