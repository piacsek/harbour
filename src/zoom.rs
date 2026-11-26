use ratatui::style::Modifier;
use ratatui::text::Span;

use crate::facts::{self, Colors, Kind};
use crate::state::State;
use crate::text::{columns, fit};

pub struct Headline {
    pub state: State,
    pub title: String,
    pub repository: String,
    pub branch: Option<String>,
    pub age: Option<String>,
}

impl Headline {
    pub fn place(&self) -> String {
        match &self.branch {
            Some(branch) => format!("{} · {branch}", self.repository),
            None => self.repository.clone(),
        }
    }

    fn parts(&self) -> Vec<(Kind, String)> {
        [
            Some((Kind::Repository, self.repository.clone())),
            self.branch.clone().map(|branch| (Kind::Branch, branch)),
        ]
        .into_iter()
        .flatten()
        .filter(|(_, text)| !text.is_empty())
        .collect()
    }
}

pub fn guaranteed(headline: &Headline) -> usize {
    let aged = headline.age.as_ref().map_or(0, |age| columns(age) + 2);
    4 + columns(&headline.title).min(16) + aged
}

pub fn info(headline: &Headline, room: usize, colors: &Colors, icons: bool) -> Vec<Span<'static>> {
    if room < 3 {
        return Vec::new();
    }
    let mut spans = vec![
        Span::from(" "),
        Span::styled(headline.state.glyph(), headline.state.style()),
        Span::from(" "),
    ];
    let body_room = room.saturating_sub(4);
    if body_room == 0 {
        return spans;
    }
    match &headline.age {
        Some(age) if body_room > columns(age) + 2 => {
            spans.extend(named(headline, body_room - columns(age) - 2, colors, icons));
            spans.push(Span::from("  "));
            spans.push(Span::styled(age.clone(), Modifier::DIM));
        }
        _ => spans.extend(named(headline, body_room, colors, icons)),
    }
    spans.push(Span::from(" "));
    spans
}

fn named(headline: &Headline, room: usize, colors: &Colors, icons: bool) -> Vec<Span<'static>> {
    let parts = headline.parts();
    if headline.title.is_empty() {
        return facts::fitted(&parts, icons, colors, room);
    }
    let title = fit(&headline.title, room);
    let left = room.saturating_sub(columns(&title) + 2);
    if title != headline.title || left < 2 || parts.is_empty() {
        return vec![Span::from(title)];
    }
    let mut spans = vec![Span::from(format!("{title}  "))];
    spans.extend(facts::fitted(&parts, icons, colors, left));
    spans
}
