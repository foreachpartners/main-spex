#!/bin/sh
if [ ! -f CHANGELOG.md ]; then
  printf '\tCHANGELOG.md is missing from %s\n' "$(basename "$PWD")"
  exit 1
fi
