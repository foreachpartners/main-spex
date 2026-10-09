#!/bin/sh
hits=$(grep -Hn 'HACK' "$@" | awk -F: '{ printf "%s:%s\tHACK marker left in code\n", $1, $2 }')
if [ -n "$hits" ]; then
  printf '%s\n' "$hits"
  exit 1
fi
