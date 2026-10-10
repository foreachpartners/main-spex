#!/bin/sh
set -u

[ "$#" -eq 0 ] && exit 0

if ! command -v vacuum >/dev/null 2>&1; then
  printf '\tvacuum is not installed; install it (https://quobix.com/vacuum/installing/) to lint OpenAPI documents\n'
  exit 1
fi

ruleset="$(dirname "$0")/vacuum-ruleset.json"

status=0
for file in "$@"; do
  [ -f "$file" ] || continue

  if ! report=$(vacuum spectral-report --stdout --no-style --no-pretty --no-update-check --ruleset "$ruleset" "$file" 2>&1); then
    printf '%s\tvacuum failed: %s\n' "$file" "$(printf '%s\n' "$report" | tail -n 1)"
    status=1
    continue
  fi

  if ! findings=$(printf '%s' "$report" | spex jq -r '.[] | "\(.range.start.line)\t\(.message)"'); then
    printf '%s\tcould not read the vacuum report\n' "$file"
    status=1
    continue
  fi

  [ -n "$findings" ] || continue
  status=1
  printf '%s\n' "$findings" | while IFS="$(printf '\t')" read -r line message; do
    printf '%s:%s\t%s\n' "$file" "$line" "$message"
  done
done

exit "$status"
