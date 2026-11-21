use harbour::tabs::{claude, editor_named, iconed, strip};

#[test]
fn both_names_show_whole_when_they_fit() {
    let [claude, editor] = strip("nvim", None, 40);

    assert_eq!(claude, "claude");
    assert_eq!(editor, "nvim");
}

#[test]
fn the_directory_follows_the_editor_when_it_fits() {
    let [claude, editor] = strip("nvim", Some("~/work/acme/api"), 40);

    assert_eq!(claude, "claude");
    assert_eq!(editor, "nvim: ~/work/acme/api");
}

#[test]
fn the_directory_loses_its_start_first_at_a_slash() {
    let [claude, editor] = strip("nvim", Some("~/work/acme/api"), 28);

    assert_eq!(claude, "claude");
    assert_eq!(editor, "nvim: …/acme/api");
}

#[test]
fn a_wide_directory_name_is_cut_by_display_width() {
    let [claude, editor] = strip("nvim", Some("~/acme/日本語"), 22);

    assert_eq!(claude, "claude");
    assert_eq!(editor, "nvim: …本語");
}

#[test]
fn the_directory_goes_when_not_even_its_last_column_fits() {
    let [claude, editor] = strip("nvim", Some("~/work/acme/api"), 18);

    assert_eq!(claude, "claude");
    assert_eq!(editor, "nvim");
}

#[test]
fn the_editor_name_is_cut_before_claude() {
    let [claude, editor] = strip("nvim", None, 13);

    assert_eq!(claude, "claude");
    assert_eq!(editor, "n…");
}

#[test]
fn a_wide_editor_name_is_cut_by_display_width() {
    let [claude, editor] = strip("編集器", None, 14);

    assert_eq!(claude, "claude");
    assert_eq!(editor, "編…");
}

#[test]
fn nothing_shows_once_the_editor_name_is_gone() {
    let [claude, editor] = strip("nvim", None, 9);

    assert_eq!(claude, "");
    assert_eq!(editor, "");
}

#[test]
fn nothing_shows_in_the_narrowest_pane() {
    let [claude, editor] = strip("nvim", Some("~/work/acme/api"), 6);

    assert_eq!(claude, "");
    assert_eq!(editor, "");
}

#[test]
fn nothing_shows_when_not_one_wide_character_of_the_editor_name_fits() {
    let [claude, editor] = strip("編集器", None, 12);

    assert_eq!(claude, "");
    assert_eq!(editor, "");
}

#[test]
fn nothing_shows_where_not_even_the_separators_fit() {
    let [claude, editor] = strip("nvim", None, 4);

    assert_eq!(claude, "");
    assert_eq!(editor, "");
}

#[test]
fn with_icons_each_name_follows_its_glyph_and_a_space() {
    let [claude, editor] = iconed("nvim", true, None, 40);

    assert_eq!(claude, format!("{SPARKLE} claude"));
    assert_eq!(editor, format!("{NEOVIM} nvim"));
}

#[test]
fn with_icons_another_editor_gets_no_glyph() {
    let [claude, editor] = iconed("acme-edit", false, None, 40);

    assert_eq!(claude, format!("{SPARKLE} claude"));
    assert_eq!(editor, "acme-edit");
}

#[test]
fn with_icons_the_directory_follows_the_editor_when_it_fits() {
    let [claude, editor] = iconed("nvim", true, Some("~/work/acme/api"), 40);

    assert_eq!(claude, format!("{SPARKLE} claude"));
    assert_eq!(editor, format!("{NEOVIM} nvim: ~/work/acme/api"));
}

#[test]
fn with_icons_the_directory_loses_its_start_first_at_a_slash() {
    let [claude, editor] = iconed("nvim", true, Some("~/work/acme/api"), 31);

    assert_eq!(claude, format!("{SPARKLE} claude"));
    assert_eq!(editor, format!("{NEOVIM} nvim: …/acme/api"));
}

#[test]
fn with_icons_a_wide_directory_name_is_cut_by_display_width() {
    let [claude, editor] = iconed("nvim", true, Some("~/acme/日本語"), 26);

    assert_eq!(claude, format!("{SPARKLE} claude"));
    assert_eq!(editor, format!("{NEOVIM} nvim: …本語"));
}

