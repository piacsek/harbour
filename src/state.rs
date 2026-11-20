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
