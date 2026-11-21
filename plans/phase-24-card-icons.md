# Phase 24: icons on a session's card

## Context

Issue #44, the second half of the icons phase 23 started. A card's third line strings its facts
together with dots and words: `acme-api · main · #40 · 2 subagents`. With `nerd_font = true`, the
setting of phase 23, each fact gets a Nerd Font glyph in front of it, the glyphs take the place
of the dots and of the word `subagents`, and a Neovim glyph closes the line while the session has
an editor open. Off by default, as the border's icons.

Out of scope: lines 1 and 2 of a card (the title, the time, the prompt or reason), the state's
dot, the help view (no new row), the demo's defaults (it never reads the config file).

## Verified facts (2026-11-21, `main` at 6fed782)

- The Nerd Fonts glyph list, `glyphnames.json` of `ryanoasis/nerd-fonts` (its `METADATA` says
  version 3.5.1, dated 2026-08-21), has every name the issue gives, under that exact name:
  `oct-repo` `f401`, `oct-git_branch` `f418`, `oct-git_pull_request` `f407`, `cod-sparkle`
  `ec10`, `custom-neovim` `e6ae` (the one phase 23 already uses as `icons::NEOVIM`).
- `~/Library/Fonts/FiraCodeNerdFont-Regular.ttf`, the font of the screenshots, maps all five in
  its `cmap` (format 12 subtable, read with a short Python script, no font library): U+F401 to
  glyph 4756, U+F418 to 4779, U+F407 to 4762, U+EC10 to 3123, U+E6AE to 2192, each with an
  advance of 1200 units in `hmtx`, the same as `c`. The Bold file (a selected card is bold) and
  the Mono variant give the same glyphs and advances.
- All five are private use, East Asian Width "ambiguous": `unicode-width`, which `text::columns`
  uses through ratatui's `CellWidth`, counts one column each. tmux 3.7c moves the cursor one
  column for each of the four new ones (a scratch server printing the glyph alone, then
  `#{cursor_x}` = 1), as phase 23 found for the other two.
- `Card::new` in `src/card.rs` builds the third line once as `place`: `Place`'s `Display`
  (`<repository> · <branch>`), then ` · #<number>`, then ` · <n> subagent(s)` unless the state is
  `Exited`, the whole run through `clean`; `lines` cuts it with `text::fit` (keep what fits in the
  room minus one column, then `…`), so today the facts are lost from the end: subagents first,
  then the pull request, the branch, the repository.
- `State::of` gives `Exited` for any ended card, an exited session and a stopped (remembered) one
  alike.
- `App::compose` in `src/app/draw.rs` builds every card; `App::nerd_font` holds the setting, and
  `App::editor` the one editor program of the app (`Program::is_neovim`: its file name is
  `nvim`). Each session's open editors are `Entry::editors`: one leaves the list when it exits, and `x` clears it; a session that exits by itself may keep its editors, so the card's own rule for ended cards hides the glyph.
- The narrowest sidebar the card tests use is 20 columns (a 60-column screen's third, the
  narrowest that still has two columns), which leaves 16 for the third line.

## Decisions

- **The glyphs are the five the issue names**, as `icons::REPOSITORY`, `BRANCH`,
  `PULL_REQUEST`, `SUBAGENTS` and the existing `NEOVIM`. No substitute was needed.
- **The line with icons:** `<R> <repository> <B> <branch> <P> #<number> <S> <count> <E> `, each
  part only when its fact is there (the repository always is). One space follows every glyph,
  the editor's included, so a terminal that draws a glyph two cells wide never covers the next
  character or the sidebar's border; that trailing space counts when the line is fitted and is
  invisible.
- **The cut keeps today's order of loss**: the line is cut from its end as `text::fit` cuts the
  plain one, so the editor glyph goes first, then the subagents, the pull request, the branch and
  the repository. The one change: when the cut would leave a glyph without at least one column
  of its text (or the editor glyph, which has none, without its space), that glyph goes too, and
  the `…` follows the space before it: `<R> acme-api <B> main …`, as the plain line reads
  `acme-api · main …` at the same cut. This is the smallest rule that meets "a glyph never shows
  without at least one column of its text" without changing which fact goes first.
- **Ended cards** (exited or stopped) show no subagents and no editor glyph, as today's rule for
  subagents gives.
