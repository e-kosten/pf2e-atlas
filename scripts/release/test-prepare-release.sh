#!/usr/bin/env sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/atlas-prepare-release-test.XXXXXX")
cleanup() {
  rm -rf "$tmp"
}
trap cleanup EXIT INT TERM

work="$tmp/work"
fake_bin="$tmp/bin"
log="$tmp/commands.log"
mkdir -p "$work/scripts" "$work/crates/atlas-cli" "$work/docs/releases" "$fake_bin"
cp "$repo_root/scripts/prepare-release.sh" "$work/scripts/prepare-release.sh"
chmod +x "$work/scripts/prepare-release.sh"
cat > "$work/crates/atlas-cli/Cargo.toml" <<'EOF_CARGO'
[package]
name = "atlas-cli"
version = "0.1.0"
EOF_CARGO
cat > "$work/docs/releases/v0.1.0.md" <<'EOF_NOTES'
# v0.1.0

## Summary

Test release.

## Install Notes

None.

## Known Issues

- None known.
EOF_NOTES

cat > "$fake_bin/git" <<'EOF_GIT'
#!/bin/sh
printf 'git %s\n' "$*" >> "$ATLAS_TEST_COMMAND_LOG"
case "$1" in
  branch)
    printf '%s\n' "${ATLAS_TEST_BRANCH:-main}"
    ;;
  fetch)
    [ "${ATLAS_TEST_FETCH_FAIL:-0}" = 1 ] && exit 1
    exit 0
    ;;
  rev-parse)
    case "$2" in
      HEAD|origin/main) printf 'abc123\n' ;;
      v0.1.0)
        [ "${ATLAS_TEST_LOCAL_TAG_EXISTS:-0}" = 1 ] && printf 'abc123\n' && exit 0
        exit 1
        ;;
      v0.1.0-rc.1)
        exit 1
        ;;
      *) printf 'abc123\n' ;;
    esac
    ;;
  status)
    [ "${ATLAS_TEST_DIRTY:-0}" = 1 ] && printf ' M file\n'
    exit 0
    ;;
  ls-remote)
    [ "${ATLAS_TEST_REMOTE_TAG_EXISTS:-0}" = 1 ] && printf 'abc123\trefs/tags/v0.1.0\n' && exit 0
    exit 2
    ;;
  ls-files)
    [ "${ATLAS_TEST_UNTRACKED_NOTES:-0}" = 1 ] && exit 1
    exit 0
    ;;
  diff)
    [ "${ATLAS_TEST_DIRTY_NOTES:-0}" = 1 ] && exit 1
    exit 0
    ;;
  tag|push)
    exit 0
    ;;
  *)
    exit 0
    ;;
esac
EOF_GIT
chmod +x "$fake_bin/git"

cat > "$fake_bin/gh" <<'EOF_GH'
#!/bin/sh
printf 'gh %s\n' "$*" >> "$ATLAS_TEST_COMMAND_LOG"
case "$1 $2" in
  "auth status") exit 0 ;;
  "release view")
    [ "${ATLAS_TEST_RELEASE_EXISTS:-0}" = 1 ] && exit 0
    exit 1
    ;;
  "release create") exit 0 ;;
esac
EOF_GH
chmod +x "$fake_bin/gh"

for command in cargo dist; do
  cat > "$fake_bin/$command" <<'EOF_CMD'
#!/bin/sh
printf '%s %s\n' "$(basename "$0")" "$*" >> "$ATLAS_TEST_COMMAND_LOG"
exit 0
EOF_CMD
  chmod +x "$fake_bin/$command"
done

run_prepare() {
  cd "$work"
  ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" scripts/prepare-release.sh "$@"
}

expect_fail() {
  if run_prepare "$@" >/dev/null 2>&1; then
    echo "prepare-release unexpectedly passed: $*" >&2
    exit 1
  fi
}

