use harbour::rename::line;

#[test]
fn a_title_that_fits_shows_whole_after_the_label() {
    assert_eq!(
        line("acme login fix", 30),
        ("Title: ", "acme login fix".to_string())
    );
}

#[test]
fn a_title_one_column_short_of_the_room_shows_whole() {
    assert_eq!(
        line("acme login", 18),
        ("Title: ", "acme login".to_string())
    );
}

#[test]
fn a_title_filling_the_room_keeps_its_end_and_a_column_for_the_cursor() {
    assert_eq!(
        line("acme login f", 18),
        ("Title: ", "…e login f".to_string())
    );
}

#[test]
fn a_long_title_keeps_its_end() {
    assert_eq!(
        line("Fix the flaky login test in acme-api", 24),
        ("Title: ", "…est in acme-api".to_string())
    );
}

#[test]
fn under_seventeen_columns_the_label_goes() {
    assert_eq!(
        line("acme login fix", 16),
        ("", "acme login fix".to_string())
    );
}

#[test]
fn at_seventeen_columns_the_label_stays() {
    assert_eq!(line("acme", 17), ("Title: ", "acme".to_string()));
}

#[test]
fn at_the_narrowest_sidebar_the_text_has_the_whole_line() {
    assert_eq!(line("acme login fix", 13), ("", "…e login fix".to_string()));
}

#[test]
fn a_cjk_title_is_cut_at_a_whole_character() {
    assert_eq!(
        line("東京の会議の準備", 14),
        ("", "…の会議の準備".to_string())
    );
}

#[test]
fn a_cjk_title_leaves_a_column_rather_than_half_a_character() {
    assert_eq!(
        line("東京の会議の準備", 13),
        ("", "…会議の準備".to_string())
    );
}

#[test]
fn an_empty_title_shows_the_label_alone() {
    assert_eq!(line("", 30), ("Title: ", String::new()));
}

#[test]
fn one_column_holds_only_the_cursor() {
    assert_eq!(line("acme", 1), ("", String::new()));
}

#[test]
fn no_column_holds_nothing() {
    assert_eq!(line("acme", 0), ("", String::new()));
}
