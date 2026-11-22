use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::text;

const LIMIT_BAR: usize = 24;
const SHARE_BAR: usize = 12;
const FULL_WIDTH: usize = 78;
const NARROWEST_BAR: usize = 4;
const FILLED: &str = "█";
const UNFILLED: &str = "░";
const HINT: &str = "  t raw text · Esc close";
const LOADING_HINT: &str = "  t text · Esc close";
const WAITING_HINT: &str = "  t animation · Esc close";
const WAITING: &str = "nothing printed yet";
const LOADING: &str = "reading usage…";

#[derive(Debug, Default)]
pub struct Report {
    pub limits: Vec<Limit>,
    pub breakdowns: Vec<Breakdown>,
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
        section.breakdown.heading = text::clean(heading);
        section
    }

    fn read(&mut self, line: &str) {
        self.indented = true;
        let topped = matches!(self.last, Some(Part::Top)) && percentage(line).is_some();
        if let Some(share) = share(line).filter(|_| !topped) {
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
                if !text.is_empty() {
                    text.push(' ');
                }
                text.push_str(&text::clean(line));
            }
        }
    }
}

fn close(report: &mut Report, section: Option<Section>) {
    let Some(mut section) = section else { return };
    let breakdown = &mut section.breakdown;
    let read = !breakdown.shares.is_empty() || !breakdown.tops.is_empty();
    let alone = !section.indented && !breakdown.heading.ends_with('?');
    if read || alone {
        for top in &mut breakdown.tops {
            top.values = top.values.replace(", ", " · ");
        }
        report.breakdowns.push(section.breakdown);
    }
}

fn share(line: &str) -> Option<Share> {
    let (percent, rest) = percentage(line)?;
    let label = text::clean(rest);
    Some(Share { percent, label })
}

fn top(line: &str) -> Option<Top> {
    let (kind, values) = line.strip_prefix("Top ")?.split_once(": ")?;
    (!kind.trim().is_empty()).then(|| Top {
        label: text::clean(kind),
        values: text::clean(values),
    })
}

pub fn lines(report: &Report, width: u16, height: u16, accent: Color) -> Vec<Line<'static>> {
    let width = usize::from(width);
    let shrink = FULL_WIDTH.saturating_sub(width);
    let bar = cells_for(width);
    let short = SHARE_BAR.saturating_sub(shrink).max(NARROWEST_BAR);
    let label = widest(report.limits.iter().map(|limit| limit.label.as_str()));
    let used = shown(report.limits.iter().map(|limit| limit.percent.as_str()));
    let spent = shown(
        report
            .breakdowns
            .iter()
            .flat_map(|breakdown| &breakdown.shares)
            .map(|share| share.percent.as_str()),
    );
    let mut rows = Vec::new();
    for limit in &report.limits {
        let mut spans = vec![Span::raw(format!("  {}   ", padded(&limit.label, label)))];
        spans.extend(filled(&limit.percent, bar, fullness(&limit.percent)));
        spans.push(Span::raw(format!(
            "  {:>used$}",
            format!("{}%", limit.percent)
        )));
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
            spans.push(Span::raw(format!(
                "  {:>spent$}",
                format!("{}%", share.percent)
            )));
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
    let height = usize::from(height);
    let above = height.saturating_sub(1);
    rows.truncate(above);
    if rows.len() < above {
        rows.push(Line::default());
    }
    if rows.len() < above {
        rows.insert(0, Line::default());
    }
    if height > 0 {
        rows.push(Line::raw(HINT));
    }
    rows.into_iter().map(|row| cut(row, width)).collect()
}

