kind: feat

- Harbour's actions now sit behind one prefix key, `C-/`: press it, then `z` to zoom, `\` for
  side by side, `{` and `}` to swap the halves, `s` to find a session, `e` for the next editor,
  `h` and `l` to focus the sidebar or the pane, `-` and `=` for the sidebar's width, `u` for
  the usage, `c` for the changelog, `?` for the help, `q` to quit, or a digit to jump to a
  card. They work from the sidebar and from every pane, with or without tmux's extended keys.
  `C-/` twice sends `C-/` to the program.
- While the prefix waits for its key, `C-/` shows on the pane's border, and the sidebar's
  bottom border shows `C-/ ? help`.
- `M-z`, `M-\`, `M-{`, `M-}`, `M-s`, `M--`, `M-=`, `C-1` to `C-9`, and `e`, `u`, `c`, `?` and
  `q` in the sidebar are gone: those keys now reach the program you are in. The sidebar keeps
  `j`, `k`, Enter, `C-h`, `C-l`, `n` and `x`.
- The `prefix` setting picks another prefix key (`C-]` or `C-Space`, for one), and a `[keys]`
  table moves an action to another key or turns it off with `"none"`; `harbour config` shows
  both and the help lists the keys in force.
