#!/bin/sh
# commit-message-format / commit-spec-id-trace
#
# A commit related to a spec must carry a matching
# spec:ref <id>[#<ac-id>][, <id>[#<ac-id>]] tag in its message, e.g.
#   feat(ui): add order endpoint spec:ref place-pizza-order, secret-handling
#   feat(orders): set order status spec:ref place-pizza-order#created-order-status
#
# Only a feat commit is required to carry a spec:ref tag — other commit types
# (fix, refactor, docs, ...) may touch spec-related code (e.g. mechanical
# spex update reformatting) without being new spec work.
#
# Runs once in the package directory and inspects the last 10 commits'
# diffs, not the working tree.
#
# Findings are file-less (git history has no working-tree path): one line per
# offending commit subject.
set -eu

# Not a git repository means there is no history to verify, so skip and report
# ok rather than warn — a non-git directory is simply out of scope here.
if [ "$(git rev-parse --show-toplevel 2>/dev/null)" != "$PWD" ]; then
  exit 0
fi

# Keep in sync with schemas/spec-id.schema.json and schemas/ac-id.schema.json
id='[a-z0-9]+(-[a-z0-9]+)*'
ref="${id}(#${id})?"
id_list=" ${ref}(, ${ref})*"

bad=0
for sha in $(git log -3 --pretty=%H); do
  subject=$(git log -1 --pretty=%s "$sha")
  msg=$(git log -1 --pretty=%B "$sha")

  if printf '%s' "$subject" | grep -qE '^feat(\([a-z0-9-]+\))?!?: '; then
    if ! printf '%s' "$msg" | grep -qE "spec:ref${id_list}"; then
      printf 'feat commit missing a spec:ref ID list: %s\n' "$subject"
      bad=1
    fi
  fi
done

exit "$bad"
