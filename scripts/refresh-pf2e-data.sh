#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA_DIR="${ROOT_DIR}/vendor/pf2e"

if [[ ! -d "${DATA_DIR}" ]]; then
  echo "PF2E data checkout not found at ${DATA_DIR}" >&2
  echo "Clone it with: git clone https://github.com/foundryvtt/pf2e.git ${DATA_DIR}" >&2
  exit 1
fi

if [[ ! -d "${DATA_DIR}/.git" ]]; then
  echo "Skipping refresh: ${DATA_DIR} is not a git checkout." >&2
  exit 0
fi

git -C "${DATA_DIR}" pull --ff-only
