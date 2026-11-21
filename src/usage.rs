use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

use crate::text;

const LIMIT_BAR: usize = 24;
const FULL_WIDTH: usize = 78;
const NARROWEST_BAR: usize = 4;
const FILLED: &str = "█";
const UNFILLED: &str = "░";

#[derive(Debug, Default)]
pub struct Report {
    pub limits: Vec<Limit>,
}

#[derive(Debug)]
pub struct Limit {
    pub label: String,
    pub percent: String,
    pub resets: String,
}

pub fn parse(text: &str) -> Option<Report> {
    let limits: Vec<Limit> = text
        .lines()
        .filter(|line| !line.starts_with(char::is_whitespace))
        .filter_map(limit)
        .collect();
    (!limits.is_empty()).then_some(Report { limits })
}

pub fn lines(report: &Report, width: u16, height: u16, _accent: Color) -> Vec<Line<'static>> {
    let width = usize::from(width);
    let shrink = FULL_WIDTH.saturating_sub(width);
    let bar = LIMIT_BAR.saturating_sub(shrink).max(NARROWEST_BAR);
    let label = widest(report.limits.iter().map(|limit| limit.label.as_str()));
    let mut rows = vec![Line::default()];
    for limit in &report.limits {
        let mut spans = vec![Span::raw(format!("  {}   ", padded(&limit.label, label)))];
        spans.extend(filled(&limit.percent, bar, fullness(&limit.percent)));
        spans.push(Span::raw(format!("  {:>4}", format!("{}%", limit.percent))));
        if !limit.resets.is_empty() {
            spans.push(Span::raw(format!("   {}", limit.resets)));
        }
        rows.push(Line::from(spans));
    }
    rows.truncate(usize::from(height));
    rows
}

fn limit(line: &str) -> Option<Limit> {
    let (name, rest) = line.trim_end().split_once(": ")?;
    let (percent, rest) = percentage(rest)?;
    let rest = rest.strip_prefix(" used")?;
    let resets = rest
        .find("resets ")
        .map_or(String::new(), |at| zoneless(&rest[at..]).to_string());
    (!name.trim().is_empty()).then(|| Limit {
        label: labelled(name.trim()),
        percent,
        resets,
    })
}

fn percentage(text: &str) -> Option<(String, &str)> {
    let (number, rest) = text.split_once('%')?;
    let (whole, part) = number.split_once('.').unwrap_or((number, "0"));
    let digits = |text: &str| !text.is_empty() && text.chars().all(|c| c.is_ascii_digit());
    let valid = digits(whole) && digits(part);
    valid.then(|| (number.to_string(), rest))
}

fn labelled(name: &str) -> String {
    let name = name.strip_prefix("Current ").unwrap_or(name);
    let name = name.strip_suffix(" (all models)").unwrap_or(name);
    let name = match name
        .strip_suffix(')')
        .and_then(|kept| kept.split_once(" ("))
    {
        Some((kind, model)) => format!("{kind}, {model}"),
        None => name.to_string(),
    };
    let mut chars = name.chars();
    chars.next().map_or(String::new(), |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

fn zoneless(text: &str) -> &str {
    match text.strip_suffix(')').and_then(|kept| kept.rfind(" (")) {
        Some(at) => &text[..at],
        None => text,
    }
}

fn widest<'a>(labels: impl Iterator<Item = &'a str>) -> usize {
    labels.map(text::columns).max().unwrap_or(0)
}

fn padded(label: &str, width: usize) -> String {
    format!(
        "{label}{}",
        " ".repeat(width.saturating_sub(text::columns(label)))
    )
}

fn hundredths(percent: &str) -> usize {
    let (whole, part) = percent.split_once('.').unwrap_or((percent, ""));
    let part = format!("{part:0<2}");
    let whole: usize = whole.parse().unwrap_or(0);
    let part: usize = part.get(..2).and_then(|two| two.parse().ok()).unwrap_or(0);
    whole.saturating_mul(100).saturating_add(part)
}

fn fullness(percent: &str) -> Color {
    match hundredths(percent) {
        at if at >= 9000 => Color::Red,
        at if at >= 7000 => Color::Yellow,
        _ => Color::Green,
    }
}

fn filled(percent: &str, cells: usize, colour: Color) -> Vec<Span<'static>> {
    let on = (hundredths(percent)
        .saturating_mul(cells)
        .saturating_add(5000)
        / 10000)
        .min(cells);
    vec![
        Span::styled(FILLED.repeat(on), Style::new().fg(colour)),
        Span::raw(UNFILLED.repeat(cells - on)),
    ]
}
