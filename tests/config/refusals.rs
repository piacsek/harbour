use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

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

fn refusal(home: &Path) -> String {
    match read_at(home, None) {
        Ok(_) => "read without a refusal".to_string(),
        Err(refusal) => refusal.to_string(),
    }
}
