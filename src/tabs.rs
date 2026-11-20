use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const CLAUDE: &str = "claude";
const SEPARATORS: usize = " ".len() + " | ".len() + " ".len();

pub fn strip(editor: &str, directory: Option<&str>, width: usize) -> [String; 2] {
    let room = width.saturating_sub(SEPARATORS);
    let left = room.saturating_sub(CLAUDE.width());
    if let Some(directory) = directory {
        let named = format!("{editor}: ");
        let whole = format!("{named}{directory}");
        if whole.width() <= left {
            return [CLAUDE.to_string(), whole];
        }
        let budget = left.saturating_sub(named.width());
        if budget >= 2 {
            return [
                CLAUDE.to_string(),
                format!("{named}…{}", tail(directory, budget - 1)),
            ];
        }
    }
    let editor = head(editor, left);
    let claude = head(CLAUDE, room.saturating_sub(editor.width()));
    [claude, editor]
}

fn head(text: &str, width: usize) -> String {
    let mut used = 0;
    text.chars()
        .take_while(|c| {
            used += c.width().unwrap_or(0);
            used <= width
        })
        .collect()
}

fn tail(text: &str, width: usize) -> &str {
    let fits = |start: &usize| text[*start..].width() <= width;
    let start = text
        .match_indices('/')
        .map(|(at, _)| at)
        .find(fits)
        .or_else(|| text.char_indices().map(|(at, _)| at).find(fits))
        .unwrap_or(text.len());
    &text[start..]
}
