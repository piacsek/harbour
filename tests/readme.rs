use harbour::help::KEYS;

const README: &str = include_str!("../README.md");

#[test]
fn the_readme_opens_with_the_name_and_the_one_line_description() {
    assert!(README.starts_with("# harbour\n\nHarbour: where shipping begins.\n\n"));
}

#[test]
fn the_readme_shows_the_details_screenshot_right_below_the_description() {
    assert!(README.starts_with(
        "# harbour\n\nHarbour: where shipping begins.\n\n![harbour: a sidebar of sessions beside a live pane](docs/details.png)\n\n## "
    ));
}

#[test]
fn the_readme_embeds_one_screenshot() {
    assert_eq!(README.matches("![").count(), 1);
}

#[test]
fn the_readme_has_exactly_the_three_user_sections_in_order() {
    let sections = sections();

    assert_eq!(sections.len(), 3, "{sections:?}");
    assert_eq!(sections[0], "## Installation");
    assert_eq!(sections[1], "## Usage");
    assert_eq!(sections[2], "## Development");
}

#[test]
fn the_readme_has_no_subsections() {
    assert!(!prose().iter().any(|line| line.starts_with("### ")));
}

#[test]
fn the_readme_names_one_h1() {
    assert_eq!(
        prose().iter().filter(|line| line.starts_with("# ")).count(),
        1
    );
}

#[test]
fn the_readme_installs_from_the_tap() {
    assert!(README.contains("\nbrew install piacsek/tap/harbour\n"));
}

#[test]
fn the_readme_nix_install_works_without_the_experimental_features_enabled() {
    assert!(README.contains(
        "\nnix --extra-experimental-features 'nix-command flakes' profile install github:piacsek/harbour/stable\n"
    ));
}

#[test]
fn the_readme_usage_shows_the_version_flag() {
    assert!(README.contains("\nharbour --version"));
}

#[test]
fn the_readme_usage_shows_the_help_flag() {
    assert!(README.contains("\nharbour --help"));
}

#[test]
fn the_readme_usage_shows_the_changelog_subcommand() {
    assert!(README.contains("\nharbour changelog"));
}

#[test]
fn the_readme_usage_shows_the_status_subcommand() {
    assert!(README.contains("\nharbour status"));
}

#[test]
fn the_readme_usage_shows_the_edit_subcommand() {
    assert!(README.contains("\nharbour edit <file>"));
}

#[test]
fn the_readme_usage_shows_the_log_subcommand() {
    assert!(README.contains("\nharbour log "));
}

#[test]
fn the_readme_builds_from_source_at_the_newest_release() {
    assert!(README.contains(
        "\ncargo install --git https://github.com/piacsek/harbour --branch stable --locked\n"
    ));
}

#[test]
fn the_readme_key_table_mirrors_the_in_app_help() {
    assert_eq!(readme_key_rows(), help_key_rows());
}

fn sections() -> Vec<&'static str> {
    prose()
        .into_iter()
        .filter(|line| line.starts_with("## "))
        .collect()
}

fn prose() -> Vec<&'static str> {
    let mut fenced = false;
    README
        .lines()
        .filter(|line| {
            if line.starts_with("```") {
                fenced = !fenced;
                return false;
            }
            !fenced
        })
        .collect()
}

fn readme_key_rows() -> String {
    README
        .lines()
        .filter(|line| line.starts_with("| `"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn help_key_rows() -> String {
    KEYS.iter()
        .map(|(key, what)| format!("| `{key}` | {what} |"))
        .collect::<Vec<_>>()
        .join("\n")
}
