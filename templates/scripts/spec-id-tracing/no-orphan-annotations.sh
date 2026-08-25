#!/bin/sh
# SPEC-ID-TRACING / no-orphan-annotations
#
# Flags spec-id annotations that reference a Spec ID with no matching spec.
# Runs once for the whole workspace from the spex directory (its working
# directory) and receives no file arguments. It reads the workspace
# repositories from spex.yaml, lists each repository's git-tracked files (so
# .gitignore is honored and each repository must be its own git repository),
# and checks their @spec / x-spec annotations against the known Spec IDs — the
# spec files in specs/ and rules/. An annotation pointing at an ID with
# no spec file is an orphan, usually a spec that was renamed or removed. Markdown
# files are skipped, as are the spex workspace's own spec and script files:
# its specs/ and rules/ only contribute their Spec ID (the filename),
# and its scripts/ holds framework helper code whose comments carry illustrative
# example annotations — neither's contents are real references. Other repositories
# are scanned in full, so an app repo's own scripts/ folder is still checked.
#
# Emits one finding per orphan following the script location contract:
#   <file><TAB>orphan annotation references unknown spec `<id>`
set -u

id='[A-Z]+(-[A-Z]+)*'

known=$(
  # specs/ and rules/ may group files one subdirectory level deep (e.g.
  # `rules/frontend/frontend-testing.yaml`); the Spec ID is always just the
  # filename, so both levels are globbed here.
  for spec in specs/*.yaml specs/*/*.yaml rules/*.yaml rules/*/*.yaml; do
    [ -f "$spec" ] || continue
    base=${spec##*/}
    echo "${base%.yaml}" | tr '[:lower:]' '[:upper:]'
  done
)

is_known() {
  for spec_id in $known; do
    [ "$1" = "$spec_id" ] && return 0
  done
  return 1
}

# Repository paths (relative to the spex directory), read from the resolved
# workspace config rather than parsed by hand from spex.yaml.
repo_paths=$(spex info | jq -r '.repositories[].path')

spex_dir=$(pwd -P)

orphans=$(
  for repo in $repo_paths; do
    [ -d "$repo" ] || continue
    repo_dir=$(cd "$repo" && pwd -P)
    # Only enumerate a path that is itself a git repository root.
    top=$(git -C "$repo" rev-parse --show-toplevel 2>/dev/null) || continue
    [ "$top" = "$repo_dir" ] || continue

    git -C "$repo" ls-files | while IFS= read -r rel; do
      case "$rel" in *.md) continue ;; esac
      # The spex workspace's own spec and script files hold example annotations,
      # not real references: specs/ and rules/ only contribute their
      # Spec ID (the filename), and scripts/ is framework helper code.
      if [ "$repo_dir" = "$spex_dir" ]; then
        case "$rel" in specs/* | rules/* | scripts/*) continue ;; esac
      fi
      file="$repo/$rel"
      [ -f "$file" ] || continue
      refs=$(grep -hE '@spec |x-spec:' "$file" 2>/dev/null | grep -oE "$id" | sort -u)
      for ref in $refs; do
        is_known "$ref" ||
          printf '%s\torphan annotation references unknown spec `%s`\n' "$file" "$ref"
      done
    done
  done
)

[ -n "$orphans" ] && {
  printf '%s\n' "$orphans"
  exit 1
}
exit 0
