#!/bin/sh
set -eu

case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*) ;;
  *) exec node "$@" ;;
esac

# Windows terminates Node on native SIGTERM without running its handlers.
# Git Bash forwards cancellation through an owned channel and awaits cleanup.
channel=$(mktemp -d "${TMPDIR:-/tmp}/atlas-validation-cancel.XXXXXX")
trap 'rm -rf "$channel"' EXIT
ATLAS_VALIDATION_CANCEL_FILE=$(cygpath -m "$channel/cancel")
export ATLAS_VALIDATION_CANCEL_FILE
interrupted=0
# shellcheck disable=SC2329
cancel() {
  interrupted=$1
  printf '%s\n' "$interrupted" >"$channel/request"
  mv "$channel/request" "$channel/cancel"
}
trap 'cancel 129' HUP
trap 'cancel 130' INT
trap 'cancel 143' TERM
node "$@" & owned=$!
while :; do
  status=0
  wait "$owned" || status=$?
  kill -0 "$owned" 2>/dev/null || break
done
[ "$interrupted" -eq 0 ] || exit "$interrupted"
exit "$status"
