use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::time::Duration;

use crate::facts::{self, Colors, Kind};
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
    parts: Vec<(Kind, String)>,
    iconed: bool,
    age: Option<String>,
    repository: String,
    header: bool,
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
            (Some(Ended::Exited(note)), _) => {
                let style = if note.is_empty() {
                    Style::default().add_modifier(Modifier::DIM)
                } else {
                    Style::default().fg(Color::Red)
                };
                Some((format!("exited{note}"), style))
            }
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
        let number = facts.and_then(|facts| facts.pull_request);
        let count = facts
            .filter(|_| state != State::Exited)
            .map_or(0, |facts| facts.subagents);
        let parts = [
            Some((Kind::Repository, clean(&shown.repository))),
            shown
                .branch
                .as_ref()
                .map(|branch| (Kind::Branch, clean(branch))),
            number.map(|number| (Kind::PullRequest, format!("#{number}"))),
            (count > 0).then(|| (Kind::Subagents, count.to_string())),
        ]
        .into_iter()
        .flatten()
        .collect();
        Card {
            state,
            title: fact(|facts| facts.title.as_ref()).unwrap_or_else(|| clean(title)),
            doing,
            moves: matches!(state, State::Working | State::Idle),
            parts,
            iconed: false,
            age: elapsed.map(age),
            repository: clean(&place.repository),
            header: false,
        }
    }

    pub fn renamed(mut self, given: Option<&str>) -> Card {
        if let Some(given) = given.map(clean).filter(|given| !given.is_empty()) {
            self.title = given;
        }
        self
    }

    pub fn with_icons(mut self) -> Card {
        self.iconed = true;
        self
    }

    pub fn in_group(mut self, first: bool) -> Card {
        self.parts.retain(|(kind, _)| *kind != Kind::Repository);
        self.header = first;
        self
    }

    fn header_line(&self, room: usize) -> Line<'static> {
        let name = fit(&self.repository, room);
        let mut text = name.clone();
        if room > columns(&name) + 1 {
            text.push(' ');
            text.push_str(&"─".repeat(room - columns(&name) - 1));
        }
        Line::styled(text, Modifier::DIM)
    }

    fn lines(&self, width: usize, selected: bool, colors: &Colors) -> [Line<'static>; HEIGHT] {
        let room = width.saturating_sub(INDENT.len());
        let first = self.first_line(width);
        let second = match self.doing.as_ref().filter(|_| !(selected && self.moves)) {
            Some((text, style)) => Line::from(vec![
                Span::from(INDENT),
                Span::styled(fit(text, room), *style),
            ]),
            None => Line::default(),
        };
        let mut third = vec![Span::from(INDENT)];
        third.extend(facts::fitted(&self.parts, self.iconed, colors, room));
        [first, second, Line::from(third)]
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
                    Span::from(format!(" {title}{}", " ".repeat(gap))),
                    Span::styled(age.clone(), Modifier::DIM),
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
    colors: Colors,
}

impl Sidebar {
    pub fn with_colors(colors: Colors) -> Sidebar {
        Sidebar { offset: 0, colors }
    }

    pub fn draw(
        &mut self,
        cards: &[Card],
        selected: usize,
        accent: Color,
        area: Rect,
        buffer: &mut Buffer,
    ) {
        if area.height == 0 {
            return;
        }
        let height = usize::from(area.height);
        let items = items(cards);
        self.scroll(&items, selected, height);
        let width = usize::from(area.width.saturating_sub(2));
        let first = items.get(self.offset).map_or(0, |item| item.top);
        for item in items.iter().skip(self.offset) {
            let top = item.top - first;
            if top >= height {
                break;
            }
            let index = item.card;
            let card = &cards[index];
            if item.header {
                if let Some(row) = u16::try_from(top).ok().filter(|_| top + 1 < height) {
                    buffer.set_line(
                        area.x + 2,
                        area.y + row,
                        &card.header_line(width),
                        area.width.saturating_sub(2),
                    );
                }
                continue;
            }
            let chosen = index == selected;
            for (line_index, line) in card.lines(width, chosen, &self.colors).iter().enumerate() {
                let Some(row) = u16::try_from(top + line_index)
                    .ok()
                    .filter(|row| *row < area.height)
                else {
                    break;
                };
                let y = area.y + row;
                if chosen && area.width > 0 {
                    buffer.set_string(area.x, y, "▌", Style::default().fg(accent));
                }
                buffer.set_line(area.x + 2, y, line, area.width.saturating_sub(2));
                if chosen {
                    buffer.set_style(Rect::new(area.x, y, area.width, 1), Modifier::BOLD);
                } else if card.state == State::Exited {
                    buffer.set_style(Rect::new(area.x, y, area.width, 1), Modifier::DIM);
                }
            }
        }
    }

    fn scroll(&mut self, items: &[Item], selected: usize, height: usize) {
        let Some(shown) = items
            .iter()
            .position(|item| item.card == selected && !item.header)
        else {
            self.offset = 0;
            return;
        };
        let start = if shown > 0 && items[shown - 1].header {
            shown - 1
        } else {
            shown
        };
        let bottom = items[shown].top + HEIGHT;
        self.offset = self.offset.min(start);
        while self.offset < shown && bottom - items[self.offset].top > height {
            self.offset += 1;
        }
        let end = items.last().map_or(0, |item| item.top + HEIGHT);
        while self.offset > 0 && end - items[self.offset - 1].top <= height {
            self.offset -= 1;
        }
    }
}

struct Item {
    top: usize,
    card: usize,
    header: bool,
}

fn items(cards: &[Card]) -> Vec<Item> {
    let mut items = Vec::new();
    let mut top = 0;
    for (card, shown) in cards.iter().enumerate() {
        if shown.header {
            items.push(Item {
                top,
                card,
                header: true,
            });
            top += 1;
        }
        items.push(Item {
            top,
            card,
            header: false,
        });
        top += STEP;
    }
    items
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
