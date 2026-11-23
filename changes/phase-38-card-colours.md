kind: feat

- A card's third line is coloured by kind: the repository blue, the branch magenta, the pull
  request cyan, the subagents and the dots between facts dimmed. With `nerd_font` on, each glyph
  takes its fact's colour.
- A card's age is dimmed, `exited` is dimmed after a clean exit and red after a failed one, and a
  card that ended is dimmed unless it is selected.
- The dot of a session in an unknown state is drawn in the terminal's default colour.
- A `[colors]` table in `config.toml` sets the repository, branch and pull request colours
  (`repository`, `branch` and `pull_request`, each a colour name or `#rrggbb`); `harbour config`
  prints each with where it comes from.
