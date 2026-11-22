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

pub const ACTIONS: [(Action, &str, char); 16] = [
    (Action::Zoom, "zoom", 'z'),
    (Action::Side, "side", '\\'),
    (Action::SwapLeft, "swap_left", '{'),
    (Action::SwapRight, "swap_right", '}'),
    (Action::Find, "find", 's'),
    (Action::New, "new", 'n'),
    (Action::End, "end", 'x'),
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
    pub fn is_prefix(&self, key: KeyEvent) -> bool {
        let KeyCode::Char(c) = key.code else {
            return false;
        };
        key.modifiers - KeyModifiers::SHIFT == KeyModifiers::CONTROL
            && same_ctrl(self.prefix).contains(&c.to_ascii_lowercase())
    }

    pub fn prefix_name(&self) -> String {
        format!("C-{}", self.prefix)
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
            .find(|(_, key)| key.is_some_and(|key| key.eq_ignore_ascii_case(&c)))
            .map(|(action, _)| *action)
    }
}

fn same_ctrl(c: char) -> &'static [char] {
    match c {
        '/' | '_' => &['/', '_'],
        _ => &[],
    }
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
