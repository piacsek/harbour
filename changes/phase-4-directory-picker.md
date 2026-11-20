kind: feat

- `n` opens a directory picker before it starts a session: fzf, fed with the directory harbour was opened from (first and selected) and the directories zoxide ranks. Enter starts the session there, typing finds another one, Esc starts nothing. Each session is named after its own directory.
- Once the picker confirms a directory, the new session's pane takes the focus, so you can type to it at once.
- `HARBOUR_DIR_PICKER` names another picker: a program that draws on the terminal and prints one directory. Without fzf and without `HARBOUR_DIR_PICKER`, `n` starts the session where harbour was opened, as before.
