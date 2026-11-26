use harbour::facts::Colors;
use harbour::icons::{BRANCH, REPOSITORY};
use harbour::state::State;
use harbour::text::columns;
use harbour::zoom::{Headline, info};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

#[test]
fn the_whole_headline_shows_when_it_fits() {
    let shown = text(&plain(&fixing(), 60));

    assert_eq!(shown, " ● Fix the login  acme-api · main  3m ");
}

#[test]
fn the_dot_takes_the_state_style() {
    let spans = plain(&fixing(), 60);

    assert_eq!(spans[1].content, "●");
    assert_eq!(spans[1].style, State::Working.style());
}

#[test]
fn a_narrow_room_cuts_the_place_first() {
    let shown = text(&plain(&fixing(), 30));

    assert_eq!(shown, " ● Fix the login  acme-a…  3m ");
}

#[test]
fn a_narrower_room_drops_the_place_and_cuts_the_title_keeping_the_time() {
    let shown = text(&plain(&fixing(), 20));

    assert_eq!(shown, " ● Fix the log…  3m ");
}

#[test]
fn the_time_stays_while_the_title_keeps_a_column() {
    let shown = text(&plain(&fixing(), 9));

    assert_eq!(shown, " ● …  3m ");
}

#[test]
fn the_time_goes_once_the_title_would_lose_its_last_column() {
    let shown = text(&plain(&fixing(), 8));

    assert_eq!(shown, " ● Fix… ");
}

#[test]
fn three_columns_show_the_dot_alone() {
    let shown = text(&plain(&fixing(), 3));

    assert_eq!(shown, " ● ");
}

#[test]
fn under_three_columns_show_nothing() {
    let shown = text(&plain(&fixing(), 2));

    assert_eq!(shown, "");
}

#[test]
fn without_a_time_the_title_and_place_take_the_room() {
    let headline = Headline {
        age: None,
        ..fixing()
    };

    let shown = text(&plain(&headline, 20));

    assert_eq!(shown, " ● Fix the login ");
}

#[test]
fn a_cjk_title_and_branch_are_cut_by_display_width() {
    let headline = Headline {
        title: "修正ログイン".to_string(),
        branch: Some("機能/ズーム".to_string()),
        ..fixing()
    };

    let shown = text(&plain(&headline, 18));

    assert_eq!(shown, " ● 修正ログ…  3m ");
    assert_eq!(columns(&shown), 17);
}

#[test]
fn a_cjk_branch_is_cut_by_display_width() {
    let headline = Headline {
        title: "修正ログイン".to_string(),
        branch: Some("機能/ズーム".to_string()),
        ..fixing()
    };

    let shown = text(&plain(&headline, 40));

    assert_eq!(shown, " ● 修正ログイン  acme-api · 機能/…  3m ");
    assert_eq!(columns(&shown), 39);
}

#[test]
fn a_cjk_headline_that_fits_shows_whole() {
    let headline = Headline {
        title: "修正ログイン".to_string(),
        branch: Some("機能/ズーム".to_string()),
        ..fixing()
    };

    let shown = text(&plain(&headline, 60));

    assert_eq!(shown, " ● 修正ログイン  acme-api · 機能/ズーム  3m ");
}

#[test]
fn without_a_title_the_place_shows_alone() {
    let headline = Headline {
        title: String::new(),
        ..fixing()
    };

    let shown = text(&plain(&headline, 60));

    assert_eq!(shown, " ● acme-api · main  3m ");
}

#[test]
fn the_repository_takes_its_colour() {
    let spans = plain(&fixing(), 60);

    assert_eq!(
        style_of(&spans, "acme-api"),
        Style::default().fg(Color::Blue)
    );
}

#[test]
fn the_branch_takes_its_colour() {
    let spans = plain(&fixing(), 60);

    assert_eq!(
        style_of(&spans, "main"),
        Style::default().fg(Color::Magenta)
    );
}

#[test]
fn the_dot_between_repository_and_branch_is_dimmed() {
    let spans = plain(&fixing(), 60);

    assert_eq!(
        style_of(&spans, " · "),
        Style::default().add_modifier(Modifier::DIM)
    );
}

#[test]
fn the_time_is_dimmed() {
    let spans = plain(&fixing(), 60);

    assert_eq!(
        style_of(&spans, "3m"),
        Style::default().add_modifier(Modifier::DIM)
    );
}

#[test]
fn the_title_has_no_colour_of_its_own() {
    let spans = plain(&fixing(), 60);

    let title = spans
        .iter()
        .find(|span| span.content.contains("Fix the login"))
        .unwrap();
    assert_eq!(title.style, Style::default());
}

