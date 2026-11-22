use std::path::Path;

use harbour::keymap::Action;

use crate::{read_at, read_with, write_config};

#[test]
fn the_prefix_is_c_slash_by_default() {
    let home = tempfile::tempdir().unwrap();

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.keymap().prefix_name(), "C-/");
}

#[test]
fn the_file_sets_the_prefix() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "prefix = \"C-]\"\n");

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.keymap().prefix_name(), "C-]");
}

#[test]
fn a_letter_prefix_is_named_in_lower_case() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "prefix = \"C-A\"\n");

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.keymap().prefix_name(), "C-a");
}

#[test]
fn ctrl_space_is_a_prefix() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "prefix = \"C-Space\"\n");

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.keymap().prefix_name(), "C-Space");
}

#[test]
fn the_prefix_variable_outranks_the_file() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "prefix = \"C-]\"\n");

    let config = read_with(home.path(), &[("HARBOUR_PREFIX", "C-_")]);

    assert_eq!(config.keymap().prefix_name(), "C-_");
}

#[test]
fn harbours_own_ctrl_key_is_refused_as_the_prefix() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "prefix = \"C-h\"\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:1: prefix: \"C-h\" is taken: C-h, C-j, C-k and C-l are harbour's, C-m is Enter, C-i is Tab",
            path.display()
        )
    );
}

#[test]
fn a_prefix_harbour_does_not_take_is_refused_naming_what_it_takes() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "prefix = \"M-a\"\n");

    assert_eq!(
        refusal(home.path()),
        format!("{}:1: prefix: {PREFIX}", path.display())
    );
}

#[test]
fn a_prefix_that_is_not_a_string_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "prefix = 31\n");

    assert_eq!(
        refusal(home.path()),
        format!("{}:1: prefix: {PREFIX}", path.display())
    );
}

#[test]
fn a_bad_prefix_variable_is_refused_by_its_name() {
    let home = tempfile::tempdir().unwrap();
    let variable = |name: &str| match name {
        "HOME" => Some(home.path().as_os_str().to_owned()),
        "HARBOUR_PREFIX" => Some("C-Enter".into()),
        _ => None,
    };

    let refused = harbour::config::Config::read(variable).err().unwrap();

    assert_eq!(refused.to_string(), format!("HARBOUR_PREFIX: {PREFIX}"));
}

#[test]
fn a_binding_moves_an_action_to_another_key() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "[keys]\nzoom = \"f\"\n");

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.keymap().action_on('f'), Some(Action::Zoom));
    assert_eq!(config.keymap().action_on('z'), None);
}

#[test]
fn a_capital_letter_binds_both_cases() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "[keys]\nzoom = \"F\"\n");

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.keymap().action_on('f'), Some(Action::Zoom));
}

#[test]
fn none_unbinds_an_action() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "[keys]\nusage = \"none\"\n");

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.keymap().action_on('u'), None);
    assert_eq!(config.keymap().action_on('c'), Some(Action::Changelog));
}

#[test]
fn a_binding_on_another_actions_default_key_is_refused_naming_both() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "[keys]\nzoom = \"s\"\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:2: keys.zoom: \"s\" is find's key too; bind find elsewhere",
            path.display()
        )
    );
}

#[test]
fn moving_the_other_action_away_lets_the_key_be_taken() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "[keys]\nzoom = \"s\"\nfind = \"/\"\n");

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.keymap().action_on('s'), Some(Action::Zoom));
    assert_eq!(config.keymap().action_on('/'), Some(Action::Find));
}

#[test]
fn two_written_bindings_on_one_key_are_refused_on_the_later_line() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "[keys]\nzoom = \"f\"\nfind = \"F\"\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:3: keys.find: \"F\" is zoom's key too; bind zoom elsewhere",
            path.display()
        )
    );
}

#[test]
fn a_sidebar_key_on_j_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "[keys]\nnew = \"j\"\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:2: keys.new: \"j\" moves the selection in the sidebar",
            path.display()
        )
    );
}

#[test]
fn a_sidebar_key_on_k_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "[keys]\nend = \"K\"\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:2: keys.end: \"K\" moves the selection in the sidebar",
            path.display()
        )
    );
}

#[test]
fn an_action_outside_the_sidebar_may_take_j() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "[keys]\nzoom = \"j\"\n");

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.keymap().action_on('j'), Some(Action::Zoom));
}

#[test]
fn a_digit_is_refused_as_a_binding() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "[keys]\nzoom = \"3\"\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:2: keys.zoom: \"3\" is a digit, and the digits are the jumps",
            path.display()
        )
    );
}

#[test]
fn two_characters_are_refused_as_a_binding() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "[keys]\nzoom = \"zz\"\n");

    assert_eq!(
        refusal(home.path()),
        format!("{}:2: keys.zoom: {KEY}", path.display())
    );
}

#[test]
fn an_empty_binding_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "[keys]\nzoom = \"\"\n");

    assert_eq!(
        refusal(home.path()),
        format!("{}:2: keys.zoom: {KEY}", path.display())
    );
}

#[test]
fn a_space_is_refused_as_a_binding() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "[keys]\nzoom = \" \"\n");

    assert_eq!(
        refusal(home.path()),
        format!("{}:2: keys.zoom: {KEY}", path.display())
    );
}

#[test]
fn a_control_character_is_refused_as_a_binding() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "[keys]\nzoom = \"\\t\"\n");

    assert_eq!(
        refusal(home.path()),
        format!("{}:2: keys.zoom: {KEY}", path.display())
    );
}

#[test]
fn a_binding_that_is_not_a_string_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "[keys]\nzoom = 3\n");

    assert_eq!(
        refusal(home.path()),
        format!("{}:2: keys.zoom: {KEY}", path.display())
    );
}

#[test]
fn an_unknown_action_is_refused_naming_the_actions() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "\n[keys]\nzoomed = \"z\"\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:3: keys.zoomed: no such action; the actions are zoom, side, swap_left, swap_right, find, new, end, editor, sidebar, pane, narrower, wider, usage, changelog, help, quit",
            path.display()
        )
    );
}

#[test]
fn keys_that_are_not_a_table_are_refused() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "keys = \"z\"\n");

    assert_eq!(
        refusal(home.path()),
        format!(
            "{}:1: keys: expected a table of action = \"key\"",
            path.display()
        )
    );
}

const PREFIX: &str = "expected C-<letter>, C-Space, C-/, C-\\, C-], C-^ or C-_";
const KEY: &str = "expected one printable character other than a space, or \"none\"";

fn refusal(home: &Path) -> String {
    match read_at(home, None) {
        Ok(_) => "read without a refusal".to_string(),
        Err(refusal) => refusal.to_string(),
    }
}
