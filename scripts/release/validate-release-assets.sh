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
atlas-cli-x86_64-unknown-linux-gnu.tar.xz
atlas-cli-aarch64-unknown-linux-gnu.tar.xz
atlas-cli-x86_64-pc-windows-msvc.zip
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
  x86_64-unknown-linux-gnu \
  aarch64-unknown-linux-gnu \
  x86_64-pc-windows-msvc
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

python3 - "$dist" <<'PY'
import sys
import tarfile
import zipfile
from pathlib import Path

dist = Path(sys.argv[1])


def contains_runtime_data(entry: str) -> bool:
    return (
        "pf2e-index.sqlite" in entry
        or "model.onnx" in entry
        or "tokenizer.json" in entry
        or "vendor/pf2e/" in entry
        or "hf-models/" in entry
    )


bad_entries = []
for archive in sorted(dist.glob("*.tar.xz")):
    with tarfile.open(archive, "r:xz") as source:
        for member in source.getmembers():
            if contains_runtime_data(member.name):
                bad_entries.append(f"{archive.name}:{member.name}")

for archive in sorted(dist.glob("*.zip")):
    with zipfile.ZipFile(archive) as source:
        for name in source.namelist():
            if contains_runtime_data(name):
                bad_entries.append(f"{archive.name}:{name}")

if bad_entries:
    for entry in bad_entries:
        print(
            f"release archive contains runtime data that must not be bundled: {entry}",
            file=sys.stderr,
        )
    sys.exit(1)
PY
