#!/bin/sh
echo "sleep $*" >> "$SLEEPS_LOG"
exec "$REAL_SLEEP" "$@"
