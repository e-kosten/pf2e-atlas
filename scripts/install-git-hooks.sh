#!/bin/sh

set -eu

REPO_ROOT="$(git rev-parse --show-toplevel)"
git -C "$REPO_ROOT" config core.hooksPath .githooks
echo "Configured core.hooksPath=.githooks" >&2

