kind: feat

- Harbour reads its settings from `$XDG_CONFIG_HOME/harbour/config.toml`, else
  `~/.config/harbour/config.toml`, at start: `editor`, `picker` and `session_command` (each a
  program with its arguments), `sidebar_width` and `accent`. A `HARBOUR_` variable still wins
  over the file; without a file nothing changes.
- `sidebar_width` sets the sidebar's columns, borders included, with the pane keeping at least
  half, and `accent` colours the focused border and the selected card's bar (cyan by default).
- `harbour config` prints the file's path and every setting with where it comes from: the file,
  a variable or the default. A bad file is named with its line and key in one line, here and at
  start.
- A `picker` or `session_command` from the file whose program is named `fzf` or `claude` runs
  as the default does (fzf over the list, claude with its session id), with the arguments given;
  a configured picker that cannot start says so instead of being skipped.
