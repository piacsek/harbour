use std::ffi::OsString;

use harbour::config::Source;

use crate::{read_with, words, write_config};

const FILE: &str = "editor = [\"nvim\", \"--clean\"]\npicker = [\"/opt/acme/pick\", \"--dirs\"]\nsession_command = [\"claude\", \"--model\", \"acme\"]\n";

#[test]
fn the_editor_variable_outranks_the_file() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), FILE);

    let config = read_with(home.path(), &[("HARBOUR_EDITOR", "/opt/acme/vim")]);

    assert_eq!(config.editor().value, vec![OsString::from("/opt/acme/vim")]);
    assert_eq!(config.editor().source, Source::Variable("HARBOUR_EDITOR"));
}

#[test]
fn the_picker_variable_outranks_the_file() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), FILE);

    let config = read_with(home.path(), &[("HARBOUR_DIR_PICKER", "acme-pick")]);

    assert_eq!(config.picker().value, words(&["acme-pick"]));
    assert_eq!(
        config.picker().source,
        Source::Variable("HARBOUR_DIR_PICKER")
    );
}

#[test]
fn the_session_command_variable_outranks_the_file() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), FILE);

    let config = read_with(home.path(), &[("HARBOUR_SESSION_COMMAND", "acme-agent")]);

    assert_eq!(config.session_command().value, words(&["acme-agent"]));
    assert_eq!(
        config.session_command().source,
        Source::Variable("HARBOUR_SESSION_COMMAND")
    );
}

#[test]
fn a_variable_outranks_the_default() {
    let home = tempfile::tempdir().unwrap();

    let config = read_with(home.path(), &[("HARBOUR_EDITOR", "acme-vim")]);

    assert_eq!(config.editor().value, vec![OsString::from("acme-vim")]);
    assert_eq!(config.editor().source, Source::Variable("HARBOUR_EDITOR"));
}

#[test]
fn a_variable_keeps_a_relative_path_as_it_did() {
    let home = tempfile::tempdir().unwrap();

    let config = read_with(home.path(), &[("HARBOUR_DIR_PICKER", "bin/acme-pick")]);

    assert_eq!(config.picker().value, words(&["bin/acme-pick"]));
}

#[test]
fn an_empty_variable_falls_through_to_the_file() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), FILE);

    let config = read_with(
        home.path(),
        &[
            ("HARBOUR_EDITOR", ""),
            ("HARBOUR_DIR_PICKER", ""),
            ("HARBOUR_SESSION_COMMAND", ""),
        ],
    );

    assert_eq!(config.editor().source, Source::File);
    assert_eq!(config.picker().source, Source::File);
    assert_eq!(config.session_command().source, Source::File);
}

#[test]
fn an_empty_session_command_variable_runs_claude() {
    let home = tempfile::tempdir().unwrap();

    let config = read_with(home.path(), &[("HARBOUR_SESSION_COMMAND", "")]);

    assert_eq!(config.session_command().value, words(&["claude"]));
    assert_eq!(config.session_command().source, Source::Default);
}

#[test]
fn a_variable_does_not_hide_a_bad_file() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "editor = \"nvim\"\n");

    let refused = harbour::config::Config::read(|name| match name {
        "HOME" => Some(home.path().as_os_str().to_owned()),
        "HARBOUR_EDITOR" => Some(OsString::from("acme-vim")),
        _ => None,
    });

    assert!(refused.is_err());
}
