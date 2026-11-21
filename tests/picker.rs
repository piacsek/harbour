use std::io;

use harbour::picker::{is_listing, listing};

#[test]
fn a_failed_list_names_the_list_and_the_error() {
    let error = listing(io::Error::other("acme disk full"));

    assert_eq!(
        error.to_string(),
        "Could not list the sessions: acme disk full"
    );
}

#[test]
fn a_failed_list_is_told_apart_from_a_failed_start() {
    let error = listing(io::Error::other("acme disk full"));

    assert!(is_listing(&error));
}

#[test]
fn a_failed_start_is_not_a_failed_list() {
    let error = io::Error::from(io::ErrorKind::NotFound);

    assert!(!is_listing(&error));
}
