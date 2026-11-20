use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use crate::helpers::{stderr, stdout, which};

#[test]
fn a_colour_set_on_one_line_starts_the_next_line_too() {
    let output = truecolor("\x1b[38;5;6m┏━┓\n┃ ┃\n");

    let second = stdout(&output).lines().nth(1).unwrap().to_string();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(
        second.starts_with("\x1b[0;38;2;148;226;213m┃"),
        "{second:?}"
    );
}

#[test]
fn the_output_ends_on_the_last_newline_of_the_input() {
    let output = truecolor("\x1b[38;5;6m┏━┓\n");

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(stdout(&output).ends_with("┏━┓\n"), "{:?}", stdout(&output));
}

fn truecolor(input: &str) -> Output {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/truecolor.py");
    let mut child = Command::new(which("python3"))
        .arg(script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}
