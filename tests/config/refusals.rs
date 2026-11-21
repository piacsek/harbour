use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::time::Duration;

use harbour::config::{Config, Source};

use crate::{config_path, read_at, write_config};

#[test]
fn a_syntax_error_names_the_file_the_line_and_the_key() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "accent = \"cyan\"\neditor = [\"nvim\"\n");

    assert_eq!(
        refusal(home.path()),
        format!("{}:2: editor: unclosed array, expected `]`", path.display())
    );
}

#[test]
fn a_syntax_error_on_a_line_without_a_key_names_the_file_and_the_line() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "accent = \"cyan\"\n\n[[\n");

    assert!(
        refusal(home.path()).starts_with(&format!("{}:3: ", path.display())),
        "{}",
        refusal(home.path())
    );
}

#[test]
fn a_repeated_key_is_a_syntax_error_on_its_second_line() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "accent = \"cyan\"\naccent = \"red\"\n");

    assert!(
        refusal(home.path()).starts_with(&format!("{}:2: accent: ", path.display())),
        "{}",
        refusal(home.path())
    );
}

#[test]
fn an_unknown_key_is_named_with_its_line() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "accent = \"cyan\"\n\nretention_days = 7\n");

    assert_eq!(
        refusal(home.path()),
        format!("{}:3: retention_days: unknown key", path.display())
    );
}

#[test]
fn a_table_is_an_unknown_key() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "\n[keys]\nquit = \"q\"\n");

    assert_eq!(
        refusal(home.path()),
        format!("{}:2: keys: unknown key", path.display())
    );
}

#[test]
fn a_program_given_as_a_string_is_the_wrong_type() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "\neditor = \"nvim\"\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:2: editor: expected a list of strings: the program, then its arguments",
            path.display()
        )
    );
}

#[test]
fn a_list_holding_a_number_is_the_wrong_type() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "picker = [\"fzf\", 3]\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:1: picker: expected a list of strings: the program, then its arguments",
            path.display()
        )
    );
}

#[test]
fn an_empty_program_list_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "session_command = []\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:1: session_command: expected a list of strings: the program, then its arguments",
            path.display()
        )
    );
}

#[test]
fn an_empty_program_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "editor = [\"\", \"--clean\"]\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:1: editor: expected a list of strings: the program, then its arguments",
            path.display()
        )
    );
}

#[test]
fn a_relative_program_path_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "editor = [\"bin/acme-vim\"]\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:1: editor: \"bin/acme-vim\" is a relative path; give an absolute path, ~/... or a bare name",
            path.display()
        )
    );
}

#[test]
fn a_program_under_home_without_home_says_home_is_not_set() {
    let home = tempfile::tempdir().unwrap();
    let xdg = home.path().join("xdg");
    let path = xdg.join("harbour/config.toml");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "editor = [\"~/bin/acme-vim\"]\n").unwrap();
    let xdg = xdg.into_os_string();

    let refused = Config::read(move |name| (name == "XDG_CONFIG_HOME").then(|| xdg.clone()));

    assert_eq!(
        refused.err().map(|refusal| refusal.to_string()),
        Some(format!(
            "{}:1: editor: \"~/bin/acme-vim\" starts with ~ but HOME is not set",
            path.display()
        ))
    );
}

#[test]
fn a_width_given_as_a_string_is_the_wrong_type() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "sidebar_width = \"40\"\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:1: sidebar_width: expected a whole number of columns above 0",
            path.display()
        )
    );
}

#[test]
fn a_width_of_zero_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "sidebar_width = 0\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:1: sidebar_width: expected a whole number of columns above 0",
            path.display()
        )
    );
}

#[test]
fn a_negative_width_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "sidebar_width = -40\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:1: sidebar_width: expected a whole number of columns above 0",
            path.display()
        )
    );
}

#[test]
fn a_fractional_width_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "sidebar_width = 1.5\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:1: sidebar_width: expected a whole number of columns above 0",
            path.display()
        )
    );
}

