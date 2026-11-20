#!/usr/bin/env bash
# Quality gates. Every step must pass; the script exits on the first failure and never
# swallows output, so do not pipe it through tail.
set -euo pipefail
cd "$(dirname "$0")/.."

step() {
  echo "==> $*"
  "$@"
}

step scripts/scrub-check.sh
step cargo fmt --check
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
