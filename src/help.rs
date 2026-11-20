pub const KEYS: [(&str, &str); 8] = [
    ("j/k", "select the next / previous session"),
    ("n", "pick a directory and start a session there"),
    (
        "C-l/Enter",
        "focus the pane; from Claude Code, C-l opens its editor",
    ),
    ("C-h", "focus the sidebar; from the editor, Claude Code"),
    ("e", "show the session's next editor, one per directory"),
    (
        "x",
        "end the session and its editors, or remove it once it has exited",
    ),
    ("?", "show or hide the help"),
    ("q", "quit, after a yes while a session or editor runs"),
];

pub const NOTES: [&str; 6] = [
    "n runs fzf over . and zoxide's list; HARBOUR_DIR_PICKER names another picker.",
    "C-l opens nvim at Claude Code's repository root; HARBOUR_EDITOR names another.",
    "In Neovim, C-h goes back only from the leftmost window, in normal mode.",
    "Most terminals send C-Backspace as C-h, so it acts as C-h.",
    "With vim-tmux-navigator, add harbour to the processes it sends C-h and C-l to.",
    "Under tmux, Shift-Enter arrives as Enter: use Alt-Enter for a newline.",
];
