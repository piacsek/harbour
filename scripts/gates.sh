#!/usr/bin/env bash
# Quality gates. Every step must pass; the script exits on the first failure and never
# swallows output, so do not pipe it through tail.
set -euo pipefail
cd "$(dirname "$0")/.."

step() {
  echo "==> $*"
  "$@"
}

if ! command -v cargo-nextest >/dev/null; then
  echo "cargo-nextest is not installed (cargo install cargo-nextest --locked)" >&2
  exit 1
fi
nextest=(cargo nextest run --config-file .github/nextest.toml)

. scripts/gates-lock.sh
lock_gates
step scripts/scrub-check.sh
step cargo fmt --check
if [ -z "${CI:-}" ] && reason="$(scripts/docs-only.sh)"; then
  echo "==> skipping the tests: $reason; CI runs them" >&2
  step "${nextest[@]}" --test readme --test plans --test agents_size
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
  step "${nextest[@]}" --lib --bins ${targets[@]+"${targets[@]}"}
else
  step "${nextest[@]}"
fi
step cargo test --doc
if [ "$separate" = e2e ]; then
  step "${nextest[@]}" --test e2e --run-ignored all
else
  echo "==> skipping the ignored e2e tests: tmux is not installed (brew install tmux); CI runs them" >&2
fi
if ! command -v nvim >/dev/null; then
  echo "==> skipping the real-Neovim tests: nvim is not installed (brew install neovim); CI runs them" >&2
fi
echo "gates passed"
