use harbour::cursor::Shape;

#[test]
fn the_default_shape_is_never_sent_to_a_terminal_that_has_it() {
    let mut shape = Shape::default();

    let written = sent(&mut shape, 0);

    assert_eq!(written, "");
}

#[test]
fn a_new_shape_is_sent_as_its_sequence() {
    let mut shape = Shape::default();

    let written = sent(&mut shape, 6);

    assert_eq!(written, "\x1b[?25l\x1b[6 q");
}

#[test]
fn a_shape_already_sent_is_not_sent_again() {
    let mut shape = Shape::default();
    sent(&mut shape, 4);

    let written = sent(&mut shape, 4);

    assert_eq!(written, "");
}

#[test]
fn going_back_to_the_default_sends_the_default() {
    let mut shape = Shape::default();
    sent(&mut shape, 2);

    let written = sent(&mut shape, 0);

    assert_eq!(written, "\x1b[?25l\x1b[0 q");
}

fn sent(shape: &mut Shape, wanted: u8) -> String {
    let mut written = Vec::new();
    shape.send(&mut written, wanted).unwrap();
    String::from_utf8(written).unwrap()
}
