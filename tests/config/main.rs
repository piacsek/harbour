use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use harbour::config::{Config, Refusal, Source};
use ratatui::style::Color;

mod command;
mod precedence;
mod refusals;

#[test]
fn the_file_is_under_the_config_folder_of_home() {
    let home = tempfile::tempdir().unwrap();

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(
        config.path(),
        Some(home.path().join(".config/harbour/config.toml").as_path())
    );
}

#[test]
fn an_absolute_xdg_config_home_holds_the_file() {
    let home = tempfile::tempdir().unwrap();
    let xdg = home.path().join("xdg");

    let config = read_at(home.path(), Some(xdg.to_str().unwrap())).unwrap();

    assert_eq!(
        config.path(),
        Some(xdg.join("harbour/config.toml").as_path())
    );
}

#[test]
fn an_empty_xdg_config_home_counts_as_unset() {
    let home = tempfile::tempdir().unwrap();

    let config = read_at(home.path(), Some("")).unwrap();

    assert_eq!(
        config.path(),
        Some(home.path().join(".config/harbour/config.toml").as_path())
    );
}

#[test]
fn a_relative_xdg_config_home_counts_as_unset() {
    let home = tempfile::tempdir().unwrap();

    let config = read_at(home.path(), Some("acme/config")).unwrap();

    assert_eq!(
        config.path(),
        Some(home.path().join(".config/harbour/config.toml").as_path())
    );
}

#[test]
fn without_a_home_there_is_no_file_and_every_default_holds() {
    let config = Config::read(|_| None).unwrap();

    assert_eq!(config.path(), None);
    assert_eq!(config.editor().value, vec![OsString::from("nvim")]);
}

#[test]
fn a_relative_home_gives_no_file() {
    let config = Config::read(|name| (name == "HOME").then(|| OsString::from("acme"))).unwrap();

    assert_eq!(config.path(), None);
}

#[test]
fn without_a_file_every_setting_is_its_default() {
    let home = tempfile::tempdir().unwrap();

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.editor().value, vec![OsString::from("nvim")]);
    assert_eq!(config.editor().source, Source::Default);
    assert_eq!(config.picker().value, words(&["fzf"]));
    assert_eq!(config.picker().source, Source::Default);
    assert_eq!(config.session_command().value, words(&["claude"]));
    assert_eq!(config.session_command().source, Source::Default);
    assert_eq!(config.sidebar_width().value, None);
    assert_eq!(config.sidebar_width().source, Source::Default);
    assert_eq!(config.accent().value, Color::Cyan);
    assert_eq!(config.accent().source, Source::Default);
}

#[test]
fn an_empty_file_keeps_every_default() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "");

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.editor().source, Source::Default);
    assert_eq!(config.picker().source, Source::Default);
    assert_eq!(config.session_command().source, Source::Default);
    assert_eq!(config.sidebar_width().source, Source::Default);
    assert_eq!(config.accent().source, Source::Default);
}

#[test]
fn the_file_sets_each_program_with_its_arguments() {
    let home = tempfile::tempdir().unwrap();
    write_config(
        home.path(),
        "editor = [\"nvim\", \"--clean\"]\npicker = [\"/opt/acme/pick\", \"--dirs\"]\nsession_command = [\"claude\", \"--model\", \"acme\"]\n",
    );

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(
        config.editor().value,
        vec![OsString::from("nvim"), OsString::from("--clean")]
    );
    assert_eq!(config.editor().source, Source::File);
    assert_eq!(config.picker().value, words(&["/opt/acme/pick", "--dirs"]));
    assert_eq!(config.picker().source, Source::File);
    assert_eq!(
        config.session_command().value,
        words(&["claude", "--model", "acme"])
    );
    assert_eq!(config.session_command().source, Source::File);
}

#[test]
fn the_file_sets_the_sidebar_width() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "sidebar_width = 40\n");

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.sidebar_width().value, Some(40));
    assert_eq!(config.sidebar_width().source, Source::File);
}

#[test]
fn a_width_beyond_what_a_terminal_holds_counts_as_the_largest() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "sidebar_width = 100000\n");

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.sidebar_width().value, Some(u16::MAX));
}

#[test]
fn the_file_sets_the_accent_by_name() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "accent = \"magenta\"\n");

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.accent().value, Color::Magenta);
    assert_eq!(config.accent().source, Source::File);
}

#[test]
fn the_accent_takes_a_light_name() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "accent = \"lightred\"\n");

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.accent().value, Color::LightRed);
}

#[test]
fn the_accent_takes_a_hex_colour() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "accent = \"#d7875f\"\n");

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.accent().value, Color::Rgb(0xd7, 0x87, 0x5f));
}

#[test]
fn a_program_under_home_is_expanded() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "editor = [\"~/bin/acme-vim\", \"~/notes\"]\n");

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(
        config.editor().value,
        vec![
            home.path().join("bin/acme-vim").into_os_string(),
            OsString::from("~/notes")
        ]
    );
}

#[test]
fn a_bare_program_name_is_kept_for_the_path_search() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "session_command = [\"acme-agent\"]\n");

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.session_command().value, words(&["acme-agent"]));
}

#[test]
fn a_symlink_to_the_file_is_read_through() {
    let home = tempfile::tempdir().unwrap();
    let elsewhere = home.path().join("dotfiles.toml");
    fs::write(&elsewhere, "sidebar_width = 30\n").unwrap();
    let path = config_path(home.path());
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&elsewhere, &path).unwrap();

    let config = read_at(home.path(), None).unwrap();

    assert_eq!(config.sidebar_width().value, Some(30));
}

fn read_at(home: &Path, xdg: Option<&str>) -> Result<Config, Refusal> {
    let home = home.as_os_str().to_owned();
    let xdg = xdg.map(OsString::from);
    Config::read(move |name| match name {
        "HOME" => Some(home.clone()),
        "XDG_CONFIG_HOME" => xdg.clone(),
        _ => None,
    })
}

fn read_with(home: &Path, variables: &[(&str, &str)]) -> Config {
    let home = home.as_os_str().to_owned();
    let variables: Vec<(String, OsString)> = variables
        .iter()
        .map(|(name, value)| (name.to_string(), OsString::from(value)))
        .collect();
    Config::read(move |name| match name {
        "HOME" => Some(home.clone()),
        _ => variables
            .iter()
            .find(|(variable, _)| variable == name)
            .map(|(_, value)| value.clone()),
    })
    .unwrap()
}

fn config_path(home: &Path) -> PathBuf {
    home.join(".config/harbour/config.toml")
}

fn write_config(home: &Path, text: &str) -> PathBuf {
    let path = config_path(home);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, text).unwrap();
    path
}

fn words(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| item.to_string()).collect()
}
