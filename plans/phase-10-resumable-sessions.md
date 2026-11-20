# Phase 10: sessions that come back after harbour closes, resumed on demand

## Context

Every session ends with harbour today: quitting, a crash, a closed terminal tab or a reboot
loses the sidebar, and nothing brings it back. Issue #10 asks harbour to remember each session
that has a Claude Code conversation and to show it again, on the next start, as a stopped card
that Enter resumes with `claude --resume <id>` in its directory. Nothing is saved at quit, so a
crash loses nothing; one small file per session means two harbours never rewrite the same file.

## Verified facts (2026-11-20 UTC, `main` at 5eec30a, Claude Code 2.1.329)

Read from `claude --help` and from the field names and value types of the files on this
machine; no `claude` was started, and nothing below quotes a value.

- `claude --resume <value>` (`-r`) resumes a conversation by session id or name, "or pick from a
  list (optionally with a search term)". So an id Claude Code does not know may open its picker
  with the id as the search term instead of failing; harbour therefore resumes only an id whose
  transcript it finds (see States), and never relies on that path.
- `--fork-session` exists to give a resumed conversation a new id "instead of reusing the
  original": without it, a resumed session keeps its id. `--session-id <uuid>` must be a valid
  UUID.
- A registry file `<config>/sessions/<pid>.json` (`<config>` is `CLAUDE_CONFIG_DIR`, else
  `~/.claude`) holds `pid` (integer), `sessionId` (string, 36 characters, four hyphens: a UUID),
  `cwd` (string), `startedAt`, `updatedAt` and `statusUpdatedAt` (integers), `procStart`
  (string), `status`, and a dozen other fields. `App::read_files` already reads `sessionId` from
  the file named after a session's pid, at most every 500 ms.
- The transcript is `<config>/projects/<folder>/<sessionId>.jsonl`; the folder is derived from
  the directory the session started in, and harbour finds it by id alone (phase 8). The id in the
  registry changes when the user runs `/clear` or `/resume` inside a session (phase 8 follows the
  new transcript for that reason).
- Not verified without starting `claude`: whether `--resume <id>` finds a conversation started
  in another directory. Harbour sidesteps it by resuming in the directory the session started in,
  the one the transcript's folder is derived from.
- The state directory rules exist in `src/summary.rs` (phase 5): an absolute `XDG_STATE_HOME`,
  else an absolute `HOME` joined with `.local/state`, else nothing; folders `0700`, refused when a
  symlink; files `0600`, written to a temporary name and renamed.
- `~/.local/state` exists on this machine; harbour's own folder there holds `status/` only.

## How harbour remembers

- One file per conversation: `<state>/harbour/sessions/<sessionId>.json`, a JSON object
  `{"version":1,"sessionId":…,"directory":…,"title":…,"position":…}`. The name is the id, so a
  conversation is one file however many harbours saw it.
- A running session is remembered once the registry gives its id and the transcript for that id
  is found. The file is written when the id, title, directory or position differs from what this
  harbour last wrote for it, checked with the registry read (at most every 500 ms). When the id
  changes, the new file is written and the old one removed.
- Writing: the folder is made `0700` and refused when it is a symlink; the file is written to a
  fresh `0600` temporary name in the folder and renamed over `<id>.json`, so a reader sees the old
  file or the new one, never half of one. A file of a newer version is never overwritten or
  removed.
- Reading, once at start: every `<id>.json` whose name and `sessionId` agree, whose id is a plain
  name (ASCII letters, digits and hyphens, at most 64), whose `directory` is absolute and whose
  `version` is 1; each read is at most 64 KiB of a regular file, never through a symlink. Anything
  else is skipped and left on disk. The cards are ordered by `position`, then by id.
- Nothing is written at quit. `x` on a stopped card removes its file.

## States

