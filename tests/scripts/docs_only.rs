use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::helpers::{committed, git, isolated, link_script, repository, stderr, stdout, which};

const DOCS_ONLY: &str = "only Markdown outside the binary changed on this branch\n";

#[test]
fn a_branch_of_markdown_outside_the_binary_is_docs_only() {
    let dir = tempfile::tempdir().unwrap();
    let work = checked_repository(dir.path());
    committed(&work, "AGENTS.md", "reworded\n");
    fs::write(work.join("README.md"), "changed\n").unwrap();
    fs::write(work.join("src/AGENTS.md"), "agents\n").unwrap();
    fs::create_dir_all(work.join(".claude/skills/sweep")).unwrap();
    fs::write(work.join(".claude/skills/sweep/SKILL.md"), "skill\n").unwrap();
    fs::create_dir_all(work.join("plans")).unwrap();
    fs::write(work.join("plans/phase-1-x.md"), "plan\n").unwrap();

    let output = docs_only(dir.path());

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(stdout(&output), DOCS_ONLY);
}

#[test]
fn a_markdown_file_turned_into_a_symlink_to_markdown_is_docs_only() {
    let dir = tempfile::tempdir().unwrap();
    let work = checked_repository(dir.path());
    committed(&work, "CLAUDE.md", "AGENTS.md\n");
    git(&work, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    fs::remove_file(work.join("CLAUDE.md")).unwrap();
    symlink("AGENTS.md", work.join("CLAUDE.md")).unwrap();

    let output = docs_only(dir.path());

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(stdout(&output), DOCS_ONLY);
}

#[test]
fn a_source_file_needs_the_tests() {
    let dir = tempfile::tempdir().unwrap();
    let work = checked_repository(dir.path());
    fs::write(work.join("README.md"), "changed\n").unwrap();
    fs::write(work.join("src/main.rs"), "fn main() { }\n").unwrap();

    let output = docs_only(dir.path());

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert_eq!(stdout(&output), "src/main.rs needs the tests\n");
}

#[test]
fn markdown_that_feeds_the_binary_needs_the_tests() {
    let dir = tempfile::tempdir().unwrap();
    let work = checked_repository(dir.path());
    fs::write(work.join("CHANGELOG.md"), "changed\n").unwrap();

    let output = docs_only(dir.path());

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert_eq!(stdout(&output), "CHANGELOG.md needs the tests\n");
}

#[test]
fn a_fragment_needs_the_tests() {
    let dir = tempfile::tempdir().unwrap();
    let work = checked_repository(dir.path());
    fs::create_dir_all(work.join("changes")).unwrap();
    fs::write(work.join("changes/topic.md"), "kind: fix\n\n- Fixed.\n").unwrap();

    let output = docs_only(dir.path());

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert_eq!(stdout(&output), "changes/topic.md needs the tests\n");
}

#[test]
fn markdown_the_script_tests_read_needs_the_tests() {
    let dir = tempfile::tempdir().unwrap();
    let work = checked_repository(dir.path());
    fs::create_dir_all(work.join(".github")).unwrap();
    fs::write(
        work.join(".github/pull_request_template.md"),
        "## Summary\n",
    )
    .unwrap();

    let output = docs_only(dir.path());

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        ".github/pull_request_template.md needs the tests\n"
    );
}

#[test]
fn an_untracked_script_needs_the_tests() {
    let dir = tempfile::tempdir().unwrap();
    let work = checked_repository(dir.path());
    fs::write(work.join("scripts/another.sh"), "").unwrap();

    let output = docs_only(dir.path());

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert_eq!(stdout(&output), "scripts/another.sh needs the tests\n");
}

#[test]
fn an_image_under_docs_needs_the_tests() {
    let dir = tempfile::tempdir().unwrap();
    let work = checked_repository(dir.path());
    fs::create_dir_all(work.join("docs")).unwrap();
    fs::write(work.join("docs/details.png"), "png").unwrap();

    let output = docs_only(dir.path());

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert_eq!(stdout(&output), "docs/details.png needs the tests\n");
}

#[test]
fn a_test_committed_earlier_on_the_branch_needs_the_tests() {
    let dir = tempfile::tempdir().unwrap();
    let work = checked_repository(dir.path());
    committed(&work, "tests/cli.rs", "");
    fs::write(work.join("README.md"), "changed\n").unwrap();

    let output = docs_only(dir.path());

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert_eq!(stdout(&output), "tests/cli.rs needs the tests\n");
}

#[test]
fn a_branch_with_no_change_needs_the_tests() {
    let dir = tempfile::tempdir().unwrap();
    checked_repository(dir.path());

    let output = docs_only(dir.path());

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "nothing on this branch differs from origin/main\n"
    );
}

#[test]
fn a_branch_with_no_known_main_needs_the_tests() {
    let dir = tempfile::tempdir().unwrap();
    let work = checked_repository(dir.path());
    fs::write(work.join("README.md"), "changed\n").unwrap();
    git(&work, &["update-ref", "-d", "refs/remotes/origin/main"]);

    let output = docs_only(dir.path());

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert_eq!(stdout(&output), "no merge base with origin/main\n");
}

#[test]
fn a_directory_outside_any_repository_needs_the_tests() {
    let dir = tempfile::tempdir().unwrap();
    let work = dir.path().join("work");
    link_script(&work, "docs-only.sh");
    link_script(&work, "feeds-binary.sh");

    let output = docs_only(dir.path());

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert_eq!(stdout(&output), "no merge base with origin/main\n");
}

fn checked_repository(root: &Path) -> PathBuf {
    let work = repository(root);
    link_script(&work, "docs-only.sh");
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-qm", "the check"]);
    git(&work, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    work
}

fn docs_only(root: &Path) -> Output {
    isolated(
        Command::new(which("bash"))
            .arg(root.join("work/scripts/docs-only.sh"))
            .env("GIT_CEILING_DIRECTORIES", root),
    )
    .output()
    .unwrap()
}
