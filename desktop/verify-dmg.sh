#!/bin/sh
set -eu

# A newly created image can briefly be busy on a hosted macOS runner.
for delay in 1 2 0; do
    if diagnostic=$(LC_ALL=C hdiutil verify "$1" 2>&1 >/dev/null); then
        exit 0
    else
        status=$?
    fi
    printf '%s\n' "$diagnostic" >&2
    if [ "$diagnostic" != 'hdiutil: verify failed - Resource temporarily unavailable' ] ||
       [ "$delay" -eq 0 ]; then
        exit "$status"
    fi
    sleep "$delay"
done
