kind: fix

- Inside tmux, the editor harbour opens beside a session keeps harbour's `TMUX` and
  `TMUX_PANE`, so a tmux command it runs acts on harbour's pane: a test runner that drives
  tmux opens its split beside harbour instead of failing with "not inside tmux session".
  Claude Code sessions and the directory picker still start outside tmux.