#[test]
fn the_configured_colours_paint_the_repository_and_branch() {
    let colors = Colors {
        repository: Color::Green,
        branch: Color::Yellow,
        ..Colors::default()
    };

    let spans = info(&fixing(), 60, &colors, false);

    assert_eq!(
        style_of(&spans, "acme-api"),
        Style::default().fg(Color::Green)
    );
    assert_eq!(style_of(&spans, "main"), Style::default().fg(Color::Yellow));
}

#[test]
fn a_cut_repository_ends_in_its_colour() {
    let spans = plain(&fixing(), 30);

    assert_eq!(style_of(&spans, "…"), Style::default().fg(Color::Blue));
}

#[test]
fn a_cut_branch_ends_in_its_colour() {
    let spans = plain(&fixing(), 36);

    assert_eq!(text(&spans), " ● Fix the login  acme-api · m…  3m ");
    assert_eq!(style_of(&spans, "…"), Style::default().fg(Color::Magenta));
}

#[test]
fn icons_take_the_place_of_the_dot() {
    let shown = text(&iconed(&fixing(), 60));

    assert_eq!(
        shown,
        format!(" ● Fix the login  {REPOSITORY} acme-api {BRANCH} main  3m ")
    );
}

#[test]
fn the_repository_glyph_takes_the_repository_colour() {
    let spans = iconed(&fixing(), 60);

    let repository = format!("{REPOSITORY} acme-api");
    assert_eq!(
        style_of(&spans, &repository),
        Style::default().fg(Color::Blue)
    );
}

#[test]
fn the_branch_glyph_takes_the_branch_colour() {
    let spans = iconed(&fixing(), 60);

    let branch = format!("{BRANCH} main");
    assert_eq!(
        style_of(&spans, &branch),
        Style::default().fg(Color::Magenta)
    );
}

#[test]
fn a_cut_branch_with_icons_ends_in_its_colour() {
    let spans = iconed(&fixing(), 38);

    assert_eq!(
        text(&spans),
        format!(" ● Fix the login  {REPOSITORY} acme-api {BRANCH} m…  3m ")
    );
    assert_eq!(style_of(&spans, "…"), Style::default().fg(Color::Magenta));
}

#[test]
fn a_branch_glyph_without_a_column_of_its_branch_goes_with_it() {
    let shown = text(&iconed(&fixing(), 37));

    assert_eq!(
        shown,
        format!(" ● Fix the login  {REPOSITORY} acme-api …  3m ")
    );
}

#[test]
fn without_a_branch_the_repository_shows_alone_in_its_colour() {
    let headline = Headline {
        branch: None,
        ..fixing()
    };

    let spans = plain(&headline, 60);

    assert_eq!(text(&spans), " ● Fix the login  acme-api  3m ");
    assert_eq!(
        style_of(&spans, "acme-api"),
        Style::default().fg(Color::Blue)
    );
}

#[test]
fn without_a_title_the_place_shows_alone_with_icons() {
    let headline = Headline {
        title: String::new(),
        ..fixing()
    };

    let shown = text(&iconed(&headline, 60));

    assert_eq!(
        shown,
        format!(" ● {REPOSITORY} acme-api {BRANCH} main  3m ")
    );
}

#[test]
fn a_cjk_repository_and_branch_take_their_colours_with_icons() {
    let headline = Headline {
        title: "修正ログイン".to_string(),
        repository: "港口".to_string(),
        branch: Some("機能".to_string()),
        ..fixing()
    };

    let spans = iconed(&headline, 60);

    assert_eq!(
        text(&spans),
        format!(" ● 修正ログイン  {REPOSITORY} 港口 {BRANCH} 機能  3m ")
    );
    let repository = format!("{REPOSITORY} 港口");
    assert_eq!(
        style_of(&spans, &repository),
        Style::default().fg(Color::Blue)
    );
}

#[test]
fn a_cut_cjk_branch_ends_in_its_colour() {
    let headline = Headline {
        title: "修正ログイン".to_string(),
        branch: Some("機能/ズーム".to_string()),
        ..fixing()
    };

    let spans = plain(&headline, 40);

    assert_eq!(style_of(&spans, "…"), Style::default().fg(Color::Magenta));
}

fn fixing() -> Headline {
    Headline {
        state: State::Working,
        title: "Fix the login".to_string(),
        repository: "acme-api".to_string(),
        branch: Some("main".to_string()),
        age: Some("3m".to_string()),
    }
}

fn text(spans: &[Span]) -> String {
    spans.iter().map(|span| span.content.as_ref()).collect()
}

fn plain(headline: &Headline, room: usize) -> Vec<Span<'static>> {
    info(headline, room, &Colors::default(), false)
}

fn iconed(headline: &Headline, room: usize) -> Vec<Span<'static>> {
    info(headline, room, &Colors::default(), true)
}

fn style_of(spans: &[Span], content: &str) -> Style {
    spans
        .iter()
        .find(|span| span.content == content)
        .unwrap_or_else(|| panic!("no span {content:?} in {spans:?}"))
        .style
}
