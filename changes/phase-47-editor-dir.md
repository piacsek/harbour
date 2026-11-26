kind: feat

- `editor_dir = "exact"` in the config file opens the editor in the folder a session started in,
  not at its repository root, even after Claude Code moves elsewhere; `harbour edit` and the
  folder tmux follows use that folder too.
