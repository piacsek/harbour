# Changelog

## v0.1.0 (2026-11-20)
- `harbour` with no argument, on a terminal, opens a sidebar of Claude Code sessions beside a live pane.
- `n` starts a session in the current directory, `j` and `k` select one, `Ctrl-l` or Enter focuses its pane and `Ctrl-h` comes back.
- Keys, pastes and the mouse reach the focused session, and the sidebar shows when a session has exited and with what code.
- `x` ends a session or removes one that has exited, `?` shows the keys, and `q` asks first while a session runs.
- `harbour --demo` shows the same with made-up sessions, no `claude` needed.

## v0.0.1 (2026-11-19)
- harbour installs from Homebrew, a release tarball or Nix, and answers `--version`, `--help` and `changelog`.
