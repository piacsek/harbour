use harbour::tui::leave_terminal;

#[test]
fn leaving_the_terminal_turns_off_mouse_capture_and_bracketed_paste() {
    let mut written = Vec::new();

    leave_terminal(&mut written).unwrap();

    let written = String::from_utf8(written).unwrap();
    assert!(written.contains("\x1b[?1000l"), "{written:?}");
    assert!(written.contains("\x1b[?2004l"), "{written:?}");
}
