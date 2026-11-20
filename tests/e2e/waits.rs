use std::path::Path;

use crate::{Server, WORKING, wait_for_status};

#[test]
#[ignore]
fn a_wait_for_text_that_never_shows_fails_naming_it_and_showing_the_screen() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("wait-screen", 80, 10);
    server.run(home.path(), &[], &["--version"]);
    server.wait_for_screen("harbour 0.0.0");

    let failure = failure(|| {
        server.wait_for_screen("acme never shown");
    });

    assert!(failure.contains(r#"for ["acme never shown"]"#), "{failure}");
    assert!(failure.contains("harbour 0.0.0"), "{failure}");
}

#[test]
#[ignore]
fn a_wait_for_a_path_never_entered_fails_naming_it_and_the_last_path() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("wait-path", 80, 10);
    server.run(home.path(), &[], &["--version"]);
    server.wait_for_screen("harbour 0.0.0");

    let failure = failure(|| {
        server.wait_for_path(Path::new("/acme/never"));
    });

    assert!(failure.contains("/acme/never"), "{failure}");
    assert!(failure.contains("acme-api"), "{failure}");
}

#[test]
#[ignore]
fn a_wait_for_a_status_line_never_printed_fails_naming_it_and_the_last_line() {
    let home = tempfile::tempdir().unwrap();

    let failure = failure(|| {
        wait_for_status(home.path(), WORKING);
    });

    assert!(failure.contains(&format!("{WORKING:?}")), "{failure}");
    assert!(failure.contains(r#"last line: """#), "{failure}");
}

fn failure(wait: impl FnOnce()) -> String {
    let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(wait))
        .expect_err("the wait returned without failing");
    unwound
        .downcast_ref::<String>()
        .cloned()
        .unwrap_or_default()
}
