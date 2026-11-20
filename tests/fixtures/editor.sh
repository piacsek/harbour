#!/bin/sh
echo "acme editor in ${PWD##*/} first arg:[${1:-}]"
while read -r line; do
  echo "acme editor got: $line"
done
exec sleep 600
