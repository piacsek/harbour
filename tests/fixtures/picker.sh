#!/bin/sh
echo "acme picker tmux:[${TMUX:-}] pane:[${TMUX_PANE:-}]" >&2
echo 'acme picker: type a directory' >&2
read -r line
printf '%s\n' "$line"
