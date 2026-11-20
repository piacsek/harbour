kind: feat

- `harbour edit <file>` opens a file in the editor of the harbour session whose repository or
  worktree holds it, from a script or another terminal: harbour selects that session and shows
  its editor with the file open and the keys in it. The selected session wins when two hold
  the file, else the one used last. The editor must be Neovim; one is started with the file
  when the session has none open. Anything else (no session holds the file, no harbour running,
  Neovim waiting for a key) prints why and exits 1.
