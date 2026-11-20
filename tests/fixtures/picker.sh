#!/bin/sh
echo 'acme picker: type a directory' >&2
read -r line
printf '%s\n' "$line"
