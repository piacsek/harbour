use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::registry::{Record, Status};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Working,
    Blocked,
    Idle,
    Exited,
    Unknown,
}

impl State {
    pub fn of(exited: bool, record: Option<&Record>) -> State {
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

    pub fn glyph(self) -> &'static str {
        match self {
            State::Working => "●",
            State::Blocked => "◉",
            State::Idle | State::Exited => "○",
            State::Unknown => "·",
        }
    }

    pub fn style(self) -> Style {
        match self {
            State::Working => Style::default().fg(Color::Yellow),
            State::Blocked => Style::default().fg(Color::Red),
            State::Idle | State::Exited => Style::default().add_modifier(Modifier::DIM),
            State::Unknown => Style::default(),
        }
    }

    pub fn sgr(self) -> Option<&'static str> {
        match self {
            State::Working => Some("33"),
            State::Blocked => Some("31"),
            State::Idle | State::Exited => Some("2"),
            State::Unknown => None,
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

pub fn counts<'a>(sessions: impl Iterator<Item = (bool, Option<&'a Record>)>) -> Counts {
    sessions.fold(Counts::default(), |mut counts, (exited, record)| {
        match State::of(exited, record) {
            State::Working => counts.working += 1,
            State::Blocked => counts.blocked += 1,
            State::Idle | State::Exited => counts.idle += 1,
            State::Unknown => counts.unknown += 1,
        }
        counts
    })
}

pub fn title(counts: Counts, width: usize) -> Line<'static> {
    let full = counted(" sessions ", counts, " ");
    if full.width() <= width {
        full
    } else if counts == Counts::default() {
        Line::default()
    } else {
        counted(" ", counts, "")
    }
}

fn counted(lead: &str, counts: Counts, gap: &str) -> Line<'static> {
    let mut spans = vec![Span::from(lead.to_string())];
    for (glyph, style, count) in [
        ("●", State::Working.style(), counts.working),
        ("◉", State::Blocked.style(), counts.blocked),
        ("○", Style::default(), counts.idle),
        ("·", Style::default(), counts.unknown),
    ] {
        if count > 0 {
            spans.push(Span::styled(glyph, style));
            spans.push(Span::from(format!("{gap}{count} ")));
        }
    }
    Line::from(spans)
}

pub fn legend() -> Vec<Line<'static>> {
    [
        (State::Working, "working: thinking, or running a command"),
        (
            State::Blocked,
            "blocked, waiting on you: the card says what for",
        ),
        (State::Idle, "idle, exited, or stopped"),
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
