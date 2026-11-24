use std::path::Path;

use harbour::app::FOCUSED_BORDER;

use crate::log::wait_for_log;
use crate::signals::wait_for_empty;
use crate::{Server, WORKING, focused_at, pane_corner, wait_for_status};

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

#[test]
#[ignore]
fn a_wait_for_a_log_line_never_written_fails_naming_it_and_the_log() {
    let home = tempfile::tempdir().unwrap();
    let logs = home.path().join(".local/state/harbour/log");
    std::fs::create_dir_all(&logs).unwrap();
    std::fs::write(logs.join("4100.log"), "acme written\n").unwrap();

    let failure = failure(|| {
        wait_for_log(home.path(), "acme never written");
    });

    assert!(failure.contains(r#""acme never written""#), "{failure}");
    assert!(failure.contains("acme written"), "{failure}");
}

#[test]
#[ignore]
fn a_wait_for_an_empty_folder_that_keeps_a_file_fails_naming_it_and_the_file() {
    let home = tempfile::tempdir().unwrap();
    let folder = home.path().join(".local/state/harbour/acme");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("acme.kept"), "").unwrap();

    let failure = failure(|| {
        wait_for_empty(home.path(), "acme");
    });

    assert!(failure.contains("empty acme folder"), "{failure}");
    assert!(failure.contains("acme.kept"), "{failure}");
}

#[test]
fn a_wide_character_before_a_corner_counts_two_columns() {
    let set = FOCUSED_BORDER.to_border_set();
    let screen = format!(
        "修{}{}\n  {}",
        set.top_left, set.horizontal_top, set.vertical_left
    );
    assert!(focused_at(&screen, 2));
}

#[test]
fn a_wide_title_before_the_pane_corner_counts_two_columns() {
    let corner = FOCUSED_BORDER.to_border_set().top_left;
    let screen = format!("{corner}修{corner}");
    assert_eq!(pane_corner(&screen), Some(3));
}

fn failure(wait: impl FnOnce()) -> String {
    let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(wait))
        .expect_err("the wait returned without failing");
    unwound
        .downcast_ref::<String>()
        .cloned()
        .unwrap_or_default()
}
