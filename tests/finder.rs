use harbour::finder::{chosen, lines};
use harbour::state::State;
use harbour::zoom::Headline;

#[test]
fn a_line_holds_the_hidden_number_the_dot_the_title_the_place_and_the_age() {
    let listed = listed(&[headline("acme web", "acme-web · main", Some("3m"))]);

    assert_eq!(listed, "0\t○\tacme web \tacme-web · main \t3m\n");
}

#[test]
fn the_places_and_ages_line_up_down_the_list() {
    let listed = listed(&[
        headline("acme web", "acme-web · main", Some("3m")),
        headline("acme api fix", "acme-api", Some("<1m")),
    ]);

    let rows: Vec<&str> = listed.lines().collect();
    assert_eq!(rows[0], "0\t○\tacme web     \tacme-web · main \t 3m");
    assert_eq!(rows[1], "1\t○\tacme api fix \tacme-api        \t<1m");
}

#[test]
fn a_title_with_a_tab_a_newline_and_an_escape_splits_no_field_and_no_line() {
    let title = "acme\tweb\nfix \u{1b}[31mred\u{1b}[0m";

    let listed = listed(&[headline(title, "acme-web", None)]);

    assert_eq!(listed.matches('\t').count(), 4, "{listed:?}");
    assert_eq!(listed.matches('\n').count(), 1, "{listed:?}");
    assert!(!listed.contains('\u{1b}'), "{listed:?}");
}

#[test]
fn a_long_title_is_kept_whole_for_fzf_to_search() {
    let title = "acme fix the flaky login test on the checkout page";

    let listed = listed(&[headline(title, "acme-web", Some("2h"))]);

    let field = listed.split('\t').nth(2).unwrap();
    assert_eq!(field, format!("{title} "), "{listed:?}");
}

#[test]
fn a_long_place_is_kept_whole_for_fzf_to_search() {
    let place = "acme-checkout-service-with-a-long-name · fix-the-flaky-login-test";

    let listed = listed(&[headline("acme web", place, None)]);

    let field = listed.split('\t').nth(3).unwrap();
    assert_eq!(field, format!("{place} "), "{listed:?}");
}

#[test]
fn a_cjk_title_is_kept_whole_and_the_place_still_lines_up() {
    let listed = listed(&[
        headline("港口の修正をする長い名前", "acme-web", None),
        headline("acme web", "acme-api", None),
    ]);

    let rows: Vec<&str> = listed.lines().collect();
    assert_eq!(rows[0], "0\t○\t港口の修正をする長い名前 \tacme-web \t");
    assert_eq!(rows[1], "1\t○\tacme web                 \tacme-api \t");
}

#[test]
fn a_hundred_cards_give_a_hundred_lines_numbered_from_zero() {
    let headlines: Vec<Headline> = (0..100)
        .map(|n| headline(&format!("acme {n}"), "acme-web", None))
        .collect();

    let listed = listed(&headlines);

    assert_eq!(listed.lines().count(), 100);
    assert!(
        listed.lines().last().unwrap().starts_with("99\t"),
        "{listed:?}"
    );
}

#[test]
fn the_number_before_the_first_tab_is_the_chosen_card() {
    assert_eq!(
        chosen(b"12\t\xe2\x97\x8b\tacme web \tacme-web \t3m"),
        Some(12)
    );
}

#[test]
fn a_line_without_a_number_chooses_nothing() {
    assert_eq!(chosen(b"acme web"), None);
}

fn listed(headlines: &[Headline]) -> String {
    String::from_utf8(lines(headlines)).unwrap()
}

fn headline(title: &str, place: &str, age: Option<&str>) -> Headline {
    Headline {
        state: State::Idle,
        title: title.to_string(),
        place: place.to_string(),
        age: age.map(String::from),
    }
}
