#!/bin/sh
# ARCH-COMMIT-MESSAGE / commit-id-format
#
# Behavioral commits (feat, fix, refactor) must end their subject line with a
# bracketed, comma-separated list of Spec IDs, e.g.
#   feat(api): add order endpoint [FUNC-ORDER-PLACE, ARCH-SECRETS]
#
# Runs in the repository directory. File arguments are ignored — this check
# inspects git history, not the working tree.
set -eu

id='(FUNC|ARCH)-[A-Z0-9]+(-[A-Z0-9]+)*'
bracket="\\[${id}(, ${id})*\\]\$"

bad=$(git log -10 --pretty=%s \
  | grep -E '^(feat|fix|refactor)' \
  | grep -vE "$bracket" || true)

if [ -n "$bad" ]; then
  echo "Behavioral commits missing a bracketed Spec ID list:"
  echo "$bad"
  exit 1
fi
