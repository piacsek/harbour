# harbour plan

The index of phases and the backlog. Each phase's plan, outcome and hurdles live in its own
file under `plans/`; its time and tokens are a row of [`plans/metrics.md`](plans/metrics.md).

## Phases

| Phase | Plan | Release |
| --- | --- | --- |
| 0 | [the foundations, scaffolded from the `rust_tui` recipe](plans/phase-0-foundations.md) | [v0.0.1](https://github.com/piacsek/harbour/releases/tag/v0.0.1) |
| 1 | [agent sessions in a sidebar, each with a live Claude Code pane](plans/phase-1-session-pane.md) | [v0.1.0](https://github.com/piacsek/harbour/releases/tag/v0.1.0) |
| 2 | [the selected session and the focused side in cyan](plans/phase-2-cyan-highlight.md) | [v0.2.0](https://github.com/piacsek/harbour/releases/tag/v0.2.0) |
| 3 | [whether each agent is working, idle or waiting on you](plans/phase-3-status-dots.md) | [v0.3.0](https://github.com/piacsek/harbour/releases/tag/v0.3.0) |
| 4 | [choose the directory a new session starts in, by fuzzy search](plans/phase-4-directory-picker.md) | [v0.4.0](https://github.com/piacsek/harbour/releases/tag/v0.4.0) |
| 5 | [the counts in the tmux status line](plans/phase-5-tmux-status-line.md) | [v0.5.0](https://github.com/piacsek/harbour/releases/tag/v0.5.0) |
| 6 | [an editor beside each session's Claude Code pane](plans/phase-6-editor-column.md) | [v0.7.0](https://github.com/piacsek/harbour/releases/tag/v0.7.0) |
| 7 | [a faster macOS flake build: a Nix store cache, and no clippy or rustfmt in the flake's toolchain](plans/phase-7-flake-build-speed.md); the cache was dropped after measuring it | [v0.5.1](https://github.com/piacsek/harbour/releases/tag/v0.5.1) |
| 8 | [each session's title, activity, repository and branch in the sidebar](plans/phase-8-session-card.md) | [v0.6.0](https://github.com/piacsek/harbour/releases/tag/v0.6.0) |
| 9 | [no script dies of SIGPIPE when its reader stops early](plans/phase-9-pipe-race.md) | |
| 10 | [sessions that come back after harbour closes, resumed on demand](plans/phase-10-resumable-sessions.md) | [v0.9.0](https://github.com/piacsek/harbour/releases/tag/v0.9.0) |
| 11 | [tmux shortcuts that use the current directory follow the selected session](plans/phase-11-follow-selected-session.md) | [v0.8.0](https://github.com/piacsek/harbour/releases/tag/v0.8.0) |
| 12 | [open a file in a session's editor from outside harbour](plans/phase-12-edit-from-outside.md) | [v0.10.0](https://github.com/piacsek/harbour/releases/tag/v0.10.0) |
| 13 | [one review coordinator runs each pull request's review loop](plans/phase-13-review-coordinator.md) | |
| 14 | [a log of what harbour saw and decided, turned on by `HARBOUR_LOG`](plans/phase-14-log.md) | [v0.11.0](https://github.com/piacsek/harbour/releases/tag/v0.11.0) |
| 15 | [room for the key and layout features: keys, the split and `KEYS` each in a place of their own](plans/phase-15-room-for-keys-and-layout.md) | [v0.12.1](https://github.com/piacsek/harbour/releases/tag/v0.12.1) |
| 16 | [only the session counts above zero in the sidebar title](plans/phase-16-counts-above-zero.md) | [v0.12.0](https://github.com/piacsek/harbour/releases/tag/v0.12.0) |
| 17 | [a still sidebar: no activity line on the selected card, the last prompt on the others](plans/phase-17-still-sidebar.md) | [v0.13.0](https://github.com/piacsek/harbour/releases/tag/v0.13.0) |
| 18 | [the help view keeps the keys, `harbour --help` the notes](plans/phase-18-help-keeps-the-keys.md) | [v0.14.0](https://github.com/piacsek/harbour/releases/tag/v0.14.0) |
| 19 | [the pane's cursor takes the shape its program asks for](plans/phase-19-cursor-shape.md) | [v0.15.0](https://github.com/piacsek/harbour/releases/tag/v0.15.0) |
| 20 | [no cursor in the pane while the sidebar has focus](plans/phase-20-no-cursor-beside-the-sidebar.md), with the sweep of phases 15 to 19 | [v0.16.0](https://github.com/piacsek/harbour/releases/tag/v0.16.0) |
| 21 | [harbour's settings in one config file](plans/phase-21-config-file.md) | [v0.17.0](https://github.com/piacsek/harbour/releases/tag/v0.17.0) |
| 22 | [jump to a card by its number](plans/phase-22-jump-by-number.md) | [v0.19.0](https://github.com/piacsek/harbour/releases/tag/v0.19.0) |
| 23 | [icons beside Claude Code and Neovim in the pane's border](plans/phase-23-border-icons.md) | [v0.18.0](https://github.com/piacsek/harbour/releases/tag/v0.18.0) |
| 24 | [icons on a session's card for its repository, branch, pull request, subagents and editor](plans/phase-24-card-icons.md) | [v0.20.0](https://github.com/piacsek/harbour/releases/tag/v0.20.0) |
| 25 | [zoom: the selected session on the whole screen](plans/phase-25-zoom.md), with the sweep of phases 20 to 24 | [v0.21.0](https://github.com/piacsek/harbour/releases/tag/v0.21.0) |
| 26 | [Claude Code's usage and limits in a popup](plans/phase-26-usage-popup.md) | [v0.22.0](https://github.com/piacsek/harbour/releases/tag/v0.22.0) |
| 27 | [Alt-minus and Alt-equals change the sidebar's width](plans/phase-27-width-keys.md) | [v0.23.0](https://github.com/piacsek/harbour/releases/tag/v0.23.0) |
| 28 | [jump to any session by typing part of its name](plans/phase-28-jump-by-name.md) | [v0.24.0](https://github.com/piacsek/harbour/releases/tag/v0.24.0) |
| 29 | [Claude Code and its editor side by side](plans/phase-29-side-by-side.md) | [v0.24.0](https://github.com/piacsek/harbour/releases/tag/v0.24.0) |
| 30 | [swap Claude Code and the editor in side by side](plans/phase-30-swap-halves.md), with the sweep of phases 25 to 29 | [v0.25.0](https://github.com/piacsek/harbour/releases/tag/v0.25.0) |
| 31 | [the usage popup draws its report as bars](plans/phase-31-usage-bars.md) | [v0.26.0](https://github.com/piacsek/harbour/releases/tag/v0.26.0) |
| 32 | [read the changelog inside harbour with c](plans/phase-32-changelog-view.md) | [v0.27.0](https://github.com/piacsek/harbour/releases/tag/v0.27.0) |
| 33 | [a loading animation in the usage popup](plans/phase-33-usage-loading.md) | [v0.28.0](https://github.com/piacsek/harbour/releases/tag/v0.28.0) |
| 34 | [C-j and C-k move to the tmux pane below or above harbour](plans/phase-34-tmux-pane-navigation.md) | [v0.29.0](https://github.com/piacsek/harbour/releases/tag/v0.29.0) |
| 35 | [choose the glyph of the Claude pane](plans/phase-35-claude-icon.md) | [v0.30.0](https://github.com/piacsek/harbour/releases/tag/v0.30.0) |
| 36 | [zoom without borders](plans/phase-36-borderless-zoom.md) | [v0.31.0](https://github.com/piacsek/harbour/releases/tag/v0.31.0) |
| 37 | [one prefix key, C-/, for harbour's actions from anywhere](plans/phase-37-prefix-key.md) | [v0.32.0](https://github.com/piacsek/harbour/releases/tag/v0.32.0) |
| 38 | [colour the card's facts, dim the age and the cards that ended](plans/phase-38-card-colours.md) | [v0.33.0](https://github.com/piacsek/harbour/releases/tag/v0.33.0) |
| 39 | [X ends a running session and removes its card](plans/phase-39-end-removes-card.md) | [v0.34.0](https://github.com/piacsek/harbour/releases/tag/v0.34.0) |
| 40 | [reorder sidebar cards with J and K](plans/phase-40-reorder-cards.md) | [v0.35.0](https://github.com/piacsek/harbour/releases/tag/v0.35.0) |
| 41 | [Ctrl+C quits harbour from the sidebar](plans/phase-41-ctrl-c-quits.md) | [v0.36.0](https://github.com/piacsek/harbour/releases/tag/v0.36.0) |
| 42 | [C-; as a prefix setting](plans/phase-42-ctrl-semicolon-prefix.md) | [v0.37.0](https://github.com/piacsek/harbour/releases/tag/v0.37.0) |
| 43 | [rename a card with the prefix then r](plans/phase-43-rename-card.md) | [v0.38.0](https://github.com/piacsek/harbour/releases/tag/v0.38.0) |
| 44 | [group the cards of one repository](plans/phase-44-group-by-repository.md) | [v0.39.0](https://github.com/piacsek/harbour/releases/tag/v0.39.0) |
| 45 | [old ended cards tidy themselves away](plans/phase-45-tidy-ended-cards.md), with the sweep of phases 41 to 44 | [v0.40.0](https://github.com/piacsek/harbour/releases/tag/v0.40.0) |
| 46 | [Ctrl may stay held for the key after the prefix](plans/phase-46-ctrl-held-after-prefix.md) | [v0.41.0](https://github.com/piacsek/harbour/releases/tag/v0.41.0) |
| 47 | [choose where the editor opens](plans/phase-47-editor-dir.md) | [v0.42.0](https://github.com/piacsek/harbour/releases/tag/v0.42.0) |
| 48 | [the zoomed top row shows the repository and branch in their card colours](plans/phase-48-zoom-fact-colours.md) | [v0.43.0](https://github.com/piacsek/harbour/releases/tag/v0.43.0) |
| 49 | [each card shows its session's tasks done](plans/phase-49-card-tasks.md) | [v0.44.0](https://github.com/piacsek/harbour/releases/tag/v0.44.0) |
| 50 | [the task count stays right when a task list is cleared](plans/phase-50-task-count-edges.md), with the sweep of phases 46 to 49 | [v0.44.1](https://github.com/piacsek/harbour/releases/tag/v0.44.1) |
| 51 | [tests that open terminals share a few slots across the machine](plans/phase-51-pty-slots.md) | |
| 52 | [Claude Code's repaints show as whole frames](plans/phase-52-synchronized-output.md) | [v0.44.3](https://github.com/piacsek/harbour/releases/tag/v0.44.3) |
| 53 | [the prefix twice switches back to the previous session](plans/phase-53-previous-session.md) | |

## Backlog

### Later phases

In this order unless a retro says otherwise.

- One git worktree per session: harbour creates it, and the session and its editor both start in
  it.
- What each session is doing, in the sidebar: its current step and its subagents.
- Sessions that outlive harbour and are picked up again on the next start.

### Fixes to weigh

- Give every file an entry in its directory's guide: `src/app/read.rs` has none in `src/AGENTS.md`,
  nor `tests/demo.rs`, `tests/editor.rs`, `tests/nvim.rs`, `tests/registry.rs` and `tests/state.rs`
  in `tests/AGENTS.md`; a test that every `src/**/*.rs` file is named there would keep it so.
- With the log off, skip the app's log bookkeeping on every key, paste, mouse event and frame
  (each session's and editor's panic count is taken under its parser lock). Needs a seam a test
  can see it through (phase 14 review).
- Replace the terminal parser. `vt100` panics on a wide character cut by a resize and gives some
  emoji the wrong width; harbour contains its panics and keeps the modes the program turned on,
  but each one blanks a session's screen until it repaints. Weigh `alacritty_terminal` as the candidate.
- Decide the findings of the sweep before phase 35, which the owner parked: missed states in
  round 1 of every phase, the parallel builds' collisions over `PLAN.md` rows and a phase
  number, tasks that could not ship apart, two token cells left `unknown`, four one-offs
  ([phase 35](plans/phase-35-claude-icon.md)).
- Make `picker::a_slow_list_holds_up_neither_harbour_nor_the_picker` wait for what cannot race:
  under the gates' load its picker reads `x`, exits and the session starts before the screen
  shows `acme picker read x` ([phase 53](plans/phase-53-previous-session.md)).

### Bring from conveyor when the interface exists

These parts of conveyor need a product to show or drive, so phase 0 left them out. Bring each
over, renamed and with its tests, in the phase that gives harbour that interface.

- `tests/stage_dispatch.rs`, once harbour has a dispatch to keep in one place.
- `scripts/screenshots.sh` takes the names of the shots to render, so a change that touches one
  view leaves the other images alone.
