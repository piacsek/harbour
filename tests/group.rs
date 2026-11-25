use harbour::group::{Member, member, order, step};

#[test]
fn no_cards_have_no_order() {
    let none: [&str; 0] = [];

    assert_eq!(order(&none), Vec::<usize>::new());
}

#[test]
fn cards_of_different_repositories_keep_their_order() {
    assert_eq!(order(&["api", "web", "docs"]), vec![0, 1, 2]);
}

#[test]
fn a_later_card_joins_the_first_card_of_its_repository() {
    assert_eq!(order(&["api", "web", "api"]), vec![0, 2, 1]);
}

#[test]
fn groups_keep_the_order_of_their_first_card() {
    assert_eq!(
        order(&["api", "web", "api", "web", "docs", "api"]),
        vec![0, 2, 5, 1, 3, 4]
    );
}

#[test]
fn a_lone_card_moves_down_past_a_lone_card() {
    assert_eq!(step(&["api", "web"], 0, true), Some(vec![1, 0]));
}

#[test]
fn a_lone_card_moves_down_past_a_whole_group() {
    assert_eq!(step(&["api", "web", "web"], 0, true), Some(vec![1, 2, 0]));
}

#[test]
fn a_lone_card_moves_up_past_a_whole_group() {
    assert_eq!(step(&["web", "web", "api"], 2, false), Some(vec![2, 0, 1]));
}

#[test]
fn a_groups_first_card_moves_the_group_down_past_a_lone_card() {
    assert_eq!(step(&["api", "api", "web"], 0, true), Some(vec![2, 0, 1]));
}

#[test]
fn a_groups_first_card_moves_the_group_down_past_another_group() {
    assert_eq!(
        step(&["api", "api", "web", "web"], 0, true),
        Some(vec![2, 3, 0, 1])
    );
}

#[test]
fn a_groups_first_card_moves_the_group_up_past_a_lone_card() {
    assert_eq!(step(&["web", "api", "api"], 1, false), Some(vec![1, 2, 0]));
}

#[test]
fn a_card_inside_a_group_moves_down_inside_it() {
    assert_eq!(step(&["api", "api", "api"], 1, true), Some(vec![0, 2, 1]));
}

#[test]
fn a_groups_second_card_moves_up_to_be_its_first() {
    assert_eq!(step(&["api", "api", "web"], 1, false), Some(vec![1, 0, 2]));
}

#[test]
fn a_groups_last_card_does_not_move_down_out_of_it() {
    assert_eq!(step(&["api", "api", "web"], 1, true), None);
}

#[test]
fn the_first_group_does_not_move_up() {
    assert_eq!(step(&["api", "api", "web"], 0, false), None);
}

#[test]
fn the_last_group_does_not_move_down() {
    assert_eq!(step(&["web", "api", "api"], 1, true), None);
}

#[test]
fn a_card_past_the_end_does_not_move() {
    assert_eq!(step(&["api", "web"], 2, false), None);
}

#[test]
fn a_card_alone_in_its_repository_is_in_no_group() {
    assert_eq!(member(&["api", "web", "web"], 0), Member::Alone);
}

#[test]
fn the_first_card_of_a_group_heads_it() {
    assert_eq!(member(&["api", "web", "web"], 1), Member::First);
}

#[test]
fn a_later_card_of_a_group_follows_its_head() {
    assert_eq!(member(&["api", "web", "web"], 2), Member::Later);
}
