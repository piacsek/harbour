# Phase 8: each session's title, activity, repository and branch in the sidebar

## Context

With several sessions open, across repositories, the user cannot tell them apart or see what
each one is doing without opening it: a row is a dot and the name of a directory with a number.
Issue #9 asks for a three-line card per session, read from files Claude Code already writes:
the title and how long the session has been in its state, what it is doing or waiting for (or
the last prompt, or its exit), and the repository, branch, pull request and running subagents.

## Verified facts (2026-11-20 UTC, `main` at 77f6354, Claude Code 2.1.325 to 2.1.329)

Read from the files on this machine; no `claude` was started, and nothing below quotes them.

- The registry file `<config>/sessions/<pid>.json` (`<config>` is `CLAUDE_CONFIG_DIR`, else
  `~/.claude`) holds `sessionId` (string), `cwd` (string, the directory the session started in),
  `status` and `statusUpdatedAt` (integer, milliseconds since the epoch, the moment `status` last
  changed), among a dozen other fields. `waitingFor` is a string when present.
- The transcript is `<config>/projects/<folder>/<sessionId>.jsonl`, one JSON object per line,
  appended to while the session runs. The folder is derived from the start directory; the issue
  finds it by `sessionId` alone (`projects/*/<sessionId>.jsonl`), which needs no rule for the
  folder name. Two of four live sessions had no transcript yet (no prompt typed).
- Transcripts reach 46 MB here, and a single line 2.2 MB (an `api-request-blob`). The lines
  the card needs are short and frequent: in one 6.8 MB transcript, 348 `ai-title` and 370
  `last-prompt` lines among about 4900.
- Line shapes, each with a `type` field:
  - `{"type":"ai-title","aiTitle":"…"}`, the title, written again as it changes;
  - `{"type":"last-prompt","lastPrompt":"…"}`, the user's last prompt, possibly many lines;
  - `{"type":"pr-link","prNumber":12,"prUrl":"…","prRepository":"…"}`;
  - `{"type":"assistant","message":{"content":[…]}}`, where a tool call is a content item
    `{"type":"tool_use","name":"Bash","input":{…}}`; `input` holds `description` (Bash, Agent,
    Task tools), `command` (Bash) or `file_path` (Read, Edit, Write), or none of them (for
    example `ToolSearch`);
  - message lines (`user`, `assistant`, `system`, `attachment`) carry `gitBranch` (the branch,
    `"HEAD"` when detached) and `cwd` (the session's current directory).
- Subagents: `<config>/projects/<folder>/<sessionId>/subagents/agent-<id>.meta.json`, a small
  object with `"finished": true` once the subagent ends; one still running has no `finished`.
  Of 492 meta files here, a handful have none, some left by sessions killed long ago.
- Git without running git: a repository's folder holds `.git`, a directory; a worktree's holds
  `.git`, a file `gitdir: <main>/.git/worktrees/<name>`, whose folder holds `commondir`
  (`../..`) and its own `HEAD`; a submodule's `.git` file points into
  `<super>/.git/modules/<name>`, with no `commondir`; a bare repository is a folder holding
  `HEAD`, `objects` and `refs`, conventionally named `<name>.git`. `HEAD` reads
  `ref: refs/heads/<branch>` or, detached, a commit id.
- `App::read_registry` already reads the registry at most every 500 ms from `App::draw`, and
  `state::row` draws one line per session through a ratatui `List`, whose highlight symbol marks
  only the first line of a taller item.
- The sidebar takes a click only to focus itself; no click selects a row today, so a taller row
  changes nothing for the mouse.

## States

What each line of a card shows. The age on line 1 is right-aligned: `<1m`, then whole minutes
(`2m`), hours (`3h`) and days (`4d`).

| State | Line 1 | Line 2 | Line 3 |
| --- | --- | --- | --- |
| working (`busy`, `shell`) | yellow `●`, title, time since `statusUpdatedAt` | the last tool call: its description, else its command, else its file, else its tool name | repository · branch · `#<pr>` · `<n> subagents` |
| blocked (`waiting`) | red `◉`, title, age | `waitingFor` in red | as working |
| blocked, no `waitingFor` | red `◉`, title, age | empty | as working |
| idle | dim `○`, title, age | the last prompt, dimmed | as working |
| exited | dim `○`, title, time since harbour saw the exit | `exited`, `exited 3` or `exited (<signal>)` | as when it last ran, without subagents |
| unknown (no registry file, not Claude Code) | grey `·`, title, no age | empty | repository · branch, read from git |

What each source does to the card:

| Situation | What the user sees |
| --- | --- |
| no title yet (no transcript, or no `ai-title` line) | line 1 shows the directory name and number, as today |
| a title, then a transcript line that is not JSON, half written, or an `ai-title` whose `aiTitle` is not a string | the last good title stays; the same for every field |
| a field of an unexpected type (`prNumber` a string, `input` not an object) | that field is left off, or keeps its last value; the card never shows an error |
| a transcript line longer than 1 MiB | skipped; the fields keep their last values |
| a huge transcript | harbour reads only its last 1 MiB once, then only what is appended, at most 1 MiB per read (when more was appended, only the last 1 MiB of it) |
| a transcript being written (the last line has no newline yet) | the partial line waits for its newline, up to 1 MiB, then is dropped |
| a transcript truncated or replaced (shorter than what was read) | read again from its last 1 MiB |
| no transcript found | line 1 the name, line 2 by state from the registry, line 3 from git; the search is repeated at most every 5 s |
| the session's id changes in its registry file | the card forgets the old transcript's fields and follows the new one |
| a registry file left by another process with the same pid, or older than the session | ignored, as today, so no transcript of another session is followed |
| a very long title, activity, prompt, repository or branch | cut with `…` to the sidebar, never wrapped |
| CJK or emoji in any field | measured in display columns, cut on a character boundary, a wide character never split |
| control characters or escape sequences in any field | each is shown as a space, runs of whitespace (line breaks too) fold into one space; nothing reaches the terminal raw |
| a directory that is not a repository | line 3 shows its folder name |
| a repository | line 3 shows the folder holding `.git` |
| a worktree | line 3 shows the main repository's folder and the worktree's branch: `acme-api · fix-login` |
| a submodule | line 3 shows the submodule's folder |
| a bare repository | line 3 shows its folder without `.git` |
| a detached HEAD | the branch reads `HEAD`, as Claude Code records it |
| the branch from the transcript and from git differ | the transcript's `gitBranch` wins (the issue's source); git's `HEAD` fills in until a line has one |
| the directory deleted | line 3 shows the nearest repository above it, else the folder name it had |
| no pull request, no subagent running | that part is left out, with its separator |
| one subagent running | `1 subagent`; more read `<n> subagents` |
| a subagent meta file older than the session, unreadable, or not an object | not counted |
| a session that exited | its card keeps the last title and line 3; nothing is read for it again |
| the narrowest sidebar (a 60-column terminal: 16 columns inside the bar) | each line is cut with `…`; line 1 keeps the dot and the age while the title has at least one column |
| a terminal too short for one card | the selected card's top lines show, as many as fit |
| more cards than fit | the sidebar scrolls so the selected card is whole on screen; it moves only when the selection would leave the view |
| the narrow single-column layout (under 60 columns) | the same cards at the full width while the sidebar has focus |

