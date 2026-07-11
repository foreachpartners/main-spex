#!/bin/sh
# RS-SPEC-ID-TRACING / trace-limit
#
# Flags a Spec ID with more than 10 total trace annotations within a single
# repository, so that searching for a spec's traces stays manageable. Runs
# once for the whole workspace from the spex directory (its working
# directory) and receives no file arguments. It reads the workspace
# repositories from spex.yaml, lists each repository's git-tracked files (so
# .gitignore is honored and each repository must be its own git repository),
# and tallies every @spec / x-spec reference by Spec ID, per repository.
# Markdown files are skipped, as are the spex workspace's own spec and script
# files: its cs-specs/ and rs-specs/ only contribute their Spec ID (the
# filename), and its scripts/ holds framework helper code whose comments
# carry illustrative example annotations — neither's contents are real
# references. Other repositories are scanned in full, so an app repo's own
# scripts/ folder is still checked.
#
# Emits one finding per over-limit Spec ID following the script location
# contract (location is the repository path, since the violation spans the
# whole repository rather than a single file):
#   <repo><TAB>too many traces for `<id>`: <count> (limit 10)
set -u

id='(CS|RS)-[A-Z]+(-[A-Z]+)*'
limit=10

repo_paths=$(sed -n 's/^[[:space:]]*path:[[:space:]]*//p' spex.yaml)
spex_dir=$(pwd -P)

status=0
for repo in $repo_paths; do
  [ -d "$repo" ] || continue
  repo_dir=$(cd "$repo" && pwd -P)
  # Only enumerate a path that is itself a git repository root.
  top=$(git -C "$repo" rev-parse --show-toplevel 2>/dev/null) || continue
  [ "$top" = "$repo_dir" ] || continue

  refs=$(
    git -C "$repo" ls-files | while IFS= read -r rel; do
      case "$rel" in *.md) continue ;; esac
      if [ "$repo_dir" = "$spex_dir" ]; then
        case "$rel" in cs-specs/* | rs-specs/* | scripts/*) continue ;; esac
      fi
      file="$repo/$rel"
      [ -f "$file" ] || continue
      grep -hE '@spec |x-spec:' "$file" 2>/dev/null | grep -oE "$id"
    done
  )
  [ -n "$refs" ] || continue

  over=$(printf '%s\n' "$refs" | sort | uniq -c | awk -v limit="$limit" '$1 > limit { print $1, $2 }')
  [ -n "$over" ] || continue

  status=1
  printf '%s\n' "$over" | while IFS= read -r count spec_id; do
    printf '%s\ttoo many traces for `%s`: %s (limit %s)\n' "$repo" "$spec_id" "$count" "$limit"
  done
done

exit "$status"