- **The editor glyph shows while the session has at least one editor open and the app's editor
  is Neovim**; one program serves every session, so another editor shows none on any card.
- **The app hands the card the fact with `Card::with_icons(editor_open)`**, called only when
  `nerd_font` is on; with it off the card is built exactly as on `main`, and `tests/card.rs`'s
  existing rows stay unchanged.
- **The `icons` shot shows the card icons too**: its scratch session starts in a scratch git
  repository on a branch and writes a registry file and a transcript that name a pull request
  and a running subagent, so all five glyphs show; the other shots do not change.

## What the user sees: reading outside harbour

Nothing new is read: the setting is phase 23's, the facts are the transcript's, the registry's
and git's, read as on `main`.

| State | What the user sees |
| --- | --- |
| `nerd_font` off or absent | every card byte for byte as on `main` |
| `nerd_font = true` | the third line of every card with glyphs |
| `nerd_font = true` on a terminal without a Nerd Font | an empty box or a missing-glyph mark before each fact; the facts still read |
| the editor program is not Neovim (`HARBOUR_EDITOR`, the file's `editor`) | no editor glyph on any card |
| a session's last editor exits | the editor glyph goes at the next frame |
| `--demo` with `nerd_font = true` | plain cards: the demo never reads the file |

## What the user sees: the third line with icons

`R`, `B`, `P`, `S`, `E` stand for the glyphs (one column each). `room` is the sidebar's inner
width minus the card's two-column indent; the narrowest sidebar (20) gives 16.

| State | Third line reads |
| --- | --- |
| icons off, any fact | exactly what it reads on `main` |
| no facts but the repository | `R acme-api` |
| a branch | `R acme-api B fix-retry` |
| a pull request | `R acme-api P #40` |
| 2 subagents running | `R acme-api S 2` |
| 1 subagent running | `R acme-api S 1` |
| an editor open (Neovim) | `R acme-api E` |
| all five, room 36 | `R acme-api B fix-retry P #40 S 2 E` |
| all five, room 34: the editor glyph goes first | `R acme-api B fix-retry P #40 S 2 …` |
| all five, room 31: `S` would show without its count | `R acme-api B fix-retry P #40 …` |
| all five, the narrowest sidebar (room 16) | `R acme-api B fi…` |
| a CJK branch `修正-東京`, room 16 | `R acme-api B 修…`, cut at a whole character |
| a CJK repository `日本語-acme` with a branch, room 16 | `R 日本語-acme …`: `B` would show without one column of `main` |
| an exited or stopped card with subagents and an editor | `R acme-api`, no `S` and no `E` |
| an editor that is not Neovim | no `E` |

## Tasks

1. This plan, its rows, phase 23's times from GitHub.
2. The card's iconed third line (`src/icons.rs`, `src/card.rs`, the `tests/card.rs` rows), and
   the app handing it the setting and whether a Neovim is open (one app test per side, icons on
   and off). One ship: `with_icons` is dead code until the app calls it.
3. The `icons` shot, `AGENTS.md`, the fragment, the outcome.

## Outcome

- The third line with icons reads as the tables above: `tests/card.rs` covers each fact alone,
  all five, none but the repository, the editor glyph going first, a glyph dropped with its
  text (`S` at room 31), the narrowest sidebar, a CJK branch and a CJK repository, and the
  exited and stopped cards; `tests/app/editor.rs` covers the app's side (the Neovim glyph while
  an editor is open, none for another editor, and the icons-off card byte for byte as on
  `main`). The existing `tests/card.rs` rows did not change.
- The `icons` shot shows all five glyphs on one card (`docs/icons.png`); no other image changed.
- Phase 23's known limit on one-column glyphs now covers the cards' glyphs too, rather than a
  second line.
- Phase 22 (#91) merged during the phase; the branch took `main` in with a merge, and its
  metrics row's ready and merged times were written from GitHub.

## Hurdles

- `test-seam` The cut, CJK and ended-card tests passed on their first run against the finished
  join (the plain `fit` already gave most cuts); each was shown red by breaking the code it
  covers (the glyph rule, the separator, the ended-card filter), then restored → none
- `process` `scripts/screenshots.sh` was edited while a ship ran, and `ship.sh`'s `git add -A`
  would have taken it into that commit; it was saved as a patch, checked out and applied again
  after the ship → none
