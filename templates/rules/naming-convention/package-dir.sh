#!/bin/sh
# naming-convention / package-dir
#
# The package directory name is a derived identifier and must equal
# <project-name>-<package-short-name>, e.g.
#   shopping-notes-api
# where <project-name> comes from spex.yaml and <package-short-name> is the
# package's short name (kebab-case).
#
# Runs once per package from that package's directory.
set -eu

dir=$(basename "$(pwd -P)")

project_name=$(spex jq -r .project_name "$SPEX_DIR/spex.yaml") || {
  echo "Could not read the project name from $SPEX_DIR/spex.yaml"
  exit 1
}

if ! printf '%s' "$dir" | grep -qE '^[a-z0-9]+(-[a-z0-9]+)*$'; then
  echo "Package directory name '$dir' is not kebab-case"
  exit 1
fi

case "$dir" in
  "$project_name"-?*) ;;
  *)
    echo "Package directory name '$dir' must be <project-name>-<package-short-name> (expected prefix '$project_name-')"
    exit 1
    ;;
esac
