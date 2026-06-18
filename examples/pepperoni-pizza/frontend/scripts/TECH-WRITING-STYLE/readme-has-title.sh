#!/bin/sh
# TECH-WRITING-STYLE / readme-has-title (frontend-only, per-repo)
#
# A frontend-specific check declared in spex.yml under `checks_additional`.
# Lives in the frontend repository's own `scripts/` because additional checks
# resolve their script path under the repository directory.
#
# The README must open with a top-level Markdown title (`# ...`).
set -eu

readme="README.md"
if [ ! -f "$readme" ]; then
  echo "missing $readme"
  exit 1
fi

case "$(head -n 1 "$readme")" in
  '# '*) exit 0 ;;
  *)
    echo "$readme must start with a '# ' title"
    exit 1
    ;;
esac
