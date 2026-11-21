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

    assert_eq!(report.breakdowns.len(), 2);
    let breakdown = &report.breakdowns[1];
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
fn a_line_with_nothing_indented_under_it_is_a_heading_alone_unless_it_asks_a_question() {
    let report = parsed(FULL);

    assert_eq!(report.breakdowns.len(), 2);
    assert_eq!(
        report.breakdowns[0].heading,
        "Estimates only · this machine only"
    );
    assert_eq!(report.breakdowns[0].shares.len(), 0);
    assert_eq!(report.breakdowns[0].tops.len(), 0);
}

#[test]
fn a_period_without_activity_stays_where_the_report_printed_it() {
    let text = format!(
        "{LIMITS}\nLast 24h · 0 requests · 0 sessions\n\nLast 7d · 9 requests · 2 sessions\n  40%  Long sessions\n"
    );

    let report = parsed(&text);
    let rows = drawn(&report, 78, 30);

    assert_eq!(rows[4], "");
    assert_eq!(rows[5], "  Last 24h · 0 requests · 0 sessions");
    assert_eq!(rows[6], "");
    assert_eq!(rows[7], "  Last 7d · 9 requests · 2 sessions");
}

#[test]
fn a_section_whose_lines_do_not_parse_is_left_out() {
    let text = format!("{LIMITS}\nAcme heading\n  nothing to read here\n  nor here\n");

    let report = parsed(&text);

    assert_eq!(report.breakdowns.len(), 0);
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
fn a_line_continuing_a_share_without_a_tip_becomes_its_tip() {
    let text = format!("{LIMITS}\nLast 7d\n  5%\n    wrapped tip\n");

    let report = parsed(&text);

    assert_eq!(report.breakdowns[0].shares[0].label, "wrapped tip");
}

#[test]
fn a_top_line_continued_with_a_percentage_stays_one_top_line() {
    let text = format!("{LIMITS}\nLast 7d\n  Top skills: tdd 12%, review\n    6%, plan 2%\n");

    let report = parsed(&text);

    let breakdown = &report.breakdowns[0];
    assert_eq!(breakdown.shares.len(), 0);
    assert_eq!(breakdown.tops.len(), 1);
    assert_eq!(breakdown.tops[0].values, "tdd 12% · review 6% · plan 2%");
}

#[test]
fn a_breakdown_draws_a_short_bar_per_share_and_its_tops_under_its_heading() {
    let report = parsed(FULL);

    let rows = drawn(&report, 78, 30);

    assert_eq!(rows[4], "");
    assert_eq!(rows[5], "  Estimates only · this machine only");
    assert_eq!(rows[6], "");
    assert_eq!(rows[7], "  Last 24h · 310 requests · 7 sessions");
    assert_eq!(
        rows[8],
        format!(
            "   {}{}   64%  Long sessions keep their whole context",
            full(8),
            empty(4)
        )
    );
    assert_eq!(
        rows[9],
        format!(
            "   {}{}   25%  Restart when the task changes",
            full(3),
            empty(9)
        )
    );
    assert_eq!(rows[10], "   skills      tdd 12% · review 4%");
    assert_eq!(rows[11], "   subagents   Explore 31%");
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

    assert_eq!(lines[8].spans[1].style.fg, Some(Color::Magenta));
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
    assert!(rows[0].starts_with("  Session   "), "{rows:?}");
    assert!(rows[1].starts_with("  Week   "), "{rows:?}");
    assert!(rows[2].starts_with("  Week, acme-large   "), "{rows:?}");
    assert_eq!(rows[3], "");
    assert_eq!(rows[4], "  t raw text · Esc close");
}

#[test]
fn a_popup_of_three_rows_gives_them_to_the_limit_bars_and_the_hint() {
    let report = parsed(FULL);

    let rows = drawn(&report, 78, 3);

    assert_eq!(rows.len(), 3);
    assert!(rows[0].starts_with("  Session   "), "{rows:?}");
    assert!(rows[1].starts_with("  Week   "), "{rows:?}");
    assert_eq!(rows[2], "  t raw text · Esc close");
}

#[test]
fn a_popup_of_two_rows_shows_the_first_limit_bar_and_the_hint() {
    let report = parsed(FULL);

    let rows = drawn(&report, 78, 2);

    assert_eq!(rows.len(), 2);
    assert!(rows[0].starts_with("  Session   "), "{rows:?}");
    assert_eq!(rows[1], "  t raw text · Esc close");
}

#[test]
fn a_popup_of_one_row_shows_the_hint_alone() {
    let report = parsed(FULL);

    let rows = drawn(&report, 78, 1);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0], "  t raw text · Esc close");
}

#[test]
fn a_narrower_popup_shrinks_the_bars_first() {
    let report = parsed(FULL);

    let rows = drawn(&report, 60, 30);

    assert_eq!(
        rows[1],
        format!(
            "  Session            {}{}   12%   resets 7:20pm",
            full(1),
            empty(5)
        )
    );
    assert_eq!(
        rows[8],
        format!(
            "   {}{}   64%  Long sessions keep their whole context",
            full(3),
            empty(1)
        )
    );
}

#[test]
fn a_line_too_long_for_the_popup_is_cut_with_an_ellipsis() {
    let report = parsed(FULL);

    let rows = drawn(&report, 40, 30);

    assert_eq!(
        rows[8],
        format!(
            "   {}{}   64%  Long sessions keep their…",
            full(3),
            empty(1)
        )
    );
    assert_eq!(harbour::text::columns(&rows[8]), 40);
}

