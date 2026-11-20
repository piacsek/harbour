kind: feat

- `harbour status` prints how many sessions of every running harbour are blocked, working, idle
  or unknown, as tmux status-line text: add `set -g status-right '#(harbour status)'` to your
  tmux configuration. It prints nothing when no harbour runs a session.
