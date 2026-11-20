#!/bin/sh
echo "acme agent in ${PWD##*/}, tmux:[${TMUX:-}]"
while read -r line; do
  echo "acme agent got: $line"
done
exec sleep 600
