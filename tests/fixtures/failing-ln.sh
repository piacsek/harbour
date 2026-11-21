#!/bin/sh
if [ -n "${LN_FAILS_ONCE:-}" ] && [ ! -e "$LN_FAILS_ONCE" ]; then
  : > "$LN_FAILS_ONCE"
  exit 1
fi
[ -z "${LN_FAILS:-}" ] || exit 1
exec "$REAL_LN" "$@"
