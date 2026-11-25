use std::path::Path;
use std::time::Duration;

use harbour::config::{Config, Source};

use crate::{read_at, read_with, write_config};

const HOUR: Duration = Duration::from_secs(60 * 60);
const TIDY: &str = "expected a whole number of hours above 0, as \"6h\", or \"off\"";

#[test]
fn an_ended_card_is_tidied_after_six_hours_by_default() {
    let home = tempfile::tempdir().unwrap();

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.tidy_after().value, Some(6 * HOUR));
    assert_eq!(config.tidy_after().source, Source::Default);
}

#[test]
fn the_file_sets_the_hours_before_an_ended_card_is_tidied() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "tidy_after = \"1h\"\n");

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.tidy_after().value, Some(HOUR));
    assert_eq!(config.tidy_after().source, Source::File);
}

#[test]
fn the_file_turns_tidying_off() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "tidy_after = \"off\"\n");

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.tidy_after().value, None);
    assert_eq!(config.tidy_after().source, Source::File);
}

#[test]
fn the_tidy_variable_outranks_the_file() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "tidy_after = \"1h\"\n");

    let config = read_with(home.path(), &[("HARBOUR_TIDY_AFTER", "12h")]);

    assert_eq!(config.tidy_after().value, Some(12 * HOUR));
    assert_eq!(
        config.tidy_after().source,
        Source::Variable("HARBOUR_TIDY_AFTER")
    );
}

#[test]
fn the_tidy_variable_turns_tidying_off() {
    let home = tempfile::tempdir().unwrap();

    let config = read_with(home.path(), &[("HARBOUR_TIDY_AFTER", "off")]);

    assert_eq!(config.tidy_after().value, None);
}

#[test]
fn an_empty_tidy_variable_keeps_the_file() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "tidy_after = \"1h\"\n");

    let config = read_with(home.path(), &[("HARBOUR_TIDY_AFTER", "")]);

    assert_eq!(config.tidy_after().value, Some(HOUR));
    assert_eq!(config.tidy_after().source, Source::File);
}

#[test]
fn zero_hours_is_refused_on_its_line() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "accent = \"red\"\ntidy_after = \"0h\"\n");

    assert_eq!(
        refused(home.path()),
        format!("{}:2: tidy_after: {TIDY}", path.display())
    );
}

#[test]
fn hours_without_their_unit_are_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "tidy_after = \"6\"\n");

    assert_eq!(
        refused(home.path()),
        format!("{}:1: tidy_after: {TIDY}", path.display())
    );
}

#[test]
fn hours_with_a_sign_are_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "tidy_after = \"+6h\"\n");

    assert_eq!(
        refused(home.path()),
        format!("{}:1: tidy_after: {TIDY}", path.display())
    );
}

#[test]
fn a_fraction_of_an_hour_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "tidy_after = \"1.5h\"\n");

    assert_eq!(
        refused(home.path()),
        format!("{}:1: tidy_after: {TIDY}", path.display())
    );
}

#[test]
fn a_capital_h_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "tidy_after = \"6H\"\n");

    assert_eq!(
        refused(home.path()),
        format!("{}:1: tidy_after: {TIDY}", path.display())
    );
}

#[test]
fn more_hours_than_it_counts_are_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "tidy_after = \"4294967296h\"\n");

    assert_eq!(
        refused(home.path()),
        format!("{}:1: tidy_after: {TIDY}", path.display())
    );
}

#[test]
fn a_number_of_hours_that_is_not_text_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "tidy_after = 6\n");

    assert_eq!(
        refused(home.path()),
        format!("{}:1: tidy_after: {TIDY}", path.display())
    );
}

#[test]
fn a_bad_tidy_variable_is_refused_by_its_name() {
    let home = tempfile::tempdir().unwrap();
    let variable = |name: &str| match name {
        "HOME" => Some(home.path().as_os_str().to_owned()),
        "HARBOUR_TIDY_AFTER" => Some("6 hours".into()),
        _ => None,
    };

    let refused = Config::read(variable).err().unwrap();

    assert_eq!(refused.to_string(), format!("HARBOUR_TIDY_AFTER: {TIDY}"));
}

fn refused(home: &Path) -> String {
    read_at(home, None).err().unwrap().to_string()
}
