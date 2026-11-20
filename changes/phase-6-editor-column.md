kind: feat

- Each session has an editor beside its Claude Code pane: `Ctrl-l` from Claude Code opens Neovim at the root of the repository or worktree Claude Code is in, and the pane shows one or the other. The pane's top border names both and marks the one shown, with the editor's directory.
- `Ctrl-h` from Neovim's leftmost window in normal mode goes back to Claude Code; anywhere else in Neovim it stays there. From the sidebar, `Ctrl-l` and Enter go to what the pane last showed.
- When Claude Code moves to another worktree, `Ctrl-l` opens a second editor there and the first keeps running, hidden, with its buffers; moving back shows it again. Quitting the editor returns to Claude Code.
- `HARBOUR_EDITOR` names another editor. `x` and `q` end a session's editors with it, and their questions say when a Neovim has unsaved changes.
