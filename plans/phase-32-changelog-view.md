# Phase 32: the changelog view

Issue #113. `c` in the sidebar shows the `CHANGELOG.md` bundled into the binary over the whole
screen, as `?` shows the help, so what changed in a release can be read without leaving harbour.

## Verified facts

- **The bundled text**: `harbour::changelog::CHANGELOG` is `include_str!("../CHANGELOG.md")`.
  On `main` it holds only `# Changelog`; a release commit (`v0.24.2`, read with `git show`)
  holds `# Changelog`, a blank line, then for each release `## vX.Y.Z (YYYY-MM-DD)` and its
  bullets, each `- ` with its later lines indented by two spaces, a blank line between releases.
- **The help view** is drawn over the whole frame (`Block::bordered().title(" help ")`), any key,
  a paste or a click closes it, and an open of `harbour edit` closes it too.
- **Wrapping by display width** is `text::wrap`: by words, a word wider than the room cut by
  grapheme, each measured in the cells ratatui fills.

## Decided

- Everything under "What changes", "Decided" and "States" in the issue, except the `--demo` row:
  the view always shows the binary's bundled text, in `--demo` too. A build from `main` shows
  the heading and one dim line, `This build has no release notes.`
- **A pure function** in `src/changelog.rs`, `shown(text, width)`, turns the text into the
  view's lines: `#` and `##` headings bold without their `#`, a bullet joined with its indented
  lines, the backticks of each code span dropped (a lone one kept), and wrapped to the width with
  `- ` and a hanging indent of two columns, blank lines kept,
  any other line wrapped as it is.
- **The view's state on `App`**: the text (the bundled one; `App::with_changelog`, a test seam
  on the `SEAMS` list, takes another) and, while it shows, its scroll, the room it had and its
  length at the last frame. Each frame wraps the text again to the view's width and clamps the
  scroll, so a resize keeps the last line at the bottom at most.
- **Keys while it shows**: `j` and Down one line on, `k` and Up one line back, Ctrl-d and Ctrl-u
  half the view's rows, PageDown and PageUp all of them, the wheel one line; any other key, a
  paste or a press of a mouse button closes it. It comes before the
  questions and after the usage popup and the picker in `key`, which cannot be open with it.
- **The help**: `?` and `c` share one row, `?/c` with `show or hide the help / the changelog`;
  no new row, so the help still fits 20 rows beside the narrowest sidebar. The README key table
  mirrors it. `harbour --help` says `c` shows the changelog on the line that says what `?` shows.
- **Log**: opening it writes `changelog shown` in place of `key name="c"`; the keys inside it
  write nothing; closing it (a key, a paste, a click, an open of `harbour edit`) writes
  `changelog closed`, as the usage popup writes `usage closed`.
- **The screenshot**: the `changelog` shot runs a scratch build whose `CHANGELOG.md` holds three
  made-up releases in the release format, built from a copy of `src/`, `Cargo.toml`,
  `Cargo.lock` and `rust-toolchain.toml` into the script's scratch `HOME`, built into
  `target/released/`, a target directory of its own, so it never replaces the binary of
  `target/release/` and only its first run compiles the dependencies.

## States

| State | The view shows, or what happens |
| --- | --- |
| A release build | every release, newest first |
| A build from `main` (`harbour-dev`, CI), and `--demo` | the heading alone, then the dim line `This build has no release notes.` |
| The text is shorter than the view | no scrolling; `j` and `k` do nothing |
| Scrolled to the end | the last line stays at the bottom, never past it |
| Opened again after scrolling | the top |
| The terminal is resized while open | wrapped again, the scroll kept within the new length |
| The narrow layout (under 60 columns) | the view fills the screen, as the help does |
| Beside the narrowest sidebar (60 columns), 20 rows | the view fills the screen; the help's `?/c` row fits on one line |
| Zoom or side by side | not reachable: the sidebar has no focus there |
| `c` from a pane | typed into the session |
| `c` with Ctrl or Alt | nothing, as for `u` |
| `C` (Shift, or Caps Lock) | opens it |
| `c` while the help shows | closes the help, does not open the changelog |
| A question, the picker, the session list or the usage popup is open | `c` goes to that, as today |
| A session exits while open | the view stays; the card shows the exit once it closes |
| A `harbour edit` request arrives while open | as under the help: an open closes the view and shows the editor; an ask is answered |
| A click or a paste while open | closes it, as the help |
| The mouse wheel while open | scrolls a line a notch, down and up |
| PageDown or PageUp while open | scrolls a whole view, clamped at both ends |
| A code span in a bullet or a heading | its backticks dropped, its text kept; a lone backtick stays |
| Caps Lock on (`J`, `K`, Ctrl-D, Ctrl-U with Shift) | scrolls as `j`, `k`, Ctrl-d and Ctrl-u |
| A line of wide characters (CJK) | wrapped by display width (`src/text.rs`), never wider than the view |
| A word wider than the view | cut by grapheme onto the next line |

## Tasks

1. This plan, its `PLAN.md` and `plans/metrics.md` rows, phase 30's after-merge cells.
2. `shown` in `src/changelog.rs` and `c` opening the view, any other key closing it
   (`tests/changelog.rs`, `tests/app/changelog.rs`).
3. Scrolling: `j`/`k`, the arrows, Ctrl-d and Ctrl-u, clamped, re-clamped after a resize,
   opening at the top.
4. The rest of the states: a click, a paste, `harbour edit`, `c` from a pane, under the help,
   with modifiers.
5. The log line.
6. The help row, the README key table, `harbour --help`, the fragment, the Layout of
   `AGENTS.md`, the `changelog` shot, `docs/keys.png` and `docs/help.png` again.
7. Outcome, Hurdles, metrics.

## Outcome

- Shipped as planned, in six tasks: the plan, the view on `c`, its scrolling, the states it
  shares with the help, its log line, and the help row, the README row and `harbour --help`.
- The text is always the bundled `CHANGELOG.md`, so a build from `main` shows the dim line
  `This build has no release notes.`; the test seam `with_changelog` hands the app other text.
- `docs/changelog.png` comes from a scratch build into `target/released/` with made-up `acme`
  releases, since a build from the tree bundles only the heading; `docs/keys.png` and
  `docs/help.png` were taken again for the new help row and usage line.
- Phase 31 merged during the build; its rows were kept, the merge needed no code change, and
  every screenshot came out the same after it.

## Hurdles

- `ci` an unrelated `harbour edit` test (`an_open_harbour_could_not_take_is_printed_and_exits_three`) failed once under load in a ship's gates and passed on every rerun → none
- `process` phase 31 was built in parallel and both branches added rows to `PLAN.md` and `plans/metrics.md`, so the merge of `main` conflicted there → none
- `review-loop` round 1 found the wording "in place of the pane" for a view drawn over the whole screen, the wheel and PageDown and PageUp closing it, the backticks of code spans shown, no log of its close, and four `AGENTS.md` lines that did not name it → code
- `review-loop` round 2 found Caps Lock closing the view on its scroll keys, backticks kept in a heading, the `jump.rs` line of `AGENTS.md` not naming the view, and `docs/keys.png` missing from the PR body → code
