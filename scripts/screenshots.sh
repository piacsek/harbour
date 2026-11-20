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

shoot() {
  local name="$1" width="$2" height="$3" args="$4" keys="$5"
  t resize-window -t "$shot_window" -x "$width" -y "$height"
  t respawn-pane -k -t "$shot_window" "cd $(printf %q "$home/acme-api") && env -u XDG_CONFIG_HOME -u XDG_STATE_HOME -u FZF_DEFAULT_OPTS -u FZF_DEFAULT_OPTS_FILE -u HARBOUR_DIR_PICKER -u HARBOUR_SESSION_COMMAND -u HARBOUR_EDITOR HOME=$(printf %q "$home") $(printf %q "$bin") $args; exec cat"
  sleep 1.5
  local key
  read -ra keys <<<"$keys"
  for key in ${keys[@]+"${keys[@]}"}; do
    t send-keys -t "$shot_window" "$key" && sleep 0.5
  done
  t capture-pane -e -p -t "$shot_window" | python3 "$home/truecolor.py" >"$home/$name.ansi"
  render "$name" --padding 20,64,20,20 --window --language text
}

mkdir -p "$home/acme-api"
shoot details 160 26 --demo ""
echo "wrote $out/details.png"
shoot pane 160 26 --demo "Enter"
echo "wrote $out/pane.png"
shoot keys 160 26 --demo "?"
echo "wrote $out/keys.png"
shoot question 160 26 --demo "x"
echo "wrote $out/question.png"
shoot picker 160 26 --demo "n"
echo "wrote $out/picker.png"
shoot editor 160 26 --demo "Enter C-l"
echo "wrote $out/editor.png"
shoot narrow 56 26 --demo ""
echo "wrote $out/narrow.png"
shoot help 160 11 --help ""
echo "wrote $out/help.png"
