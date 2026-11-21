kind: fix

- An editor or the directory picker that asks for extended keys, as Neovim does, now gets each
  key in the form it asked for, so Ctrl+Esc, Ctrl+Tab, Ctrl+Enter, Shift+Enter, Ctrl with a
  digit or punctuation, Ctrl+Shift with a letter, and F13 and above reach it as themselves.
  Ctrl+I, Ctrl+M and Ctrl+[ reach it apart from Tab, Enter and Esc when the terminal tells them
  apart (under tmux, with `extended-keys on`).
- Claude Code gets the same keys as before, whatever it asks for.
- With extended keys on in tmux, Ctrl+M in the sidebar acts as Enter, and Ctrl+Shift+H and
  Ctrl+Shift+L act as Ctrl+H and Ctrl+L where harbour uses those keys.
