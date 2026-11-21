#!/bin/sh
printf '\033[?2004h\033[?1000h\033[?1006h'
echo "acme agent ready"
stty raw -echo
printf 'read:'
while :; do
  dd bs=1 count=1 2>/dev/null | tr '\033\r\001' '^|@'
done
