# Phase 13: one review coordinator per pull request

## Context

Issue #28. In the autonomous flow the main session (the orchestrator) ran every review round
itself: one autonomous run shipped phases 2 to 12 in 41 rounds over 11 pull requests (phase 1,
before it, took 6 more), and every reviewer and builder report landed in its context. The owner chose option 1 of the issue: one sub-agent, the review
coordinator, runs the whole review loop of a pull request and reports to the orchestrator once
at the end. Option 2 (stop at the first round with no must finding) was not chosen: every must
and should finding is still fixed, and the last round is still unprimed. This phase changes only
the guidelines and `plans/metrics.md`; it feeds nothing into the binary.

## Verified facts (2026-11-20, `main` at 4190913, codex-cli 0.156.1)

Probed from a sub-agent (the builder of this phase), which is where a coordinator would run.

- **A sub-agent can launch Codex.** The `/pr-review` skill starts Codex through Bash, not
  through an agent type: `node <codex-companion.mjs> task --background --sandbox read-only
  "<prompt>"`, then `status` and `result <job-id>`. From this sub-agent that call printed
  `Codex Task started in the background as task-…`, exit 0; the job completed in 5 s and
  `result` returned `OK`. It ran without `dangerouslyDisableSandbox`, so whether the Bash
  sandbox is on in these sessions is not known; the skill still asks for it, and a coordinator
  passes it. What fails from a sub-agent is the `codex:codex-rescue` agent type, which the skill
  already forbids, not Codex itself. "Run it from the main session" in `AGENTS.md` was a reading
  of that rule, not a limit.
- **A sub-agent can spawn an agent.** The Agent tool from this sub-agent started a
  `general-purpose` agent that answered `PROBE-OK` (62706 `subagent_tokens` for a reply with no
  tool call: the floor of any agent's figure).
- **A sub-agent can resume an agent it started, with its context.** `SendMessage` to that
  agent's id resumed it, it answered `PROBE-RESUMED-OK`, and its completion notice (63498
  `subagent_tokens`) came back to this sub-agent, the sender.
- **Not verified: resuming an agent another agent started.** The builder is spawned by the
  orchestrator, so a coordinator resuming it is a sibling resume. `SendMessage` documents any
  agent id as an address, so it should reach the builder; whether the builder's completion
  notice then comes back to the coordinator or to the orchestrator was not tried, because a
  probe would have needed a live builder. Phase 13's own review is the first run that shows it.
- **The orchestrator's context can be read after the fact.** Its session transcript,
  `~/.claude/projects/<project>/<session>.jsonl`, holds every turn's `usage`; the main
  session's turns have `"isSidechain": false` and sub-agents live in a folder beside it. The
  context of a turn is `input_tokens + cache_read_input_tokens + cache_creation_input_tokens`.
  The session that ran phases 1 to 12 went from 68040 at its first turn to 705399 at phase
  12's clean review and 753099 at its last, with compactions at 2026-11-20 03:36:41Z and
  18:50:49Z (neither inside a phase's start to clean review).

## Design

- The orchestrator spawns one coordinator (latest Opus) per pull request once the builder has
  shipped the phase's last task (step 3 of "Work autonomously"), and hands it the PR, the
  repository and its worktree path, the intent of the change, the builder's agent id, and a
  pointer to `AGENTS.md`.
- The coordinator runs the `/pr-review` mechanics itself: a fresh Claude reviewer through the
  Agent tool, and Codex from its own Bash with the skill's command, every round: the skill's
  "from the main loop" does not bind a sub-agent. It merges the findings, rates them by the
  severity definition, sends each round's fixes to the builder with `SendMessage` (the builder
  keeps its context and ships once per round), takes the round as shipped when the PR head
  moves past the sha it reviewed (`gh pr view <N> --json headRefOid`), not from the builder's
  reply, and decides when the unprimed round is clean. It never posts the review.
- Two strikes, then redesign moves to the coordinator. A finding that needs a product decision,
  or a redesign that changes behaviour, ends its run at once with that question; the
  orchestrator asks the owner and resumes it with the answer.
- If the builder cannot be reached, the coordinator ends its run with the fix list, and the
  orchestrator relays it in one message and resumes the coordinator: the fallback for the
  unverified sibling resume. If the builder's reply or completion notice reaches the orchestrator
  instead, the orchestrator forwards it to the coordinator without acting on it, and adds those
  figures to the metrics row itself.
- Its single final report carries the rounds, the end of the clean round, the findings fixed,
  the minors left, any redesign call, and every `subagent_tokens` figure it received. The
  orchestrator adds the coordinator's own figure and its own context, read from its transcript.
- `plans/metrics.md` keeps its columns: the Rounds column already holds the rounds, and the
  Orchestrator column now holds its context at the phase's start and at its clean review,
  measured, backfilled for phases 1 to 12 from the transcript. The coordinator's own figure goes
  in the Notes cell.
- The first phase to run under the coordinator is this one, phase 13: its own review is the
  first coordinated loop. It is small (guidelines only), so the fair comparison is phase 9
  (scripts and guidelines, 1 round, orchestrator context 365492 → 411067). The first feature
  phase after it compares with phases 10 to 12.

## Tasks

1. This plan, its rows in `PLAN.md` and `plans/metrics.md`, and the backfill of phase 12's
   ready, merged and released cells.
2. `AGENTS.md`: step 3 of "Work autonomously", "Review before ready" and "Time and tokens" name
   the coordinator; the Orchestrator cells of `plans/metrics.md` carry the measured context.

## Outcome

## Hurdles
