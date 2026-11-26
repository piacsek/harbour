use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

use crate::icons;
use crate::text::{columns, fit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Colors {
    pub repository: Color,
    pub branch: Color,
    pub pull_request: Color,
}

impl Default for Colors {
    fn default() -> Colors {
        Colors {
            repository: Color::Blue,
            branch: Color::Magenta,
            pull_request: Color::Cyan,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Repository,
    Branch,
    PullRequest,
    Subagents,
    Tasks,
    TasksDone,
}

impl Kind {
    fn glyph(self) -> &'static str {
        match self {
            Kind::Repository => icons::REPOSITORY,
            Kind::Branch => icons::BRANCH,
            Kind::PullRequest => icons::PULL_REQUEST,
            Kind::Subagents => icons::SUBAGENTS,
            Kind::Tasks => icons::TASKS_OPEN,
            Kind::TasksDone => icons::TASKS_DONE,
        }
    }

    fn style(self, colors: &Colors) -> Style {
        match self {
            Kind::Repository => Style::default().fg(colors.repository),
            Kind::Branch => Style::default().fg(colors.branch),
            Kind::PullRequest => Style::default().fg(colors.pull_request),
            Kind::Subagents | Kind::Tasks => Style::default().add_modifier(Modifier::DIM),
            Kind::TasksDone => Style::default().fg(Color::Green),
        }
    }
}

pub fn fitted(
    parts: &[(Kind, String)],
    iconed: bool,
    colors: &Colors,
    room: usize,
) -> Vec<Span<'static>> {
    let segments = segments(parts, iconed, colors);
    let whole: String = segments.iter().map(|(text, _)| text.as_str()).collect();
    let shown = if iconed {
        fit_parts(parts, room)
    } else {
        fit(&whole, room)
    };
    cut(&segments, &whole, &shown)
}

fn segments(parts: &[(Kind, String)], iconed: bool, colors: &Colors) -> Vec<(String, Style)> {
    let mut segments = Vec::new();
    for (kind, text) in parts {
        if !segments.is_empty() {
            let (gap, style) = if iconed {
                (" ", Style::default())
            } else {
                (" · ", Style::default().add_modifier(Modifier::DIM))
            };
            segments.push((gap.to_string(), style));
        }
        let shown = match (iconed, kind) {
            (true, _) => format!("{} {text}", kind.glyph()),
            (false, Kind::Subagents) if text == "1" => "1 subagent".to_string(),
            (false, Kind::Subagents) => format!("{text} subagents"),
            (false, Kind::Tasks | Kind::TasksDone) => format!("tasks {text}"),
            (false, _) => text.clone(),
        };
        segments.push((shown, kind.style(colors)));
    }
    segments
}

fn cut(segments: &[(String, Style)], whole: &str, shown: &str) -> Vec<Span<'static>> {
    let (kept, ended) = match shown.strip_suffix('…') {
        Some(kept) if shown != whole => (kept.len(), true),
        _ => (shown.len(), false),
    };
    let mut spans = Vec::new();
    let mut start = 0;
    let mut cut_style = None;
    for (text, style) in segments {
        let end = start + text.len();
        if start < kept {
            spans.push(Span::styled(
                text[..kept.min(end) - start].to_string(),
                *style,
            ));
        }
        if cut_style.is_none() && kept < end {
            cut_style = Some(*style);
        }
        start = end;
    }
    if ended {
        spans.push(Span::styled("…", cut_style.unwrap_or_default()));
    }
    spans
}

fn fit_parts(parts: &[(Kind, String)], room: usize) -> String {
    let mut whole = String::new();
    let mut starts = Vec::new();
    for (kind, text) in parts {
        let glyph = kind.glyph();
        if !whole.is_empty() {
            whole.push(' ');
        }
        let start = whole.len();
        whole.push_str(&format!("{glyph} {text}"));
        starts.push((start, start + glyph.len() + 1));
    }
    let cut = fit(&whole, room);
    if cut == whole {
        return cut;
    }
    let kept = cut.len().saturating_sub('…'.len_utf8());
    match starts.iter().rev().find(|(start, _)| *start < kept) {
        Some((start, text)) if columns(&whole[(*text).min(kept)..kept]) == 0 => {
            format!("{}…", &whole[..*start])
        }
        _ => cut,
    }
}
