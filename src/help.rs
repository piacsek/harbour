use crate::keymap::{Action, Keymap};

pub const KEYS: &[(&str, &str)] = &[
    ("j/k J/K", "select / move the card down / up"),
    ("Enter/C-l", "focus or resume; C-l in Claude Code: editor"),
    ("C-h", "focus the sidebar; from an editor, Claude Code"),
    ("n/x/X", "start / end or forget / end & forget, C-/ too"),
    ("C-/ 1..9", "focus or resume that card; 9 the last"),
    ("C-/ z", "zoom the session to the whole screen, or back"),
    ("C-/ \\ {}", "side by side (120+ columns); swap the halves"),
    ("C-/ s/r", "jump to a session by name / rename the card"),
    ("C-/ e", "show the next editor, one per directory"),
    ("C-/ h/l", "focus the sidebar / the pane"),
    ("C-/ -/=", "narrow / widen the sidebar by 2 columns"),
    ("C-/ u/c", "Claude Code's usage / the changelog"),
    ("C-/ ?/q", "this help / quit, asking while anything runs"),
    (
        "C-/ C-/ p",
        "the previous session / send C-/ to the program",
    ),
];

pub fn rows(keymap: &Keymap) -> Vec<(String, String)> {
    let prefix = keymap.prefix_name().replace("Space", "\u{2423}");
    let key = |action| keymap.key_of(action);
    let mut rows = vec![owned(KEYS[0]), owned(KEYS[1]), owned(KEYS[2])];
    rows.extend(sessions(keymap).map(|(keys, what)| (keys, format!("{what}, {prefix} too"))));
    rows.push((format!("{prefix} 1..9"), KEYS[4].1.to_string()));
    rows.extend(single(&prefix, key(Action::Zoom), KEYS[5].1));
    rows.extend(side(&prefix, keymap));
    rows.extend(pair(
        &prefix,
        [key(Action::Find), key(Action::Rename)],
        [
            KEYS[7].1,
            "jump to a session by part of its name",
            "rename the selected card",
        ],
    ));
    rows.extend(single(&prefix, key(Action::Editor), KEYS[8].1));
    rows.extend(pair(
        &prefix,
        [key(Action::Sidebar), key(Action::Pane)],
        [KEYS[9].1, "focus the sidebar", "focus the pane"],
    ));
    rows.extend(pair(
        &prefix,
        [key(Action::Narrower), key(Action::Wider)],
        [
            KEYS[10].1,
            "narrow the sidebar by 2 columns",
            "widen the sidebar by 2 columns",
        ],
    ));
    rows.extend(pair(
        &prefix,
        [key(Action::Usage), key(Action::Changelog)],
        [KEYS[11].1, "Claude Code's usage", "the changelog"],
    ));
    rows.extend(pair(
        &prefix,
        [key(Action::Help), key(Action::Quit)],
        [KEYS[12].1, "this help", "quit, asking while anything runs"],
    ));
    rows.push(match key(Action::SendPrefix) {
        Some(send) => (
            format!("{prefix} {prefix} {send}"),
            format!("the previous session / send {prefix} to the program"),
        ),
        None => (
            format!("{prefix} {prefix}"),
            "the previous session".to_string(),
        ),
    });
    rows
}

fn sessions(keymap: &Keymap) -> Option<(String, String)> {
    let bound: Vec<(char, &str)> = [
        (Action::New, "start"),
        (Action::End, "end or forget"),
        (Action::Close, "end & forget"),
    ]
    .into_iter()
    .filter_map(|(action, what)| keymap.key_of(action).map(|key| (key, what)))
    .collect();
    if bound.is_empty() {
        return None;
    }
    let keys: Vec<String> = bound.iter().map(|(key, _)| key.to_string()).collect();
    let what: Vec<&str> = bound.iter().map(|(_, what)| *what).collect();
    let closing = keymap.key_of(Action::Close).is_some();
    let noun = if closing && bound.len() > 1 {
        ""
    } else {
        " a session"
    };
    Some((keys.join("/"), format!("{}{noun}", what.join(" / "))))
}

fn side(prefix: &str, keymap: &Keymap) -> Option<(String, String)> {
    let swaps: String = [Action::SwapLeft, Action::SwapRight]
        .into_iter()
        .filter_map(|action| keymap.key_of(action))
        .collect();
    match (keymap.key_of(Action::Side), swaps.is_empty()) {
        (Some(side), false) => Some((format!("{prefix} {side} {swaps}"), KEYS[6].1.to_string())),
        (Some(side), true) => Some((
            format!("{prefix} {side}"),
            "side by side (120+ columns)".to_string(),
        )),
        (None, false) => Some((format!("{prefix} {swaps}"), "swap the halves".to_string())),
        (None, true) => None,
    }
}

fn single(prefix: &str, key: Option<char>, what: &str) -> Option<(String, String)> {
    key.map(|key| (format!("{prefix} {key}"), what.to_string()))
}

fn pair(
    prefix: &str,
    keys: [Option<char>; 2],
    [both, first, second]: [&str; 3],
) -> Option<(String, String)> {
    let lead = if prefix.is_empty() {
        String::new()
    } else {
        format!("{prefix} ")
    };
    let (keys, what) = match keys {
        [Some(a), Some(b)] => (format!("{a}/{b}"), both),
        [Some(a), None] => (a.to_string(), first),
        [None, Some(b)] => (b.to_string(), second),
        [None, None] => return None,
    };
    Some((format!("{lead}{keys}"), what.to_string()))
}

fn owned((key, what): (&str, &str)) -> (String, String) {
    (key.to_string(), what.to_string())
}
