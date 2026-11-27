use std::fs;
use std::path::{Path, PathBuf};

use harbour::repo::{common, identity, place};

#[test]
fn a_folder_outside_any_repository_shows_its_name() {
    let scratch = tempfile::tempdir().unwrap();
    let folder = folder(scratch.path(), "acme-notes");

    assert_eq!(place(&folder, None).to_string(), "acme-notes");
}

#[test]
fn a_repository_shows_its_folder_and_branch() {
    let scratch = tempfile::tempdir().unwrap();
    let repository = repository(scratch.path(), "acme-api", "ref: refs/heads/main\n");

    assert_eq!(place(&repository, None).to_string(), "acme-api · main");
}

#[test]
fn a_folder_inside_a_repository_shows_the_repository() {
    let scratch = tempfile::tempdir().unwrap();
    let repository = repository(scratch.path(), "acme-api", "ref: refs/heads/main\n");
    let inside = folder(&repository, "src/billing");

    assert_eq!(place(&inside, None).to_string(), "acme-api · main");
}

#[test]
fn a_branch_with_slashes_keeps_them() {
    let scratch = tempfile::tempdir().unwrap();
    let repository = repository(scratch.path(), "acme-api", "ref: refs/heads/fix/login\n");

    assert_eq!(place(&repository, None).to_string(), "acme-api · fix/login");
}

#[test]
fn a_detached_head_reads_head() {
    let scratch = tempfile::tempdir().unwrap();
    let repository = repository(
        scratch.path(),
        "acme-api",
        "8c1f2a7d9e04b6c35a1f0e2d7b9c4a6e1f3d5b70\n",
    );

    assert_eq!(place(&repository, None).to_string(), "acme-api · HEAD");
}

#[test]
fn a_worktree_shows_the_main_repository_and_its_own_branch() {
    let scratch = tempfile::tempdir().unwrap();
    let main = repository(scratch.path(), "acme-api", "ref: refs/heads/main\n");
    let admin = folder(&main, ".git/worktrees/fix-login");
    fs::write(admin.join("commondir"), "../..\n").unwrap();
    fs::write(admin.join("HEAD"), "ref: refs/heads/fix-login\n").unwrap();
    let worktree = folder(scratch.path(), "elsewhere/fix-login");
    fs::write(
        worktree.join(".git"),
        format!("gitdir: {}\n", admin.display()),
    )
    .unwrap();

    assert_eq!(place(&worktree, None).to_string(), "acme-api · fix-login");
}

