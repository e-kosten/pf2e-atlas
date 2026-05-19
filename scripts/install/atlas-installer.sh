#!/bin/sh
set -eu

REPO="${ATLAS_REPO:-e-kosten/pf2e-atlas}"
VERSION="${ATLAS_VERSION:-latest}"
INSTALL_DIR="${ATLAS_INSTALL_DIR:-$HOME/.local/bin}"
YES="${ATLAS_YES:-0}"
DRY_RUN=0

usage() {
  cat <<'USAGE'
Usage: atlas-installer.sh [options]

Options:
  --install-dir <dir>   Install atlas into this directory
  --version <version>   Install vX.Y.Z, vX.Y.Z-rc.N, or latest
  --yes                 Replace an existing atlas without prompting
  --dry-run             Print planned actions without downloading or installing
  --help                Show this help

Environment:
  ATLAS_INSTALL_DIR     Install directory override
  ATLAS_VERSION         Version override, default latest
  ATLAS_REPO            GitHub repo override, default e-kosten/pf2e-atlas
  ATLAS_YES=1           Non-interactive replacement confirmation
USAGE
}

die() {
  printf 'error: %s\n' "$*" >&2
  printf '\nManual install fallback:\n' >&2
  printf '  1. Open https://github.com/%s/releases\n' "$REPO" >&2
  printf '  2. Download the archive for your OS/CPU and SHA256SUMS\n' >&2
  printf '  3. Verify the checksum, extract atlas, and place it on PATH\n' >&2
  printf '  4. Run atlas setup\n' >&2
  printf '\nSource fallback for contributors with Rust 1.95+:\n' >&2
  printf '  git clone https://github.com/%s.git\n' "$REPO" >&2
  printf '  cd pf2e-atlas\n' >&2
  printf '  cargo install --path crates/atlas-cli --locked\n' >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing required tool: $1"
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --install-dir)
      [ "$#" -ge 2 ] || die "--install-dir requires a value"
      INSTALL_DIR="$2"
      shift 2
      ;;
    --version)
      [ "$#" -ge 2 ] || die "--version requires a value"
      VERSION="$2"
      shift 2
      ;;
    --yes)
      YES=1
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      die "unknown argument: $1"
      ;;
  esac
done

os=$(uname -s)
arch=$(uname -m)
case "$os:$arch" in
  Darwin:arm64) target="aarch64-apple-darwin" ;;
  Darwin:x86_64) target="x86_64-apple-darwin" ;;
  Linux:x86_64|Linux:amd64) target="x86_64-unknown-linux-gnu" ;;
  Linux:aarch64|Linux:arm64) target="aarch64-unknown-linux-gnu" ;;
  *) die "unsupported OS/architecture: $os/$arch" ;;
esac

case "$VERSION" in
  latest) tag="latest"; base_url="https://github.com/$REPO/releases/latest/download" ;;
  v*) tag="$VERSION"; base_url="https://github.com/$REPO/releases/download/$VERSION" ;;
  *) tag="v$VERSION"; base_url="https://github.com/$REPO/releases/download/v$VERSION" ;;
esac

manifest_url="$base_url/atlas-release-manifest.json"
archive=""

install_path="$INSTALL_DIR/atlas"
path_status="not-on-path"
old_ifs=$IFS
IFS=:
for entry in $PATH; do
  if [ "$entry" = "$INSTALL_DIR" ]; then
    path_status="on-path"
  fi
done
IFS=$old_ifs

existing="none"
existing_version="unknown"
if [ -e "$install_path" ]; then
  existing="at-destination"
  existing_version=$("$install_path" --version 2>/dev/null || printf 'unknown')
fi

shadow=""
found_at=$(command -v atlas 2>/dev/null || true)
if [ -n "$found_at" ] && [ "$found_at" != "$install_path" ]; then
  shadow="$found_at"
fi

printf 'Atlas installer\n'
printf '  target:      %s\n' "$target"
printf '  version:     %s\n' "$VERSION"
printf '  manifest:    %s\n' "$manifest_url"
printf '  install dir: %s\n' "$INSTALL_DIR"
printf '  PATH:        %s\n' "$path_status"
printf '  existing:    %s\n' "$existing"
[ "$existing_version" != "unknown" ] && printf '  current:     %s\n' "$existing_version"
[ -n "$shadow" ] && printf '  warning: another atlas appears earlier/on PATH at %s\n' "$shadow"

if [ "$DRY_RUN" -eq 1 ]; then
  printf 'Dry run only; no download or install performed.\n'
  exit 0
fi

need_cmd uname
need_cmd tar
if command -v curl >/dev/null 2>&1; then
  downloader="curl"
elif command -v wget >/dev/null 2>&1; then
  downloader="wget"
else
  die "missing required downloader: curl or wget"
fi
if command -v shasum >/dev/null 2>&1; then
  checksum_cmd="shasum -a 256"
elif command -v sha256sum >/dev/null 2>&1; then
  checksum_cmd="sha256sum"
else
  die "missing checksum tool: shasum or sha256sum"
fi

mkdir -p "$INSTALL_DIR" || die "could not create install dir: $INSTALL_DIR"
[ -w "$INSTALL_DIR" ] || die "install dir is not writable: $INSTALL_DIR"

tmp=$(mktemp -d "${TMPDIR:-/tmp}/atlas-install.XXXXXX")
cleanup() {
  rm -rf "$tmp"
}
trap cleanup EXIT INT TERM

