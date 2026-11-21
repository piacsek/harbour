#!/bin/sh
echo "acme agent in ${PWD##*/}, tmux:[${TMUX:-}] pane:[${TMUX_PANE:-}]"
while read -r line; do
  echo "acme agent got: $line"
done
exec sleep 600
