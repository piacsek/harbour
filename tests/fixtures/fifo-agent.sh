#!/bin/sh
dir="${CLAUDE_CONFIG_DIR:-$HOME/.claude}/sessions"
mkdir -p "$dir"
mkfifo "$dir/$$.json"
echo "acme agent registered"
exec sleep 600
