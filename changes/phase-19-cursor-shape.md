kind: feat

- The cursor in a pane takes the shape its program asks for: a thin bar in Neovim's insert mode,
  an underline in its replace mode, a block in normal mode. It is the terminal's own cursor,
  so it blinks and is coloured as the terminal draws it, and harbour gives the terminal back
  its default shape when it quits.
