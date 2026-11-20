kind: feat

- `harbour status` prints how many sessions of every running harbour are blocked, working, idle
  or unknown, as tmux status-line text: put `#(harbour status)` in your tmux `status-right`, for
  example `set -g status-right '#(harbour status) %H:%M'`, and raise `status-right-length` if
  the counts are cut off. It prints nothing when no harbour runs a session.
