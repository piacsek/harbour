#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
message="${1:?commit message}"
branch="$(git rev-parse --abbrev-ref HEAD)"
if [ "$branch" = "main" ]; then
  echo "refusing to ship on main" >&2
  exit 1
fi
cargo fmt
. scripts/gates-lock.sh
lock_gates
scripts/gates.sh
if reason="$(scripts/dev-install-skippable.sh)"; then
  echo "skipping dev-install: $reason"
else
  scripts/dev-install.sh
fi
release
git add -A
scripts/scrub-check.sh
if git diff --cached --quiet && [ -n "$(git rev-list HEAD --not --remotes)" ]; then
  echo "ship: nothing staged, pushing the unpushed commits as they are"
else
  git commit -q -m "$message"
fi
if ! scripts/scrub-check.sh --outgoing; then
  if git rev-parse -q --verify HEAD^2 >/dev/null; then
    echo "ship: nothing was pushed; HEAD is a merge, so keep it: fix what is named above in the files and git add them, then git commit --amend (with -m <message> for its message; git branch -m <new name> for the branch name), then ship again, which pushes the amended merge without a new commit" >&2
  else
    echo "ship: nothing was pushed; undo this commit with git reset --soft HEAD~1, fix what is named above (the files or the message of HEAD, git branch -m <new name> for the branch name; an earlier commit has to be rewritten by hand), then ship again" >&2
  fi
  exit 1
fi
title="${message%%$'\n'*}"
opening=
if ! gh pr view "$branch" --json number >/dev/null 2>&1; then
  opening=1
  if ! scripts/scrub-check.sh --title <<<"$title"; then
    echo "ship: nothing was pushed and no pull request opened; ship again with a message whose first line holds none of what is named above" >&2
    exit 1
  fi
fi
git push -q -u origin "$branch"
git --no-pager log --oneline -1
if [ -n "$opening" ]; then
  template=.github/pull_request_template.md
  if [ -f "$template" ]; then
    body=(--body-file "$template")
  else
    body=(--body "")
  fi
  gh pr create --draft --assignee @me --title "$title" "${body[@]}" >/dev/null
  echo "opened draft PR for $branch"
fi
