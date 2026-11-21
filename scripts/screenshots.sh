#!/usr/bin/env bash
# Usage: scripts/screenshots.sh [out-dir]   (needs tmux, freeze, Google Chrome, cargo build --release)
set -euo pipefail

command -v fzf >/dev/null || { echo "screenshots.sh: fzf is not on PATH; the picker shot needs it (brew install fzf)" >&2; exit 1; }

out="${1:-docs}"
root="$(cd "$(dirname "$0")/.." && pwd)"
bin="$root/target/release/harbour"
cargo build --release --manifest-path "$root/Cargo.toml"
mkdir -p "$out"
font=()
for f in "$HOME/Library/Fonts/FiraCodeNerdFont-Regular.ttf" /usr/share/fonts/truetype/firacode/FiraCodeNerdFont-Regular.ttf; do
  [ -f "$f" ] && font=(--font.file "$f") && break
done

sock="harbour-shots-$$"
home="$(cd "$(mktemp -d /tmp/harbour-shots.XXXXXX)" && pwd -P)"
trap 'tmux -L "$sock" kill-server 2>/dev/null || true; rm -rf "$home"' EXIT
t() { tmux -L "$sock" -f /dev/null "$@"; }

shot_window="$(t new-session -d -s main -x 160 -y 14 -c "$home" -P -F '#{window_id}')"
t set -g status off

cp "$(dirname "$0")/truecolor.py" "$home/truecolor.py"

chrome="${CHROME:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}"
[ -x "$chrome" ] || chrome="$(command -v google-chrome || command -v chromium || true)"

render() {
  local name="$1"
  shift
  local svg="$home/$name.svg"
  [ -s "$home/$name.ansi" ] || { echo "empty capture for $name" >&2; exit 1; }
  if [ -z "$chrome" ]; then
    freeze "$home/$name.ansi" -o "$out/$name.png" --theme catppuccin-mocha \
      --font.size 14 --margin 0 "$@" </dev/null
    return
  fi
  freeze "$home/$name.ansi" -o "$svg" --theme catppuccin-mocha ${font[@]+"${font[@]}"} \
    --font.family "FiraCode Nerd Font" --font.size 14 --margin 0 "$@" </dev/null
  # FiraCode has no ✗ ⊘ ▸; Chrome's default fallback is wider and shifts the rest of the row, so
  # the borders jog. Menlo has them at (almost) the same advance, and DejaVu is its Linux twin.
  sed -i.bak 's/font-family="FiraCode Nerd Font"/font-family="FiraCode Nerd Font, Menlo, DejaVu Sans Mono, monospace"/' "$svg"
  local w h
  w="$(grep -o 'width="[0-9.]*"' "$svg" | sed -n 1p | grep -o '[0-9]*' | sed -n 1p)"
  h="$(grep -o 'height="[0-9.]*"' "$svg" | sed -n 1p | grep -o '[0-9]*' | sed -n 1p)"
  "$chrome" --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=2 \
    --window-size="$w,$h" --screenshot="$out/$name.png" "file://$svg" >/dev/null 2>&1
}

wait_for() {
  local tries=0
  until t capture-pane -p -t "$shot_window" | grep -F "$1" >/dev/null || [ "$tries" -ge 40 ]; do
    sleep 0.25
    tries=$((tries + 1))
  done
  sleep 0.6
}

press() {
  local before tries sent=0
  before="$(t capture-pane -e -p -t "$shot_window")"
  until [ "$sent" -ge 3 ]; do
    t send-keys -t "$shot_window" "$1"
    sent=$((sent + 1))
    tries=0
    until [ "$(t capture-pane -e -p -t "$shot_window")" != "$before" ] || [ "$tries" -ge 20 ]; do
      sleep 0.25
      tries=$((tries + 1))
    done
    [ "$tries" -lt 20 ] && break
  done
  sleep 0.6
}

shoot() {
  local name="$1" width="$2" height="$3" args="$4" keys="$5" shown="${6:-}"
  t resize-window -t "$shot_window" -x "$width" -y "$height"
  t respawn-pane -k -t "$shot_window" "exec cat"
  t send-keys -R -t "$shot_window"
  t clear-history -t "$shot_window"
  t respawn-pane -k -t "$shot_window" "cd $(printf %q "$home/acme-api") && env -u XDG_CONFIG_HOME -u XDG_STATE_HOME -u FZF_DEFAULT_OPTS -u FZF_DEFAULT_OPTS_FILE -u HARBOUR_DIR_PICKER -u HARBOUR_SESSION_COMMAND -u HARBOUR_EDITOR -u CLAUDE_CONFIG_DIR HOME=$(printf %q "$home") TMPDIR=$(printf %q "$home/tmp") $(printf %q "$bin") $args; exec cat"
  sleep 1.5
  [ "$args" = --demo ] && wait_for "permission prompt"
  local key
  read -ra keys <<<"$keys"
  for key in ${keys[@]+"${keys[@]}"}; do
    press "$key"
  done
  [ -n "$shown" ] && wait_for "$shown"
  t capture-pane -e -p -t "$shot_window" | sed -E 's/harbour-demo-[A-Za-z0-9]{6}/harbour-demo-x7q2mf/g' | python3 "$home/truecolor.py" >"$home/$name.ansi"
  render "$name" --padding 20,64,20,20 --window --language text
}

