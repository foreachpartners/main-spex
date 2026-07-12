#!/bin/sh
# RS-COMMIT-MESSAGE-FORMAT / commit-spec-id-trace
#
# Commits scoped to `fs` mark functional changes and must end their
# subject line with a comma-separated list of Spec IDs, e.g.
#   feat(fs): add order endpoint FS-PLACE-PIZZA-ORDER, RS-SECRET-HANDLING
#
# Runs once in the repository directory and inspects git history, not the
# working tree.
#
# Findings are file-less (git history has no working-tree path): one line per
# offending commit subject.
set -eu

# Not a git repository means there is no history to verify, so skip and report
# ok rather than warn — a non-git directory is simply out of scope here.
if [ "$(git rev-parse --show-toplevel 2>/dev/null)" != "$PWD" ]; then
  exit 0
fi

# Keep in sync with schemas/fs-spec-id.schema.json and schemas/rs-spec-id.schema.json
id='(FS|RS)-[A-Z]+(-[A-Z]+)*'
trailing=" ${id}(, ${id})*\$"

bad=$(git log -2 --pretty=%s \
  | grep -E '^[a-z]+\(fs\)!?:' \
  | grep -vE "$trailing" || true)

if [ -n "$bad" ]; then
  printf '%s\n' "$bad" | while IFS= read -r subject; do
    printf 'fs-scoped commit missing a trailing Spec ID list: %s\n' "$subject"
  done
  exit 1
fi
