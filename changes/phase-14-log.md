kind: feat

- With `HARBOUR_LOG` set to anything but empty, harbour keeps a private log of what it saw and
  decided: its start and quit, the keys it acted on, where input went (by byte count only),
  focus and selection, what it refused, each session and editor starting and exiting, every
  `harbour edit` request and Neovim call, frames of 100 ms or more, and the screen errors it
  recovered from. It never records what was typed, pasted or printed, or a session's title.
  One file a run under harbour's state folder, removed after 7 days.
- `harbour log` prints the path of the newest log, so `tail -f "$(harbour log)"` follows it.