#[test]
fn a_branch_name_that_is_not_utf8_still_shows() {
    let scratch = tempfile::tempdir().unwrap();
    let repository = repository(scratch.path(), "acme-api", "");
    fs::write(
        repository.join(".git/HEAD"),
        b"ref: refs/heads/fix-\xff-login\n",
    )
    .unwrap();

    assert_eq!(
        place(&repository, None).to_string(),
        "acme-api \u{b7} fix-\u{fffd}-login"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn a_worktree_whose_gitdir_path_is_not_utf8_still_shows_its_repository() {
    use std::ffi::OsStr;
    use std::io::Write;
    use std::os::unix::ffi::OsStrExt;

    let scratch = tempfile::tempdir().unwrap();
    let latin = scratch.path().join(OsStr::from_bytes(b"caf\xe9"));
    fs::create_dir_all(&latin).unwrap();
    let main = repository(&latin, "acme-api", "ref: refs/heads/main\n");
    let admin = folder(&main, ".git/worktrees/fix-login");
    fs::write(admin.join("commondir"), "../..\n").unwrap();
    fs::write(admin.join("HEAD"), "ref: refs/heads/fix-login\n").unwrap();
    let worktree = folder(scratch.path(), "elsewhere/fix-login");
    let mut file = fs::File::create(worktree.join(".git")).unwrap();
    file.write_all(b"gitdir: ").unwrap();
    file.write_all(admin.as_os_str().as_bytes()).unwrap();
    file.write_all(b"\n").unwrap();

    assert_eq!(
        place(&worktree, None).to_string(),
        "acme-api \u{b7} fix-login"
    );
}

#[test]
fn a_worktree_with_a_relative_gitdir_is_found_too() {
    let scratch = tempfile::tempdir().unwrap();
    let main = repository(scratch.path(), "acme-api", "ref: refs/heads/main\n");
    let admin = folder(&main, ".git/worktrees/fix-login");
    fs::write(admin.join("commondir"), "../..\n").unwrap();
    fs::write(admin.join("HEAD"), "ref: refs/heads/fix-login\n").unwrap();
    let worktree = folder(&main, ".claude/worktrees/fix-login");
    fs::write(
        worktree.join(".git"),
        "gitdir: ../../../.git/worktrees/fix-login\n",
    )
    .unwrap();

    assert_eq!(place(&worktree, None).to_string(), "acme-api · fix-login");
}

#[test]
fn a_submodule_shows_its_own_folder() {
    let scratch = tempfile::tempdir().unwrap();
    let parent = repository(scratch.path(), "acme-api", "ref: refs/heads/main\n");
    let module = folder(&parent, ".git/modules/acme-vendor");
    fs::write(module.join("HEAD"), "ref: refs/heads/stable\n").unwrap();
    let submodule = folder(&parent, "vendor/acme-vendor");
    fs::write(
        submodule.join(".git"),
        "gitdir: ../../.git/modules/acme-vendor\n",
    )
    .unwrap();

    assert_eq!(place(&submodule, None).to_string(), "acme-vendor · stable");
}

#[test]
fn a_bare_repository_drops_the_git_suffix() {
    let scratch = tempfile::tempdir().unwrap();
    let bare = folder(scratch.path(), "acme-api.git");
    folder(&bare, "objects");
    folder(&bare, "refs");
    fs::write(bare.join("HEAD"), "ref: refs/heads/main\n").unwrap();

    assert_eq!(place(&bare, None).to_string(), "acme-api · main");
}

#[test]
fn a_worktree_of_a_bare_repository_shows_the_bare_repository_without_its_suffix() {
    let scratch = tempfile::tempdir().unwrap();
    let bare = folder(scratch.path(), "repos/acme-api.git");
    folder(&bare, "objects");
    folder(&bare, "refs");
    fs::write(bare.join("HEAD"), "ref: refs/heads/main\n").unwrap();
    let admin = folder(&bare, "worktrees/topic");
    fs::write(admin.join("commondir"), "../..\n").unwrap();
    fs::write(admin.join("HEAD"), "ref: refs/heads/topic\n").unwrap();
    let worktree = folder(scratch.path(), "work/topic");
    fs::write(
        worktree.join(".git"),
        format!("gitdir: {}\n", admin.display()),
    )
    .unwrap();

    assert_eq!(place(&worktree, None).to_string(), "acme-api · topic");
}

#[test]
fn a_deleted_folder_shows_the_repository_above_it() {
    let scratch = tempfile::tempdir().unwrap();
    let repository = repository(scratch.path(), "acme-api", "ref: refs/heads/main\n");
    let gone = repository.join("build/tmp");

    assert_eq!(place(&gone, None).to_string(), "acme-api · main");
}

#[test]
fn a_deleted_folder_outside_any_repository_shows_the_name_it_had() {
    let scratch = tempfile::tempdir().unwrap();
    let gone = scratch.path().join("acme-scratch");

    assert_eq!(place(&gone, None).to_string(), "acme-scratch");
}

#[test]
fn an_unreadable_head_leaves_the_branch_out() {
    let scratch = tempfile::tempdir().unwrap();
    let repository = repository(scratch.path(), "acme-api", "\u{0}\u{1}not a head");

    assert_eq!(place(&repository, None).to_string(), "acme-api");
}

#[test]
fn a_reftable_repository_s_placeholder_head_leaves_the_branch_out() {
    let scratch = tempfile::tempdir().unwrap();
    let repository = repository(scratch.path(), "acme-api", "ref: refs/heads/.invalid\n");

    assert_eq!(place(&repository, None).to_string(), "acme-api");
}

#[test]
fn a_gitdir_file_that_points_nowhere_shows_the_folder() {
    let scratch = tempfile::tempdir().unwrap();
    let broken = folder(scratch.path(), "acme-broken");
    fs::write(broken.join(".git"), "gitdir: /nowhere/acme\n").unwrap();

    assert_eq!(place(&broken, None).to_string(), "acme-broken");
}

#[test]
fn a_repository_at_the_home_directory_does_not_count() {
    let scratch = tempfile::tempdir().unwrap();
    let home = repository(scratch.path(), "acme-home", "ref: refs/heads/main\n");
    let notes = folder(&home, "acme-notes");

    assert_eq!(place(&notes, Some(&home)).to_string(), "acme-notes");
}

#[test]
fn a_repository_below_the_home_directory_still_counts() {
    let scratch = tempfile::tempdir().unwrap();
    let home = repository(scratch.path(), "acme-home", "ref: refs/heads/main\n");
    let api = repository(&home, "acme-api", "ref: refs/heads/fix-login\n");
    let inside = folder(&api, "src");

    assert_eq!(
        place(&inside, Some(&home)).to_string(),
        "acme-api · fix-login"
    );
}

#[test]
fn a_repository_at_a_home_directory_reached_through_a_link_does_not_count() {
    let scratch = tempfile::tempdir().unwrap();
    let home = repository(scratch.path(), "acme-home", "ref: refs/heads/main\n");
    let notes = folder(&home, "acme-notes");
    let linked = scratch.path().join("acme-link");
    std::os::unix::fs::symlink(&home, &linked).unwrap();

    assert_eq!(place(&notes, Some(&linked)).to_string(), "acme-notes");
}

#[test]
fn the_common_git_folder_of_a_worktree_reached_through_a_link_follows_its_relative_paths() {
    let scratch = tempfile::tempdir().unwrap();
    let real = folder(scratch.path(), "real");
    let app = repository(&real, "acme-app", "ref: refs/heads/main\n");
    let git_dir = folder(&app, ".git/worktrees/acme-app-feature");
    fs::write(git_dir.join("commondir"), "../..\n").unwrap();
    let feature = folder(&real, "acme-app-feature");
    fs::write(
        feature.join(".git"),
        "gitdir: ../acme-app/.git/worktrees/acme-app-feature\n",
    )
    .unwrap();
    let elsewhere = folder(scratch.path(), "elsewhere");
    let link = elsewhere.join("acme-link");
    std::os::unix::fs::symlink(&feature, &link).unwrap();

    let common = common(&link).unwrap();

    assert_eq!(identity(&common), identity(&app.join(".git")));
    assert!(identity(&common).is_some());
}

fn folder(parent: &Path, name: &str) -> PathBuf {
    let folder = parent.join(name);
    fs::create_dir_all(&folder).unwrap();
    folder
}

fn repository(parent: &Path, name: &str, head: &str) -> PathBuf {
    let repository = folder(parent, name);
    let git = folder(&repository, ".git");
    fs::write(git.join("HEAD"), head).unwrap();
    repository
}
