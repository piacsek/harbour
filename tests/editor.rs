use std::fs;
use std::path::{Path, PathBuf};

use harbour::editor::{EditorDir, directory, socket_base};

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

    assert_eq!(
        directory(&notes, &home, Some(&home), EditorDir::Root),
        notes
    );
}

#[test]
fn a_repository_below_the_home_directory_is_the_root() {
    let scratch = tempfile::tempdir().unwrap();
    let home = scratch.path().join("acme-home");
    let api = home.join("acme-api");
    fs::create_dir_all(home.join(".git")).unwrap();
    fs::create_dir_all(api.join(".git")).unwrap();
    fs::create_dir_all(api.join("src")).unwrap();

    assert_eq!(
        directory(&api.join("src"), &home, Some(&home), EditorDir::Root),
        api
    );
}

#[test]
fn the_exact_directory_is_the_folder_the_session_started_in_wherever_claude_code_went() {
    let scratch = tempfile::tempdir().unwrap();
    let api = scratch.path().join("acme-api");
    fs::create_dir_all(api.join(".git")).unwrap();
    fs::create_dir_all(api.join("rust/tests")).unwrap();

    assert_eq!(
        directory(
            &api.join("rust/tests"),
            &api.join("rust"),
            None,
            EditorDir::Exact
        ),
        api.join("rust")
    );
}

#[test]
fn the_exact_directory_of_a_session_started_at_a_repository_root_is_that_root() {
    let scratch = tempfile::tempdir().unwrap();
    let api = scratch.path().join("acme-api");
    fs::create_dir_all(api.join(".git")).unwrap();
    fs::create_dir_all(api.join("src")).unwrap();

    assert_eq!(
        directory(&api.join("src"), &api, None, EditorDir::Exact),
        api
    );
}

#[test]
fn the_exact_directory_of_a_start_folder_that_is_gone_is_that_folder() {
    let scratch = tempfile::tempdir().unwrap();
    let api = scratch.path().join("acme-api");
    fs::create_dir_all(api.join(".git")).unwrap();
    let gone = api.join("gone");

    assert_eq!(directory(&api, &gone, None, EditorDir::Exact), gone);
}

fn long_directory(length: usize) -> PathBuf {
    let path = PathBuf::from(format!("/{}", "a".repeat(length - 1)));
    assert_eq!(path.as_os_str().len(), length);
    path
}