The selected card has the cyan `▌` on all three lines and bold text; the blank line between cards
has no bar. The title counts, `harbour status`, the dot and its colours are unchanged.

## Decisions

- **The card is drawn by harbour, not by ratatui's `List`**, so the bar runs down all three lines
  and the scroll keeps a whole card in view. `src/card.rs` builds and draws the cards,
  `src/transcript.rs` follows a transcript and counts subagents, `src/repo.rs` reads git's files;
  `src/app.rs` changes by a few lines, as phase 6 edits the same file.
- **Line 3 is read for every session**, Claude Code or not, from git's files in the session's
  current directory (the transcript's last `cwd`, else the directory it started in). It is not
  read once the session has exited.
- **Git is read, never run**: `.git`, `gitdir`, `commondir` and `HEAD` are small files, read at
  most every 500 ms per session. No subprocess is started from the draw thread.
- **What a tool call shows**: its description, else its command, else its file path, else the
  tool's name, so a working session never shows a blank line while the transcript has a call.
- **Exited shows the time since harbour saw the exit**, measured from the read that found it.
- **Unknown shows no age**: harbour cannot tell when its state began.
- **Everything is read on the draw thread, bounded**: per running session, at most every 500 ms,
  the registry file, at most 1 MiB of the transcript, the subagents folder (a finished subagent is
  never read again) and git's small files; the search for a transcript lists `projects/` at most
  every 5 s. A slow or network file system stalls the interface for that long: a known limit.
- **The transcripts are found under the same config directory as the registry**
  (`CLAUDE_CONFIG_DIR`, else `~/.claude`); in-process tests hand both to `App` in scratch
  directories.
- **No new key and no click selection**: the help view, the README key table and the mouse stay
  as they are; the help legend says the card, not the row, says what a blocked session waits for.

## Tasks

One red-green-refactor cycle each, shipped with `scripts/ship.sh`.

1. This plan, its rows in `PLAN.md` and `plans/metrics.md`, and the phase 5 backfill.
2. Cards: three lines per session with the bar down all three, the name on line 1, the exit and
   the waiting reason on line 2, a blank line between, the scroll that keeps the selection in
   view; the fragment.
3. Line 3 from git: repository, worktree, submodule, bare, not a repository, deleted, branch.
4. The age on line 1 from `statusUpdatedAt` and from the exit.
5. The transcript: title, activity, last prompt, branch, current directory and pull request, read
   from its tail and followed.
6. Subagents still running.
7. The binary reads transcripts under `CLAUDE_CONFIG_DIR` (an e2e test), the help legend.
8. The demo shows a card in each state, and the screenshots are rerun.

## Outcome

## Hurdles