| State | What the user sees | Test |
| --- | --- | --- |
| no state folder, or no file | nothing new: the sidebar starts empty, the hint shows | app |
| one file, everything in place | a stopped card at its position: `○`, its title, `stopped` on line 2 in dim, its repository and branch on line 3; the pane says Enter resumes it; nothing is started | app |
| Enter or `Ctrl-l` on a stopped card | `<command> --resume <id>` starts in its directory, the pane takes focus, the card runs like any other | app, e2e |
| its directory is gone | line 2 says `directory gone`; Enter does nothing; `x` forgets it | app |
| the transcript is gone (Claude Code no longer has it) | the card is not shown; its file stays on disk, so a harbour pointed at another `CLAUDE_CONFIG_DIR` cannot wipe it; the same check runs at Enter, which then drops the card | app |
| the id is running elsewhere: a registry file whose pid is alive names it (another harbour, a plain terminal, or another card of this harbour) | line 2 says `open elsewhere`; Enter starts no second copy; `x` hides the card in this harbour and keeps the file the running one owns; checked at start, with every registry read, and again at Enter | app |
| the other harbour quits | with the next registry read the card says `stopped` again and resumes | app |
| a registry file left by a crash whose pid now belongs to another process | the card says `open elsewhere` until that process ends: the safe side, no second copy (known limit) | none, documented |
| `claude --resume` exits with an error | the card shows the exit on line 2 as today and stays resumable; the pane shows what the program printed | app |
| a running remembered session exits by itself, or is ended with `x` then `y` | an exited card, resumable with Enter; `x` then forgets it | app |
| resumed twice | the second Enter on a running card focuses the pane, as today; no second process | app |
| `/clear` or `/resume` inside the session (its id changes) | the file of the new id is written, the old one removed | app |
| two harbours at once, different conversations | each writes only its own ids' files; neither rewrites the other's | memory |
| two harbours, same conversation | the second sees it `open elsewhere` and does not resume it | app |
| harbour killed by a signal, or its tab closed | the files are as of the last change; the next start shows the cards | e2e (quit, start again) |
| a temporary file left by a crash between write and rename | ignored: not a `<id>.json` name | memory |
| a file half written by a foreign tool, malformed JSON, a wrong field type, a relative or non-UTF-8 directory, an id that is not a plain name, a name that disagrees with its `sessionId`, larger than 64 KiB, a symlink, a folder, unreadable | skipped, never removed; every good file still shows | memory |
| a file from a newer harbour (`version` above 1) | skipped, never overwritten or removed | memory |
| `HARBOUR_SESSION_COMMAND` names another program | resuming runs that program with `--resume <id>`; a program that writes no registry file with a `sessionId` and no transcript is never remembered | app, e2e |
| no `HOME` and no `XDG_STATE_HOME` | nothing is remembered or read | memory |
| the state folder is a symlink, or cannot be made | nothing is written; the sessions run as today | memory |
| many remembered sessions | the sidebar scrolls as today; `j`/`k` reach every card | app |
| narrowest sidebar (60 columns, 18 inside) and narrow layout (under 60) | line 2's `directory gone`, `open elsewhere` and `stopped` are cut by `src/text.rs`; a CJK title fits | app |
| a title or directory holding control characters, bidi marks or a CJK name | cleaned and cut by `src/text.rs` like any card | app |
| `q` with a running session | `Quit? y/n` as today, and a line above it saying sessions come back stopped | app |
| `q` with only stopped cards | quits at once, as with only exited sessions today | app |
| counts in the sidebar title and `harbour status` | a card that never ran in this harbour counts nowhere, so a session open in another harbour is not counted twice; an exited one counts as today | app |
| the directory picker, the mouse | unchanged; a click in the pane of a card with no process does not focus the pane | app |

## Keys

| Key | Running card | Exited card | Stopped card |
| --- | --- | --- | --- |
| Enter, `Ctrl-l` | focus the pane | resume it when remembered, else focus the pane | resume it (nothing when its directory is gone or it is open elsewhere) |
| `x` | ask, then end it | remove it and forget it | forget it (only hide it when open elsewhere) |
| `q` | ask, saying sessions come back | quit | quit |

## Decisions

- **The issue's open readings, taken conservatively without the owner**:
  - A card whose transcript is gone is not shown, but its file is kept: dropping files on a
    missing transcript would wipe every memory the day harbour runs with another
    `CLAUDE_CONFIG_DIR`.
  - `x` on a card open elsewhere hides it here and leaves the file, which the running harbour
    owns.
  - Resuming runs in the directory the session started in, not the last `cwd` of its
    transcript, since the transcript's folder is derived from the start directory.
  - A stopped card that never ran in this harbour is not counted in the title or by
    `harbour status`, so two harbours never count one conversation twice.
  - Resuming runs the session command (`claude`, or `HARBOUR_SESSION_COMMAND`) with
    `--resume <id>`, so a scripted program stands in for Claude Code in every test.
- **The code**: `src/memory.rs` reads and writes the files; `src/app/remember.rs`, a child module
  of `src/app.rs`, holds what the app does with them. `src/app.rs` stays where it is with small
  hooks, so the phase 6 branch, which moves it to `src/app/mod.rs`, merges with little to
  resolve.
