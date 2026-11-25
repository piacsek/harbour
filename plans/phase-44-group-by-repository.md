# Phase 44: group the cards of one repository

Issue #190. Cards sit in one flat list and every card's third line repeats its repository. When
two or more cards share a repository, the sidebar puts them under one header with its name, and
their third lines drop it.

## Verified facts

- **The sidebar draws `App::sessions` in its order** (`src/app/draw.rs` builds one `Card` per
  entry, `Sidebar::draw` in `src/card.rs` lays them out at a fixed step of 4 rows). `j`, `k`, the
  prefix then a digit (`jump_to`) and the session finder's list (`find::list`) all index that
  same vector, so keeping the vector in the order on screen keeps every one of them right.
- **A card's repository is `Entry::place`**, read by `repo::place` from where the session works
  (`seen`), again on every read of its files (`read_entry`); outside any repository it is the
  folder's name. It can change while a session runs (Claude Code moved to another checkout).
- **`J` and `K` swap the selected entry with its neighbour** (`move_card` in
  `src/app/remember.rs`) and write every remembered card's index as its position at once
  (`keep_positions`); a running session's position is rewritten with the rest of its file on
  every read (`remember_all`, `wanted`). Recall sorts by position.
- **Stopped cards come back in one `recall`**, pushed after any running session.
- **The sidebar scrolls by whole cards** (`Sidebar::scroll`): the selected card stays whole and
  the offset moves only when it would leave the view.
- **The demo already has two groups**: two sessions in `acme-api` and two in `acme-web`.

## Decided

- The orchestrator's decisions are in issue #190 and the build brief; these are the builder's.
- The order lives in `App::sessions`: after every change that can break it (a start, a recall, a
  read that changed a place) a stable regroup pulls each card behind the first card of its
  repository, and the selection follows its card. When it moved a card, every remembered
  card's position is written as `J` and `K` write it, so the saved order stays the shown one.
- Grouping and moving are pure functions over the cards' repositories (`src/group.rs`); the
  header and the shortened third line are drawn by `Sidebar::draw`, which groups adjacent cards
  of one repository.
- A card other than a group's first moves only inside its group: `K` on the second card makes it
  the first, `J` on the last does nothing. The first card moves the group past the next or
  previous unit (a lone card or a whole group).
- The header starts where a card's text starts (two columns in), dimmed, the name, a space and
  `─` to the sidebar's inner edge; when the name alone fills the room it is cut with `…` and no
  rule shows.
- The sidebar scrolls by items (headers and cards): selecting a group's first card shows its
  header too.

## States

| What | State | What the user sees |
| --- | --- | --- |
| repository | one card | no header, the full third line |
| repository | two or more cards | a header, then its cards, third lines without the repository |
| repository | outside any repository | grouped by folder name like a repository |
| card | running, exited or stopped | grouped the same way |
| card | its session moves to another repository | it leaves its group and joins the other's, behind its first card |
| card | removed (`x`, `X`) leaving one in its group | the header goes, the last card's full third line comes back |
| card | started in a repository that has a card | it joins that group at its end, selected |
| key | `J` / `K` on a group's first card | the whole group moves past the next or previous unit |
| key | `J` / `K` on another card of a group | swaps inside the group; nothing at the group's end |
| key | `J` / `K` on a lone card | moves past the next or previous unit (a whole group) |
| key | `j`, `k`, the prefix then a digit, the finder | follow the order on screen; headers are never selected |
| remembered | positions saved before this phase, cards of one repository apart | recalled grouped, positions rewritten on the next move or regroup |
| sidebar | narrowest (15 columns, 11 for the header) | the name cut with `…`, the rule only when a column is left after a space |
| name | CJK (`東京リポ`, 2 columns each) | cut by display width, never half a character; the rule fills to the edge |
| sidebar | taller than the cards | headers take one row each, no blank between a header and its first card |
| sidebar | shorter than the cards | scrolls by items, the selected card whole, a group's first card with its header |
| `nerd_font` | on | a grouped card's third line keeps the branch and pull request glyphs, no repository glyph |
| layout | zoomed, side by side, narrow showing the pane | no sidebar, nothing changes |

## Tasks

1. The order: `group::order` and `group::step` (`tests/group.rs`).
2. The header and the third line without the repository (`tests/card/`).
3. The app keeps its cards grouped: a start, a recall, a moved place; jump and finder follow
   (`tests/app/grouping.rs`).
4. `J` and `K` by group, positions saved (`tests/app/reordering.rs`).
5. The scroll by items with headers (`tests/card/`).
6. The end-to-end test, the guides, the fragment, the screenshot.

## Outcome

Cards of one repository (a folder's name outside one) sit together in the sidebar under a dimmed
header, the name and a rule to the edge, and drop the repository from their third line; a lone
card is drawn as before. `App::sessions` holds the order on screen: a stable regroup after every
start and every frame's reads (a recall, a session that moved) pulls each card behind the first
of its repository, the selection following its card, and writes the remembered positions when it
moved one, so `j`, `k`, the prefix then a digit and the finder follow the screen with no change
of their own. `J` and `K` go through `group::step`: inside a group a card moves only within it,
and a group's first card moves the whole group past the next lone card or group. The sidebar
scrolls by items, showing a group's header with its first card when the height allows.
`harbour --help` says it in two lines. The demo already had two groups (`acme-api`,
`acme-web`); `docs/details.png` and the other shots that show the demo's sidebar were rerendered.
Review round 1 found no must; two fixes landed in one ship: a group header could sit alone on the
sidebar's last row, now left out when its first card has no room below it, and a session that
moved repository just before a write (at quit, or before the frame's regroup) was saved at its
old position, now regrouped before every write. Its other findings (two unrelated repositories
sharing a name group together; the scroll can shift a row when a header appears above the view)
went to #216. Round 2 found no must; its findings (a moved session drags its new group up to its
own place, writes in the draw path, the order of rows an end-to-end test checks) went to #216
too, and the loop ended at the owner's 2-round limit. Phase 43's ready, merged and released, its third review round and its builder's last notice
were written into `plans/metrics.md`.

## Hurdles

- `test-seam` existing in-process tests start every session in one folder, so they all grouped: seven layout tests moved one row down for the header, and the `J`/`K` tests now start their two cards in two folders through a picker that hands out a list of directories in turn → code
- `test-seam` a test read the agent's recorded session id before the third session had written it and moved the second card instead; it now waits for the file of each start → code
- `tooling` `scripts/screenshots.sh` rewrites every image, and shots with no sidebar (zoom, changelog, side by side) differ by a few bytes of raster noise; only the images the change alters were kept → none