reset_flags() {
  unset ATLAS_TEST_BRANCH
  unset ATLAS_TEST_DIRTY
  unset ATLAS_TEST_UNTRACKED_NOTES
  unset ATLAS_TEST_DIRTY_NOTES
  unset ATLAS_TEST_LOCAL_TAG_EXISTS
  unset ATLAS_TEST_REMOTE_TAG_EXISTS
  unset ATLAS_TEST_RELEASE_EXISTS
  unset ATLAS_TEST_FETCH_FAIL
}

expect_fail --version v0.1.0 --dry-run
expect_fail --version invalid --dry-run

ATLAS_TEST_BRANCH=dev expect_fail --version 0.1.0
reset_flags
ATLAS_TEST_DIRTY=1 expect_fail --version 0.1.0
reset_flags
ATLAS_TEST_UNTRACKED_NOTES=1 expect_fail --version 0.1.0
reset_flags
ATLAS_TEST_DIRTY_NOTES=1 expect_fail --version 0.1.0
reset_flags
ATLAS_TEST_LOCAL_TAG_EXISTS=1 expect_fail --version 0.1.0
reset_flags
ATLAS_TEST_REMOTE_TAG_EXISTS=1 expect_fail --version 0.1.0
reset_flags
ATLAS_TEST_RELEASE_EXISTS=1 expect_fail --version 0.1.0
reset_flags
expect_fail --version 0.1.0 --notes-file docs/releases/missing.md

cp "$work/crates/atlas-cli/Cargo.toml" "$tmp/Cargo.toml.good"
sed 's/version = "0.1.0"/version = "0.2.0"/' "$tmp/Cargo.toml.good" > "$work/crates/atlas-cli/Cargo.toml"
expect_fail --version 0.1.0
cp "$tmp/Cargo.toml.good" "$work/crates/atlas-cli/Cargo.toml"

ATLAS_TEST_BRANCH=dev ATLAS_TEST_DIRTY=1 ATLAS_TEST_FETCH_FAIL=1 run_prepare --version 0.1.0 --dry-run >/dev/null
if grep -q 'git tag -a v0.1.0' "$log"; then
  echo "prepare-release dry-run mutated git state" >&2
  exit 1
fi
reset_flags

printf 'y\n' | run_prepare --version 0.1.0 >/dev/null
grep -q 'git tag -a v0.1.0 -m Release v0.1.0' "$log" || {
  echo "prepare-release did not create the expected annotated tag" >&2
  exit 1
}
grep -q 'git push origin v0.1.0' "$log" || {
  echo "prepare-release did not push the expected tag" >&2
  exit 1
}
grep -q 'gh release create v0.1.0 --draft --verify-tag --notes-file docs/releases/v0.1.0.md --title v0.1.0' "$log" || {
  echo "prepare-release did not create the expected draft release" >&2
  exit 1
}
grep -q 'dist plan --tag v0.1.0 --allow-dirty' "$log" || {
  echo "prepare-release did not run dist plan" >&2
  exit 1
}

cp "$tmp/Cargo.toml.good" "$work/crates/atlas-cli/Cargo.toml"
sed 's/version = "0.1.0"/version = "0.1.0-rc.1"/' "$tmp/Cargo.toml.good" > "$work/crates/atlas-cli/Cargo.toml"
cat > "$work/docs/releases/v0.1.0-rc.1.md" <<'EOF_RC'
# v0.1.0-rc.1

## Summary

Test release candidate.

## Install Notes

None.

## Known Issues

- None known.
EOF_RC
printf 'y\n' | run_prepare --version 0.1.0-rc.1 >/dev/null
grep -q 'gh release create v0.1.0-rc.1 --draft --verify-tag --notes-file docs/releases/v0.1.0-rc.1.md --prerelease --latest=false --title v0.1.0-rc.1' "$log" || {
  echo "prepare-release did not create the expected prerelease draft" >&2
  exit 1
}

printf 'prepare-release smoke tests passed\n'
