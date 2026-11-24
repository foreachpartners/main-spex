#!/bin/sh
set -u

[ "$#" -eq 0 ] && exit 0

baseline=${SPEX_OPENAPI_BASELINE_DIR:-}
case "$baseline" in
  /*) ;;
  *) printf '\tset SPEX_OPENAPI_BASELINE_DIR to an absolute baseline checkout directory\n'; exit 1 ;;
esac
if ! baseline_root=$(CDPATH='' cd -P -- "$baseline" 2>/dev/null && pwd -P); then
  printf '\tOpenAPI baseline directory is unavailable: %s\n' "$baseline"
  exit 1
fi

status=0
for file in "$@"; do
  if ! directory=$(CDPATH='' cd -P -- "$(dirname -- "$file")" 2>/dev/null && pwd -P); then
    printf '%s\tcould not resolve the contract directory\n' "$file"
    status=1
    continue
  fi
  current="$directory/$(basename -- "$file")"
  if ! owner=$(GIT_OPTIONAL_LOCKS=0 git -C "$directory" rev-parse --show-toplevel 2>/dev/null); then
    printf '%s\tcould not find the Git checkout that stores this contract\n' "$file"
    status=1
    continue
  fi
  repository_prefix="${owner%/}/"
  relative=${current#"$repository_prefix"}
  if [ "$current" = "$relative" ]; then
    printf '%s\tcontract is outside its Git checkout: %s\n' "$file" "$owner"
    status=1
    continue
  fi
  if [ "$baseline_root" = "$owner" ]; then
    printf '%s\tOpenAPI baseline must be a different checkout\n' "$file"
    status=1
    continue
  fi
  previous="$baseline_root/$relative"
  if [ ! -f "$previous" ]; then
    printf '%s\tOpenAPI baseline contract is missing: %s\n' "$file" "$previous"
    status=1
    continue
  fi

  if report=$(oasdiff breaking "$previous" "$current" --fail-on=WARN --fetch=false --open=false 2>&1); then
    continue
  else
    result=$?
  fi
  status=1

  if [ "$result" -eq 1 ]; then
    printf '%s\tbreaking OpenAPI change\n' "$file"
  else
    printf '%s\toasdiff could not compare contracts (exit %s)\n' "$file" "$result"
  fi
  printf '%s\n' "$report" | while IFS= read -r line; do
    [ -n "$line" ] && printf '%s\t%s\n' "$file" "$line"
  done
done

exit "$status"
