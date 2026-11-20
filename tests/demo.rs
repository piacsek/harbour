use std::panic;

use harbour::demo::Registry;

#[test]
fn the_demo_registry_is_created_and_removed_when_dropped() {
    let scratch = tempfile::tempdir().unwrap();
    let path = scratch.path().join("harbour-demo");

    let registry = Registry::create(path.clone()).unwrap();
    std::fs::write(registry.path().join("1.json"), "{}").unwrap();
    assert!(path.is_dir());
    drop(registry);

    assert!(!path.exists());
}

#[test]
fn the_demo_registry_is_removed_when_a_panic_unwinds() {
    let scratch = tempfile::tempdir().unwrap();
    let path = scratch.path().join("harbour-demo");

    let unwound = panic::catch_unwind(|| {
        let _registry = Registry::create(path.clone()).unwrap();
        panic!("acme demo failed");
    });

    assert!(unwound.is_err());
    assert!(!path.exists());
}
