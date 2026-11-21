kind: feat

- Alt-z zooms the selected session to the whole screen, and back. The sidebar hides and the
  pane's top border shows the session's dot, title, repository and branch, and time, then the
  names, with the counts at the right. C-h from Claude Code ends the zoom, and C-1 to C-9 jump
  to another session still zoomed.
- Alt-z zooms only where the terminal sends it as an extended key (under tmux,
  `extended-keys on`), and never from an editor: C-h first. Under tmux, Esc then z typed fast in Claude Code
  arrives as Alt-z and zooms.
- The help says `q` quits "after a yes while anything runs", a shorter line that leaves room for
  the new key.
