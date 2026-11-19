#!/bin/sh
echo "acme agent in ${PWD##*/}, tmux:[${TMUX:-}]"
exec sleep 600
