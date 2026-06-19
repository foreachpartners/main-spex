#!/bin/sh
# TECH-WRITING-STYLE / readme-has-title (frontend-only, per-repo)
#
# A frontend-specific check declared in spex.yaml under `checks_additional`.
# Like every check script, it lives in the spex repository's `scripts/`; the
# `per-repo` run scope runs it once in the frontend repository's directory.
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
