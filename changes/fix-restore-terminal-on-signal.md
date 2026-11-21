kind: fix

- When harbour is ended by `kill`, an interrupt sent from outside, or its terminal or tmux pane
  closing, it now ends its sessions and cleans up as a quit does, and gives the terminal back
  with the mouse, bracketed paste and extended keys turned off, so Ctrl+C and Ctrl+D work again
  in the shell after it. It exits with 143, 130 or 129 for those signals, so a script can tell
  them from a quit. Only `kill -9` still leaves the terminal needing `reset`.
