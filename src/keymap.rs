use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Zoom,
    Side,
    SwapLeft,
    SwapRight,
    Find,
    New,
    End,
    Close,
    Editor,
    Sidebar,
    Pane,
    Narrower,
    Wider,
    Usage,
    Changelog,
    Help,
    Quit,
}

pub const ACTIONS: [(Action, &str, char); 17] = [
    (Action::Zoom, "zoom", 'z'),
    (Action::Side, "side", '\\'),
    (Action::SwapLeft, "swap_left", '{'),
    (Action::SwapRight, "swap_right", '}'),
    (Action::Find, "find", 's'),
    (Action::New, "new", 'n'),
    (Action::End, "end", 'x'),
    (Action::Close, "close", 'X'),
    (Action::Editor, "editor", 'e'),
    (Action::Sidebar, "sidebar", 'h'),
    (Action::Pane, "pane", 'l'),
    (Action::Narrower, "narrower", '-'),
    (Action::Wider, "wider", '='),
    (Action::Usage, "usage", 'u'),
    (Action::Changelog, "changelog", 'c'),
    (Action::Help, "help", '?'),
    (Action::Quit, "quit", 'q'),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum After {
    Jump(usize),
    Act(Action),
    Prefix,
    Cancel,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keymap {
    prefix: char,
    bound: Vec<(Action, Option<char>)>,
}

impl Default for Keymap {
    fn default() -> Keymap {
        Keymap {
            prefix: '/',
            bound: ACTIONS
                .iter()
                .map(|(action, _, key)| (*action, Some(*key)))
                .collect(),
        }
    }
}

impl Keymap {
    pub fn with_prefix(mut self, prefix: char) -> Keymap {
        self.prefix = prefix;
        self
    }

    pub fn bind(mut self, action: Action, key: Option<char>) -> Keymap {
        for (each, bound) in &mut self.bound {
            if *each == action {
                *bound = key;
            }
        }
        self
    }

    pub fn key_of(&self, action: Action) -> Option<char> {
        self.bound
            .iter()
            .find(|(each, _)| *each == action)
            .and_then(|(_, key)| *key)
    }

    pub fn clash(&self, action: Action) -> Option<Action> {
        let key = self.key_of(action)?;
        self.bound
            .iter()
            .find(|(each, other)| *each != action && *other == Some(key))
            .map(|(each, _)| *each)
    }

    pub fn is_prefix(&self, key: KeyEvent) -> bool {
        let KeyCode::Char(c) = key.code else {
            return false;
        };
        key.modifiers - KeyModifiers::SHIFT == KeyModifiers::CONTROL
            && same_ctrl(self.prefix, c.to_ascii_lowercase())
    }

    pub fn prefix_name(&self) -> String {
        prefix_name(self.prefix)
    }

    pub fn prefix_key(&self) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(self.prefix), KeyModifiers::CONTROL)
    }

    pub fn after(&self, key: KeyEvent) -> After {
        if self.is_prefix(key) {
            return After::Prefix;
        }
        if key.code == KeyCode::Esc && key.modifiers.is_empty() {
            return After::Cancel;
        }
        let KeyCode::Char(c) = key.code else {
            return After::Unknown;
        };
        if !(key.modifiers - KeyModifiers::SHIFT).is_empty() {
            return After::Unknown;
        }
        if let Some(digit @ 1..=9) = c.to_digit(10) {
            return After::Jump(usize::try_from(digit).unwrap_or(9));
        }
        self.action_on(c).map_or(After::Unknown, After::Act)
    }

    pub fn action_on(&self, c: char) -> Option<Action> {
        self.bound
            .iter()
            .find(|(_, key)| *key == Some(c))
            .or_else(|| {
                self.bound.iter().find(|(action, key)| {
                    *action != Action::Close && key.is_some_and(|key| key.eq_ignore_ascii_case(&c))
                })
            })
            .map(|(action, _)| *action)
    }
}

fn same_ctrl(prefix: char, typed: char) -> bool {
    match prefix {
        '/' | '_' => typed == '/' || typed == '_',
        prefix => typed == prefix,
    }
}

pub const PREFIX: &str = "expected C-<letter>, C-Space, C-/, C-\\, C-], C-^, C-_ or C-;";
pub const KEY: &str = "expected one printable character other than a space, or \"none\"";

pub fn prefix_name(prefix: char) -> String {
    match prefix {
        ' ' => "C-Space".to_string(),
        prefix => format!("C-{prefix}"),
    }
}

pub fn parse_prefix(name: &str) -> Result<char, String> {
    let Some(rest) = name.strip_prefix("C-") else {
        return Err(PREFIX.to_string());
    };
    if rest.eq_ignore_ascii_case("space") {
        return Ok(' ');
    }
    let mut chars = rest.chars();
    let (Some(c), None) = (chars.next(), chars.next()) else {
        return Err(PREFIX.to_string());
    };
    match c.to_ascii_lowercase() {
        'h' | 'i' | 'j' | 'k' | 'l' | 'm' => Err(format!(
            "{name:?} is taken: C-h, C-j, C-k and C-l are harbour's, C-m is Enter, C-i is Tab"
        )),
        c if c.is_ascii_lowercase() || matches!(c, '/' | '\\' | ']' | '^' | '_' | ';') => Ok(c),
        _ => Err(PREFIX.to_string()),
    }
}

pub fn parse_key(value: &str) -> Result<Option<char>, String> {
    if value == "none" {
        return Ok(None);
    }
    let mut chars = value.chars();
    let (Some(c), None) = (chars.next(), chars.next()) else {
        return Err(KEY.to_string());
    };
    if c.is_ascii_digit() {
        return Err(format!(
            "{value:?} is a digit, and the digits are the jumps"
        ));
    }
    if c.is_whitespace() || c.is_control() {
        return Err(KEY.to_string());
    }
    Ok(Some(c))
}

pub fn action_named(name: &str) -> Option<Action> {
    ACTIONS
        .iter()
        .find(|(_, each, _)| *each == name)
        .map(|(action, _, _)| *action)
}

pub fn key_name(key: KeyEvent) -> String {
    let base = match key.code {
        KeyCode::Char(' ') => "Space".to_string(),
        KeyCode::Char(c) => c.to_string(),
        KeyCode::F(n) => format!("F{n}"),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Tab | KeyCode::BackTab => "Tab".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        _ => "key".to_string(),
    };
    let mut name = String::new();
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        name.push_str("C-");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        name.push_str("M-");
    }
    name + &base
}

pub fn name(action: Action) -> &'static str {
    ACTIONS
        .iter()
        .find(|(each, _, _)| *each == action)
        .map_or("", |(_, name, _)| name)
}
