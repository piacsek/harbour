#!/usr/bin/env bash
# Quality gates. Every step must pass; the script exits on the first failure and never
# swallows output, so do not pipe it through tail.
set -euo pipefail
cd "$(dirname "$0")/.."

step() {
  echo "==> $*"
  "$@"
}

holder() {
  readlink "$1" 2>/dev/null || true
}

running() {
  local pid="${1%% *}" command
  case "$pid" in
    '' | *[!0-9]*) return 1 ;;
  esac
  kill -0 "$pid" 2>/dev/null || return 1
  command="$(ps -ww -p "$pid" -o command= 2>/dev/null)" || return 0
  case "$command" in
    *gates.sh*) return 0 ;;
  esac
  return 1
}

described() {
  if [ -n "$1" ]; then
    echo "the gates of pid ${1%% *} in ${1#* }"
  else
    echo "whoever holds $lock"
  fi
}

take_over() {
  local target="$1" stale="$2" guard="$1.takeover-${2%% *}" guarding
  if ! ln -sn "$mine" "$guard" 2>/dev/null; then
    guarding="$(holder "$guard")"
    if [ -n "$guarding" ] && ! running "$guarding"; then
      take_over "$guard" "$guarding"
      return
    fi
    return 1
  fi
  if [ "$(holder "$target")" = "$stale" ]; then
    rm -f "$target"
    if [ "$target" = "$lock" ]; then
      echo "==> taking over the gates lock of pid ${stale%% *} in ${stale#* }, whose gates no longer run" >&2
    fi
  fi
  rm -f "$guard"
}

release() {
  if [ "$(holder "$lock")" = "$mine" ]; then
    rm -f "$lock"
  fi
}

claim() {
  if [ -d "$lock" ] && [ ! -L "$lock" ]; then
    return 1
  fi
  ln -sn "$mine" "$lock" 2>/dev/null
}

lock_gates() {
  [ -z "${CI:-}" ] || return 0
  local bound="${HARBOUR_GATES_WAIT:-1800}" common told='' asked='' held
  case "$bound" in
    *[!0-9]*)
      echo "gates: HARBOUR_GATES_WAIT must be a whole number of seconds, not $bound" >&2
      exit 1
      ;;
  esac
  if ! common="$(git rev-parse --git-common-dir 2>/dev/null)"; then
    echo "==> no gates lock: not in a git repository" >&2
    return 0
  fi
  common="$(cd "$common" && pwd -P)"
  if [ ! -w "$common" ]; then
    echo "==> no gates lock: $common cannot be written" >&2
    return 0
  fi
  lock="$common/harbour-gates.lock"
  mine="$$ $(pwd -P)"
  trap release EXIT
  trap 'exit 129' HUP
  trap 'exit 130' INT
  trap 'exit 143' TERM
  SECONDS=0
  until claim; do
    held="$(holder "$lock")"
    if [ -n "$held" ] && ! running "$held" && take_over "$lock" "$held"; then
      continue
    fi
    if [ -z "$asked" ] || [ "$held" != "$told" ]; then
      echo "==> waiting for $(described "$held") to finish" >&2
      told="$held"
      asked=1
    fi
    if [ "$SECONDS" -ge "$bound" ]; then
      echo "gates: gave up after $bound s waiting for $(described "$held")" >&2
      exit 1
    fi
    sleep 1
  done
}

lock_gates
step scripts/scrub-check.sh
step cargo fmt --check
if [ -z "${CI:-}" ] && reason="$(scripts/docs-only.sh)"; then
  echo "==> skipping the tests: $reason; CI runs them" >&2
  step cargo test --test readme --test plans
  echo "gates passed"
  exit 0
fi
step cargo clippy --all-targets -- -D warnings
separate=scripts
if command -v tmux >/dev/null; then
  separate=e2e
fi
if [ -z "${CI:-}" ] && reason="$(scripts/script-tests-skippable.sh)"; then
  targets=()
  for file in tests/*.rs tests/*/main.rs; do
    [ -e "$file" ] || continue
    name="${file#tests/}"
    name="${name%/main.rs}"
    name="${name%.rs}"
    case "$name" in
      scripts | "$separate") ;;
      *) targets+=(--test "$name") ;;
    esac
  done
  echo "==> skipping the script tests: $reason; CI runs them" >&2
  step cargo test --lib --bins ${targets[@]+"${targets[@]}"}
  step cargo test --doc
else
  step cargo test
fi
if [ "$separate" = e2e ]; then
  step cargo test --test e2e -- --include-ignored --test-threads=2
else
  echo "==> skipping the ignored e2e tests: tmux is not installed (brew install tmux); CI runs them" >&2
fi
echo "gates passed"
