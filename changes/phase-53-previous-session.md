kind: feat

- Pressing the prefix twice (`C-/ C-/`) switches to the session selected before the current one,
  and pressing it twice again switches back, so it toggles between the last two sessions.
- The prefix then `p` sends the prefix key to the program in the focused pane, as pressing the
  prefix twice did before; `send_prefix` in `[keys]` moves it to another key.
