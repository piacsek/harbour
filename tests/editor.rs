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

#[test]
fn the_mirrored_directory_is_the_start_subfolder_in_the_worktree_claude_code_moved_to() {
    let scratch = tempfile::tempdir().unwrap();
    let app = checkout(scratch.path(), "acme-app");
    let feature = worktree(scratch.path(), &app, "acme-app-feature");
    fs::create_dir_all(app.join("rust")).unwrap();
    fs::create_dir_all(feature.join("rust")).unwrap();

    assert_eq!(
        directory(&feature, &app.join("rust"), None, EditorDir::Mirror),
        feature.join("rust")
    );
}

#[test]
fn the_mirrored_directory_keeps_the_start_subfolder_wherever_claude_code_went_in_the_worktree() {
    let scratch = tempfile::tempdir().unwrap();
    let app = checkout(scratch.path(), "acme-app");
    let feature = worktree(scratch.path(), &app, "acme-app-feature");
    fs::create_dir_all(app.join("rust/tests")).unwrap();
    fs::create_dir_all(feature.join("rust/tests")).unwrap();
    fs::create_dir_all(feature.join("docs")).unwrap();

    assert_eq!(
        directory(
            &feature.join("docs"),
            &app.join("rust/tests"),
            None,
            EditorDir::Mirror
        ),
        feature.join("rust/tests")
    );
}

#[test]
fn the_mirrored_directory_is_the_worktree_root_when_it_lacks_the_start_subfolder() {
    let scratch = tempfile::tempdir().unwrap();
    let app = checkout(scratch.path(), "acme-app");
    let feature = worktree(scratch.path(), &app, "acme-app-feature");
    fs::create_dir_all(app.join("rust")).unwrap();
    fs::create_dir_all(feature.join("docs")).unwrap();

    assert_eq!(
        directory(
            &feature.join("docs"),
            &app.join("rust"),
            None,
            EditorDir::Mirror
        ),
        feature
    );
}

#[test]
fn the_mirrored_directory_of_a_session_started_at_a_root_is_the_worktree_root() {
    let scratch = tempfile::tempdir().unwrap();
    let app = checkout(scratch.path(), "acme-app");
    let feature = worktree(scratch.path(), &app, "acme-app-feature");
    fs::create_dir_all(feature.join("docs")).unwrap();

    assert_eq!(
        directory(&feature.join("docs"), &app, None, EditorDir::Mirror),
        feature
    );
}

#[test]
fn the_mirrored_directory_follows_claude_code_back_from_a_worktree_to_the_main_checkout() {
    let scratch = tempfile::tempdir().unwrap();
    let app = checkout(scratch.path(), "acme-app");
    let feature = worktree(scratch.path(), &app, "acme-app-feature");
    fs::create_dir_all(app.join("rust")).unwrap();
    fs::create_dir_all(feature.join("rust")).unwrap();

    assert_eq!(
        directory(&app, &feature.join("rust"), None, EditorDir::Mirror),
        app.join("rust")
    );
}

#[test]
fn the_mirrored_directory_stays_in_the_start_folder_while_claude_code_stays_in_its_checkout() {
    let scratch = tempfile::tempdir().unwrap();
    let app = checkout(scratch.path(), "acme-app");
    fs::create_dir_all(app.join("rust")).unwrap();
    fs::create_dir_all(app.join("docs")).unwrap();

    assert_eq!(
        directory(
            &app.join("docs"),
            &app.join("rust"),
            None,
            EditorDir::Mirror
        ),
        app.join("rust")
    );
}

#[test]
fn the_mirrored_directory_stays_in_the_start_folder_when_claude_code_reaches_its_checkout_by_a_symlink()
 {
    let scratch = tempfile::tempdir().unwrap();
    let app = checkout(scratch.path(), "acme-app");
    fs::create_dir_all(app.join("rust")).unwrap();
    fs::create_dir_all(app.join("docs")).unwrap();
    let link = scratch.path().join("acme-link");
    std::os::unix::fs::symlink(&app, &link).unwrap();

    assert_eq!(
        directory(
            &link.join("docs"),
            &app.join("rust"),
            None,
            EditorDir::Mirror
        ),
        app.join("rust")
    );
}

#[test]
fn the_mirrored_directory_is_the_start_folder_when_claude_codes_directory_is_gone() {
    let scratch = tempfile::tempdir().unwrap();
    let app = checkout(scratch.path(), "acme-app");
    let feature = worktree(scratch.path(), &app, "acme-app-feature");
    fs::create_dir_all(app.join("rust")).unwrap();
    fs::create_dir_all(feature.join("rust")).unwrap();

    assert_eq!(
        directory(
            &feature.join("gone"),
            &app.join("rust"),
            None,
            EditorDir::Mirror
        ),
        app.join("rust")
    );
}

#[test]
fn the_mirrored_directory_is_the_start_folder_when_claude_code_left_every_repository() {
    let scratch = tempfile::tempdir().unwrap();
    let app = checkout(scratch.path(), "acme-app");
    let plain = scratch.path().join("acme-plain");
    fs::create_dir_all(app.join("rust")).unwrap();
    fs::create_dir_all(&plain).unwrap();

    assert_eq!(
        directory(&plain, &app.join("rust"), None, EditorDir::Mirror),
        app.join("rust")
    );
}

#[test]
fn the_mirrored_directory_is_the_start_folder_when_the_session_started_outside_a_repository() {
    let scratch = tempfile::tempdir().unwrap();
    let app = checkout(scratch.path(), "acme-app");
    let feature = worktree(scratch.path(), &app, "acme-app-feature");
    let plain = scratch.path().join("acme-plain/notes");
    fs::create_dir_all(&plain).unwrap();

    assert_eq!(directory(&feature, &plain, None, EditorDir::Mirror), plain);
}

fn checkout(scratch: &Path, name: &str) -> PathBuf {
    let checkout = scratch.join(name);
    fs::create_dir_all(checkout.join(".git")).unwrap();
    checkout
}

fn worktree(scratch: &Path, main: &Path, name: &str) -> PathBuf {
    let git_dir = main.join(".git/worktrees").join(name);
    fs::create_dir_all(&git_dir).unwrap();
    let worktree = scratch.join(name);
    fs::create_dir_all(&worktree).unwrap();
    fs::write(
        worktree.join(".git"),
        format!("gitdir: {}\n", git_dir.display()),
    )
    .unwrap();
    worktree
}

fn long_directory(length: usize) -> PathBuf {
    let path = PathBuf::from(format!("/{}", "a".repeat(length - 1)));
    assert_eq!(path.as_os_str().len(), length);
    path
}
