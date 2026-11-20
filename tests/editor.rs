use std::path::{Path, PathBuf};

use harbour::editor::socket_base;

#[test]
fn a_short_temporary_directory_holds_the_socket() {
    let temporary = Path::new("/var/folders/ab/acme/T");

    assert_eq!(socket_base(temporary), temporary);
}

#[test]
fn a_temporary_directory_too_long_for_a_socket_path_gives_way_to_tmp() {
    let temporary = long_directory(78);

    assert_eq!(socket_base(&temporary), Path::new("/tmp"));
}

#[test]
fn the_longest_temporary_directory_that_still_fits_holds_the_socket() {
    let temporary = long_directory(77);

    assert_eq!(socket_base(&temporary), temporary);
}

fn long_directory(length: usize) -> PathBuf {
    let path = PathBuf::from(format!("/{}", "a".repeat(length - 1)));
    assert_eq!(path.as_os_str().len(), length);
    path
}
