#!/bin/sh
# TECH-NAMING-CONVENTION / repo-dir-kebab-case
#
# The repository directory name is a derived identifier and must be kebab-case:
# lowercase letters and digits joined by single hyphens, e.g.
#   pepperoni-pizza-api
#
# Runs once per workspace repository from that repository's directory.
set -eu

dir=$(basename "$(pwd -P)")

if ! printf '%s' "$dir" | grep -qE '^[a-z0-9]+(-[a-z0-9]+)*$'; then
  echo "Repository directory name '$dir' is not kebab-case"
  exit 1
fi
