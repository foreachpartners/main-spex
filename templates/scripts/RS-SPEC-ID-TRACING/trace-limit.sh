#!/bin/sh
# RS-SPEC-ID-TRACING / trace-limit
#
# Flags a Spec ID with more than 20 total trace annotations within this
# repository, so that searching for a spec's traces stays manageable. Runs
# once per workspace repository, in that repository's directory, and
# receives no arguments. It lists the repository's git-tracked files (so
# .gitignore is honored and the repository must be its own git repository)
# and tallies every @spec / x-spec reference by Spec ID. Markdown files are
# skipped, as are the spex repository's own spec and script files when this
# runs there (detected by the presence of spex.yaml): its fs-specs/ and
# rs-specs/ only contribute their Spec ID (the filename), and its scripts/
# holds framework helper code whose comments carry illustrative example
# annotations — neither's contents are real references. Other repositories
# are scanned in full, so an app repo's own scripts/ folder is still checked.
#
# Findings are file-less (the violation spans the whole repository, not a
# single file), one per over-limit Spec ID, following the script location
# contract:
#   <TAB>too many traces for `<id>`: <count> (limit 20)
set -u

id='(FS|RS)-[A-Z]+(-[A-Z]+)*'
limit=20

# Not a git repository means there are no tracked files to tally.
top=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
[ "$top" = "$(pwd -P)" ] || exit 0

is_spex_repo=false
[ -f spex.yaml ] && is_spex_repo=true

refs=$(
  git ls-files | while IFS= read -r rel; do
    case "$rel" in *.md) continue ;; esac
    if $is_spex_repo; then
      case "$rel" in fs-specs/* | rs-specs/* | scripts/*) continue ;; esac
    fi
    [ -f "$rel" ] || continue
    grep -hE '@spec |x-spec:' "$rel" 2>/dev/null | grep -oE "$id"
  done
)
[ -n "$refs" ] || exit 0

over=$(printf '%s\n' "$refs" | sort | uniq -c | awk -v limit="$limit" '$1 > limit { print $1, $2 }')
[ -n "$over" ] || exit 0

printf '%s\n' "$over" | while read -r count spec_id; do
  printf '\ttoo many traces for `%s`: %s (limit %s)\n' "$spec_id" "$count" "$limit"
done
exit 1
