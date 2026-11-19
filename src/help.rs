pub const KEYS: [(&str, &str); 7] = [
    ("j/k", "select the next / previous session"),
    ("n", "start a session"),
    ("C-l/Enter", "focus the pane"),
    ("C-h", "focus the sidebar, from the pane"),
    ("x", "end the session, or remove it once it has exited"),
    ("?", "show or hide the help"),
    ("q", "quit, after a yes while a session runs"),
];

pub const TMUX: [&str; 2] = [
    "Inside tmux with vim-tmux-navigator, add harbour to the pattern of processes it passes C-h and C-l to.",
    "Under tmux, Shift-Enter arrives as Enter: use Alt-Enter for a newline.",
];
