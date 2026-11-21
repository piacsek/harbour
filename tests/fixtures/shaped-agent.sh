#!/bin/sh
echo "acme agent ready"
while read -r line; do
  printf '\033[%s q' "$line"
  echo "acme agent shape $line"
done
exec sleep 600
