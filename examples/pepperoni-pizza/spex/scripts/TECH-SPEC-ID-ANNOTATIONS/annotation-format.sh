#!/bin/sh
# TECH-SPEC-ID-ANNOTATIONS / annotation-format
#
# Validates spec-id annotation syntax. A comment annotation must read
# `@spec contract|impl|test <spec-id>[, <spec-id>]...` (e.g. `@spec impl
# BIZ-LIST-PIZZA-MENU`) and an OpenAPI annotation must read `x-spec: <spec-id>`
# (e.g. `x-spec: BIZ-LIST-PIZZA-MENU`). Flags any line that opens an annotation
# but does not match its grammar. Receives candidate files as arguments
# (Markdown excluded by the check's exclude patterns).
set -u

[ "$#" -eq 0 ] && exit 0

id='(BIZ|TECH)-[A-Z]+(-[A-Z]+)*'
comment="@spec (contract|impl|test) ${id}(, ${id})*"
xspec="x-spec:[[:space:]]+\"?${id}\"?"

status=0
for file in "$@"; do
  [ -f "$file" ] || continue
  bad=$(
    grep -nE '@spec ' "$file" | grep -Ev "$comment"
    grep -nE 'x-spec:' "$file" | grep -Ev "$xspec"
  )
  if [ -n "$bad" ]; then
    echo "$file:"
    echo "$bad"
    status=1
  fi
done

exit "$status"