download() {
  url="$1"
  dest="$2"
  if [ "$downloader" = "curl" ]; then
    curl --proto '=https' --tlsv1.2 -fsSL "$url" -o "$dest" || die "failed to download $url"
  else
    wget -q "$url" -O "$dest" || die "failed to download $url"
  fi
}

download "$base_url/SHA256SUMS" "$tmp/SHA256SUMS"
download "$manifest_url" "$tmp/atlas-release-manifest.json"
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
' "$tmp/atlas-release-manifest.json")
[ -n "$archive" ] || die "manifest did not contain an asset for $target"
printf '  asset:       %s\n' "$archive"
download "$base_url/$archive" "$tmp/$archive"

expected=$(awk -v file="$archive" '$2 == file { print $1 }' "$tmp/SHA256SUMS")
[ -n "$expected" ] || die "checksum for $archive not found in SHA256SUMS"
actual=$($checksum_cmd "$tmp/$archive" | awk '{ print $1 }')
[ "$expected" = "$actual" ] || die "checksum mismatch for $archive"

mkdir "$tmp/extract"
tar -xJf "$tmp/$archive" -C "$tmp/extract" || die "failed to extract $archive"
new_binary=$(find "$tmp/extract" -type f -name atlas | head -n 1)
[ -n "$new_binary" ] || die "archive did not contain atlas binary"
chmod +x "$new_binary"
new_version=$("$new_binary" --version 2>/dev/null || printf 'unknown')
printf '  target bin:  %s\n' "$new_version"

version_number() {
  printf '%s\n' "$1" | awk '{ print $NF }'
}

classify_change() {
  old=$(version_number "$1")
  new=$(version_number "$2")
  case "$old:$new" in
    unknown:*|*:unknown) printf 'unknown'; return ;;
  esac
  [ "$old" = "$new" ] && { printf 'same-version'; return; }
  old_core=${old%%-*}
  new_core=${new%%-*}
  old_pre=
  new_pre=
  [ "$old_core" != "$old" ] && old_pre=1
  [ "$new_core" != "$new" ] && new_pre=1
  old_major=$(printf '%s\n' "$old_core" | awk -F . '{ print $1 }')
  old_minor=$(printf '%s\n' "$old_core" | awk -F . '{ print $2 }')
  old_patch=$(printf '%s\n' "$old_core" | awk -F . '{ print $3 }')
  new_major=$(printf '%s\n' "$new_core" | awk -F . '{ print $1 }')
  new_minor=$(printf '%s\n' "$new_core" | awk -F . '{ print $2 }')
  new_patch=$(printf '%s\n' "$new_core" | awk -F . '{ print $3 }')
  case "$old_major$old_minor$old_patch$new_major$new_minor$new_patch" in
    *[!0-9]*) printf 'unknown'; return ;;
  esac
  if [ "$new_major" -gt "$old_major" ] || { [ "$new_major" -eq "$old_major" ] && [ "$new_minor" -gt "$old_minor" ]; } || { [ "$new_major" -eq "$old_major" ] && [ "$new_minor" -eq "$old_minor" ] && [ "$new_patch" -gt "$old_patch" ]; }; then
    printf 'upgrade'
  elif [ "$new_major" -lt "$old_major" ] || { [ "$new_major" -eq "$old_major" ] && [ "$new_minor" -lt "$old_minor" ]; } || { [ "$new_major" -eq "$old_major" ] && [ "$new_minor" -eq "$old_minor" ] && [ "$new_patch" -lt "$old_patch" ]; }; then
    printf 'downgrade'
  elif [ -n "$old_pre" ] && [ -z "$new_pre" ]; then
    printf 'upgrade'
  elif [ -z "$old_pre" ] && [ -n "$new_pre" ]; then
    printf 'downgrade'
  else
    printf 'unknown'
  fi
}

if [ -e "$install_path" ]; then
  change=$(classify_change "$existing_version" "$new_version")
  printf '  action:      replace existing atlas (%s)\n' "$change"
  if [ "$YES" != "1" ]; then
    if [ ! -r /dev/tty ]; then
      die "existing atlas requires confirmation; rerun with --yes or ATLAS_YES=1 to replace it"
    fi
    printf 'Replace existing atlas at %s? [y/N] ' "$install_path" >/dev/tty
    read answer </dev/tty
    case "$answer" in
      y|Y|yes|YES) ;;
      *) die "install cancelled" ;;
    esac
  fi
fi

tmp_dest="$INSTALL_DIR/.atlas.install.$$"
backup=""
cp "$new_binary" "$tmp_dest" || die "failed to stage atlas binary"
chmod +x "$tmp_dest"
if [ -e "$install_path" ]; then
  backup="$INSTALL_DIR/.atlas.backup.$$"
  cp "$install_path" "$backup" || die "failed to back up existing atlas"
fi
if mv "$tmp_dest" "$install_path"; then
  if ! "$install_path" --version >/dev/null 2>&1; then
    [ -n "$backup" ] && mv "$backup" "$install_path"
    die "installed atlas failed version check; restored previous binary if one existed"
  fi
else
  [ -n "$backup" ] && rm -f "$backup"
  die "failed to replace atlas at $install_path"
fi
rm -f "$backup"

printf 'Installed atlas to %s\n' "$install_path"
if [ "$path_status" != "on-path" ]; then
  printf '\n%s is not on PATH. Add this to your shell profile:\n' "$INSTALL_DIR"
  printf '  export PATH="%s:$PATH"\n' "$INSTALL_DIR"
fi