mkdir -p "$home/acme-api" "$home/tmp"
shoot details 160 26 --demo "j" "Do you want to proceed?"
echo "wrote $out/details.png"
shoot pane 160 26 --demo "Enter"
echo "wrote $out/pane.png"
shoot keys 160 26 --demo "?"
echo "wrote $out/keys.png"
shoot question 160 26 --demo "x"
echo "wrote $out/question.png"
shoot quit 160 26 --demo "q" "come back stopped"
echo "wrote $out/quit.png"
shoot ended 160 26 --demo "j x y" "○ 3 · 1"
echo "wrote $out/ended.png"
shoot picker 160 26 --demo "n" "5/5"
echo "wrote $out/picker.png"
shoot editor 160 26 --demo "Enter C-l" "src/retry.rs"
echo "wrote $out/editor.png"
shoot usage 160 26 --demo "u" "Subagents: 7 runs"
echo "wrote $out/usage.png"
shoot narrow 56 26 --demo ""
echo "wrote $out/narrow.png"
t set -s extended-keys on
shoot zoom 160 26 --demo "M-z"
shoot side-by-side 160 26 --demo "M-\\" "src/retry.rs"
shoot sessions 160 26 --demo "M-s" "6/6"
t set -s extended-keys off
echo "wrote $out/zoom.png"
echo "wrote $out/side-by-side.png"
echo "wrote $out/sessions.png"
mkdir -p "$home/.config/harbour"
cat >"$home/.config/harbour/config.toml" <<'TOML'
accent = "#d7875f"
sidebar_width = 48
picker = ["pwd"]
session_command = ["sh", "-c", "printf 'acme agent ready in acme-api\\n'; exec sleep 600"]
TOML
shoot accent 160 26 "" "n" "acme agent ready"
rm -r "$home/.config"
echo "wrote $out/accent.png"
mkdir -p "$home/.config/harbour" "$home/bin"
cat >"$home/bin/nvim" <<'SH'
#!/bin/sh
printf 'pub fn retry(attempts: u32) -> Backoff {\n    Backoff::exponential(attempts)\n}\n'
exec sleep 600
SH
chmod +x "$home/bin/nvim"
cat >"$home/bin/agent" <<'SH'
#!/bin/sh
id="5d2c8e41-7a3f-4b9e-9c1d-8f0e6a2b4c7d"
projects="$HOME/.claude/projects/acme-api"
mkdir -p "$HOME/.claude/sessions" "$projects/$id/subagents"
printf '%s\n' '{"type":"ai-title","aiTitle":"Retry with backoff"}' '{"type":"pr-link","prNumber":128}' >"$projects/$id.jsonl"
printf '%s' '{"agentType":"Explore"}' >"$projects/$id/subagents/agent-1.meta.json"
printf '%s' '{"agentType":"Explore"}' >"$projects/$id/subagents/agent-2.meta.json"
printf '{"status":"busy","sessionId":"%s"}' "$id" >"$HOME/.claude/sessions/$$.json"
printf 'acme agent ready in acme-api\n'
exec sleep 600
SH
chmod +x "$home/bin/agent"
git -C "$home/acme-api" init -q -b fix-retry-backoff
cat >"$home/.config/harbour/config.toml" <<TOML
nerd_font = true
picker = ["pwd"]
editor = ["$home/bin/nvim"]
session_command = ["$home/bin/agent"]
TOML
shoot icons 160 26 "" "n Enter C-l" "Backoff::exponential"
rm -rf "$home/.config" "$home/bin" "$home/.claude" "$home/acme-api/.git"
echo "wrote $out/icons.png"
usage="$("$bin" --help)"
usage_rows="$(printf '%s\n' "$usage" | wc -l)"
shoot help 160 "$((usage_rows + 1))" --help ""
grep -qF -- "${usage%%$'\n'*}" "$home/help.ansi" || { echo "screenshots.sh: the help shot lost the top of harbour --help; give it more rows" >&2; exit 1; }
echo "wrote $out/help.png"
