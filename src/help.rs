pub const KEYS: &[(&str, &str)] = &[
    ("j/k", "select the next / previous session"),
    ("n", "pick a directory and start a session there"),
    (
        "C-l/Enter",
        "focus the pane or resume; from Claude Code, C-l opens its editor",
    ),
    ("C-h", "focus the sidebar; from the editor, Claude Code"),
    ("e", "show the session's next editor, one per directory"),
    (
        "x",
        "end the session and its editors, or forget it once it has ended",
    ),
    ("?", "show or hide the help"),
    ("q", "quit, after a yes while a session or editor runs"),
];

pub const REST: &str = "harbour --help has the rest.";
