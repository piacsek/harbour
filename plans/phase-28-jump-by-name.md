# Phase 28: jump to any session by typing part of its name

Issue #39. Alt-s opens a popup that lists every card in fzf; typing filters by fuzzy match over
the title, the repository and the branch, Enter jumps to the chosen card as Ctrl-1 to Ctrl-9 do,
Esc closes it and changes nothing. It works from the sidebar and from a Claude Code pane.

## Verified facts (2026-11-21, `main` at 5ca6a1c)

- **Claude Code binds no Alt-s.** Its keybindings documentation (code.claude.com, read
  2026-11-21, not run) lists Meta+P, Meta+O, Meta+T, Meta+Up and Meta+Down as its Alt defaults,
  plus Alt+V on Windows and WSL; nothing on Meta+S or Alt+S.
- **The bytes Alt-s arrives as**, captured in a scratch tmux 3.7c server (`-L <scratch> -f
  /dev/null`, `TMUX_TMPDIR` a short folder under `/tmp`) by a raw-tty reader that asks for
  extended keys with `ESC[>4;2m`, as harbour does:

  | tmux | `send-keys M-s` | `send-keys Escape s` | `send-keys -H 1b 73` | `send-keys M-S` |
  | --- | --- | --- | --- | --- |
  | `extended-keys on`, format `xterm` | `1b 5b 32 37 3b 33 3b 31 31 35 7e` (`CSI 27;3;115~`) | `1b 73` | `1b 73` | `CSI 27;3;83~` |
  | `extended-keys on`, format `csi-u` | `1b 5b 31 31 35 3b 33 75` (`CSI 115;3u`) | `1b 73` | `1b 73` | `CSI 83;3u` |
  | `extended-keys off` | `1b 73` | `1b 73` | `1b 73` | `1b 53` |

  Typed through a tmux client (an outer pane attached to the inner server), Esc then s sent
  together, 50 ms apart or 300 ms apart reached the pane as the extended Alt-s with
  `extended-keys on` (both formats); 2.5 s apart they arrived as `1b`, then `73`. With
  `extended-keys off` every one of them arrived as `1b 73`. So phase 25's rule for Alt-z holds
  for Alt-s too.
- **fzf 0.74.0 (Homebrew)**, checked with `--filter` over tab-separated lines
  `<n>\t<dot>\t<title>\t<place>\t<age>`:
  - `--delimiter=<TAB> --with-nth=2..` shows fields 2 to 5; `--nth` counts the fields after
    that transform, so `--nth=2,3` matches the title and the place only. A query for the hidden
    number, the dot or the age matched nothing; one for a title word or `acme-docs` matched its
    line.
  - fzf prints the whole original line, the hidden number first, so harbour reads the number
    before the first tab. `--accept-nth` exists in 0.74 (`--accept-nth=.. --version` exits 0;
    an unknown option exits 2), and a later `--accept-nth=..` overrides the user's own.
  - Rendered in a 50-column tmux pane with `--tabstop=1 --layout=reverse`, each line takes a
    2-column gutter (`▌ `) and fzf cuts a longer line with `··`; a list longer than the window
    adds a 1-column scroll bar. Each tab shows as one space.
- `Entry.id` is 0 for every stopped card (`src/app/remember.rs`), so a stopped card is known by
  its remembered session id, a running or exited one by `Entry.id`.
- While a popup is open no key, paste or mouse event reaches the sidebar, so `x` and Enter
  cannot run; a session can still exit (its exited card stays until `x`), and a stopped card can
  still be merged into a live session whose registry names its id (a conversation resumed
  inside harbour).

## Decided

- **The key rule (phase 25's).** Harbour takes Alt-s only in an extended form (`Char('s' | 'S')`,
  modifiers exactly Alt, a press, not `input::ESCAPED`), never while the pane shows an editor.
  The legacy `ESC s` goes to the session. Where it does not act and why, said in this plan, the
  PR body, the fragment and `harbour --help`:
  - a terminal or tmux without extended keys sends `ESC s`, which is also what Esc then s typed
    in Neovim or a shell sends, so harbour leaves it to the session;
  - from an editor it never acts, so Neovim keeps every Alt key: C-h first (from Neovim, Esc
    then C-h);
  - the cost: under tmux with extended keys, Esc then s typed within about 300 ms in a Claude
    Code pane opens the popup instead of reaching Claude Code.
- **The finder is fzf, and only fzf.** It runs the configured picker only when that program's
  file name is `fzf` (the default, the config file's `picker`, or `HARBOUR_DIR_PICKER` naming
  an fzf binary), since the finder's arguments are fzf's. Any other picker says
  `Could not start the session finder: it needs fzf, not <program>`, and nothing changes. Why:
  a custom picker gets one directory per line and prints one path; a session list needs fzf's
  hidden field and match options, which another program would not read the same way. With no
  picker set in the app (only tests build one so) it runs `fzf`.
