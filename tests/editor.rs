use std::fs;
use std::path::{Path, PathBuf};

use harbour::editor::{root, socket_base};

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

#[test]
fn a_repository_at_the_home_directory_is_not_the_root() {
    let scratch = tempfile::tempdir().unwrap();
    let home = scratch.path().join("acme-home");
    let notes = home.join("acme-notes");
    fs::create_dir_all(home.join(".git")).unwrap();
    fs::create_dir_all(&notes).unwrap();

    assert_eq!(root(&notes, Some(&home)), notes);
}

#[test]
fn a_repository_below_the_home_directory_is_the_root() {
    let scratch = tempfile::tempdir().unwrap();
    let home = scratch.path().join("acme-home");
    let api = home.join("acme-api");
    fs::create_dir_all(home.join(".git")).unwrap();
    fs::create_dir_all(api.join(".git")).unwrap();
    fs::create_dir_all(api.join("src")).unwrap();

    assert_eq!(root(&api.join("src"), Some(&home)), api);
}

fn long_directory(length: usize) -> PathBuf {
    let path = PathBuf::from(format!("/{}", "a".repeat(length - 1)));
    assert_eq!(path.as_os_str().len(), length);
    path
}
