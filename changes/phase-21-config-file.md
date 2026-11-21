kind: feat

- Harbour reads its settings from `$XDG_CONFIG_HOME/harbour/config.toml`, else
  `~/.config/harbour/config.toml`, at start: `editor`, `picker` and `session_command` (each a
  program with its arguments), `sidebar_width` and `accent`. A `HARBOUR_` variable still wins
  over the file; without a file nothing changes.
- `harbour config` prints the file's path and every setting with where it comes from: the file,
  a variable or the default. A bad file is named with its line and key in one line, here and at
  start.
