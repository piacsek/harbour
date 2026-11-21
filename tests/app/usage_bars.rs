use crate::usage_states::app;
use crate::{Screen, script};

const ROW: &str = "  Session   ███░░░░░░░░░░░░░░░░░░░░░   12%   resets 7:20pm";

#[test]
fn a_finished_report_that_parses_shows_a_bar_per_limit() {
    let mut screen = Screen::of(app().with_usage(report("")));

    screen.press('u');

    let shown = screen.wait_for(ROW);
    assert!(shown.contains("  Week      ██"), "{shown}");
    assert!(!shown.contains("Current session: 12% used"), "{shown}");
}

#[test]
fn a_report_still_running_shows_its_text_as_it_arrives() {
    let mut screen = Screen::of(app().with_usage(report("exec sleep 600")));

    screen.press('u');

    let shown = screen.wait_for("Current week (all models): 90% used");
    assert!(shown.contains("Current session: 12% used"), "{shown}");
    assert!(!shown.contains(ROW), "{shown}");
}

fn report(then: &str) -> Vec<String> {
    script(&format!(
        "printf 'You are on the acme team plan\\n\\n'; \
         printf 'Current session: 12%% used · resets 7:20pm (Acme/Harbour)\\n'; \
         printf 'Current week (all models): 90%% used · resets Sun 5am (Acme/Harbour)\\n'; \
         {then}"
    ))
}
