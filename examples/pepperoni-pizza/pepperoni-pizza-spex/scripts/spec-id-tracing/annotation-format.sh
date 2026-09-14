#!/bin/sh
# spec-id-tracing / annotation-format
#
# Validates spec-id annotation syntax. A comment annotation must read
# `@spec contract|impl|test <spec-id>[#<ac-id>][, <spec-id>[#<ac-id>]]...`
# (e.g. `@spec impl list-pizza-menu#in-stock-only`) and an OpenAPI annotation
# must read `x-spec: <spec-id>` (e.g. `x-spec: list-pizza-menu`; no `#<ac-id>`
# form since contracts have no per-criterion granularity). Flags any line
# that opens an annotation but does not match its grammar. Receives
# candidate files as arguments (Markdown excluded by the check's exclude
# patterns).
#
# The spex workspace repo's own specs/, rules/ and scripts/ hold illustrative
# example annotations in prose and framework helper code, not real
# references, so they are skipped there (identified by a spex.yaml at the
# repository root, since only the spex workspace repo has one).
#
# Emits one finding per malformed line following the script location contract:
#   <file>:<line><TAB>malformed spec-id annotation
set -u

[ "$#" -eq 0 ] && exit 0

id='[a-z]+(-[a-z]+)*'
token="${id}(#${id})?"
comment="@spec (contract|impl|test) ${token}(, ${token})*"
xspec="x-spec:[[:space:]]+\"?${id}\"?"

status=0
for file in "$@"; do
  [ -f "$file" ] || continue
  if [ -f spex.yaml ]; then
    case "$file" in specs/* | rules/* | scripts/*) continue ;; esac
  fi
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
