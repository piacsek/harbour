use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::time::Duration;

use crate::registry::Record;
use crate::repo::Place;
use crate::state::State;
use crate::text::{clean, columns, fit};
use crate::transcript::Facts;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

const HEIGHT: usize = 3;
const STEP: usize = HEIGHT + 1;
const INDENT: &str = "  ";

pub enum Ended {
    Exited(String),
    Stopped(&'static str),
}

pub struct Card {
    state: State,
    title: String,
    doing: Option<(String, Style)>,
    moves: bool,
    place: String,
    age: Option<String>,
}

impl Card {
    pub fn new(
        title: &str,
        ended: Option<Ended>,
        record: Option<&Record>,
        place: &Place,
        elapsed: Option<Duration>,
        facts: Option<&Facts>,
    ) -> Card {
        let state = State::of(ended.is_some(), record);
        let fact = |pick: fn(&Facts) -> Option<&String>| {
            facts
                .and_then(pick)
                .map(|text| clean(text))
                .filter(|text| !text.is_empty())
        };
        let doing = match (ended, state) {
            (Some(Ended::Exited(note)), _) => Some((format!("exited{note}"), Style::default())),
            (Some(Ended::Stopped(why)), _) => Some((
                why.to_string(),
                Style::default().add_modifier(Modifier::DIM),
            )),
            (None, State::Blocked) => record
                .and_then(|record| record.waiting_for.as_deref())
                .map(|reason| (clean(reason), state.style())),
            (None, State::Working | State::Idle) => fact(|facts| facts.prompt.as_ref())
                .map(|prompt| (prompt, Style::default().add_modifier(Modifier::DIM))),
            _ => None,
        };
        let shown = Place {
            repository: place.repository.clone(),
            branch: facts
                .and_then(|facts| facts.branch.clone())
                .or_else(|| place.branch.clone()),
        };
        let pull_request = facts
            .and_then(|facts| facts.pull_request)
            .map(|number| format!(" · #{number}"))
            .unwrap_or_default();
        let subagents = match facts
            .filter(|_| state != State::Exited)
            .map_or(0, |facts| facts.subagents)
        {
            0 => String::new(),
            1 => " · 1 subagent".to_string(),
            many => format!(" · {many} subagents"),
        };
        Card {
            state,
            title: fact(|facts| facts.title.as_ref()).unwrap_or_else(|| clean(title)),
            doing,
            moves: matches!(state, State::Working | State::Idle),
            place: clean(&format!("{shown}{pull_request}{subagents}")),
            age: elapsed.map(age),
        }
    }

    fn lines(&self, width: usize, selected: bool) -> [Line<'static>; HEIGHT] {
        let room = width.saturating_sub(INDENT.len());
        let first = self.first_line(width);
        let second = match self.doing.as_ref().filter(|_| !(selected && self.moves)) {
            Some((text, style)) => Line::from(vec![
                Span::from(INDENT),
                Span::styled(fit(text, room), *style),
            ]),
            None => Line::default(),
        };
        let third = Line::from(vec![Span::from(INDENT), Span::from(fit(&self.place, room))]);
        [first, second, third]
    }

    fn first_line(&self, width: usize) -> Line<'static> {
        let glyph = Span::styled(self.state.glyph(), self.state.style());
        let room = width.saturating_sub(INDENT.len());
        match &self.age {
            Some(age) if room > columns(age) + 1 => {
                let title = fit(&self.title, room.saturating_sub(columns(age) + 1));
                let gap = room.saturating_sub(columns(&title) + columns(age));
                Line::from(vec![
                    glyph,
                    Span::from(format!(" {title}{}{age}", " ".repeat(gap))),
                ])
            }
            _ => Line::from(vec![
                glyph,
                Span::from(format!(" {}", fit(&self.title, room))),
            ]),
        }
    }
}

#[derive(Default)]
pub struct Sidebar {
    offset: usize,
}

impl Sidebar {
    pub fn draw(&mut self, cards: &[Card], selected: usize, area: Rect, buffer: &mut Buffer) {
        let height = usize::from(area.height);
        self.scroll(cards.len(), selected, height);
        let width = usize::from(area.width.saturating_sub(2));
        for (index, card) in cards.iter().enumerate().skip(self.offset) {
            let top = (index - self.offset) * STEP;
            if top >= height {
                break;
            }
            let chosen = index == selected;
            for (line_index, line) in card.lines(width, chosen).iter().enumerate() {
                let Some(row) = u16::try_from(top + line_index)
                    .ok()
                    .filter(|row| *row < area.height)
                else {
                    break;
                };
                let y = area.y + row;
                if chosen {
                    buffer.set_string(area.x, y, "▌", Style::default().fg(Color::Cyan));
                }
                buffer.set_line(area.x + 2, y, line, area.width.saturating_sub(2));
                if chosen {
                    buffer.set_style(Rect::new(area.x, y, area.width, 1), Modifier::BOLD);
                }
            }
        }
    }

    fn scroll(&mut self, count: usize, selected: usize, height: usize) {
        let above = height.saturating_sub(HEIGHT) / STEP;
        self.offset = self
            .offset
            .min(selected)
            .max(selected.saturating_sub(above));
        while self.offset > 0 && (count - self.offset + 1) * STEP - 1 <= height {
            self.offset -= 1;
        }
    }
}

pub fn age(elapsed: Duration) -> String {
    let minutes = elapsed.as_secs() / 60;
    match minutes {
        0 => "<1m".to_string(),
        1..60 => format!("{minutes}m"),
        60..1440 => format!("{}h", minutes / 60),
        _ => format!("{}d", minutes / 1440),
    }
}
