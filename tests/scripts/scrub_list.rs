use std::fs;
use std::path::Path;

use crate::helpers::{stderr, stdout};
use crate::scrub_check::{denylist, scrub, scrub_outgoing, scrub_sandbox};

#[test]
fn a_denylist_line_ending_with_a_carriage_return_fails_the_scrub() {
    let dir = tempfile::tempdir().unwrap();
    let work = scrub_sandbox(dir.path());
    let list = denylist(dir.path(), "zz-other-term\nzz-secret-term\r\n");

    let output = scrub_outgoing(&work, &list);

    assert!(!output.status.success(), "{}", stdout(&output));
    assert!(
        stderr(&output).contains(&refusal(&list, 2, "ends with whitespace")),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_denylist_line_ending_with_a_space_fails_the_scrub() {
    let dir = tempfile::tempdir().unwrap();
    let work = scrub_sandbox(dir.path());
    let list = denylist(dir.path(), "zz-other-term\nzz-secret-term \n");

    let output = scrub(&work, Some(&list));

    assert!(!output.status.success(), "{}", stdout(&output));
    assert!(
        stderr(&output).contains(&refusal(&list, 2, "ends with whitespace")),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_denylist_line_ending_with_a_tab_fails_the_scrub() {
    let dir = tempfile::tempdir().unwrap();
    let work = scrub_sandbox(dir.path());
    let list = denylist(dir.path(), "zz-other-term\nzz-secret-term\t\n");

    let output = scrub(&work, Some(&list));

    assert!(!output.status.success(), "{}", stdout(&output));
    assert!(
        stderr(&output).contains(&refusal(&list, 2, "ends with whitespace")),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_blank_denylist_line_fails_the_scrub() {
    let dir = tempfile::tempdir().unwrap();
    let work = scrub_sandbox(dir.path());
    let list = denylist(dir.path(), "zz-other-term\n\nzz-secret-term\n");

    let output = scrub(&work, Some(&list));

    assert!(!output.status.success(), "{}", stdout(&output));
    assert!(
        stderr(&output).contains(&refusal(&list, 2, "is blank")),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_denylist_line_starting_with_a_space_fails_the_scrub() {
    let dir = tempfile::tempdir().unwrap();
    let work = scrub_sandbox(dir.path());
    let list = denylist(dir.path(), "zz-other-term\n zz-secret-term\n");

    let output = scrub(&work, Some(&list));

    assert!(!output.status.success(), "{}", stdout(&output));
    assert!(
        stderr(&output).contains(&refusal(&list, 2, "starts with whitespace")),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_denylist_starting_with_a_byte_order_mark_fails_the_scrub() {
    let dir = tempfile::tempdir().unwrap();
    let work = scrub_sandbox(dir.path());
    let list = denylist(dir.path(), "\u{feff}zz-secret-term\n");

    let output = scrub(&work, Some(&list));

    assert!(!output.status.success(), "{}", stdout(&output));
    assert!(
        stderr(&output).contains(&refusal(&list, 1, "holds a byte-order mark")),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_denylist_line_ending_with_a_no_break_space_fails_the_scrub() {
    let dir = tempfile::tempdir().unwrap();
    let work = scrub_sandbox(dir.path());
    let list = denylist(dir.path(), "zz-other-term\nzz-secret-term\u{a0}\n");

    let output = scrub(&work, Some(&list));

    assert!(!output.status.success(), "{}", stdout(&output));
    assert!(
        stderr(&output).contains(&refusal(&list, 2, "holds a no-break space")),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_denylist_line_holding_a_zero_width_space_fails_the_scrub() {
    let dir = tempfile::tempdir().unwrap();
    let work = scrub_sandbox(dir.path());
    let list = denylist(dir.path(), "zz-other-term\nzz-secret\u{200b}term\n");

    let output = scrub(&work, Some(&list));

    assert!(!output.status.success(), "{}", stdout(&output));
    assert!(
        stderr(&output).contains(&refusal(&list, 2, "holds a zero-width character")),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_refused_denylist_under_a_path_with_a_backslash_is_named_as_it_is() {
    let dir = tempfile::tempdir().unwrap();
    let work = scrub_sandbox(dir.path());
    let slashed = dir.path().join("back\\slash");
    fs::create_dir_all(&slashed).unwrap();
    let list = denylist(&slashed, "zz-other-term\n\nzz-secret-term\n");

    let output = scrub(&work, Some(&list));

    assert!(!output.status.success(), "{}", stdout(&output));
    assert!(
        stderr(&output).contains(&refusal(&list, 2, "is blank")),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_denylist_term_with_an_accented_letter_is_accepted() {
    let dir = tempfile::tempdir().unwrap();
    let work = scrub_sandbox(dir.path());
    let list = denylist(dir.path(), "acm\u{e9}-secret\n");

    let output = scrub(&work, Some(&list));

    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(stdout(&output), "scrub-check: clean\n");
}

fn refusal(list: &Path, line: usize, why: &str) -> String {
    format!("scrub-check: {}:{line} {why}", list.display())
}
