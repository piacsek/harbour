use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::time::Duration;

use crate::icons;
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
enum Kind {
    Repository,
    Branch,
    PullRequest,
    Subagents,
    Editor,
}

impl Kind {
    fn glyph(self) -> &'static str {
        match self {
            Kind::Repository => icons::REPOSITORY,
            Kind::Branch => icons::BRANCH,
            Kind::PullRequest => icons::PULL_REQUEST,
            Kind::Subagents => icons::SUBAGENTS,
            Kind::Editor => icons::NEOVIM,
        }
    }

    fn style(self, colors: &Colors) -> Style {
        match self {
            Kind::Repository => Style::default().fg(colors.repository),
            Kind::Branch => Style::default().fg(colors.branch),
            Kind::PullRequest => Style::default().fg(colors.pull_request),
            Kind::Subagents => Style::default().add_modifier(Modifier::DIM),
            Kind::Editor => Style::default(),
        }
    }
}

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
        }
    }

    pub fn with_icons(mut self, editor: bool) -> Card {
        if editor && self.state != State::Exited {
            self.parts.push((Kind::Editor, String::new()));
        }
        self.iconed = true;
        self
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
        let segments = self.segments(colors);
        let whole: String = segments.iter().map(|(text, _)| text.as_str()).collect();
        let shown = if self.iconed {
            fit_parts(&self.parts, room)
        } else {
            fit(&whole, room)
        };
        let mut third = vec![Span::from(INDENT)];
        third.extend(cut(&segments, &whole, &shown));
        [first, second, Line::from(third)]
    }

    fn segments(&self, colors: &Colors) -> Vec<(String, Style)> {
        let mut segments = Vec::new();
        for (kind, text) in &self.parts {
            if !segments.is_empty() {
                let (gap, style) = if self.iconed {
                    (" ", Style::default())
                } else {
                    (" · ", Style::default().add_modifier(Modifier::DIM))
                };
                segments.push((gap.to_string(), style));
            }
            let shown = match (self.iconed, kind) {
                (true, _) => format!("{} {text}", kind.glyph()),
                (false, Kind::Subagents) if text == "1" => "1 subagent".to_string(),
                (false, Kind::Subagents) => format!("{text} subagents"),
                (false, _) => text.clone(),
            };
            segments.push((shown, kind.style(colors)));
        }
        segments
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
        self.scroll(cards.len(), selected, height);
        let width = usize::from(area.width.saturating_sub(2));
        for (index, card) in cards.iter().enumerate().skip(self.offset) {
            let top = (index - self.offset) * STEP;
            if top >= height {
                break;
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

pub fn age(elapsed: Duration) -> String {
    let minutes = elapsed.as_secs() / 60;
    match minutes {
        0 => "<1m".to_string(),
        1..60 => format!("{minutes}m"),
        60..1440 => format!("{}h", minutes / 60),
        _ => format!("{}d", minutes / 1440),
    }
}
