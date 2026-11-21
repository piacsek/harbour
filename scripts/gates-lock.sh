holder() {
  readlink "$1" 2>/dev/null || true
}

running() {
  local pid="${1%% *}" command
  case "$pid" in
    '' | *[!0-9]*) return 1 ;;
  esac
  kill -0 "$pid" 2>/dev/null || return 1
  command="$(ps -ww -p "$pid" -o command= 2>/dev/null)" || return 0
  case "$command" in
    *gates.sh* | *ship.sh*) return 0 ;;
  esac
  return 1
}

described() {
  if [ -n "$1" ]; then
    echo "the gates of pid ${1%% *} in ${1#* }"
  else
    echo "whoever holds $lock"
  fi
}

clearing() {
  [ -z "$1" ] || return 0
  if [ -d "$lock" ] && [ ! -L "$lock" ]; then
    echo "; if no gates run, remove it with: rmdir $lock"
  else
    echo "; if no gates run, remove it with: rm $lock"
  fi
}

take_over() {
  local target="$1" stale="$2" guard="$1.takeover-${2%% *}" guarding
  if ! ln -sn "$mine" "$guard" 2>/dev/null; then
    guarding="$(holder "$guard")"
    if [ -n "$guarding" ] && ! running "$guarding"; then
      take_over "$guard" "$guarding"
      return
    fi
    return 1
  fi
  if [ "$(holder "$target")" = "$stale" ]; then
    rm -f "$target"
    if [ "$target" = "$lock" ]; then
      echo "==> taking over the gates lock of pid ${stale%% *} in ${stale#* }, whose gates no longer run" >&2
    fi
  fi
  rm -f "$guard"
}

release() {
  if [ -n "${lock:-}" ] && [ "$(holder "$lock")" = "$mine" ]; then
    rm -f "$lock"
  fi
}

claim() {
  if [ -d "$lock" ] && [ ! -L "$lock" ]; then
    return 1
  fi
  ln -sn "$mine" "$lock" 2>/dev/null
}

inherited() {
  local held="${HARBOUR_GATES_HELD:-}"
  [ -n "$held" ] && [ "${held%% *}" = "$PPID" ] && [ "$(holder "$lock")" = "$held" ]
}

lock_gates() {
  [ -z "${CI:-}" ] || return 0
  local bound="${HARBOUR_GATES_WAIT:-1800}" common told='' asked='' held
  case "$bound" in
    *[!0-9]* | ??????????*)
      echo "gates: HARBOUR_GATES_WAIT must be a whole number of seconds, not $bound" >&2
      exit 1
      ;;
  esac
  if ! common="$(git rev-parse --git-common-dir 2>/dev/null)"; then
    echo "==> no gates lock: not in a git repository" >&2
    return 0
  fi
  common="$(cd "$common" && pwd -P)"
  if [ ! -w "$common" ]; then
    echo "==> no gates lock: $common cannot be written" >&2
    return 0
  fi
  lock="$common/harbour-gates.lock"
  if inherited; then
    return 0
  fi
  mine="$$ $(pwd -P)"
  trap release EXIT
  trap 'exit 129' HUP
  trap 'exit 130' INT
  trap 'exit 143' TERM
  SECONDS=0
  until claim; do
    if [ ! -e "$lock" ] && [ ! -L "$lock" ]; then
      continue
    fi
    held="$(holder "$lock")"
    if [ -n "$held" ] && ! running "$held" && take_over "$lock" "$held"; then
      continue
    fi
    if [ -z "$asked" ] || [ "$held" != "$told" ]; then
      echo "==> waiting for $(described "$held") to finish" >&2
      told="$held"
      asked=1
    fi
    if [ "$SECONDS" -ge "$bound" ]; then
      echo "gates: gave up after $bound s waiting for $(described "$held")$(clearing "$held")" >&2
      exit 1
    fi
    sleep 1
  done
  export HARBOUR_GATES_HELD="$mine"
}
