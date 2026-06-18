#!/bin/sh
# TECH-SPEX-CONFORMANCE / spec-files-conform
#
# Validates the spex repository's own spec files: spex.yaml and every file in
# biz-specs/ and tech-specs/ must satisfy its JSON Schema, each spec file must
# be named <id>.yaml, and every script-type check in a tech-spec must point at
# a script that exists under scripts/<spec-id>/. Schema validation runs through
# `spex check-schema`, so the spex binary must be on PATH.
#
# Runs once in the spex repository directory.
set -u

status=0
fail() {
  echo "$1"
  status=1
}

spec_id() {
  sed -n 's/^id:[[:space:]]*//p' "$1" | head -n1
}

# Schema validity plus filename-matches-id for one spec directory.
validate_specs() {
  dir=$1
  schema=$2
  [ -d "$dir" ] || return 0
  for file in "$dir"/*.yaml; do
    [ -e "$file" ] || continue
    spex check-schema --schema "$schema" --files "$file" || status=1

    id=$(spec_id "$file")
    stem=$(basename "$file" .yaml)
    if [ "$id" != "$stem" ]; then
      fail "$file: filename stem '$stem' does not match id '$id'"
    fi
  done
}

if [ -f spex.yaml ]; then
  spex check-schema --schema schemas/spex.schema.json --files spex.yaml || status=1
else
  fail "spex.yaml not found"
fi

validate_specs biz-specs schemas/biz-spec.schema.json
validate_specs tech-specs schemas/tech-spec.schema.json

# Every script a tech-spec check references must exist at scripts/<id>/<script>.
for file in tech-specs/*.yaml; do
  [ -e "$file" ] || continue
  id=$(spec_id "$file")
  references=$(sed -n 's/^[[:space:]]*script:[[:space:]]*//p' "$file")
  for script in $references; do
    if [ ! -f "scripts/$id/$script" ]; then
      fail "$file: missing script scripts/$id/$script"
    fi
  done
done

exit "$status"
