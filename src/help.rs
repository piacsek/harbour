pub const KEYS: [(&str, &str); 7] = [
    ("j/k", "select the next / previous session"),
    ("n", "start a session"),
    ("C-l/Enter", "focus the pane"),
    ("C-h", "focus the sidebar, from the pane"),
    ("x", "end the session, or remove it once it has exited"),
    ("?", "show or hide the help"),
    ("q", "quit, after a yes while a session runs"),
];

pub const NOTES: [&str; 3] = [
    "C-Backspace arrives as C-h on most terminals, so in the pane it focuses the sidebar instead of deleting.",
    "Inside tmux with vim-tmux-navigator, add harbour to the pattern of processes it passes C-h and C-l to.",
    "Under tmux, Shift-Enter arrives as Enter: use Alt-Enter for a newline.",
];