- **On the draw thread**: the files are read once at start; a write is a few hundred bytes, only
  when something changed; "open elsewhere" lists the registry folder and reads each file (at most
  64 KiB) once per registry read while a card is stopped; the transcript search lists
  `projects/` once per stopped card at start and at Enter. A slow file system stalls the
  interface for that long: a known limit.

## Tasks

One red-green-refactor cycle each, shipped with `scripts/ship.sh`.

1. This plan, its rows in `PLAN.md` and `plans/metrics.md`, and the phase 8 backfill.
2. `src/memory.rs`: write, read and forget, with every file state of the table; the fragment.
3. Stopped cards at start, resume with Enter and `Ctrl-l`, `x` forgets, the pane's message.
4. Running sessions are remembered and followed (id change, title, position).
5. Directory gone, transcript gone, open elsewhere.
6. Quit line, counts, mouse, narrow layout.
7. The binary wires it in (e2e: quit, start again, resume), help, README, `AGENTS.md`.
8. The demo shows a stopped card, the screenshots are rerun.

## Outcome

### Sweep

Range: phases 6 to 9, read from `scripts/sweep-report.sh` and each plan's `## Outcome` and
`## Hurdles`; phases 5 and 7 to 9 are on `main`, phase 6 (PR #18) was still in review, so its
hurdles were read from its branch and may still change. The builder ran this sweep without the
owner: every line is a **proposal awaiting the owner's yes**, and no `AGENTS.md` changed for it.
The report counted 18 hurdles: `review-loop` 6, `test-seam` 5, `ci` 3, `process` 2,
`tooling` 2; four went `→ none`.

| # | Finding | Phases | Proposal (awaiting the owner) | Where |
| --- | --- | --- | --- | --- |
| 1 | A test green on its first run, proved only by breaking the code by hand (`wait_for` returns after 5 s without failing; the tui wiring), each `→ none` | 8, and 1 to 3 before | Phase 5's proposal 1 came back: take it. A green first run is not red: break the code it covers, show the red run, restore, say so | root `AGENTS.md`, "TDD is never skipped" |
| 2 | Waits in tests that time out without failing: phase 8 made it a rule that every `wait_for` asserts on what it returned; phase 6 still hit a wait that timed out under load (`→ none`) | 8, 6 | Issue: make `wait_for` and `wait_for_all` fail on timeout, naming the text they waited for, so the rule is enforced by the harness, not by review | issue |
| 3 | Tests that depend on the checkout's folder name: phase 2, phase 8 round 2 (became a rule in `tests/AGENTS.md`), phase 6 again despite the rule | 2, 8, 6 | Issue: a test (like `tests/source_size.rs`) that runs the in-process and e2e suites from a checkout whose folder name differs, or a gate that greps tests for `current_dir`/`CARGO_MANIFEST_DIR` folder names; the rule alone did not hold | issue |
| 4 | Review rounds that found states of an outside program the plan's table missed: fzf and directories (phase 4), locks and hosts (phase 5), Neovim's states over three of phase 6's rounds | 4, 5, 6 | Phase 5's proposal 8 came back, a third time: take it. A plan's state table lists, for each thing read from outside harbour, what the user sees when it is missing, changed in between, or shared | root `AGENTS.md`, "Plan before code" |
| 5 | Width arithmetic in more than one place (phase 8 round 1's emoji, phases 1 and 3 before) | 1, 3, 8 | Phase 5's proposal 4 is now mostly done by code: `src/text.rs` is the one place widths are measured and cut. Narrow the rule to "a plan that draws text adds the narrowest sidebar and a CJK name to its state table"; decline the rest | root `AGENTS.md`, "Plan before code" |
| 6 | `main` moved mid-phase and conflicted in `PLAN.md` and `plans/metrics.md`; a brief named a wrong test file | 7 | Decline: one occurrence each, and the merge rule already covers catching up | none |
| 7 | Prose through a shell argument blocked by the em dash hook | 2 | Phase 5's proposal 2 did not come back in phases 6 to 9: decline it unless it recurs | none |

Metrics (from `plans/metrics.md`): review rounds 5, 2, 4, 1 for phases 5, 7, 8, 9 (phase 6 not
merged); start to release 1 h 50 min, 56 min, 2 h 49 min (phase 8, read from GitHub in this
phase), and 35 min to merge for phase 9, which released nothing. Claude reviewer tokens per round
stay between 97k and 158k, with phase 8 rising over its rounds (136k, 98k, 154k, 158k); builder
notices per phase 10, 2, 5, 2. The rounds track the size of the change, not a drift; Codex and
orchestrator usage are still unknown in every row, so no trend in total tokens can be read.

## Hurdles
