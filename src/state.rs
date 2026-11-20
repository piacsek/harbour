use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

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
            State::Idle | State::Exited | State::Unknown => "○",
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

pub fn row(name: &str, exit_note: Option<String>, record: Option<&Record>) -> Line<'static> {
    let state = State::of(exit_note.is_some(), record);
    let mut spans = vec![
        Span::styled(state.glyph(), state.style()),
        Span::from(format!(" {name}")),
    ];
    match (
        exit_note,
        record.and_then(|record| record.waiting_for.as_ref()),
    ) {
        (Some(note), _) => spans.push(Span::from(format!(" exited{note}"))),
        (None, Some(reason)) if state == State::Blocked => {
            spans.push(Span::from(" "));
            spans.push(Span::styled(reason.clone(), state.style()));
        }
        _ => {}
    }
    Line::from(spans)
}

pub fn title<'a>(sessions: impl Iterator<Item = (bool, Option<&'a Record>)>) -> Line<'static> {
    let mut counts = [0; 3];
    for (exited, record) in sessions {
        counts[match State::of(exited, record) {
            State::Working => 0,
            State::Blocked => 1,
            State::Idle | State::Exited | State::Unknown => 2,
        }] += 1;
    }
    Line::from(vec![
        Span::from(" sessions "),
        Span::styled("●", State::Working.style()),
        Span::from(format!(" {} ", counts[0])),
        Span::styled("◉", State::Blocked.style()),
        Span::from(format!(" {} ", counts[1])),
        Span::from("○"),
        Span::from(format!(" {} ", counts[2])),
    ])
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
