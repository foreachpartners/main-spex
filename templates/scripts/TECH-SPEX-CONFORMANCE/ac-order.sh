#!/bin/sh
# TECH-SPEX-CONFORMANCE / ac-ordered
#
# Verifies each spec file lists its acceptance_criteria in RFC 2119 commitment
# order: MUST, MUST NOT, SHOULD, SHOULD NOT, MAY. Reads each criterion's leading
# keyword (after the optional [ ]/[x] implementation checkbox) and reports a
# finding wherever a criterion ranks below the criterion before it.
#
# Runs once in the spex repository directory over biz-specs/ and tech-specs/.
#
# Emits one finding per out-of-order criterion following the script location
# contract:
#   <file>:<line><TAB><message>
set -u

status=0

check_file() {
  file=$1
  [ -f "$file" ] || return 0
  awk -v file="$file" '
    function rank(text,   t) {
      t = text
      sub(/^\[[ xX]\][ \t]*/, "", t)
      if (t ~ /^MUST NOT/)   { label = "MUST NOT";   return 2 }
      if (t ~ /^MUST/)       { label = "MUST";       return 1 }
      if (t ~ /^SHOULD NOT/) { label = "SHOULD NOT"; return 4 }
      if (t ~ /^SHOULD/)     { label = "SHOULD";     return 3 }
      if (t ~ /^MAY/)        { label = "MAY";        return 5 }
      label = ""
      return 0
    }
    /^acceptance_criteria:/ { in_ac = 1; prev = 0; prev_label = ""; next }
    in_ac && /^[^ \t#-]/    { in_ac = 0 }
    in_ac && /^[ \t]+-[ \t]/ {
      item = $0
      sub(/^[ \t]+-[ \t]*/, "", item)
      if (item ~ /^".*"$/) { sub(/^"/, "", item); sub(/"$/, "", item) }
      r = rank(item)
      if (r == 0) next
      if (prev != 0 && r < prev) {
        printf "%s:%d\tacceptance criteria out of order: %s must not follow %s (order: MUST, MUST NOT, SHOULD, SHOULD NOT, MAY)\n", file, NR, label, prev_label
        found = 1
      }
      prev = r
      prev_label = label
    }
    END { exit (found ? 1 : 0) }
  ' "$file" || status=1
}

for dir in biz-specs tech-specs; do
  [ -d "$dir" ] || continue
  for file in "$dir"/*.yaml; do
    [ -e "$file" ] || continue
    check_file "$file"
  done
done

exit "$status"
