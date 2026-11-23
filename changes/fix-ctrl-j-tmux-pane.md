kind: fix

- Under tmux 3.7, Ctrl+J and Ctrl+K each stay in harbour when tmux does not bind that key, instead
  of jumping to another tmux pane: the key reaches Claude Code, or moves between cards in the sidebar.
