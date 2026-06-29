#!/bin/sh
# ES-SPEC-ID-ANNOTATIONS / annotation-format
#
# Validates spec-id annotation syntax. A comment annotation must read
# `@spec contract|impl|test <spec-id>[, <spec-id>]...` (e.g. `@spec impl
# CS-LIST-PIZZA-MENU`) and an OpenAPI annotation must read `x-spec: <spec-id>`
# (e.g. `x-spec: CS-LIST-PIZZA-MENU`). Flags any line that opens an annotation
# but does not match its grammar. Receives candidate files as arguments
# (Markdown excluded by the check's exclude patterns).
#
# Emits one finding per malformed line following the script location contract:
#   <file>:<line><TAB>malformed spec-id annotation
set -u

[ "$#" -eq 0 ] && exit 0

id='(CS|ES)-[A-Z]+(-[A-Z]+)*'
comment="@spec (contract|impl|test) ${id}(, ${id})*"
xspec="x-spec:[[:space:]]+\"?${id}\"?"

status=0
for file in "$@"; do
  [ -f "$file" ] || continue
  bad=$(
    grep -nE '@spec ' "$file" | grep -Ev "$comment"
    grep -nE 'x-spec:' "$file" | grep -Ev "$xspec"
  )
  [ -n "$bad" ] || continue
  status=1
  printf '%s\n' "$bad" | while IFS= read -r line; do
    printf '%s:%s\tmalformed spec-id annotation\n' "$file" "${line%%:*}"
  done
done

exit "$status"
