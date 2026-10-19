#!/bin/sh
# Sourced by validation entrypoints. Caller owns tmp_dir and verbose.
# shellcheck disable=SC2154

run_check() {

  label="$1"
  shift
  check_started=$(date +%s)
  printf 'Running %s...\n' "$label" >&2
  check_status=0
  if [ "$verbose" -eq 1 ]; then
    "$@" || check_status=$?
  else
    output="$tmp_dir/$label.log"
    "$@" >"$output" 2>&1 || check_status=$?
  fi
  check_elapsed=$(($(date +%s) - check_started))
  if [ "$check_status" -eq 0 ]; then
    printf 'Passed %s (%ss)\n' "$label" "$check_elapsed" >&2
  else
    printf '\nFailed %s (%ss)\nCommand: %s\n' "$label" "$check_elapsed" "$*" >&2
    if [ "$verbose" -eq 0 ]; then cat "$output" >&2; fi
  fi
  if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
    printf -- '- %s: %ss (exit %s)\n' "$label" "$check_elapsed" "$check_status" >>"$GITHUB_STEP_SUMMARY"
  fi
  return "$check_status"
}
