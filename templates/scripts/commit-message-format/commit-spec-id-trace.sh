#!/bin/sh
# commit-message-format / commit-spec-id-trace
#
# Commits scoped to `spec` mark functional changes and must include
# a comma-separated list of Spec IDs in their subject or body, e.g.
#   feat(spec): add order endpoint place-pizza-order, secret-handling
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

# Keep in sync with schemas/spec-id.schema.json and schemas/rule-id.schema.json
id='[a-z]+(-[a-z]+)*'
spec_ids=" ${id}(, ${id})*"

bad=""
for sha in $(git log -2 --pretty=%H); do
  msg=$(git log -1 --pretty=%B "$sha")
  subject=$(printf '%s' "$msg" | head -1)
  if printf '%s' "$subject" | grep -qE '^[a-z]+\(spec\)!?:'; then
    if ! printf '%s' "$msg" | grep -qE "$spec_ids"; then
      bad="${bad}${subject}\n"
    fi
  fi
done
bad=$(printf '%b' "$bad" | sed '/^$/d')

if [ -n "$bad" ]; then
  printf '%s\n' "$bad" | while IFS= read -r subject; do
    printf 'spec-scoped commit missing a Spec ID list: %s\n' "$subject"
  done
  exit 1
fi
