#!/bin/sh
echo "${0##*/} $*" >> "$CALLS_LOG"
if [ -n "${GATE_ALONE:-}" ]; then
  if ln -s "$$" "$GATE_ALONE" 2>/dev/null; then
    sleep 0.05
    rm -f "$GATE_ALONE"
  else
    echo "$$" >> "$GATE_ALONE.overlaps"
  fi
fi
if [ -n "${GATE_HELD:-}" ]; then
  : > "$GATE_HELD"
  while [ -e "$GATE_HELD" ] && [ ! -e "$GATE_HELD.release" ]; do sleep 0.1; done
fi
exit "${GATE_STATUS:-0}"