#[test]
fn an_unknown_colour_name_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "accent = \"pink\"\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:1: accent: expected one of the 16 colour names or #rrggbb",
            path.display()
        )
    );
}

#[test]
fn a_nerd_font_that_is_not_true_or_false_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "accent = \"cyan\"\nnerd_font = \"yes\"\n");

    assert_eq!(
        refusal(home.path()),
        format!("{}:2: nerd_font: expected true or false", path.display())
    );
}

#[test]
fn a_nerd_font_given_as_a_number_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "nerd_font = 1\n");

    assert_eq!(
        refusal(home.path()),
        format!("{}:1: nerd_font: expected true or false", path.display())
    );
}

#[test]
fn a_short_hex_colour_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "accent = \"#12345\"\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:1: accent: expected one of the 16 colour names or #rrggbb",
            path.display()
        )
    );
}

#[test]
fn an_indexed_colour_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "accent = \"12\"\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:1: accent: expected one of the 16 colour names or #rrggbb",
            path.display()
        )
    );
}

#[test]
fn a_colour_given_as_a_number_is_the_wrong_type() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "accent = 3\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:1: accent: expected one of the 16 colour names or #rrggbb",
            path.display()
        )
    );
}

#[test]
fn a_directory_in_place_of_the_file_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = config_path(home.path());
    fs::create_dir_all(&path).unwrap();

    assert!(
        refusal(home.path()).starts_with(&format!("{}: ", path.display())),
        "{}",
        refusal(home.path())
    );
}

#[test]
fn an_unreadable_file_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "accent = \"red\"\n");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();

    assert_eq!(
        refusal(home.path()),
        format!("{}: Permission denied (os error 13)", path.display())
    );
}

#[test]
fn a_file_that_is_not_utf8_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "");
    fs::write(&path, b"accent = \"\xff\"\n").unwrap();

    assert_eq!(
        refusal(home.path()),
        format!("{}: stream did not contain valid UTF-8", path.display())
    );
}

#[test]
fn a_config_folder_that_is_a_file_counts_as_no_file() {
    let home = tempfile::tempdir().unwrap();
    fs::write(home.path().join(".config"), "").unwrap();

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.accent().source, Source::Default);
}

#[test]
fn a_fifo_in_place_of_the_file_is_refused_without_waiting() {
    let home = tempfile::tempdir().unwrap();
    let path = config_path(home.path());
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let made = Command::new("mkfifo").arg(&path).status().unwrap();
    assert!(made.success());

    let (sender, receiver) = mpsc::channel();
    let folder = home.path().to_owned();
    std::thread::spawn(move || sender.send(refusal(&folder)).unwrap());

    assert_eq!(
        receiver.recv_timeout(Duration::from_secs(5)).unwrap(),
        format!("{}: not a regular file", path.display())
    );
}

#[test]
fn a_file_over_64_kib_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), &"#".repeat(64 * 1024 + 1));

    assert_eq!(
        refusal(home.path()),
        format!("{}: larger than 64 KiB", path.display())
    );
}

#[test]
fn a_file_of_64_kib_is_read() {
    let home = tempfile::tempdir().unwrap();
    let padding = "#".repeat(64 * 1024 - "accent = \"red\"\n".len());
    write_config(home.path(), &format!("accent = \"red\"\n{padding}"));

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.accent().source, Source::File);
}

#[test]
fn a_config_folder_that_cannot_be_searched_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "accent = \"red\"\n");
    let folder = path.parent().unwrap().to_owned();
    fs::set_permissions(&folder, fs::Permissions::from_mode(0o000)).unwrap();

    let found = refusal(home.path());
    fs::set_permissions(&folder, fs::Permissions::from_mode(0o700)).unwrap();

    assert_eq!(
        found,
        format!("{}: Permission denied (os error 13)", path.display())
    );
}

fn refusal(home: &Path) -> String {
    match read_at(home, None) {
        Ok(_) => "read without a refusal".to_string(),
        Err(refusal) => refusal.to_string(),
    }
}
