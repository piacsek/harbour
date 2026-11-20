kind: feat

- Harbour works in the directory of the selected session: the root of its repository or
  worktree, or the directory itself outside one. A tmux shortcut that opens a split, a window
  or a popup at `#{pane_current_path}` now opens where the selected session works, and follows
  when another session is selected or Claude Code moves into a worktree. With no session, or
  when that directory is gone, it is where harbour was opened.
  It follows when harbour is typed at a shell prompt, or when a tmux pane command or a
  wrapper script ends with `exec harbour`.
