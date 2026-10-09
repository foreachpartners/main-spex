#!/bin/sh
# naming-convention / repo-dir
#
# The repository directory name is a derived identifier and must equal
# <project-name>-<repository-short-name>, e.g.
#   shopping-notes-api
# where <project-name> comes from spex.yaml and <repository-short-name> is the
# repository's short name (kebab-case).
#
# Runs once per workspace repository from that repository's directory.
set -eu

dir=$(basename "$(pwd -P)")

project_name=$(spex jq -r .project_name "$SPEX_DIR/spex.yaml") || {
  echo "Could not read the project name from $SPEX_DIR/spex.yaml"
  exit 1
}

if ! printf '%s' "$dir" | grep -qE '^[a-z0-9]+(-[a-z0-9]+)*$'; then
  echo "Repository directory name '$dir' is not kebab-case"
  exit 1
fi

case "$dir" in
  "$project_name"-?*) ;;
  *)
    echo "Repository directory name '$dir' must be <project-name>-<repository-short-name> (expected prefix '$project_name-')"
    exit 1
    ;;
esac