#[test]
fn with_icons_the_directory_goes_when_not_even_its_last_column_fits() {
    let [claude, editor] = iconed("nvim", true, Some("~/work/acme/api"), 22);

    assert_eq!(claude, format!("{SPARKLE} claude"));
    assert_eq!(editor, format!("{NEOVIM} nvim"));
}

#[test]
fn with_icons_the_glyphs_stay_while_the_editor_name_fits_whole() {
    let [claude, editor] = iconed("nvim", true, None, 19);

    assert_eq!(claude, format!("{SPARKLE} claude"));
    assert_eq!(editor, format!("{NEOVIM} nvim"));
}

#[test]
fn with_icons_the_glyphs_go_before_the_editor_name_is_cut() {
    let [claude, editor] = iconed("nvim", true, None, 18);

    assert_eq!(claude, "claude");
    assert_eq!(editor, "nvim");
}

#[test]
fn with_icons_the_directory_does_not_come_back_once_the_glyphs_go() {
    let [claude, editor] = iconed("nvim", true, Some("~/work/acme/api"), 18);

    assert_eq!(claude, "claude");
    assert_eq!(editor, "nvim");
}

#[test]
fn with_icons_the_editor_name_is_cut_once_the_glyphs_are_gone() {
    let [claude, editor] = iconed("nvim", true, None, 13);

    assert_eq!(claude, "claude");
    assert_eq!(editor, "n…");
}

#[test]
fn with_icons_a_wide_editor_name_keeps_its_glyph_only_whole() {
    let [claude, editor] = iconed("編集器", true, None, 21);

    assert_eq!(claude, format!("{SPARKLE} claude"));
    assert_eq!(editor, format!("{NEOVIM} 編集器"));
}

#[test]
fn with_icons_a_wide_editor_name_too_wide_for_its_glyph_shows_without_one() {
    let [claude, editor] = iconed("編集器", true, None, 20);

    assert_eq!(claude, "claude");
    assert_eq!(editor, "編集器");
}

#[test]
fn with_icons_nothing_shows_in_the_narrowest_pane() {
    let [claude, editor] = iconed("nvim", true, Some("~/work/acme/api"), 6);

    assert_eq!(claude, "");
    assert_eq!(editor, "");
}

#[test]
fn the_claude_name_alone_is_plain_without_icons() {
    assert_eq!(claude(false), "claude");
}

#[test]
fn the_claude_name_alone_has_its_glyph_with_icons() {
    assert_eq!(claude(true), format!("{SPARKLE} claude"));
}

#[test]
fn the_editor_name_alone_keeps_its_directory_when_it_fits() {
    assert_eq!(
        editor_named("nvim", false, Some("~/work/acme/api"), 30),
        "nvim: ~/work/acme/api"
    );
}

#[test]
fn the_editor_name_alone_cuts_its_directory_from_the_start() {
    assert_eq!(
        editor_named("nvim", false, Some("~/work/acme/api"), 16),
        "nvim: …/acme/api"
    );
}

#[test]
fn the_editor_name_alone_cuts_a_cjk_directory_by_display_width() {
    assert_eq!(
        editor_named("nvim", false, Some("~/コード/acme-api"), 16),
        "nvim: …/acme-api"
    );
}

#[test]
fn the_editor_name_alone_has_the_neovim_glyph_with_icons() {
    assert_eq!(
        editor_named("nvim", true, None, 10),
        format!("{NEOVIM} nvim")
    );
}

#[test]
fn the_editor_name_alone_drops_its_glyph_before_it_is_cut() {
    assert_eq!(editor_named("nvim", true, None, 5), "nvim");
}

#[test]
fn the_editor_name_alone_is_cut_when_it_does_not_fit() {
    assert_eq!(editor_named("acme-edit", false, None, 5), "acme…");
}

#[test]
fn the_editor_name_alone_shows_nothing_under_two_columns() {
    assert_eq!(editor_named("acme-edit", false, None, 1), "");
}

const SPARKLE: &str = "\u{F0674}";
const NEOVIM: &str = "\u{E6AE}";
