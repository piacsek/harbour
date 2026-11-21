kind: fix

- The editor harbour opens beside a session sees the tmux harbour runs in, so a plugin that
  drives tmux (a test runner opening a split, for one) works there as in a pane of its own.
  Claude Code sessions and the directory picker still start outside tmux.
