kind: fix

- When harbour is ended by `kill`, an interrupt sent from outside, or its terminal or tmux pane
  closing, it now ends its sessions and cleans up as a quit does, and gives the terminal back
  with the mouse, bracketed paste and extended keys turned off, so Ctrl+C and Ctrl+D work again
  in the shell after it. It then ends by that same signal, so a shell or a script sees it was
  killed or interrupted. A second `kill` or interrupt ends a harbour that is stuck at once.
  Any other signal, SIGKILL included, still leaves the terminal needing `reset`.
