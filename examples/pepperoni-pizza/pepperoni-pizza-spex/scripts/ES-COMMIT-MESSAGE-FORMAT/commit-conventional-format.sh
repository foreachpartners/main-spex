#!/bin/sh
# ES-COMMIT-MESSAGE-FORMAT / commit-conventional-format
#
# Recent commit subjects must follow the Conventional Commits format:
#   <type>[optional scope][!]: <description>
# e.g.
#   feat(cs): list available pizzas CS-LIST-PIZZA-MENU
# See https://www.conventionalcommits.org/
#
# Runs once in the repository directory and inspects git history, not the
# working tree. The initial commit subject ("initial commit", any case) is
# exempt.
#
# Findings are file-less (git history has no working-tree path): one line per
# offending commit subject.
set -eu

# Not a git repository means there is no history to verify, so skip and report
# ok rather than warn — a non-git directory is simply out of scope here.
if [ "$(git rev-parse --show-toplevel 2>/dev/null)" != "$PWD" ]; then
  exit 0
fi

type='(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)'
conventional="^${type}(\\([a-z0-9-]+\\))?!?: .+"

bad=$(git log -2 --no-merges --pretty=%s \
  | grep -viE '^initial commit$' \
  | grep -vE "$conventional" || true)

if [ -n "$bad" ]; then
  printf '%s\n' "$bad" | while IFS= read -r subject; do
    printf 'commit not in Conventional Commits format: %s\n' "$subject"
  done
  exit 1
fi
