pub const KEYS: [(&str, &str); 7] = [
    ("j/k", "select the next / previous session"),
    ("n", "pick a directory and start a session there"),
    (
        "C-l/Enter",
        "focus the pane; from Claude Code, C-l opens its editor",
    ),
    (
        "C-h",
        "focus the sidebar; from the editor, Claude Code (Neovim: leftmost window, normal mode)",
    ),
    (
        "x",
        "end the session and its editors, or remove it once it has exited",
    ),
    ("?", "show or hide the help"),
    ("q", "quit, after a yes while a session or editor runs"),
];

pub const NOTES: [&str; 5] = [
    "n runs fzf over the current directory and zoxide's list; HARBOUR_DIR_PICKER names a picker that prints one directory.",
    "C-l opens nvim at the root of Claude Code's repository; HARBOUR_EDITOR names another editor.",
    "C-Backspace arrives as C-h on most terminals, so in the pane it focuses the sidebar instead of deleting.",
    "Inside tmux with vim-tmux-navigator, add harbour to the pattern of processes it passes C-h and C-l to.",
    "Under tmux, Shift-Enter arrives as Enter: use Alt-Enter for a newline.",
];
