#!/bin/sh
# TECH-SPEC-ID-ANNOTATIONS / no-orphan-annotations
#
# Flags spec-id annotations that reference a Spec ID with no matching spec.
# Runs once for the whole workspace: the working directory is the spex
# directory, candidate files from every repository arrive as arguments (as
# paths relative to the spex directory), and the known Spec IDs are the spec
# files in biz-specs/ and tech-specs/. An annotation pointing at an ID with no
# spec file is an orphan — usually a spec that was renamed or removed.
set -u

[ "$#" -eq 0 ] && exit 0

id='(BIZ|TECH)-[A-Z]+(-[A-Z]+)*'

known=$(
  for spec in biz-specs/*.yaml tech-specs/*.yaml; do
    [ -f "$spec" ] || continue
    base=${spec##*/}
    echo "${base%.yaml}"
  done
)

is_known() {
  for spec_id in $known; do
    [ "$1" = "$spec_id" ] && return 0
  done
  return 1
}

status=0
for file in "$@"; do
  [ -f "$file" ] || continue
  refs=$(grep -hE '@spec |x-spec:' "$file" 2>/dev/null | grep -oE "$id" | sort -u)
  for ref in $refs; do
    is_known "$ref" || {
      echo "$file: orphan annotation references unknown spec \`$ref\`"
      status=1
    }
  done
done

exit "$status"
