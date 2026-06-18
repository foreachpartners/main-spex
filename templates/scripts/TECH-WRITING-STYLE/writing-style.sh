#!/bin/sh
# TECH-WRITING-STYLE / writing
#
# Flags filler, hedging, motivational, apologetic, and subjective phrasing in
# documentation. Receives candidate files as arguments (scoped to *.md by the
# check's include patterns).
set -u

[ "$#" -eq 0 ] && exit 0

words='\b(maybe|perhaps|I think|basically|essentially|actually|sorry|unfortunately|elegant|beautiful|clean)\b'
exclamations='(Great job|Perfect|Excellent)!'
checkins='(Make sense|Got it)\?'
pattern="${words}|${exclamations}|${checkins}"

status=0
for file in "$@"; do
  [ -f "$file" ] || continue
  matches=$(grep -nEi "$pattern" "$file" || true)
  if [ -n "$matches" ]; then
    echo "$file:"
    echo "$matches"
    status=1
  fi
done

exit "$status"
