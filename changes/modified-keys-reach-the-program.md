kind: fix

- A program in a session that asks for extended keys, as Neovim does, now gets the keys a plain
  terminal cannot tell apart: Ctrl+Esc, Ctrl+Tab, Ctrl+Enter, Shift+Enter, Ctrl with a digit or
  punctuation, Ctrl+Shift with a letter, and F13 and above. A program that did not ask gets the
  same bytes as before.
