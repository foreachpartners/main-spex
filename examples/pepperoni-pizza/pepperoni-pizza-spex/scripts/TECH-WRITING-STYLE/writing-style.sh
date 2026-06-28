#!/bin/sh
# TECH-WRITING-STYLE / writing
#
# Flags filler, hedging, motivational, apologetic, and subjective phrasing in
# documentation. Receives candidate files as arguments (scoped to *.md by the
# check's include patterns).
#
# Emits one finding per match following the script location contract:
#   <file>:<line><TAB>matches forbidden writing pattern
set -u

[ "$#" -eq 0 ] && exit 0

words='\b(maybe|perhaps|I think|basically|essentially|actually|sorry|unfortunately|elegant|beautiful)\b'
exclamations='(Great job|Perfect|Excellent)!'
checkins='(Make sense|Got it)\?'
pattern="${words}|${exclamations}|${checkins}"

status=0
for file in "$@"; do
  [ -f "$file" ] || continue
  matches=$(grep -nEi "$pattern" "$file" || true)
  [ -n "$matches" ] || continue
  status=1
  printf '%s\n' "$matches" | while IFS= read -r match; do
    printf '%s:%s\tmatches forbidden writing pattern\n' "$file" "${match%%:*}"
  done
done

exit "$status"
