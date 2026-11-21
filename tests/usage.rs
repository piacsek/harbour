use harbour::usage::{self, Report};
use ratatui::style::Color;

const LIMITS: &str = "\
You are on the acme team plan

Current session: 12% used · resets 7:20pm (Acme/Harbour)
Current week (all models): 90% used · resets Sun 5am (Acme/Harbour)
Current week (acme-large): 73% used · resets Sun 5am (Acme/Harbour)
";

const FULL: &str = "\
You are on the acme team plan

Current session: 12% used · resets 7:20pm (Acme/Harbour)
Current week (all models): 90% used · resets Sun 5am (Acme/Harbour)
Current week (acme-large): 73% used · resets Sun 5am (Acme/Harbour)

What's contributing to your usage?
Estimates only · this machine only

Last 24h · 310 requests · 7 sessions
  64%  Long sessions keep their whole context
  25%  Restart when the task changes
  Top skills: tdd 12%, review 4%
  Top subagents: Explore 31%
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

#[test]
fn a_breakdown_keeps_its_heading_shares_and_tops() {
    let report = parsed(FULL);

    assert_eq!(report.breakdowns.len(), 1);
    let breakdown = &report.breakdowns[0];
    assert_eq!(breakdown.heading, "Last 24h · 310 requests · 7 sessions");
    assert_eq!(breakdown.shares.len(), 2);
    assert_eq!(breakdown.shares[0].percent, "64");
    assert_eq!(
        breakdown.shares[0].label,
        "Long sessions keep their whole context"
    );
    assert_eq!(breakdown.shares[1].percent, "25");
    assert_eq!(breakdown.shares[1].label, "Restart when the task changes");
    assert_eq!(breakdown.tops.len(), 2);
    assert_eq!(breakdown.tops[0].label, "skills");
    assert_eq!(breakdown.tops[0].values, "tdd 12% · review 4%");
    assert_eq!(breakdown.tops[1].label, "subagents");
    assert_eq!(breakdown.tops[1].values, "Explore 31%");
}

#[test]
fn a_line_with_nothing_indented_under_it_is_a_note_unless_it_asks_a_question() {
    let report = parsed(FULL);

    assert_eq!(report.notes, vec!["Estimates only · this machine only"]);
}

#[test]
fn a_section_whose_lines_do_not_parse_is_left_out() {
    let text = format!("{LIMITS}\nAcme heading\n  nothing to read here\n  nor here\n");

    let report = parsed(&text);

    assert_eq!(report.breakdowns.len(), 0);
    assert_eq!(report.notes.len(), 0);
}

#[test]
fn a_share_without_a_tip_keeps_its_percentage() {
    let text = format!("{LIMITS}\nLast 7d\n  5%\n");

    let report = parsed(&text);

    assert_eq!(report.breakdowns[0].shares[0].percent, "5");
    assert_eq!(report.breakdowns[0].shares[0].label, "");
}

#[test]
fn an_indented_line_that_is_no_share_continues_the_line_before() {
    let text = format!(
        "{LIMITS}\nLast 7d\n  40%  Long sessions keep\n    their whole context\n  Top skills: tdd 3%,\n    review 1%\n"
    );

    let report = parsed(&text);

    let breakdown = &report.breakdowns[0];
    assert_eq!(breakdown.shares.len(), 1);
    assert_eq!(
        breakdown.shares[0].label,
        "Long sessions keep their whole context"
    );
    assert_eq!(breakdown.tops.len(), 1);
    assert_eq!(breakdown.tops[0].values, "tdd 3% · review 1%");
}

#[test]
fn a_breakdown_draws_a_short_bar_per_share_and_its_tops_under_its_heading() {
    let report = parsed(FULL);

    let rows = drawn(&report, 78, 30);

    assert_eq!(rows[4], "");
    assert_eq!(rows[5], "  Last 24h · 310 requests · 7 sessions");
    assert_eq!(
        rows[6],
        format!(
            "   {}{}   64%  Long sessions keep their whole context",
            full(8),
            empty(4)
        )
    );
    assert_eq!(
        rows[7],
        format!(
            "   {}{}   25%  Restart when the task changes",
            full(3),
            empty(9)
        )
    );
    assert_eq!(rows[8], "   skills      tdd 12% · review 4%");
    assert_eq!(rows[9], "   subagents   Explore 31%");
    assert_eq!(rows[10], "");
    assert_eq!(rows[11], "  Estimates only · this machine only");
}

#[test]
fn a_share_without_a_tip_draws_its_bar_and_number_alone() {
    let report = parsed(&format!("{LIMITS}\nLast 7d\n  50%\n"));

    let rows = drawn(&report, 78, 30);

    assert_eq!(rows[6], format!("   {}{}   50%", full(6), empty(6)));
}

#[test]
fn a_cjk_top_kind_is_padded_by_its_display_width() {
    let report = parsed(&format!(
        "{LIMITS}\nLast 7d\n  Top 技能: 検索 3%\n  Top subagents: Explore 1%\n"
    ));

    let rows = drawn(&report, 78, 30);

    assert_eq!(rows[6], "   技能        検索 3%");
    assert_eq!(rows[7], "   subagents   Explore 1%");
}

#[test]
fn breakdown_bars_take_the_accent() {
    let report = parsed(FULL);

    let lines = usage::lines(&report, 78, 30, Color::Magenta);

    assert_eq!(lines[6].spans[1].style.fg, Some(Color::Magenta));
}

#[test]
fn the_drawing_ends_with_a_blank_line_and_the_hint() {
    let report = parsed(LIMITS);

    let rows = drawn(&report, 78, 30);

    assert_eq!(rows.len(), 6);
    assert_eq!(rows[4], "");
    assert_eq!(rows[5], "  t raw text · Esc close");
}

#[test]
fn a_short_popup_keeps_the_hint_on_its_last_row() {
    let report = parsed(FULL);

    let rows = drawn(&report, 78, 5);

    assert_eq!(rows.len(), 5);
    assert!(rows[1].starts_with("  Session   "), "{rows:?}");
    assert!(rows[2].starts_with("  Week   "), "{rows:?}");
    assert_eq!(rows[3], "");
    assert_eq!(rows[4], "  t raw text · Esc close");
}

#[test]
fn a_popup_of_one_row_shows_the_hint_alone() {
    let report = parsed(FULL);

    let rows = drawn(&report, 78, 1);

    assert_eq!(rows, vec!["  t raw text · Esc close"]);
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
