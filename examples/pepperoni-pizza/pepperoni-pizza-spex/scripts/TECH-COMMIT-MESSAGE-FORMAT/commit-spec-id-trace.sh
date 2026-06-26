#!/bin/sh
# TECH-COMMIT-MESSAGE-FORMAT / commit-spec-id-trace
#
# Behavioral commits (feat, fix, refactor) must end their subject line with a
# comma-separated list of Spec IDs, e.g.
#   feat(api): add order endpoint BIZ-PLACE-PIZZA-ORDER, TECH-SECRET-HANDLING
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

# Keep in sync with schemas/biz-spec-id.schema.json and schemas/tech-spec-id.schema.json
id='(BIZ|TECH)-[A-Z]+(-[A-Z]+)*'
trailing=" ${id}(, ${id})*\$"

bad=$(git log -10 --pretty=%s \
  | grep -E '^(feat|fix|refactor)' \
  | grep -vE "$trailing" || true)

if [ -n "$bad" ]; then
  printf '%s\n' "$bad" | while IFS= read -r subject; do
    printf 'behavioral commit missing a trailing Spec ID list: %s\n' "$subject"
  done
  exit 1
fi
