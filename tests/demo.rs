use std::fs;
use std::panic;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use harbour::demo::{self, Registry};

#[test]
fn the_demo_registry_is_created_in_its_parent_and_removed_when_dropped() {
    let scratch = tempfile::tempdir().unwrap();

    let registry = Registry::create_in(scratch.path()).unwrap();
    let path = registry.path().to_path_buf();
    std::fs::write(path.join("1.json"), "{}").unwrap();
    assert!(path.is_dir());
    assert_eq!(path.parent(), Some(scratch.path()));
    drop(registry);

    assert!(!path.exists());
}

#[test]
fn two_demo_registries_in_one_parent_never_share_a_directory() {
    let scratch = tempfile::tempdir().unwrap();

    let first = Registry::create_in(scratch.path()).unwrap();
    let second = Registry::create_in(scratch.path()).unwrap();

    assert_ne!(first.path(), second.path());
}

#[test]
fn the_demo_registry_is_removed_when_a_panic_unwinds() {
    let scratch = tempfile::tempdir().unwrap();
    let parent = scratch.path().to_path_buf();

    let unwound = panic::catch_unwind(|| {
        let _registry = Registry::create_in(&parent).unwrap();
        panic!("acme demo failed");
    });

    assert!(unwound.is_err());
    assert_eq!(std::fs::read_dir(scratch.path()).unwrap().count(), 0);
}

#[test]
fn every_demo_transcript_names_a_checkout_inside_the_demo() {
    let scratch = tempfile::tempdir().unwrap();
    let registry = Registry::create_in(scratch.path()).unwrap();
    let root = registry.path();

    let idle = transcript_cwd(
        root,
        &demo::idle(root),
        "1d9f4c62-8e07-4a3b-b5c1-f2a6e8d03c97",
    );
    let working = transcript_cwd(
        root,
        &demo::working(root),
        "6f0c2a9e-31d4-4b7a-9e58-0d2c7f41a3b6",
    );
    let blocked = transcript_cwd(
        root,
        &demo::blocked(root),
        "b83e5d17-c0a2-4f69-8d14-7e9a2b6c05f1",
    );

    assert_eq!(idle, root.join("acme-web"));
    assert_eq!(working, root.join("acme-api"));
    assert_eq!(blocked, root.join("acme-api"));
    assert!(root.join("acme-web/.git/HEAD").is_file());
    assert!(root.join("acme-api/.git/HEAD").is_file());
}

#[test]
fn the_demos_stopped_session_has_a_title_of_its_own() {
    let scratch = tempfile::tempdir().unwrap();
    let registry = Registry::create_in(scratch.path()).unwrap();

    let recalled = demo::stopped(registry.path()).recall();

    assert_eq!(
        recalled[0].given_title.as_deref(),
        Some("acme-docs cleanup")
    );
}

fn transcript_cwd(registry: &Path, command: &[String], id: &str) -> PathBuf {
    let file = registry.join(format!("projects/demo/{id}.jsonl"));
    let mut child = Command::new(&command[0])
        .args(&command[1..])
        .current_dir(registry)
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    for _ in 0..100 {
        if fs::read_to_string(&file).is_ok_and(|text| text.ends_with('\n')) {
            break;
        }
        sleep(Duration::from_millis(50));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    fs::read_to_string(&file)
        .unwrap()
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .find_map(|fields| fields.get("cwd")?.as_str().map(PathBuf::from))
        .unwrap()
}
