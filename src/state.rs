use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::registry::{Record, Status};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Working,
    Blocked,
    Idle,
    Exited,
    Unknown,
}

impl State {
    fn of(exited: bool, record: Option<&Record>) -> State {
        if exited {
            return State::Exited;
        }
        match record.map(|record| record.status) {
            Some(Status::Busy | Status::Shell) => State::Working,
            Some(Status::Waiting) => State::Blocked,
            Some(Status::Idle) => State::Idle,
            _ => State::Unknown,
        }
    }

    fn glyph(self) -> &'static str {
        match self {
            State::Working => "●",
            State::Blocked => "◉",
            State::Idle | State::Exited => "○",
            State::Unknown => "·",
        }
    }

    fn style(self) -> Style {
        match self {
            State::Working => Style::default().fg(Color::Yellow),
            State::Blocked => Style::default().fg(Color::Red),
            State::Idle | State::Exited => Style::default().add_modifier(Modifier::DIM),
            State::Unknown => Style::default().fg(Color::DarkGray),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    pub blocked: usize,
    pub working: usize,
    pub idle: usize,
    pub unknown: usize,
}

impl Counts {
    pub fn plus(self, other: Counts) -> Counts {
        Counts {
            blocked: self.blocked.saturating_add(other.blocked),
            working: self.working.saturating_add(other.working),
            idle: self.idle.saturating_add(other.idle),
            unknown: self.unknown.saturating_add(other.unknown),
        }
    }
}

pub fn status_line(counts: Counts) -> String {
    [
        (State::Blocked, "fg=red", counts.blocked),
        (State::Working, "fg=yellow", counts.working),
        (State::Idle, "dim", counts.idle),
        (State::Unknown, "fg=brightblack", counts.unknown),
    ]
    .into_iter()
    .filter(|(_, _, count)| *count > 0)
    .map(|(state, style, count)| format!("#[{style}]{} {count}#[default]", state.glyph()))
    .collect::<Vec<_>>()
    .join(" ")
}

pub fn row(
    name: &str,
    exit_note: Option<String>,
    record: Option<&Record>,
    width: usize,
) -> Line<'static> {
    let state = State::of(exit_note.is_some(), record);
    let reason = record
        .and_then(|record| record.waiting_for.as_ref())
        .filter(|_| state == State::Blocked);
    let name = match reason {
        Some(reason) => shortened(name, reason, width),
        None => name.to_string(),
    };
    let mut spans = vec![
        Span::styled(state.glyph(), state.style()),
        Span::from(format!(" {name}")),
    ];
    match (exit_note, reason) {
        (Some(note), _) => spans.push(Span::from(format!(" exited{note}"))),
        (None, Some(reason)) => {
            spans.push(Span::from(" "));
            spans.push(Span::styled(reason.clone(), state.style()));
        }
        _ => {}
    }
    Line::from(spans)
}

fn shortened(name: &str, reason: &str, width: usize) -> String {
    let room = width.saturating_sub(2);
    let length = name.width();
    let wanted = reason.width() + 1;
    if length + wanted <= room {
        return name.to_string();
    }
    let budget = room.saturating_sub(wanted).max(room / 2);
    if length <= budget {
        return name.to_string();
    }
    let mut kept = String::new();
    for character in name.chars() {
        if kept.width() + character.width().unwrap_or(0) > budget.saturating_sub(1) {
            break;
        }
        kept.push(character);
    }
    format!("{kept}…")
}

pub fn title<'a>(
    sessions: impl Iterator<Item = (bool, Option<&'a Record>)>,
    width: usize,
) -> Line<'static> {
    let mut counts = [0; 4];
    for (exited, record) in sessions {
        counts[match State::of(exited, record) {
            State::Working => 0,
            State::Blocked => 1,
            State::Idle | State::Exited => 2,
            State::Unknown => 3,
        }] += 1;
    }
    let full = counted(" sessions ", counts, " ");
    if full.width() <= width {
        full
    } else {
        counted(" ", counts, "")
    }
}

fn counted(lead: &str, counts: [usize; 4], gap: &str) -> Line<'static> {
    let mut spans = vec![Span::from(lead.to_string())];
    for (glyph, style, count) in [
        ("●", State::Working.style(), counts[0]),
        ("◉", State::Blocked.style(), counts[1]),
        ("○", Style::default(), counts[2]),
        ("·", Style::default(), counts[3]),
    ] {
        spans.push(Span::styled(glyph, style));
        spans.push(Span::from(format!("{gap}{count} ")));
    }
    Line::from(spans)
}

pub fn legend() -> Vec<Line<'static>> {
    [
        (State::Working, "working: thinking, or running a command"),
        (
            State::Blocked,
            "blocked, waiting on you: the row says what for",
        ),
        (State::Idle, "idle, or exited"),
        (
            State::Unknown,
            "status unknown: not Claude Code, or not written yet",
        ),
    ]
    .into_iter()
    .map(|(state, what)| {
        Line::from(vec![
            Span::styled(state.glyph(), state.style()),
            Span::from(format!(" {what}")),
        ])
    })
    .collect()
}
