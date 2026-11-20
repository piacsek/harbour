#!/bin/sh
last=
for argument in "$@"; do last=$argument; done
echo "acme editor in ${PWD##*/} first arg:[${1:-}] last arg:[$last]"
while read -r line; do
  echo "acme editor got: $line"
done
exec sleep 600
