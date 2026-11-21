use harbour::state::State;
use harbour::text::columns;
use harbour::zoom::{Headline, info};
use ratatui::text::Span;

#[test]
fn the_whole_headline_shows_when_it_fits() {
    let shown = text(&info(&fixing(), 60));

    assert_eq!(shown, " ● Fix the login  acme-api · main  3m ");
}

#[test]
fn the_dot_takes_the_state_style() {
    let spans = info(&fixing(), 60);

    assert_eq!(spans[1].content, "●");
    assert_eq!(spans[1].style, State::Working.style());
}

#[test]
fn a_narrow_room_cuts_the_place_first() {
    let shown = text(&info(&fixing(), 30));

    assert_eq!(shown, " ● Fix the login  acme-a…  3m ");
}

#[test]
fn a_narrower_room_drops_the_place_and_cuts_the_title_keeping_the_time() {
    let shown = text(&info(&fixing(), 20));

    assert_eq!(shown, " ● Fix the log…  3m ");
}

#[test]
fn the_time_stays_while_the_title_keeps_a_column() {
    let shown = text(&info(&fixing(), 9));

    assert_eq!(shown, " ● …  3m ");
}

#[test]
fn the_time_goes_once_the_title_would_lose_its_last_column() {
    let shown = text(&info(&fixing(), 8));

    assert_eq!(shown, " ● Fix… ");
}

#[test]
fn three_columns_show_the_dot_alone() {
    let shown = text(&info(&fixing(), 3));

    assert_eq!(shown, " ● ");
}

#[test]
fn under_three_columns_show_nothing() {
    let shown = text(&info(&fixing(), 2));

    assert_eq!(shown, "");
}

#[test]
fn without_a_time_the_title_and_place_take_the_room() {
    let headline = Headline {
        age: None,
        ..fixing()
    };

    let shown = text(&info(&headline, 20));

    assert_eq!(shown, " ● Fix the login ");
}

#[test]
fn a_cjk_title_and_branch_are_cut_by_display_width() {
    let headline = Headline {
        title: "修正ログイン".to_string(),
        place: "acme-api · 機能/ズーム".to_string(),
        ..fixing()
    };

    let shown = text(&info(&headline, 18));

    assert_eq!(shown, " ● 修正ログ…  3m ");
    assert_eq!(columns(&shown), 17);
}

#[test]
fn a_cjk_branch_is_cut_by_display_width() {
    let headline = Headline {
        title: "修正ログイン".to_string(),
        place: "acme-api · 機能/ズーム".to_string(),
        ..fixing()
    };

    let shown = text(&info(&headline, 40));

    assert_eq!(shown, " ● 修正ログイン  acme-api · 機能/…  3m ");
    assert_eq!(columns(&shown), 39);
}

#[test]
fn a_cjk_headline_that_fits_shows_whole() {
    let headline = Headline {
        title: "修正ログイン".to_string(),
        place: "acme-api · 機能/ズーム".to_string(),
        ..fixing()
    };

    let shown = text(&info(&headline, 60));

    assert_eq!(shown, " ● 修正ログイン  acme-api · 機能/ズーム  3m ");
}

fn fixing() -> Headline {
    Headline {
        state: State::Working,
        title: "Fix the login".to_string(),
        place: "acme-api · main".to_string(),
        age: Some("3m".to_string()),
    }
}

fn text(spans: &[Span]) -> String {
    spans.iter().map(|span| span.content.as_ref()).collect()
}
