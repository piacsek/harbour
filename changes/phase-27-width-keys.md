kind: feat

- Alt-minus makes the sidebar 2 columns narrower and Alt-equals 2 columns wider, from the
  sidebar or the Claude Code pane, between 15 columns and half the terminal, until harbour
  quits. A held key repeats.
- They work only where the terminal sends them as extended keys (under tmux, `extended-keys on`),
  and never from the editor. Under tmux, Esc then `-` or `=` typed fast in Claude Code arrives
  as the Alt key and changes the width.
