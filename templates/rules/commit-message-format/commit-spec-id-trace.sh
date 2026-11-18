#!/bin/sh
set -euf
export LC_ALL=C

root=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
[ "$root" = "$(pwd -P)" ] || exit 0
git rev-parse --verify HEAD >/dev/null 2>&1 || exit 0

if [ -n "${SPEX_COMMIT_BASE:-}" ]; then
  base=$(git rev-parse --verify --end-of-options "$SPEX_COMMIT_BASE^{commit}" 2>/dev/null) || {
    printf 'SPEX_COMMIT_BASE must name a reachable commit: %s\n' "$SPEX_COMMIT_BASE"
    exit 1
  }
  if ! git merge-base --is-ancestor "$base" HEAD; then
    printf 'SPEX_COMMIT_BASE must be an ancestor of HEAD: %s\n' "$SPEX_COMMIT_BASE"
    exit 1
  fi
  commits=$(git log --no-merges --format=%H "$base..HEAD")
else
  commits=$(git log --no-merges --format=%H -3)
fi

references() {
  awk -v source="$1" -v diff_prefix="${2:-}" '
    diff_prefix != "" && /^diff --git / { in_hunk = 0; next }
    diff_prefix != "" && /^@@ / { in_hunk = 1; next }
    diff_prefix != "" && (!in_hunk || substr($0, 1, 1) != diff_prefix) { next }
    {
      text = diff_prefix == "" ? $0 : substr($0, 2)
      while (match(text, /@spec[[:space:]]+/)) {
        text = substr(text, RSTART + RLENGTH)
        trace = sub(/^(contract|impl|test)[[:space:]]+/, "", text)
        if ((source == "trace") != trace) continue
        while (match(text, /^[a-zA-Z0-9_.#-]+/)) {
          id = substr(text, 1, RLENGTH)
          text = substr(text, RLENGTH + 1)
          sub(/[.]$/, "", id)
          print id
          if (!sub(/^[[:space:]]*,[[:space:]]*/, "", text)) break
        }
      }
    }
  ' | sort -u
}

contains_id() {
  printf '%s\n' "$1" | grep -Fxq -e "$2"
}

supports_tag() {
  case "$2" in
    *\#*) contains_id "$1" "$2" ;;
    *) printf '%s\n' "$1" | awk -F '#' -v spec="$2" '("x" $1) == ("x" spec) { seen = 1 } END { exit !seen }' ;;
  esac
}

patch_for() {
  parent=$(git cat-file commit "$1" | awk '/^$/ { exit } /^parent / { print $2; exit }')
  case "$parent" in
    '') git diff-tree --root --no-commit-id -r -p -U0 --no-color --no-ext-diff --no-textconv "$1" ;;
    *)
      if git cat-file -e "$parent^{commit}" 2>/dev/null; then
        git diff -U0 --find-renames --no-color --no-ext-diff --no-textconv "$parent" "$1" --
      fi
      ;;
  esac
}

catalog_loaded=0
bad=0
for sha in $commits; do
  subject=$(git show -s --format=%s "$sha")
  if ! printf '%s\n' "$subject" | grep -qE '^(feat|fix)(\([a-z0-9-]+\))?!?: '; then
    continue
  fi
  kind=${subject%%[(!:]*}
  message=$(git show -s --format=%B "$sha")
  tags=$(printf '%s\n' "$message" | references tag)
  patch=$(patch_for "$sha")
  added=$(printf '%s\n' "$patch" | references trace '+')

  if [ -z "$tags" ] && [ -z "$added" ] && [ "$kind" = feat ]; then
    printf 'feat commit missing an @spec ID list: %s\n' "$subject"
    bad=1
  fi

  if [ -n "$tags" ] && [ "$catalog_loaded" -eq 0 ]; then
    specs=$(cd "${SPEX_DIR:?SPEX_DIR must name the spex directory}" && spex specs --fields id acceptance_criteria)
    known=$(printf '%s\n' "$specs" | spex jq -r '.[] | .id as $id | $id, (.acceptance_criteria // [] | .[] | $id + "#" + .id)')
    catalog_loaded=1
  fi

  removed_loaded=0
  for tag in $tags; do
    if contains_id "$known" "$tag"; then
      continue
    fi
    if supports_tag "$added" "$tag"; then
      if [ "$removed_loaded" -eq 0 ]; then
        later=$(git diff -U0 --find-renames --no-color --no-ext-diff --no-textconv "$sha" HEAD --)
        removed=$(printf '%s\n' "$later" | references trace '-')
        removed_loaded=1
      fi
      if supports_tag "$removed" "$tag"; then
        continue
      fi
    fi
    printf '%s commit references an unknown @spec ID %s: %s\n' "$kind" "$tag" "$subject"
    bad=1
  done

  for trace in $added; do
    if contains_id "$tags" "${trace%%\#*}" || supports_tag "$tags" "$trace"; then
      continue
    fi
    printf '%s commit missing an @spec tag for added trace %s: %s\n' "$kind" "$trace" "$subject"
    bad=1
  done
done

exit "$bad"
