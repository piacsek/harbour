use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

use crate::text;

const LIMIT_BAR: usize = 24;
const SHARE_BAR: usize = 12;
const FULL_WIDTH: usize = 78;
const NARROWEST_BAR: usize = 4;
const FILLED: &str = "█";
const UNFILLED: &str = "░";

#[derive(Debug, Default)]
pub struct Report {
    pub limits: Vec<Limit>,
    pub breakdowns: Vec<Breakdown>,
    pub notes: Vec<String>,
}

#[derive(Debug)]
pub struct Limit {
    pub label: String,
    pub percent: String,
    pub resets: String,
}

#[derive(Debug, Default)]
pub struct Breakdown {
    pub heading: String,
    pub shares: Vec<Share>,
    pub tops: Vec<Top>,
}

#[derive(Debug)]
pub struct Share {
    pub percent: String,
    pub label: String,
}

#[derive(Debug)]
pub struct Top {
    pub label: String,
    pub values: String,
}

#[derive(Default)]
struct Section {
    breakdown: Breakdown,
    indented: bool,
    last: Option<Part>,
}

#[derive(Clone, Copy)]
enum Part {
    Share,
    Top,
}

pub fn parse(text: &str) -> Option<Report> {
    let mut report = Report::default();
    let mut section: Option<Section> = None;
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if line.starts_with(char::is_whitespace) {
            if let Some(section) = section.as_mut() {
                section.read(line.trim());
            }
        } else if let Some(limit) = limit(line) {
            close(&mut report, section.take());
            report.limits.push(limit);
        } else if !report.limits.is_empty() {
            close(&mut report, section.take());
            section = Some(Section::headed(line.trim()));
        }
    }
    close(&mut report, section);
    (!report.limits.is_empty()).then_some(report)
}

impl Section {
    fn headed(heading: &str) -> Self {
        let mut section = Self::default();
        section.breakdown.heading = heading.to_string();
        section
    }

    fn read(&mut self, line: &str) {
        self.indented = true;
        if let Some(share) = share(line) {
            self.breakdown.shares.push(share);
            self.last = Some(Part::Share);
        } else if let Some(top) = top(line) {
            self.breakdown.tops.push(top);
            self.last = Some(Part::Top);
        } else {
            let continued = match self.last {
                Some(Part::Share) => self.breakdown.shares.last_mut().map(|s| &mut s.label),
                Some(Part::Top) => self.breakdown.tops.last_mut().map(|t| &mut t.values),
                None => None,
            };
            if let Some(text) = continued {
                text.push(' ');
                text.push_str(line);
            }
        }
    }
}

fn close(report: &mut Report, section: Option<Section>) {
    let Some(mut section) = section else { return };
    let breakdown = &mut section.breakdown;
    if !breakdown.shares.is_empty() || !breakdown.tops.is_empty() {
        for top in &mut breakdown.tops {
            top.values = top.values.replace(", ", " · ");
        }
        report.breakdowns.push(section.breakdown);
    } else if !section.indented && !breakdown.heading.ends_with('?') {
        report.notes.push(section.breakdown.heading);
    }
}

fn share(line: &str) -> Option<Share> {
    let (percent, rest) = percentage(line)?;
    let label = rest.trim().to_string();
    (rest.is_empty() || rest.starts_with(char::is_whitespace)).then_some(Share { percent, label })
}

fn top(line: &str) -> Option<Top> {
    let (kind, values) = line.strip_prefix("Top ")?.split_once(": ")?;
    (!kind.trim().is_empty()).then(|| Top {
        label: kind.trim().to_string(),
        values: values.trim().to_string(),
    })
}

pub fn lines(report: &Report, width: u16, height: u16, accent: Color) -> Vec<Line<'static>> {
    let width = usize::from(width);
    let shrink = FULL_WIDTH.saturating_sub(width);
    let bar = LIMIT_BAR.saturating_sub(shrink).max(NARROWEST_BAR);
    let short = SHARE_BAR.saturating_sub(shrink).max(NARROWEST_BAR);
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
    for breakdown in &report.breakdowns {
        rows.push(Line::default());
        rows.push(Line::raw(format!("  {}", breakdown.heading)));
        for share in &breakdown.shares {
            let mut spans = vec![Span::raw("   ")];
            spans.extend(filled(&share.percent, short, accent));
            spans.push(Span::raw(format!("  {:>4}", format!("{}%", share.percent))));
            if !share.label.is_empty() {
                spans.push(Span::raw(format!("  {}", share.label)));
            }
            rows.push(Line::from(spans));
        }
        let kind = widest(breakdown.tops.iter().map(|top| top.label.as_str()));
        for top in &breakdown.tops {
            rows.push(Line::raw(format!(
                "   {}   {}",
                padded(&top.label, kind),
                top.values
            )));
        }
    }
    if !report.notes.is_empty() {
        rows.push(Line::default());
        rows.extend(
            report
                .notes
                .iter()
                .map(|note| Line::raw(format!("  {note}"))),
        );
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
