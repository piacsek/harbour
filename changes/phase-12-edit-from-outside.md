kind: feat

- `harbour edit <file>` opens a file in the editor of the harbour session whose repository or
  worktree holds it, from a script or another terminal: harbour selects that session and shows
  its editor with the file open and the keys in it. When several hold the file, the one whose
  repository or worktree lies deepest wins, then the selected one, then the one used last. The editor must be Neovim; one is started with the file
  when the session has none open. Anything else prints why: it exits 3 when no harbour runs or
  answers, no session holds the file, or a Neovim harbour started did not answer in time or exited, so a script can open it itself, and 2 when harbour or
  Neovim refused it (Neovim waiting for a key, for one) or a harbour went silent once it had the
  file, since it may still open it.
