use ratatui::buffer::CellWidth;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const ELLIPSIS: &str = "…";
const LONGEST: usize = 512;
const JOINER: char = '\u{200d}';

pub fn columns(text: &str) -> usize {
    text.graphemes(true)
        .filter(|grapheme| !grapheme.contains(char::is_control))
        .map(|grapheme| usize::from(grapheme.cell_width()))
        .sum()
}

pub fn fit(text: &str, width: usize) -> String {
    if columns(text) <= width {
        return text.to_string();
    }
    if width == 0 {
        return String::new();
    }
    let budget = width.saturating_sub(columns(ELLIPSIS));
    let mut kept = String::new();
    for grapheme in text.graphemes(true) {
        let length = kept.len();
        kept.push_str(grapheme);
        if columns(&kept) > budget {
            kept.truncate(length);
            break;
        }
    }
    kept.push_str(ELLIPSIS);
    kept
}

pub fn wrap(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut lines: Vec<String> = Vec::new();
    if text.is_empty() {
        return lines;
    }
    for word in text.split(' ') {
        if let Some(line) = lines.last_mut()
            && columns(line) + 1 + columns(word) <= width
        {
            line.push(' ');
            line.push_str(word);
            continue;
        }
        let mut line = String::new();
        for grapheme in word.graphemes(true) {
            let length = line.len();
            line.push_str(grapheme);
            if columns(&line) > width && length > 0 {
                line.truncate(length);
                lines.push(std::mem::take(&mut line));
                line.push_str(grapheme);
            }
        }
        lines.push(line);
    }
    lines
}

pub fn clean(text: &str) -> String {
    let mut out = String::new();
    let mut count = 0;
    let mut gap = false;
    for grapheme in text.graphemes(true) {
        let shown = visible(grapheme);
        if shown.trim().is_empty() {
            gap = true;
            continue;
        }
        let space = usize::from(gap && count > 0);
        let length = shown.chars().count();
        if count + space + length > LONGEST {
            break;
        }
        if space > 0 {
            out.push(' ');
        }
        gap = false;
        count += space + length;
        out.push_str(&shown);
    }
    out
}

fn visible(grapheme: &str) -> String {
    let chars: Vec<char> = grapheme.chars().collect();
    chars
        .iter()
        .enumerate()
        .filter(|(index, c)| match **c {
            JOINER => chars
                .get(index + 1)
                .is_some_and(|next| next.to_string().width() > 0),
            c => !hidden(c),
        })
        .map(|(_, c)| *c)
        .collect()
}

fn hidden(c: char) -> bool {
    c.is_control()
        || matches!(
            c,
            '\u{61c}'
                | '\u{200b}'
                | '\u{200e}'
                | '\u{200f}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2066}'..='\u{2069}'
        )
}