- **fzf's arguments**, after the user's `FZF_DEFAULT_OPTS` and the directory picker's
  `--no-expect --no-print-query --no-read0 --no-print0`:
  `--delimiter=<TAB> --with-nth=2.. --nth=2,3 --tabstop=1 --no-sort --layout=reverse
  --bind=enter:accept --no-preview --header-lines=0`, then `--accept-nth=..` when the popup's
  shell finds that `fzf --accept-nth=.. --version` exits 0 (fzf 0.60 and later), so neither an
  `enter` binding nor an `--accept-nth` in `FZF_DEFAULT_OPTS` changes the line fzf prints, a
  `--preview` there runs nothing on a session line, a `--header-lines` there leaves the first
  session pickable, and an older fzf still runs (`--no-preview` and `--header-lines` are far
  older than 0.60, so they need no probe). `--no-sort` keeps the sidebar's order, `--layout=reverse` lists it top-down as the
  sidebar does.
- **A line**: `<n>\t<dot>\t<title>\t<place>\t<age>`, where `<n>` is the card's position when the
  popup opened, never shown or matched, and `<age>` is the word `stopped` for a stopped card and
  `exited` for an ended session, which share the idle `○` and no colour reaches fzf, so two
  sessions of one title can be told apart; a running session keeps its time. The title and place are `text::clean`ed as the card
  does them (no tab, newline, escape or other control character survives, so no title can
  split a field or a line, and each kept to 512 characters), then padded, never cut further, to
  the widest title and the widest place,
  measured by display width; with `--tabstop=1` each tab is one column, so the place and age
  line up, and fzf cuts what overflows its window with `··` and scrolls the line to the match,
  so every word of a title can be matched. The title and place are the zoomed border's (`headline`): the
  transcript's title, else the card's name; `repository · branch`; the time in its state,
  right-aligned with the words.
- **The list is the one from when the popup opened.** The finder keeps, for each position, who
  was there: a session (running or exited) by its id and its remembered session id, a stopped
  card by its remembered session id. A pick finds that session whether it still runs or not;
  for a stopped card no longer there, the entry that now holds its remembered id. A card is not
  added or redrawn while the popup is open.
- **The list goes to fzf through a file**: written by harbour into the pick's private (0700)
  scratch directory, read by `"$@" <"$listed" >"$out"` in the popup's shell. No title is ever
  part of a shell word.
- **A pick goes through `App::jump`**, as Ctrl-1 to Ctrl-9 do: it selects the card and focuses
  its pane, or resumes a stopped card with the same refusals. A refused pick leaves the sidebar
  focused, as a refused jump does. Zoom stays on after a pick of a live session.
- **The popup is the directory picker's**: every key, paste and mouse event goes to fzf;
  `harbour edit` answers `none` or refuses while it is open; Ctrl-1 to Ctrl-9, Alt-z, `u` and
  the width keys and every other harbour key do nothing. It is never a card and never remembered. Its border's
  title is ` sessions `.
- **The log** names the key `M-s` when harbour acts on it. What is typed into the popup is
  logged as `input to=picker bytes=<n>`, never its text.
- **The help view** gets one row, `M-s  jump to a session by part of its name`. To keep the
  whole help on 20 rows beside the narrowest sidebar (59 and 60 columns), the `C-l/Enter` row
  becomes `focus or resume; C-l in Claude Code: editor` (43 columns, one row at 59), naming the
  key, since only C-l opens the editor from Claude Code. The
  headroom test goes from 2 spare rows to 1 on purpose, and to 0 once phase 27's width row is merged.

## States

