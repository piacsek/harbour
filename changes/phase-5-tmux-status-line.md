kind: feat

- `harbour status` prints how many sessions of every running harbour are blocked, working, idle
  or unknown, as tmux status-line text: add `set -g status-right-length 80` and
  `set -ga status-right ' #(harbour status)'` to your tmux configuration, which keeps your
  status-right and shows the counts after it. It prints nothing when no harbour runs a session.
