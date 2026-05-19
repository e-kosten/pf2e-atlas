#!/usr/bin/env sh
set -eu

if [ "$#" -ne 2 ]; then
  echo "usage: scripts/release/validate-release-assets.sh <tag> <dist-dir>" >&2
  exit 2
fi

tag="$1"
dist="$2"

required="
atlas-cli-aarch64-apple-darwin.tar.xz
atlas-cli-x86_64-apple-darwin.tar.xz
atlas-cli-x86_64-unknown-linux-gnu.tar.xz
atlas-cli-aarch64-unknown-linux-gnu.tar.xz
atlas-cli-x86_64-pc-windows-msvc.zip
atlas-cli-aarch64-pc-windows-msvc.zip
SHA256SUMS
dist-manifest.json
atlas-installer.sh
atlas-installer.ps1
atlas-release-manifest.json
THIRD-PARTY-NOTICES.md
"

missing=0
for file in $required; do
  if [ ! -s "$dist/$file" ]; then
    echo "missing required release asset: $file" >&2
    missing=1
  fi
done
[ "$missing" -eq 0 ] || exit 1

for target in \
  aarch64-apple-darwin \
  x86_64-apple-darwin \
  x86_64-unknown-linux-gnu \
  aarch64-unknown-linux-gnu \
  x86_64-pc-windows-msvc \
  aarch64-pc-windows-msvc
do
  archive=$(awk -v target="$target" '
    /^[[:space:]]*\{/ {
      name = ""
      matched = 0
    }
    $0 ~ "\"name\"" {
      sub(/^.*"name"[[:space:]]*:[[:space:]]*"/, "")
      sub(/".*$/, "")
      name = $0
    }
    $0 ~ "\"target\"[[:space:]]*:[[:space:]]*\"" target "\"" {
      matched = 1
    }
    matched && name != "" {
      print name
      exit
    }
  ' "$dist/atlas-release-manifest.json")
  if [ -z "$archive" ] || [ ! -s "$dist/$archive" ]; then
    echo "manifest is missing required target asset: $target" >&2
    exit 1
  fi
  manifest_sha=$(awk -v target="$target" '
    /^[[:space:]]*\{/ {
      sha = ""
      matched = 0
    }
    $0 ~ "\"sha256\"" {
      sub(/^.*"sha256"[[:space:]]*:[[:space:]]*"/, "")
      sub(/".*$/, "")
      sha = $0
    }
    $0 ~ "\"target\"[[:space:]]*:[[:space:]]*\"" target "\"" {
      matched = 1
    }
    matched && sha != "" {
      print sha
      exit
    }
  ' "$dist/atlas-release-manifest.json")
  sums_sha=$(awk -v file="$archive" '$2 == file { print $1 }' "$dist/SHA256SUMS")
  actual_sha=$(python3 - "$dist/$archive" <<'PY'
import hashlib
import sys
from pathlib import Path

digest = hashlib.sha256()
with Path(sys.argv[1]).open("rb") as handle:
    for chunk in iter(lambda: handle.read(1024 * 1024), b""):
        digest.update(chunk)
print(digest.hexdigest())
PY
)
  if [ "$manifest_sha" != "$actual_sha" ] || [ "$sums_sha" != "$actual_sha" ]; then
    echo "checksum metadata mismatch for $archive" >&2
    exit 1
  fi
done

python3 - "$dist" <<'PY'
import json
import sys
from pathlib import Path

dist = Path(sys.argv[1])
manifest = json.loads((dist / "dist-manifest.json").read_text())
missing = []
for name, artifact in manifest.get("artifacts", {}).items():
    kind = artifact.get("kind")
    if kind in {"executable-zip", "checksum", "unified-checksum"} and not (dist / name).is_file():
        missing.append(name)
if missing:
    for name in missing:
        print(f"dist-manifest.json references missing asset: {name}", file=sys.stderr)
    sys.exit(1)
PY

if find "$dist" \( -name 'pf2e-index.sqlite' -o -name 'model.onnx' -o -name 'tokenizer.json' -o -path '*/vendor/pf2e/*' -o -path '*/hf-models/*' \) | grep -q .; then
  echo "release assets contain runtime data that must not be bundled" >&2
  exit 1
fi

check_archive_entry() {
  entry="$1"
  case "$entry" in
    *pf2e-index.sqlite*|*model.onnx*|*tokenizer.json*|*vendor/pf2e/*|*hf-models/*)
      echo "release archive contains runtime data that must not be bundled: $entry" >&2
      return 1
      ;;
  esac
}

for archive in "$dist"/*.tar.xz; do
  [ -e "$archive" ] || continue
  tar -tf "$archive" | while IFS= read -r entry; do
    check_archive_entry "$entry"
  done
done

for archive in "$dist"/*.zip; do
  [ -e "$archive" ] || continue
  if command -v zipinfo >/dev/null 2>&1; then
    zipinfo -1 "$archive" | while IFS= read -r entry; do
      check_archive_entry "$entry"
    done
  elif command -v unzip >/dev/null 2>&1; then
    unzip -Z1 "$archive" | while IFS= read -r entry; do
      check_archive_entry "$entry"
    done
  else
    echo "missing zip listing tool: zipinfo or unzip" >&2
    exit 1
  fi
done