| State | Alt-s |
| --- | --- |
| Sessions exist, fzf present | the popup opens over the screen, ` sessions ` on its border, one line per card in sidebar order |
| No session and no stopped card | nothing happens |
| fzf missing | `Could not start fzf: is it installed and on PATH?` (`not an executable file` for a path), nothing changes |
| fzf cannot start, or exits 2, 126 or 127 | `Could not start fzf: it exited with <code>`, nothing changes |
| The picker is not fzf (`HARBOUR_DIR_PICKER=sk`, a config `picker` of another program) | `Could not start the session finder: it needs fzf, not sk`, nothing changes |
| Esc, or Enter with no match | the popup closes, selection and focus as before |
| A running session picked | it is selected and its pane focused, as Ctrl-N |
| An exited session picked | as Ctrl-N on its card: one with a remembered conversation resumes, another is selected and its pane focused |
| A stopped card picked | it resumes in its directory, as Enter |
| The picked session exited while the popup was open | as an exited session picked: its exited card is still in the sidebar |
| The picked stopped card was merged into a live session while the popup was open | that live session is selected and its pane focused, as Ctrl-N |
| The picked card is no longer anywhere | nothing is selected, `That session is gone: it was removed while the list was open.` (no path removes a card while the popup is open; kept for safety) |
| fzf prints a line without the number (an `FZF_DEFAULT_OPTS` binding that prints something else) | nothing is selected, `The session finder printed a line harbour cannot read; check FZF_DEFAULT_OPTS.` |
| A picked stopped card cannot resume (`directory gone`, `open elsewhere`) | its card is selected and says why, the sidebar focused |
| The list changes while the popup is open (a session exits, a title or branch changes) | the popup keeps the list from when it opened; a pick goes by the rows above |
| 100 stopped cards | 100 lines plus the running sessions; fzf scrolls and filters them |
| A CJK title | padded by display width, the place still lines up; fzf cuts it with `··` |
| A title longer than the popup | kept whole up to 512 characters: a word at its end matches, and fzf scrolls the line to it |
| Two sessions of one title, one stopped or exited | the last column says `stopped` or `exited` for it, the running one its time |
| The list file cannot be written | nothing opens, `Could not list the sessions: <error>` |
| `--preview` or `--header-lines` in `FZF_DEFAULT_OPTS` | no preview pane, every session pickable: `--no-preview --header-lines=0` come after the user's options |
| A title holding a tab, a newline, an escape sequence | cleaned as on the card: no field or line is split, no escape reaches fzf |
| The narrow layout (under 60 columns) | the popup is 80% of the screen as the directory picker's; the pick shows the pane as Ctrl-N does |
| Zoomed | the popup opens over the zoomed pane; a pick of a live session stays zoomed |
| A question, help or the directory picker shown | the question or help takes the key as any key; the directory picker gets it as any key |
| The pane shows an editor | the editor gets Alt-s; nothing opens |
| Alt-s as legacy `ESC s` | the session gets it |
| Alt-s with Caps Lock (`CSI 27;3;83~`, `CSI 83;3u`) | opens, as Alt-s |
| While open: Ctrl-1 to Ctrl-9, Alt-z, `u`, `n`, `q`, a paste, a click | all go to fzf; harbour does nothing else |
| While open: the width keys (`M--`, `M-=`) | nothing happens; the sidebar keeps its width |
| While open: `harbour edit <file>` | answered `none` to an ask, refused to an open, as for the directory picker |
| `--demo` | lists the made-up sessions and the stopped card |

