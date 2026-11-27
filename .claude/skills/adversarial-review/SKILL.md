---
name: adversarial-review
description: Adversarial review of the harbour repository: attack the code, the running binary, the release pipeline and the process, report what is bad ranked by impact, and on request file each fix as a GitHub issue labelled `adversarial-review-finding`. Use when asked for an adversarial analysis, "what's bad about this repo", "tear it apart", "don't take it easy", or to turn such findings into issues.
---

# Adversarial review

Two phases. Phase 1 always ends in a report in chat. Phase 2, filing issues, runs only when the user
asks for it, as its own step.

## Ground rules

- **Read-only until phase 2.** No commits, no pushes, no GitHub writes, no calls to outside
  services beyond reading GitHub. Probes live in the scratchpad or as temporary test files that are
  deleted before the report; end with `git status --short` clean and say so.
- **This repository is public** (root `AGENTS.md`). Repros, issues and examples use invented
  `acme/*` data and `<owner>/<repo>`, never a real organisation, login or host.
- **Read `docs/known-limits.md` first.** A known limit is reported only when it shows new
  impact, and the finding says what is new (as that file allows).
- **Severity by reachability**, as `docs/process.md` defines it under "Review before ready": the user
  hits it, the release pipeline hits it, or a pull request reaches code or a secret. Rank the rest
  lower.
- **Evidence or it is not a finding.** Each claim is reproduced (say how) or cites `file:line`.
  Keep "reproduced", "read in the code" and "inferred" apart; never state what was not checked.
- **Be fair.** End the report with what held up, so the rest can be trusted.

## Phase 1: attack

1. **Survey.** `git log --oneline | head -20`, line counts of `src/`, `tests/`, `scripts/`, the
   `AGENTS.md` files and `PLAN.md`; the releases and the open issues (`gh release list`,
   `gh issue list`). A finding already tracked cites its issue instead of being re-reported as new.
2. **Build and test everything.** `cargo build --all-targets` and `cargo build --release`, then
   `cargo test --no-fail-fast -- --include-ignored`. A shallow clone or one without tags reads as
   "never released" to the release scripts: `git fetch --unshallow --tags origin`, rerun, and count
   it as a contributor-experience note, not a code bug.
3. **Read the paths where bugs hide**, in this order:
   - `src/`: what each argument does, and what is read and silently dropped.
   - `scripts/` and `.github/workflows/`: `pull_request_target`, tokens, what runs from a pull
     request's head, guards that fail open, parsers of tool output.
4. **Attack the running binary.** Run `target/debug/harbour` and `target/release/harbour` with no
   argument, every documented argument, unknown and repeated arguments, empty strings and non-UTF-8
   bytes; check the exit code and which stream each output lands on. Debug panics on overflow that
   release wraps silently, so a debug panic points at a release bug. Then drive the interface at
   sizes from tiny to huge with `drive-demo.sh`, which runs `harbour --demo` in a scratch tmux
   server, sends the keys, prints `quit` when harbour exits 0 after the last key (`quit early` before
   it), `died: exit <code>` with stderr when it exits otherwise, or `alive`, and leaves the screen in
   `screen.txt`:

   ```bash
   keys=(j k J K C-_ '?' Escape C-_ c Escape C-_ u Escape C-_ z C-_ z C-_ '\' C-_ '}' C-_ '{' C-_ '\' C-_ - C-_ = C-_ l C-_ h C-_ e C-_ h C-_ 1 a C-_ h Enter a C-h C-l C-h C-_ s Escape C-_ r Escape C-_ C-_ C-_ p C-h n Escape C-c n x y X y C-_ q y)
   for size in "1 1" "2 2" "3 3" "30 1" "200 3" "300 80"; do .claude/skills/adversarial-review/drive-demo.sh target/debug/harbour $size "${keys[@]}"; done
   ```

   `C-_` is how tmux names the `C-/` prefix. The keys open each view once (help, changelog, usage,
   zoom, side by side and its swaps, the widths, the focus keys, the editor, a jump, the finder, the
   rename popup, the picker), switch to the previous session and send the prefix on, type into a focused session and come back with `C-h`, move a card
   with `J` and `K`, answer no to Ctrl+C's quit question, end a session with `x` and `X`, then quit
   with the prefix, `q` and `y`; every size should print `quit`. The sequence follows `KEYS` of `src/help.rs`:
   whoever adds a key there adds it here, after `C-_` when it needs the prefix (`tests/skills.rs`
   fails otherwise). Run it under bash: zsh does not split `$size`. A panic in stderr is a bug.
5. **Attack the process.** Look for:
   - claims in the `AGENTS.md` files that the code contradicts
   - rules the code breaks
   - meta-tests that are text heuristics
   - guards that skip or pass on an unreadable input
   - steps that stage or publish more than they checked (`ship.sh`, `scrub-check.sh`)

## Report

In chat, not a file. Offer a shareable page in one line. Sections, most severe first:

1. **The worst:** anything that can leak private data, publish a wrong release, or block releases.
2. **Bugs reproduced:** each with how it was reproduced, `file:line`, and its impact.
3. **Design limits that will bite real users.**
4. **Process and repository hygiene.**
5. **What held up:** probes that found nothing.
6. **What I'd fix first:** a numbered list, one fix per item.

## Phase 2: file the fixes as issues (only when asked)

- **One issue per "fix first" item.** Split an item whose parts ship separately; fold a sibling
  with the same root cause into its issue. Say which in the final reply.
- **Labels.**
  - Every issue gets `adversarial-review-finding`, plus the existing labels that fit
    (`gh label list`).
  - Besides GitHub's default labels, `tui_repo` creates `adversarial-review-finding`,
    `development-cycle-improvement` and `Type: Maintenance` with the repository. When
    `adversarial-review-finding` is missing, create it with `gh label create` first.
- **Body**, following the issue forms (`.github/ISSUE_TEMPLATE/`):
  - `## What happens` for a bug, or `## What you are trying to do` / `## What gets in the way` for
    an enhancement: the repro, pinned to the commit sha.
  - `## Where`: files and functions.
  - `## What would fix it`.
  - `## Tests to start from`.
  - The fragment the fix needs (`kind: fix`, `feat`, `internal`, or none when nothing feeds the
    binary, per `scripts/feeds-binary.sh`).
  - Then `Found during an adversarial review of the repository at <sha>.`
- **No `@name` outside backticks** (write `` `@me` ``): GitHub notifies real logins.
- **Final reply:** a table of number, title and labels, then:
  - the label calls a maintainer may want to undo
  - what was merged or split
  - what was found but not filed
