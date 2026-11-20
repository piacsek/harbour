use crate::text;

const CLAUDE: &str = "claude";
const SEPARATORS: usize = " ".len() + " | ".len() + " ".len();

pub fn strip(editor: &str, directory: Option<&str>, width: usize) -> [String; 2] {
    let room = width.saturating_sub(SEPARATORS);
    let left = room.saturating_sub(text::columns(CLAUDE));
    if let Some(directory) = directory {
        let named = format!("{editor}: ");
        let whole = format!("{named}{directory}");
        if text::columns(&whole) <= left {
            return [CLAUDE.to_string(), whole];
        }
        let budget = left.saturating_sub(text::columns(&named));
        if budget >= 2 {
            return [
                CLAUDE.to_string(),
                format!("{named}…{}", tail(directory, budget - 1)),
            ];
        }
    }
    if left < 2 {
        return [String::new(), String::new()];
    }
    [CLAUDE.to_string(), text::fit(editor, left)]
}

fn tail(directory: &str, width: usize) -> &str {
    let fits = |start: &usize| text::columns(&directory[*start..]) <= width;
    let start = directory
        .match_indices('/')
        .map(|(at, _)| at)
        .find(fits)
        .or_else(|| directory.char_indices().map(|(at, _)| at).find(fits))
        .unwrap_or(directory.len());
    &directory[start..]
}