fn cut(line: Line<'static>, width: usize) -> Line<'static> {
    let total: usize = line
        .spans
        .iter()
        .map(|span| text::columns(&span.content))
        .sum();
    if total <= width {
        return line;
    }
    let mut room = width;
    let mut spans = Vec::new();
    for span in line.spans {
        let columns = text::columns(&span.content);
        if columns < room {
            room -= columns;
            spans.push(span);
        } else {
            let kept = text::fit(&format!("{} ", span.content), room);
            spans.push(Span::styled(kept, span.style));
            break;
        }
    }
    Line::from(spans)
}

fn limit(line: &str) -> Option<Limit> {
    let (name, rest) = line.trim_end().split_once(": ")?;
    let (percent, rest) = percentage(rest)?;
    let rest = rest.strip_prefix(" used")?;
    let resets = rest
        .find("resets ")
        .map_or(String::new(), |at| text::clean(zoneless(&rest[at..])));
    let name = text::clean(name);
    (!name.is_empty()).then(|| Limit {
        label: labelled(&name),
        percent,
        resets,
    })
}

fn percentage(text: &str) -> Option<(String, &str)> {
    let (number, rest) = text.split_once('%')?;
    let unsigned = number.strip_prefix(['<', '~', '>']).unwrap_or(number);
    let (whole, part) = unsigned.split_once(['.', ',']).unwrap_or((unsigned, "0"));
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

fn shown<'a>(percents: impl Iterator<Item = &'a str>) -> usize {
    percents
        .map(|percent| text::columns(percent) + 1)
        .max()
        .unwrap_or(0)
        .max(4)
}

fn padded(label: &str, width: usize) -> String {
    format!(
        "{label}{}",
        " ".repeat(width.saturating_sub(text::columns(label)))
    )
}

fn hundredths(percent: &str) -> usize {
    let percent = percent.trim_start_matches(['<', '~', '>']);
    let (whole, part) = percent.split_once(['.', ',']).unwrap_or((percent, ""));
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

pub fn waiting(width: u16, height: u16) -> Vec<Line<'static>> {
    let width = usize::from(width);
    let mut rows = vec![Line::default(); usize::from(height)];
    if let Some(last) = rows.last_mut() {
        *last = Line::raw(WAITING_HINT);
    }
    if let Some(first) = rows.first_mut() {
        *first = Line::styled(WAITING, Style::new().add_modifier(Modifier::DIM));
    }
    rows.into_iter().map(|row| cut(row, width)).collect()
}

pub fn loading(step: usize, width: u16, height: u16, accent: Color) -> Vec<Line<'static>> {
    let width = usize::from(width);
    let height = usize::from(height);
    let cells = cells_for(width);
    let label = Line::from(vec![
        Span::raw(" ".repeat(width.saturating_sub(text::columns(LOADING)) / 2)),
        Span::raw(LOADING),
    ]);
    let mut rows = vec![Line::default(); height];
    if height >= 3 {
        let top = ((height - 3) / 2).min(height.saturating_sub(4));
        let mut bar = vec![Span::raw(" ".repeat(width.saturating_sub(cells) / 2))];
        bar.extend(sweep(step % cells, cells, accent));
        rows[top] = Line::from(bar);
        rows[top + 2] = label;
        if height >= 4 {
            rows[height - 1] = Line::raw(LOADING_HINT);
        }
    } else if height > 0 {
        rows[(height - 1) / 2] = label;
    }
    rows.into_iter().map(|row| cut(row, width)).collect()
}

fn sweep(centre: usize, cells: usize, accent: Color) -> Vec<Span<'static>> {
    (0..cells)
        .map(|cell| match cell.abs_diff(centre) {
            0 => Span::styled(FILLED, Style::new().fg(accent)),
            1 => Span::styled("▓", Style::new().fg(accent)),
            2 => Span::styled("▒", Style::new().fg(accent)),
            _ => Span::styled(UNFILLED, Style::new().add_modifier(Modifier::DIM)),
        })
        .collect()
}

fn cells_for(width: usize) -> usize {
    LIMIT_BAR
        .saturating_sub(FULL_WIDTH.saturating_sub(width))
        .max(NARROWEST_BAR)
}
