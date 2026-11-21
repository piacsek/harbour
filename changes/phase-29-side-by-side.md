kind: feat

- Alt-\ hides the sidebar and shows the selected session's Claude Code on the left and its
  editor on the right, each in half the terminal; Alt-\ again brings the sidebar back. It needs
  a terminal at least 120 columns wide and one that sends Alt-\ as an extended key (under tmux,
  `extended-keys` on), and never acts from the editor half: C-h there first.
- Side by side, C-l and C-h move the focus between the halves, a click focuses the half it
  lands in, and only the focused half shows a cursor. Below 120 columns the focused half fills
  the screen until the terminal is wide enough again.