What harbour reads from outside: the key bytes above, fzf (found on `PATH` as the directory
picker is, from where harbour was opened), and what fzf prints. fzf missing or not executable is
the `could not start` row; fzf replaced between the check and the run exits 126 or 127, the same
row; another harbour shares nothing here (the list file is in this pick's private directory).

## Tasks

Each task ships whole: every new `pub` item ships with its first caller.

1. This plan, the `PLAN.md` and `plans/metrics.md` rows, phases 25 and 26's after-merge cells.
2. Alt-s opens the finder and a pick jumps: `src/finder.rs` (lines, the number read back), the
   `Picker::Listed` kind, the popup's title, `App::jump` on a pick, Esc.
   Red: a test that presses Alt-s and waits for the fixture's `acme fzf ready`.
3. The edge states: gone, refusals, not fzf, fzf missing, exit 2, not from an editor, not
   `ESCAPED`, no card, the modal keys, `harbour edit`, zoom kept, the log name `M-s`.
   Tests of an absence (no card, `ESC s`, from an editor, the modal keys) pass on their first run
   once task 2 is in; each is shown red by breaking the guard it covers, then the guard restored.
4. The help row, the README key table, the headroom test (2 to 1), `harbour --help`.
5. The e2e through tmux with `extended-keys on` (Alt-s opens, `ESC s` reaches the session), the
   demo, the `sessions` shot, `docs/keys.png` and `docs/help.png` shot again, the fragment, the
   Layout of `AGENTS.md`, `tests/AGENTS.md` on the fixture.
6. Outcome and Hurdles.

Tests use `tests/fixtures/finder/fzf`, a script named `fzf` that prints its arguments and the
list it read, then reads a line from its terminal and prints the first listed line holding it
(a line starting with Esc exits 130, an empty one is read past, `broken` exits 2), never a real
fzf.

## Outcome

- Alt-s opens the directory picker's popup with the border title ` sessions ` and runs fzf over
  one line per card, `<n>\t<dot>\t<title>\t<place>\t<age>`, in sidebar order, the age `stopped`
  or `exited` for a card that does not run. Typing filters by
  title, repository and branch, Enter jumps to the pick through `App::jump`, as Ctrl-1 to Ctrl-9
  do (a stopped card resumes), Esc closes it and changes nothing.
- The key rule is Alt-z's: taken only in an extended form (`CSI 27;3;115~` or `CSI 115;3u`,
  Caps Lock's `S` too), never in the legacy `ESC s` form, which reaches the session, and never
  from an editor. Its cost, under tmux with `extended-keys on`, Esc then `s` typed fast in
  Claude Code opens the list, is in this plan, the PR body, the fragment and `harbour --help`.
  `tests/e2e/finding.rs` checks both formats through tmux 3.7c, and `ESC s` with it off.
- fzf only: the configured picker runs only when its file name is `fzf`; any other says
  `Could not start the session finder: it needs fzf, not <program>`. fzf's arguments, after the
  directory picker's: `--delimiter=<TAB> --with-nth=2.. --nth=2,3 --tabstop=1 --no-sort
  --layout=reverse --bind=enter:accept --no-preview --header-lines=0`, plus `--accept-nth=..`
  where `fzf --accept-nth=.. --version` succeeds, so the user's options cannot change the line
  fzf prints, add a preview or hide a session, and an older fzf still runs. The list goes to fzf through a file in the pick's private
  directory, so no title is ever part of a shell word, and harbour reads the number before the
  first tab of what fzf prints; a line without one says to check `FZF_DEFAULT_OPTS`, and a list
  that cannot be written says `Could not list the sessions: <error>`. Titles (up to 512
  characters, as cleaning keeps them) and places go to fzf whole, padded to the widest, so every
  word can be matched.
- The list is the one from when the popup opened: a session is kept by its id and remembered
  id, a stopped card by its remembered id. A session that ended while the list was open is still
  picked as its exited card (a remembered one resumes, as Ctrl-N), and a stopped card merged
  into a live session meanwhile jumps to that session. A refused resume selects its card and leaves the sidebar focused; zoom stays on after a pick.
- The popup is modal like the directory picker: every key (Ctrl-1 to Ctrl-9, Alt-z, `u`, the
  width keys), paste and mouse event goes to fzf, and `harbour edit` is answered `none` or
  refused. The log names the key `M-s` and logs what is typed into the list as a byte count.
- The help gained `M-s  jump to a session by part of its name`. To keep it whole at 59 and 60
  columns by 20 rows, `C-l/Enter` now reads `focus or resume; C-l in Claude Code: editor`.
  After merging phase 27 (the width keys, which added its own row and shortened `x`) the help
  fills a 20-row terminal with no row to spare; the headroom test says 0 and is renamed. The
  next key row needs a shorter row elsewhere or the pointer line moved first.
- `scripts/screenshots.sh` shoots `docs/sessions.png` (the demo after M-s, with `extended-keys`
  on for that shot); `docs/keys.png` and `docs/help.png` were shot again.
- 2 review rounds. Round 1 found the exited and merged picks, cut titles, the user's
  `FZF_DEFAULT_OPTS` changing the printed line, and stale docs; the clean round 2 left minor
  points fixed in the last ship: `stopped` and `exited` in place of the age, the help row naming
  C-l, `--no-preview --header-lines=0`, its own message for a list that cannot be written, and
  the Esc test pressing Esc.

## Hurdles

- `test-seam` The fixture's messages wrapped in the narrow sidebar, so whole-message asserts
  failed for the wrong reason; a helper joins the sidebar's text between its borders → code
- `test-seam` A first `harbour edit` test (an ask, answered `none`) passed with the guard broken,
  since `none` is also the answer when no session holds the file; it became an open of a file in
  a session's root, refused → code
- `test-seam` `read -t` in a fixture is not portable to dash on Linux; a background watcher that
  kills the script replaced it → code
- `test-seam` The e2e's first absence check matched the sidebar's own ` sessions · 1 ` title
  instead of the popup's; it now looks for the fixture's `acme fzf` lines → code
- `process` Phase 27 merged mid-phase and took the help's last spare row; the merge reconciled
  both rows and set the headroom to 0 → none
- `review-loop` Round 1 found that the plan's "gone" row treated an exited card as removed, that
  cutting titles to the popup made their ends unsearchable, and that a user's `--accept-nth` or
  `enter` binding dropped the pick silently; the states table had no rows for a merge or for the
  user's fzf options → code
- `review-loop` Round 1 found states the plan's table had not listed (a session ending or a
  stopped card merging while the list was open, a user's `FZF_DEFAULT_OPTS`); they became rows
  of the table and tests → code
- `test-seam` The fixture closed on an empty line, so the Esc test passed without Esc reaching
  fzf; it now closes only on a line starting with Esc, read with `dd` since `read` cuts a line
  at the NUL that Ctrl-2 sends (`head` first, which the e2e's scratch `PATH` lacks) → code
