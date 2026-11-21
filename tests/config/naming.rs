use crate::{read_at, read_with, write_config};

#[test]
fn a_session_command_named_claude_from_the_file_names_its_sessions() {
    let home = tempfile::tempdir().unwrap();
    write_config(
        home.path(),
        "session_command = [\"claude\", \"--model\", \"acme\"]\n",
    );

    let config = read_at(home.path(), None).unwrap();

    assert!(config.names_sessions());
}

#[test]
fn a_session_command_under_a_folder_named_claude_names_its_sessions() {
    let home = tempfile::tempdir().unwrap();
    write_config(
        home.path(),
        "session_command = [\"/opt/acme/bin/claude\"]\n",
    );

    let config = read_at(home.path(), None).unwrap();

    assert!(config.names_sessions());
}

#[test]
fn a_default_session_command_names_its_sessions() {
    let home = tempfile::tempdir().unwrap();

    let config = read_at(home.path(), None).unwrap();

    assert!(config.names_sessions());
}

#[test]
fn a_session_command_from_the_file_not_named_claude_names_none() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "session_command = [\"acme-agent\"]\n");

    let config = read_at(home.path(), None).unwrap();

    assert!(!config.names_sessions());
}

#[test]
fn a_claude_that_continues_a_conversation_names_none() {
    let home = tempfile::tempdir().unwrap();
    write_config(
        home.path(),
        "session_command = [\"claude\", \"--continue\"]\n",
    );

    let config = read_at(home.path(), None).unwrap();

    assert!(!config.names_sessions());
}

#[test]
fn a_claude_given_c_names_none() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "session_command = [\"claude\", \"-c\"]\n");

    let config = read_at(home.path(), None).unwrap();

    assert!(!config.names_sessions());
}

#[test]
fn a_claude_that_resumes_a_conversation_names_none() {
    let home = tempfile::tempdir().unwrap();
    write_config(
        home.path(),
        "session_command = [\"claude\", \"--resume\", \"acme\"]\n",
    );

    let config = read_at(home.path(), None).unwrap();

    assert!(!config.names_sessions());
}

#[test]
fn a_claude_given_r_names_none() {
    let home = tempfile::tempdir().unwrap();
    write_config(
        home.path(),
        "session_command = [\"claude\", \"-r\", \"acme\"]\n",
    );

    let config = read_at(home.path(), None).unwrap();

    assert!(!config.names_sessions());
}

#[test]
fn a_claude_given_its_own_session_id_names_none() {
    let home = tempfile::tempdir().unwrap();
    write_config(
        home.path(),
        "session_command = [\"claude\", \"--session-id\", \"0f1e2d3c-4b5a-4968-8776-655443322110\"]\n",
    );

    let config = read_at(home.path(), None).unwrap();

    assert!(!config.names_sessions());
}

#[test]
fn a_claude_that_resumes_with_an_equals_sign_names_none() {
    let home = tempfile::tempdir().unwrap();
    write_config(
        home.path(),
        "session_command = [\"claude\", \"--resume=acme\"]\n",
    );

    let config = read_at(home.path(), None).unwrap();

    assert!(!config.names_sessions());
}

#[test]
fn a_claude_that_continues_with_an_equals_sign_names_none() {
    let home = tempfile::tempdir().unwrap();
    write_config(
        home.path(),
        "session_command = [\"claude\", \"--continue=true\"]\n",
    );

    let config = read_at(home.path(), None).unwrap();

    assert!(!config.names_sessions());
}

#[test]
fn a_claude_given_its_own_session_id_with_an_equals_sign_names_none() {
    let home = tempfile::tempdir().unwrap();
    write_config(
        home.path(),
        "session_command = [\"claude\", \"--session-id=0f1e2d3c-4b5a-4968-8776-655443322110\"]\n",
    );

    let config = read_at(home.path(), None).unwrap();

    assert!(!config.names_sessions());
}

#[test]
fn a_session_command_named_claude_from_a_variable_names_none() {
    let home = tempfile::tempdir().unwrap();

    let config = read_with(home.path(), &[("HARBOUR_SESSION_COMMAND", "claude")]);

    assert!(!config.names_sessions());
}
