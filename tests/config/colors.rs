use std::path::Path;

use ratatui::style::Color;

use crate::{read_at, read_with, write_config};

#[test]
fn the_fact_colours_default_to_blue_magenta_and_cyan() {
    let home = tempfile::tempdir().unwrap();

    let colors = read_at(home.path(), None).unwrap().colors();

    assert_eq!(colors.repository, Color::Blue);
    assert_eq!(colors.branch, Color::Magenta);
    assert_eq!(colors.pull_request, Color::Cyan);
}

#[test]
fn the_file_sets_each_fact_colour() {
    let home = tempfile::tempdir().unwrap();
    write_config(
        home.path(),
        "[colors]\nrepository = \"green\"\nbranch = \"#d7875f\"\npull_request = \"LightRed\"\n",
    );

    let colors = read_at(home.path(), None).unwrap().colors();

    assert_eq!(colors.repository, Color::Green);
    assert_eq!(colors.branch, Color::Rgb(0xd7, 0x87, 0x5f));
    assert_eq!(colors.pull_request, Color::LightRed);
}

#[test]
fn a_fact_colour_the_file_leaves_out_keeps_its_default() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "[colors]\nbranch = \"red\"\n");

    let colors = read_at(home.path(), None).unwrap().colors();

    assert_eq!(colors.repository, Color::Blue);
    assert_eq!(colors.branch, Color::Red);
}

#[test]
fn a_variable_never_sets_a_fact_colour() {
    let home = tempfile::tempdir().unwrap();

    let colors = read_with(home.path(), &[("HARBOUR_COLORS", "red")]).colors();

    assert_eq!(colors.repository, Color::Blue);
}

#[test]
fn an_unknown_fact_is_refused_on_its_line() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(
        home.path(),
        "[colors]\nbranch = \"red\"\nsubagents = \"red\"\n",
    );

    assert_eq!(
        refused(home.path()),
        format!("{}:3: colors.subagents: unknown key", path.display())
    );
}

#[test]
fn a_colour_that_is_not_one_is_refused_on_its_line() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "[colors]\nbranch = \"pink\"\n");

    assert_eq!(
        refused(home.path()),
        format!(
            "{}:2: colors.branch: expected one of the 16 colour names or #rrggbb",
            path.display()
        )
    );
}

#[test]
fn a_short_hex_colour_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "[colors]\nrepository = \"#12345\"\n");

    assert_eq!(
        refused(home.path()),
        format!(
            "{}:2: colors.repository: expected one of the 16 colour names or #rrggbb",
            path.display()
        )
    );
}

#[test]
fn a_colour_that_is_not_text_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "[colors]\npull_request = 3\n");

    assert_eq!(
        refused(home.path()),
        format!(
            "{}:2: colors.pull_request: expected one of the 16 colour names or #rrggbb",
            path.display()
        )
    );
}

#[test]
fn colors_that_is_not_a_table_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "accent = \"red\"\ncolors = 3\n");

    assert_eq!(
        refused(home.path()),
        format!(
            "{}:2: colors: expected a table of fact = \"colour\"",
            path.display()
        )
    );
}

#[test]
fn a_fact_colour_from_the_file_says_so() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "[colors]\nbranch = \"red\"\n");

    let config = read_at(home.path(), None).unwrap();

    assert!(
        config
            .report()
            .contains("\ncolors.branch = \"red\"  (file)\n")
    );
    assert!(
        config
            .report()
            .contains("\ncolors.repository = \"blue\"  (default)\n")
    );
}

fn refused(home: &Path) -> String {
    match read_at(home, None) {
        Ok(_) => "read without a refusal".to_string(),
        Err(refusal) => refusal.to_string(),
    }
}