#[test]
fn a_span_that_fills_the_room_exactly_still_ends_with_an_ellipsis() {
    let report = parsed(FULL);

    let rows = drawn(&report, 21, 30);

    assert_eq!(rows[1], "  Session           …");
}

#[test]
fn the_narrowest_popup_cuts_every_line_to_its_width() {
    let report = parsed(FULL);

    let rows = drawn(&report, 10, 30);

    assert_eq!(rows[1], "  Session…");
    assert_eq!(rows[7], "  Last 24…");
    assert_eq!(rows[rows.len() - 1], "  t raw t…");
}

#[test]
fn a_cjk_tip_is_cut_by_its_display_width() {
    let report = parsed(&format!("{LIMITS}\nLast 7d\n  30%  長いセッションが続く\n"));

    let rows = drawn(&report, 30, 30);

    assert_eq!(
        rows[6],
        format!("   {}{}   30%  長いセッション…", full(1), empty(3))
    );
    assert_eq!(harbour::text::columns(&rows[6]), 30);
}

#[test]
fn a_limit_bar_is_green_under_70_percent() {
    let lines = limit_lines("69.9");

    assert_eq!(lines[1].spans[1].style.fg, Some(Color::Green));
}

#[test]
fn a_limit_bar_is_yellow_from_70_percent() {
    let lines = limit_lines("70");

    assert_eq!(lines[1].spans[1].style.fg, Some(Color::Yellow));
}

#[test]
fn a_limit_bar_is_red_from_90_percent() {
    let lines = limit_lines("90");

    assert_eq!(lines[1].spans[1].style.fg, Some(Color::Red));
}

#[test]
fn a_percentage_above_100_fills_a_red_bar_and_prints_as_given() {
    let report = parsed("Current session: 130% used · resets 7:20pm\n");

    let rows = drawn(&report, 78, 30);
    let lines = usage::lines(&report, 78, 30, Color::Cyan);

    assert_eq!(
        rows[1],
        format!("  Session   {}  130%   resets 7:20pm", full(24))
    );
    assert_eq!(lines[1].spans[1].style.fg, Some(Color::Red));
}

#[test]
fn a_share_shaped_like_a_percentage_is_its_own_row_and_not_part_of_the_tip_above() {
    let report = parsed(&format!("{LIMITS}\n{SHAPED}"));

    let shares = &report.breakdowns[0].shares;
    assert_eq!(shares.len(), 4);
    assert_eq!(shares[0].label, "Long sessions");
    assert_eq!(shares[1].percent, "<1");
    assert_eq!(shares[1].label, "Rare tools");
    assert_eq!(shares[2].percent, "~5");
    assert_eq!(shares[2].label, "Some tools");
    assert_eq!(shares[3].percent, "12,5");
    assert_eq!(shares[3].label, "Comma tools");
}

#[test]
fn shares_shaped_like_percentages_draw_their_bars_and_line_up_with_the_widest() {
    let report = parsed(&format!("{LIMITS}\n{SHAPED}"));

    let rows = drawn(&report, 78, 30);

    assert_eq!(
        rows[6],
        format!("   {}{}    40%  Long sessions", full(5), empty(7))
    );
    assert_eq!(rows[7], format!("   {}    <1%  Rare tools", empty(12)));
    assert_eq!(
        rows[8],
        format!("   {}{}    ~5%  Some tools", full(1), empty(11))
    );
    assert_eq!(
        rows[9],
        format!("   {}{}  12,5%  Comma tools", full(2), empty(10))
    );
}

#[test]
fn limit_percentages_line_up_with_the_widest() {
    let report = parsed(
        "Current session: 12.5% used · resets 7pm\nCurrent week (all models): 9% used · resets Sun 5am\n",
    );

    let rows = drawn(&report, 78, 30);

    assert_eq!(
        rows[1],
        format!("  Session   {}{}  12.5%   resets 7pm", full(3), empty(21))
    );
    assert_eq!(
        rows[2],
        format!(
            "  Week      {}{}     9%   resets Sun 5am",
            full(2),
            empty(22)
        )
    );
}

#[test]
fn every_text_the_report_gives_is_cleaned_of_hidden_characters() {
    let text = "Current\u{200b} session: 12% used · resets 7:20pm\u{202e}\n\
        \nLast 7d\u{202e}\n  40%  Long\u{202e} sessions\u{200b}\n  Top skills\u{200b}: tdd\u{202e} 3%\n\
        \nEstimates\u{200b} only\n";

    let report = parsed(text);

    assert_eq!(report.limits[0].label, "Session");
    assert_eq!(report.limits[0].resets, "resets 7:20pm");
    assert_eq!(report.breakdowns[0].heading, "Last 7d");
    assert_eq!(report.breakdowns[0].shares[0].label, "Long sessions");
    assert_eq!(report.breakdowns[0].tops[0].label, "skills");
    assert_eq!(report.breakdowns[0].tops[0].values, "tdd 3%");
    assert_eq!(report.breakdowns[1].heading, "Estimates only");
}

const SHAPED: &str = "\
Last 7d
  40%  Long sessions
  <1%  Rare tools
  ~5%  Some tools
  12,5%  Comma tools
";

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

fn limit_lines(percent: &str) -> Vec<ratatui::text::Line<'static>> {
    let report = parsed(&format!("Current session: {percent}% used\n"));
    usage::lines(&report, 78, 30, Color::Cyan)
}
