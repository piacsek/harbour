pub const KEYS: &[(&str, &str)] = &[
    ("j/k", "select the next / previous session"),
    ("n", "pick a directory and start a session there"),
    (
        "C-l/Enter",
        "focus the pane or resume; from Claude Code, C-l opens its editor",
    ),
    ("C-h", "focus the sidebar; from an editor, Claude Code"),
    ("C-1..C-9", "focus or resume that card; C-9 the last"),
    ("M-z", "zoom the session to the whole screen, or back"),
    ("M--/M-=", "narrow / widen the sidebar by 2 columns"),
    ("e", "show the session's next editor, one per directory"),
    ("x", "end it and its editors, or forget an ended one"),
    ("?", "show or hide the help"),
    ("q", "quit, after a yes while anything runs"),
];

pub const REST: &str = "harbour --help has the rest.";
