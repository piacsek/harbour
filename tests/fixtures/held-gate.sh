#!/bin/sh
echo "${0##*/} $*" >> "$CALLS_LOG"
if [ -n "${GATE_HELD:-}" ]; then
  : > "$GATE_HELD"
  while [ -e "$GATE_HELD" ] && [ ! -e "$GATE_HELD.release" ]; do sleep 0.1; done
fi
exit "${GATE_STATUS:-0}"
