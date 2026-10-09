#!/bin/sh
expected=$(cd "$(dirname "$0")/../.." && pwd -P)
actual=$(cd "${SPEX_DIR:?SPEX_DIR is not set}" && pwd -P)
if [ "$actual" = "$expected" ]; then
  printf '%s\tSPEX_DIR is the spex directory\n' "${1-}"
else
  printf '%s\tSPEX_DIR is %s, not the spex directory\n' "${1-}" "$SPEX_DIR"
fi
exit 1
