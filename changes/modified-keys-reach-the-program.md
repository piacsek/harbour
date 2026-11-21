kind: fix

- A program in a session that asks for extended keys, as Neovim does, now gets the keys a plain
  terminal cannot tell apart: Ctrl+Esc, Ctrl+Tab, Ctrl+Enter, Shift+Enter, Ctrl+I, Ctrl+M and
  Ctrl+[ apart from Tab, Enter and Esc, Ctrl with a digit or punctuation, Ctrl+Shift with a
  letter, and F13 and above. Ctrl+], Ctrl+\ and the other control keys still arrive as
  themselves. A program that did not ask gets the same bytes as before.
