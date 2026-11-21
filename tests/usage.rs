use harbour::usage::{self, Report};
use ratatui::style::Color;

const LIMITS: &str = "\
You are on the acme team plan

Current session: 12% used · resets 7:20pm (Acme/Harbour)
Current week (all models): 90% used · resets Sun 5am (Acme/Harbour)
Current week (acme-large): 73% used · resets Sun 5am (Acme/Harbour)
";

#[test]
fn the_limit_lines_parse_into_labels_percentages_and_resets() {
    let report = parsed(LIMITS);

    assert_eq!(report.limits.len(), 3);
    assert_eq!(report.limits[0].label, "Session");
    assert_eq!(report.limits[0].percent, "12");
    assert_eq!(report.limits[0].resets, "resets 7:20pm");
    assert_eq!(report.limits[1].label, "Week");
    assert_eq!(report.limits[1].percent, "90");
    assert_eq!(report.limits[1].resets, "resets Sun 5am");
    assert_eq!(report.limits[2].label, "Week, acme-large");
    assert_eq!(report.limits[2].percent, "73");
}

#[test]
fn a_report_without_limit_lines_does_not_parse() {
    let report = usage::parse("acme usage: not logged in\nTry again later\n");

    assert!(report.is_none());
}

#[test]
fn a_report_without_a_model_line_has_no_bar_for_it() {
    let text = "Current session: 4% used · resets 9pm\nCurrent week (all models): 8% used\n";

    let report = parsed(text);

    assert_eq!(report.limits.len(), 2);
    assert_eq!(report.limits[1].label, "Week");
    assert_eq!(report.limits[1].resets, "");
}

#[test]
fn a_limit_line_of_another_name_is_a_limit_under_that_name() {
    let report = parsed("Extra usage: 5% used · resets Dec 1\n");

    assert_eq!(report.limits[0].label, "Extra usage");
    assert_eq!(report.limits[0].percent, "5");
    assert_eq!(report.limits[0].resets, "resets Dec 1");
}

#[test]
fn a_limit_row_shows_its_label_bar_percentage_and_reset_at_80_columns() {
    let report = parsed(LIMITS);

    let rows = drawn(&report, 78, 30);

    assert_eq!(rows[0], "");
    assert_eq!(
        rows[1],
        format!(
            "  Session            {}{}   12%   resets 7:20pm",
            full(3),
            empty(21)
        )
    );
    assert_eq!(
        rows[2],
        format!(
            "  Week               {}{}   90%   resets Sun 5am",
            full(22),
            empty(2)
        )
    );
    assert_eq!(
        rows[3],
        format!(
            "  Week, acme-large   {}{}   73%   resets Sun 5am",
            full(18),
            empty(6)
        )
    );
}

fn parsed(text: &str) -> Report {
    usage::parse(text).unwrap()
}

fn drawn(report: &Report, width: u16, height: u16) -> Vec<String> {
    usage::lines(report, width, height, Color::Cyan)
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect()
        })
        .collect()
}

fn full(cells: usize) -> String {
    "█".repeat(cells)
}

fn empty(cells: usize) -> String {
    "░".repeat(cells)
}
