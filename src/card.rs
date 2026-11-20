use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::registry::Record;
use crate::state::State;

const HEIGHT: usize = 3;
const STEP: usize = HEIGHT + 1;
const INDENT: &str = "  ";

pub struct Card {
    state: State,
    title: String,
    doing: Option<(String, Style)>,
}

impl Card {
    pub fn new(title: &str, exit_note: Option<String>, record: Option<&Record>) -> Card {
        let state = State::of(exit_note.is_some(), record);
        let doing = match (exit_note, state) {
            (Some(note), _) => Some((format!("exited{note}"), Style::default())),
            (None, State::Blocked) => record
                .and_then(|record| record.waiting_for.as_deref())
                .map(|reason| (clean(reason), state.style())),
            _ => None,
        };
        Card {
            state,
            title: clean(title),
            doing,
        }
    }

    fn lines(&self, width: usize) -> [Line<'static>; HEIGHT] {
        let room = width.saturating_sub(INDENT.len());
        let first = Line::from(vec![
            Span::styled(self.state.glyph(), self.state.style()),
            Span::from(format!(" {}", fit(&self.title, room))),
        ]);
        let second = match &self.doing {
            Some((text, style)) => Line::from(vec![
                Span::from(INDENT),
                Span::styled(fit(text, room), *style),
            ]),
            None => Line::default(),
        };
        [first, second, Line::default()]
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
            for (line_index, line) in card.lines(width).iter().enumerate() {
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

pub fn fit(text: &str, width: usize) -> String {
    if text.width() <= width {
        return text.to_string();
    }
    if width == 0 {
        return String::new();
    }
    let budget = width - 1;
    let mut kept = String::new();
    let mut used = 0;
    for character in text.chars() {
        let columns = character.width().unwrap_or(0);
        if used + columns > budget {
            break;
        }
        used += columns;
        kept.push(character);
    }
    format!("{kept}…")
}

pub fn clean(text: &str) -> String {
    text.chars()
        .map(|c| if hidden(c) { ' ' } else { c })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn hidden(c: char) -> bool {
    c.is_control()
        || matches!(c, '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}
