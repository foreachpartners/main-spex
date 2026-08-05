#!/bin/sh
# RS-SPEX-CONFORMANCE / spec-files-conform
#
# Validates the spex repository's own spec files: spex.yaml and every file in
# fs-specs/ and rs-specs/ must satisfy its JSON Schema, each spec file must
# be named <id-lowercase>.yaml (RS-NAMING-CONVENTION: uppercase id, lowercase
# filename), and every script-type check in a rs-spec must point at a script
# that exists under scripts/<spec-id-lowercase>/. Schema validation runs
# through `spex check-schema`, so the spex binary must be on PATH.
#
# Runs once in the spex repository directory.
#
# Emits one finding per failure following the script location contract:
#   <file><TAB><message>     (file-less when no spec file applies)
set -u

status=0

# A finding with a file location.
fail() {
  printf '%s\t%s\n' "$1" "$2"
  status=1
}

# A file-less finding.
fail_global() {
  printf '%s\n' "$1"
  status=1
}

spec_id() {
  sed -n 's/^id:[[:space:]]*//p' "$1" | head -n1
}

# Validate one file against its schema, re-emitting each schema violation
# (printed by `spex check-schema` as `<file>: <message>`) as a per-file finding.
validate_schema() {
  file=$1
  schema=$2
  errors=$(spex check-schema --schema "$schema" --files "$file" 2>&1 >/dev/null) || {
    status=1
    if [ -n "$errors" ]; then
      printf '%s\n' "$errors" | while IFS= read -r line; do
        printf '%s\t%s\n' "$file" "${line#*: }"
      done
    else
      printf '%s\tschema validation failed\n' "$file"
    fi
  }
}

# Schema validity plus filename-matches-id for one spec directory.
validate_specs() {
  dir=$1
  schema=$2
  [ -d "$dir" ] || return 0
  for file in "$dir"/*.yaml; do
    [ -e "$file" ] || continue
    validate_schema "$file" "$schema"

    id=$(spec_id "$file")
    stem=$(basename "$file" .yaml)
    id_lower=$(printf '%s' "$id" | tr '[:upper:]' '[:lower:]')
    if [ "$id_lower" != "$stem" ]; then
      fail "$file" "filename stem '$stem' does not match lowercased id '$id_lower'"
    fi
  done
}

if [ -f spex.yaml ]; then
  validate_schema spex.yaml schemas/spex.schema.json
else
  fail_global "spex.yaml not found"
fi

validate_specs fs-specs schemas/fs-spec.schema.json
validate_specs rs-specs schemas/rs-spec.schema.json

# Every script a rs-spec check references must exist at scripts/<id>/<script>.
for file in rs-specs/*.yaml; do
  [ -e "$file" ] || continue
  id=$(spec_id "$file")
  id_lower=$(printf '%s' "$id" | tr '[:upper:]' '[:lower:]')
  references=$(sed -n 's/^[[:space:]]*script:[[:space:]]*//p' "$file")
  for script in $references; do
    if [ ! -f "scripts/$id_lower/$script" ]; then
      fail "$file" "missing script scripts/$id_lower/$script"
    fi
  done
done

exit "$status"
