# Phase 55: the session finder's dots in their state colours

The session finder (the prefix then `s`) draws every card's dot in the terminal's plain
foreground, so a blocked session looks like a working one there, while the sidebar, the
counts and the tmux status line colour the same dot by state. The finder now colours it the
same way: working yellow, blocked red, idle and exited dim, unknown plain.

## Verified facts

- **The finder's list carries no colour.** `src/finder.rs` writes `state.glyph()` alone in
  the second field, and `finder::OPTIONS` does not pass `--ansi`, so fzf would print any
  escape code as text.
- **The sidebar's colours are `State::style`** (`src/state.rs`): `Working` yellow, `Blocked`
  red, `Idle` and `Exited` dim, `Unknown` the default style.
- **fzf 0.74.0 with `--ansi` matches and prints the text without the codes.** Checked live:
  `printf '1\t\033[2m○\033[0m\tacme api \ty \t4m\n' | fzf --ansi --delimiter='\t'
  --with-nth=2.. --nth=2,3 --accept-nth=.. --filter=api` prints `1\t○\tacme api \ty \t4m`, so
  `finder::chosen` still reads the number from the first field, and `--nth=2,3` never matches
  the dot field anyway.
- **Titles and places cannot carry an escape code into the list.** Both go through
  `text::clean`, which drops every control character (`text::hidden`), ESC included.

## Decided

- The orchestrator's: the codes are SGR `33` (yellow), `31` (red) and `2` (dim), each closed by
  `0`, the same 16-colour names `State::style` and the tmux status line use; `Unknown` gets no
  code. The colours come from one place in `src/state.rs` next to `State::style`.
- The orchestrator's: no screenshot. The finder is a popup over fzf, not the main interface
  `docs/details.png` shows.

## States

What the user sees on a finder line's dot.

| State | What the user sees |
| --- | --- |
| Working | `●` in yellow |
| Blocked, waiting on the user | `◉` in red |
| Idle, or exited (`exited` as its age) | `○` dimmed |
| A stopped card (`stopped` as its age) | `○` dimmed, as idle |
| Status unknown | `·` in the plain foreground |
| The selected line | the same colour over fzf's highlight |

## What it reads from outside harbour

| Read | What the user sees |
| --- | --- |
| fzf without colour (`--no-color`, `--color=bw`, `NO_COLOR` in `FZF_DEFAULT_OPTS` or the environment) | the dots plain, as today; matching and the pick unchanged |
| `FZF_DEFAULT_OPTS` with `--ansi` already | no change, the flag is idempotent |
| `FZF_DEFAULT_OPTS` with `--with-nth` or `--nth` | harbour's options come after and win, as today |

## Tasks

1. The red test in `tests/state.rs` for each state's colour code, then `State` gets it.
2. The red tests in `tests/finder.rs`: each state's dot wrapped in its code; the line tests
   that pin an exact string take the coloured dot. Then `finder::lines` writes it and
   `finder::OPTIONS` gains `--ansi`.
3. An end-to-end check in `tests/e2e/finding.rs` that the popup draws a working session's dot
   in yellow; the `src/AGENTS.md` lines of `finder.rs` and `app/find.rs`; the fragment.

## Outcome

### Sweep

Range: phases 51 to 54 (`scripts/sweep-report.sh`, after phase 50's sweep). The owner's
standing answer made every finding an issue; a finding that repeats an open issue became a
comment on it, as phases 45 and 50 did. No rule, `AGENTS.md` or `docs/process.md` changed.

- Tests green on their first run (52: the timeout tests and a fresh-copy test): repeats #96, the
  third sweep in a row to see it, comment there.
- Tests that depend on the machine's load (51 shared the PTY slots for it, 53's picker test
  failed twice under the gates, 54's machine-wide PTY counts swung by 110, `→ none`): repeats
  #249, comment there.
- Phase 51 merged with its `## Outcome` and `## Hurdles` empty, and `tests/plans.rs` let it:
  issue #252.
- The two `→ none` (52's 100 ms window, 54's PTY counting) are different problems, not a repeat.
  One-offs, not findings: a commit pushed over the 700-line cap (54), a hangup lost when
  children moved off the PTY (54).
- Metrics: review rounds none recorded, 1, 2, 5; build start to the clean round 19 min 32 s
  (52), 54 min 56 s (53), 2 h 39 min 31 s (54); the reviewer's tokens a round about 104k, 128k
  to 143k, 136k to 177k, growing with the size of each change; the orchestrator's context at the
  start 134851, 183582, 62664, and 54's grew to 430158 by its clean round. Too little comparable
  data for a trend. Process drift: 51 and 54 were built in the main session without a worktree,
  both at the owner's word; only 51 skipped its retro (#252).

### The finder's dots

The finder's list wraps each dot in `State::sgr`, a colour code derived from `State::style`, so
the finder and the sidebar take each state's colour from one place, and fzf reads it with
`--ansi`. tmux and harbour's popup redraw fzf's `ESC[33m` as the indexed `38;5;3`, so the
end-to-end check asserts only that the working dot is drawn with a colour code, not plain.
Round 1 found only nits (the colour copied by hand, literals pinned twice, an exact literal at
the end-to-end level), all fixed; round 2 found no must or should. Its two nits stay in the pull
request body: the end-to-end check would pass for any colour, and the fake fzf's `grep` sees the
escape bytes that real fzf ignores, which no test types today.

## Hurdles
- `test-seam` tmux reports the finder's yellow as `38;5;3`, not the `33` fzf was given, so the first end-to-end check pinned a literal the encoding chose; it now asserts the colour loosely → code
- `ci` An unrelated end-to-end test (`navigating::ctrl_j_in_claude_code_with_nothing_below_reaches_it_and_ctrl_k_moves_above`) failed once in the gates and passed three times alone, the load sensitivity of #249 → backlog
- `process` A merge of `main` pushed to the branch while the builder worked refused its next push; it merged the remote branch and shipped again → none
