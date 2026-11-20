#!/usr/bin/env bash
set -euo pipefail
binary="${1:?binary}"
binary="$(cd "$(dirname "$binary")" && pwd)/$(basename "$binary")"
width="${2:?width}"
height="${3:?height}"
shift 3
command -v tmux >/dev/null || { echo "drive-demo.sh needs tmux" >&2; exit 1; }
out="${OUT:-$(mktemp -d)}"
mkdir -p "$out/home/acme-api"
rm -f "$out/exit" "$out/stderr" "$out/screen.txt"
socket="harbour-drive-demo-$$"
t() { tmux -L "$socket" -f /dev/null "$@"; }
run="$(printf '%q ' env -u XDG_CONFIG_HOME -u XDG_STATE_HOME "HOME=$out/home" "$binary" --demo)"
trap 't kill-server 2>/dev/null || true' EXIT
t new-session -d -s demo -x "$width" -y "$height" -c "$out/home/acme-api" \
  "$run 2>$(printf '%q' "$out/stderr"); echo \$? >$(printf '%q' "$out/exit"); sleep 30"
sleep 1.2
for key in "$@"; do
  t send-keys -t demo "$key"
  sleep 0.25
done
sleep 0.5
t capture-pane -t demo -p >"$out/screen.txt"
if [ -f "$out/exit" ]; then
  echo "died: exit $(cat "$out/exit")"
  head -n 8 "$out/stderr"
else
  echo alive
fi
echo "$out"
