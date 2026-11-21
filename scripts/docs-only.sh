#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

run() {
  echo "$*"
  exit 1
}

base="$(git merge-base HEAD origin/main 2>/dev/null)" || run "no merge base with origin/main"
changed="$(git diff --name-only --no-renames "$base")"
untracked="$(git ls-files --others --exclude-standard)"
paths="$(printf '%s\n%s\n' "$changed" "$untracked" | sed '/^$/d')"
[ -n "$paths" ] || run "nothing on this branch differs from origin/main"
feeding="$(scripts/feeds-binary.sh <<<"$paths")"
while IFS= read -r path; do
  case "$path" in
    changes/* | .github/*) run "$path needs the tests" ;;
    *.md) if grep -qxF -- "$path" <<<"$feeding"; then run "$path needs the tests"; fi ;;
    *) run "$path needs the tests" ;;
  esac
done <<<"$paths"
echo "only Markdown outside the binary changed on this branch"
