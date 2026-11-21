kind: feat

- Alt-s lists every session in fzf, running, ended and stopped alike, by title, repository and
  branch; typing filters the list, Enter jumps to the one picked as Ctrl-1 to Ctrl-9 do (a
  stopped one resumes), and Esc closes it and changes nothing. With a picker other than fzf it
  says the finder needs fzf.
- Alt-s works only where the terminal sends it as an extended key (under tmux, `extended-keys
  on`), and never from the editor. Under tmux, Esc then `s` typed fast in Claude Code arrives as
  Alt-s and opens the list.
- The help view says `C-l/Enter  focus or resume; from Claude Code, its editor`, so the whole
  help still fits 20 rows beside the narrowest sidebar.
