use std::panic;

use harbour::demo::Registry;

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
