pub const KEYS: &[(&str, &str)] = &[
    ("j/k", "select the next / previous session"),
    ("n", "pick a directory and start a session there"),
    (
        "C-l/Enter",
        "focus the pane or resume; from Claude Code, C-l opens its editor",
    ),
    ("C-h", "focus the sidebar; from an editor, Claude Code"),
    ("C-1..C-9", "focus or resume that card; C-9 the last"),
    ("e", "show the session's next editor, one per directory"),
    (
        "x",
        "end the session and its editors, or forget it once it has ended",
    ),
    ("u", "show Claude Code's usage and limits"),
    ("?", "show or hide the help"),
    ("q", "quit, after a yes while anything runs"),
];

pub const REST: &str = "harbour --help has the rest.";
