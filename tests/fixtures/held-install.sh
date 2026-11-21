#!/bin/sh
echo "${0##*/} $*" >> "$CALLS_LOG"
if [ -n "${INSTALL_HELD:-}" ]; then
  : > "$INSTALL_HELD"
  while [ -e "$INSTALL_HELD" ] && [ ! -e "$INSTALL_HELD.release" ]; do sleep 0.1; done
fi
