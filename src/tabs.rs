use crate::icons::{self, ClaudeIcon};
use crate::text;

const CLAUDE: &str = "claude";
const SEPARATORS: usize = " ".len() + " | ".len() + " ".len();

pub fn strip(editor: &str, directory: Option<&str>, width: usize) -> [String; 2] {
    let room = width.saturating_sub(SEPARATORS);
    let left = room.saturating_sub(text::columns(CLAUDE));
    if let Some(directory) = directory
        && let Some(named) = with_directory(editor, directory, left)
    {
        return [CLAUDE.to_string(), named];
    }
    if left < 2 {
        return [String::new(), String::new()];
    }
    [CLAUDE.to_string(), text::fit(editor, left)]
}

pub fn claude(icon: Option<ClaudeIcon>) -> String {
    match icon {
        Some(icon) => format!("{} {CLAUDE}", icon.glyph()),
        None => CLAUDE.to_string(),
    }
}

pub fn editor_named(editor: &str, neovim: bool, directory: Option<&str>, width: usize) -> String {
    let named = glyphed(editor, neovim);
    if let Some(directory) = directory
        && let Some(named) = with_directory(&named, directory, width)
    {
        return named;
    }
    if text::columns(&named) <= width {
        return named;
    }
    if neovim {
        return editor_named(editor, false, directory, width);
    }
    if width < 2 {
        return String::new();
    }
    text::fit(editor, width)
}

pub fn iconed(
    icon: ClaudeIcon,
    editor: &str,
    neovim: bool,
    directory: Option<&str>,
    width: usize,
) -> [String; 2] {
    let claude = claude(Some(icon));
    let room = width.saturating_sub(SEPARATORS);
    let left = room.saturating_sub(text::columns(&claude));
    let named = glyphed(editor, neovim);
    if let Some(directory) = directory
        && let Some(named) = with_directory(&named, directory, left)
    {
        return [claude, named];
    }
    if text::columns(&named) <= left {
        return [claude, named];
    }
    strip(editor, directory, width)
}

fn glyphed(editor: &str, neovim: bool) -> String {
    if neovim {
        format!("{} {editor}", icons::NEOVIM)
    } else {
        editor.to_string()
    }
}

fn with_directory(editor: &str, directory: &str, left: usize) -> Option<String> {
    let named = format!("{editor}: ");
    let whole = format!("{named}{directory}");
    if text::columns(&whole) <= left {
        return Some(whole);
    }
    let budget = left.saturating_sub(text::columns(&named));
    (budget >= 2).then(|| format!("{named}…{}", tail(directory, budget - 1)))
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
