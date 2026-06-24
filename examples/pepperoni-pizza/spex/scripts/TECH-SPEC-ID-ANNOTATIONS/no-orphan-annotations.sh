#!/bin/sh
# TECH-SPEC-ID-ANNOTATIONS / no-orphan-annotations
#
# Flags spec-id annotations that reference a Spec ID with no matching spec.
# Runs once for the whole workspace from the spex directory (its working
# directory) and receives no file arguments. It reads the workspace
# repositories from spex.yaml, lists each repository's git-tracked files (so
# .gitignore is honored and each repository must be its own git repository),
# and checks their @spec / x-spec annotations against the known Spec IDs — the
# spec files in biz-specs/ and tech-specs/. An annotation pointing at an ID with
# no spec file is an orphan, usually a spec that was renamed or removed. Markdown
# files are skipped.
#
# Emits one finding per orphan following the script location contract:
#   <file><TAB>orphan annotation references unknown spec `<id>`
set -u

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

# Repository paths (relative to the spex directory) declared in spex.yaml. A
# contract's `path:` sits under a `- ` list item, so it never matches here.
repo_paths=$(sed -n 's/^[[:space:]]*path:[[:space:]]*//p' spex.yaml)

orphans=$(
  for repo in $repo_paths; do
    [ -d "$repo" ] || continue
    # Only enumerate a path that is itself a git repository root.
    top=$(git -C "$repo" rev-parse --show-toplevel 2>/dev/null) || continue
    [ "$top" = "$(cd "$repo" && pwd -P)" ] || continue

    git -C "$repo" ls-files | while IFS= read -r rel; do
      case "$rel" in *.md) continue ;; esac
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
