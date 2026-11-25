use std::time::Duration;

use harbour::tidy::{self, Card};

const AFTER: Duration = Duration::from_secs(6 * 60 * 60);
const SECOND: Duration = Duration::from_secs(1);

#[test]
fn an_ended_card_as_old_as_the_setting_is_due() {
    let cards = [ended(SECOND), ended(AFTER)];

    assert_eq!(tidy::due(&cards, 0, AFTER), vec![1]);
}

#[test]
fn an_ended_card_younger_than_the_setting_is_kept() {
    let cards = [ended(SECOND), ended(AFTER - SECOND)];

    assert_eq!(tidy::due(&cards, 0, AFTER), Vec::<usize>::new());
}

#[test]
fn a_running_card_is_never_due() {
    let cards = [ended(SECOND), running(AFTER * 2)];

    assert_eq!(tidy::due(&cards, 0, AFTER), Vec::<usize>::new());
}

#[test]
fn the_selected_card_is_never_due() {
    let cards = [ended(AFTER * 2), ended(SECOND)];

    assert_eq!(tidy::due(&cards, 0, AFTER), Vec::<usize>::new());
}

#[test]
fn a_card_of_no_known_age_is_never_due() {
    let cards = [
        ended(SECOND),
        Card {
            age: None,
            running: false,
        },
    ];

    assert_eq!(tidy::due(&cards, 0, AFTER), Vec::<usize>::new());
}

#[test]
fn every_card_due_is_named_in_order() {
    let cards = [
        ended(AFTER),
        ended(SECOND),
        ended(AFTER * 3),
        running(AFTER),
        ended(AFTER + SECOND),
    ];

    let due = tidy::due(&cards, 1, AFTER);

    assert_eq!(due.len(), 3);
    assert_eq!(due[0], 0);
    assert_eq!(due[1], 2);
    assert_eq!(due[2], 4);
}

fn ended(age: Duration) -> Card {
    Card {
        age: Some(age),
        running: false,
    }
}

fn running(age: Duration) -> Card {
    Card {
        age: Some(age),
        running: true,
    }
}
