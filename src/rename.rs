use unicode_segmentation::UnicodeSegmentation;

use crate::text::columns;

const LABEL: &str = "Title: ";
const LABELLED: usize = 17;
const ELLIPSIS: &str = "…";

pub fn line(text: &str, width: usize) -> (&'static str, String) {
    let label = if width >= LABELLED { LABEL } else { "" };
    let room = width.saturating_sub(columns(label) + 1);
    (label, end_of(text, room))
}

fn end_of(text: &str, room: usize) -> String {
    if columns(text) <= room {
        return text.to_string();
    }
    let budget = room.saturating_sub(columns(ELLIPSIS));
    if room == 0 {
        return String::new();
    }
    let mut used = 0;
    let mut kept = Vec::new();
    for grapheme in text.graphemes(true).rev() {
        used += columns(grapheme);
        if used > budget {
            break;
        }
        kept.push(grapheme);
    }
    kept.reverse();
    format!("{ELLIPSIS}{}", kept.concat())
}
