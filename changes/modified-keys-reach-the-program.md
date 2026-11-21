kind: fix

- An editor or the directory picker that asks for extended keys, as Neovim does, now gets each
  key in the form it asked for, so Ctrl+Esc, Ctrl+Tab, Ctrl+Enter, Shift+Enter, Ctrl+I, Ctrl+M
  and Ctrl+[ no longer arrive as a plain Esc, Tab or Enter. Claude Code gets the same keys as
  before.
- With extended keys on in tmux, Ctrl+M in the sidebar acts as Enter, and Ctrl+Shift+H and
  Ctrl+Shift+L act as Ctrl+H and Ctrl+L.
