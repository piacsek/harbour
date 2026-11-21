use ratatui::text::Span;

use crate::state::State;
use crate::text::{columns, fit};

pub struct Headline {
    pub state: State,
    pub title: String,
    pub place: String,
    pub age: Option<String>,
}

pub fn info(headline: &Headline, room: usize) -> Vec<Span<'static>> {
    if room < 3 {
        return Vec::new();
    }
    let dot = Span::styled(headline.state.glyph(), headline.state.style());
    let body_room = room.saturating_sub(4);
    if body_room == 0 {
        return vec![Span::from(" "), dot, Span::from(" ")];
    }
    let body = match &headline.age {
        Some(age) if body_room > columns(age) + 2 => {
            let named = named(headline, body_room - columns(age) - 2);
            format!("{named}  {age}")
        }
        _ => named(headline, body_room),
    };
    vec![Span::from(" "), dot, Span::from(format!(" {body} "))]
}

fn named(headline: &Headline, room: usize) -> String {
    if headline.title.is_empty() {
        return fit(&headline.place, room);
    }
    let title = fit(&headline.title, room);
    let left = room.saturating_sub(columns(&title) + 2);
    if title != headline.title || left < 2 || headline.place.is_empty() {
        return title;
    }
    format!("{title}  {}", fit(&headline.place, left))
}
