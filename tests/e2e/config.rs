use std::path::{Path, PathBuf};

use crate::Server;

#[test]
#[ignore]
fn a_syntax_error_stops_harbour_before_the_interface_opens() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "accent = \"cyan\"\neditor = [\"nvim\"\n");
    let server = Server::start("config-syntax", 200, 20);

    server.run(home.path(), &[], &[]);

    let screen = server.wait_for_screen("harbour exited 1");
    assert!(
        screen.contains(&format!("harbour: {}:2: editor: ", path.display())),
        "{screen}"
    );
    assert!(!screen.contains("No session yet"), "{screen}");
}

#[test]
#[ignore]
fn a_wrong_type_stops_harbour_before_the_interface_opens() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "sidebar_width = 0\n");
    let server = Server::start("config-type", 200, 20);

    server.run(home.path(), &[], &[]);

    let screen = server.wait_for_screen("harbour exited 1");
    assert!(
        screen.contains(&format!(
            "harbour: {}:1: sidebar_width: expected a whole number of columns above 0",
            path.display()
        )),
        "{screen}"
    );
}

#[test]
#[ignore]
fn an_unknown_key_stops_harbour_before_the_interface_opens() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "retention_days = 7\n");
    let server = Server::start("config-unknown", 200, 20);

    server.run(home.path(), &[], &[]);

    let screen = server.wait_for_screen("harbour exited 1");
    assert!(
        screen.contains(&format!(
            "harbour: {}:1: retention_days: unknown key",
            path.display()
        )),
        "{screen}"
    );
}

#[test]
#[ignore]
fn the_demo_never_reads_the_file() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "retention_days = 7\n");
    let server = Server::start("config-demo", 160, 26);

    server.run(home.path(), &[], &["--demo"]);

    let screen = server.wait_for_screen("permission prompt");
    assert!(!screen.contains("retention_days"), "{screen}");
}

fn write_config(home: &Path, text: &str) -> PathBuf {
    let path = home.join(".config/harbour/config.toml");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, text).unwrap();
    path
}
