#!/bin/sh
# commit-message-format / commit-spec-id-trace
#
# A commit related to a spec must carry a matching @spec <id>[, <id>] tag in
# its message; a commit related to a rule must carry a matching @rule
# <id>[, <id>] tag, e.g.
#   feat(ui): add order endpoint @spec place-pizza-order, secret-handling
#
# "Related" can't be determined exactly from git history alone, so this check
# is a proxy: it flags a commit whose diff adds a @spec contract|impl|test /
# x-spec annotation, or one that touches rules/ — other spec/rule-related
# commits can still slip through untagged.
#
# Runs once in the repository directory and inspects the last 10 commits'
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

# Keep in sync with schemas/spec-id.schema.json and schemas/rule-id.schema.json
id='[a-z]+(-[a-z]+)*'
id_list=" ${id}(, ${id})*"

bad=0
for sha in $(git log -3 --pretty=%H); do
  subject=$(git log -1 --pretty=%s "$sha")
  msg=$(git log -1 --pretty=%B "$sha")

  if git diff-tree -p --no-commit-id -r "$sha" \
      | grep -E '^\+' \
      | grep -qE '@spec (contract|impl|test) |x-spec: *'; then
    if ! printf '%s' "$msg" | grep -qE "@spec${id_list}"; then
      printf 'commit adds a @spec annotation but is missing an @spec ID list: %s\n' "$subject"
      bad=1
    fi
  fi

  if git diff-tree --no-commit-id --name-only -r "$sha" | grep -q '^rules/'; then
    if ! printf '%s' "$msg" | grep -qE "@rule${id_list}"; then
      printf 'rules/ commit missing an @rule ID list: %s\n' "$subject"
      bad=1
    fi
  fi
done

exit "$bad"
