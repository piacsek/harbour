#!/bin/sh
dir="${CLAUDE_CONFIG_DIR:-$HOME/.claude}/sessions"
mkdir -p "$dir"
printf '{"status":"idle","cwd":"%s"}' "$ACME_MOVED_TO" > "$dir/$$.json"
echo "acme agent moved"
exec sleep 600
