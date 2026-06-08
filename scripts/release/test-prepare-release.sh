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
mkdir -p "$work/scripts/release" "$work/scripts/git-hooks" "$work/crates/atlas-cli" "$work/docs/releases" "$fake_bin"
cp "$repo_root/scripts/prepare-release.sh" "$work/scripts/prepare-release.sh"
chmod +x "$work/scripts/prepare-release.sh"
cp "$repo_root/scripts/verify.sh" "$work/scripts/verify.sh"
chmod +x "$work/scripts/verify.sh"
for release_check in validate-release-tooling.sh test-prepare-release.sh; do
  cat > "$work/scripts/release/$release_check" <<'EOF_RELEASE_CHECK'
#!/bin/sh
printf '%s\n' "$0" >> "$ATLAS_TEST_COMMAND_LOG"
exit 0
EOF_RELEASE_CHECK
  chmod +x "$work/scripts/release/$release_check"
done
cat > "$work/scripts/git-hooks/test-common.sh" <<'EOF_GIT_HOOK_CHECK'
#!/bin/sh
printf '%s\n' "$0" >> "$ATLAS_TEST_COMMAND_LOG"
exit 0
EOF_GIT_HOOK_CHECK
chmod +x "$work/scripts/git-hooks/test-common.sh"
cat > "$work/scripts/release/generate-notices.py" <<'EOF_NOTICES'
#!/bin/sh
printf '%s\n' "$0" >> "$ATLAS_TEST_COMMAND_LOG"
exit 0
EOF_NOTICES
chmod +x "$work/scripts/release/generate-notices.py"
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
  switch)
    exit 0
    ;;
  pull)
    exit 0
    ;;
  fetch)
    [ "${ATLAS_TEST_FETCH_FAIL:-0}" = 1 ] && exit 1
    exit 0
    ;;
  rev-parse)
    if [ "$2" = "--show-toplevel" ]; then
      pwd
      exit 0
    fi
    if [ "$2" = "--verify" ]; then
      case "$3" in
        release/v0.1.0|release/v0.1.0-rc.1)
          [ "${ATLAS_TEST_RELEASE_BRANCH_EXISTS:-0}" = 1 ] && printf 'abc123\n' && exit 0
          exit 1
          ;;
        release/v*)
          exit 1
          ;;
      esac
    fi
    case "$2" in
      HEAD|origin/main) printf 'abc123\n' ;;
      v0.1.0)
        [ "${ATLAS_TEST_LOCAL_TAG_EXISTS:-0}" = 1 ] && printf 'abc123\n' && exit 0
        exit 1
        ;;
      v0.1.0-rc.1)
        exit 1
        ;;
      v0.1.0-rc.*)
        exit 1
        ;;
      v[0-9]*)
        exit 1
        ;;
      *) printf 'abc123\n' ;;
    esac
    ;;
  status)
    if [ -n "${ATLAS_TEST_STATUS:-}" ]; then
      printf '%s\n' "$ATLAS_TEST_STATUS"
      exit 0
    fi
    [ "${ATLAS_TEST_DIRTY:-0}" = 1 ] && printf ' M file\n'
    exit 0
    ;;
  ls-remote)
    if [ "$2" = "--exit-code" ] && [ "$3" = "--heads" ]; then
      [ "${ATLAS_TEST_REMOTE_RELEASE_BRANCH_EXISTS:-0}" = 1 ] && printf 'abc123\trefs/heads/%s\n' "$5" && exit 0
      exit 2
    fi
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
  tag)
    if [ "$2" = "-l" ]; then
      [ "${ATLAS_TEST_EXISTING_RC_TAGS:-0}" = 1 ] && printf 'v0.1.0-rc.1\nv0.1.0-rc.2\n'
      exit 0
    fi
    exit 0
    ;;
  push)
    exit 0
    ;;
  add|commit)
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
  "pr view")
    [ "${ATLAS_TEST_PR_EXISTS:-0}" = 1 ] && exit 0
    exit 1
    ;;
  "pr create") exit 0 ;;
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
  ATLAS_RELEASE_TEST_PROMPTS="${ATLAS_RELEASE_TEST_PROMPTS:-0}" \
    ATLAS_RELEASE_TEST_DISABLE_FZF="${ATLAS_RELEASE_TEST_DISABLE_FZF:-1}" \
    ATLAS_TEST_COMMAND_LOG="$log" \
    ATLAS_TEST_BRANCH="${ATLAS_TEST_BRANCH:-}" \
    ATLAS_TEST_DIRTY="${ATLAS_TEST_DIRTY:-0}" \
    ATLAS_TEST_UNTRACKED_NOTES="${ATLAS_TEST_UNTRACKED_NOTES:-0}" \
    ATLAS_TEST_DIRTY_NOTES="${ATLAS_TEST_DIRTY_NOTES:-0}" \
    ATLAS_TEST_LOCAL_TAG_EXISTS="${ATLAS_TEST_LOCAL_TAG_EXISTS:-0}" \
    ATLAS_TEST_REMOTE_TAG_EXISTS="${ATLAS_TEST_REMOTE_TAG_EXISTS:-0}" \
    ATLAS_TEST_RELEASE_EXISTS="${ATLAS_TEST_RELEASE_EXISTS:-0}" \
    ATLAS_TEST_FETCH_FAIL="${ATLAS_TEST_FETCH_FAIL:-0}" \
    ATLAS_TEST_RELEASE_BRANCH_EXISTS="${ATLAS_TEST_RELEASE_BRANCH_EXISTS:-0}" \
    ATLAS_TEST_REMOTE_RELEASE_BRANCH_EXISTS="${ATLAS_TEST_REMOTE_RELEASE_BRANCH_EXISTS:-0}" \
    ATLAS_TEST_EXISTING_RC_TAGS="${ATLAS_TEST_EXISTING_RC_TAGS:-0}" \
    ATLAS_TEST_STATUS="${ATLAS_TEST_STATUS:-}" \
    ATLAS_TEST_PR_EXISTS="${ATLAS_TEST_PR_EXISTS:-0}" \
    ATLAS_TEST_FZF_CHOICE="${ATLAS_TEST_FZF_CHOICE:-}" \
    ATLAS_RELEASE_TEST_SKIP_EDITOR="${ATLAS_RELEASE_TEST_SKIP_EDITOR:-1}" \
    PATH="$fake_bin:$PATH" \
    scripts/prepare-release.sh "$@"
}

expect_fail() {
  if run_prepare "$@" >/dev/null 2>&1; then
    echo "prepare-release unexpectedly passed: $*" >&2
    exit 1
  fi
}

expect_fail_without_stdin() {
  if run_prepare "$@" </dev/null >/dev/null 2>&1; then
    echo "prepare-release unexpectedly passed without stdin: $*" >&2
    exit 1
  fi
}

expect_log_absent() {
  if grep -q "$1" "$log"; then
    echo "$2" >&2
    exit 1
  fi
}

first_log_line() {
  grep -n "$1" "$log" | sed -n '1s/:.*//p'
}

last_log_line() {
  grep -n "$1" "$log" | sed -n '$s/:.*//p'
}

write_fake_fzf() {
  cat > "$fake_bin/fzf" <<'EOF_FZF'
#!/bin/sh
input=$(cat)
case "${ATLAS_TEST_FZF_CHOICE:-}" in
  cancel) exit 1 ;;
  "")
    printf '%s\n' "$input" | sed -n '1p'
    ;;
  *)
    printf '%s\n' "$input" | grep "^$ATLAS_TEST_FZF_CHOICE" | sed -n '1p'
    ;;
esac
EOF_FZF
  chmod +x "$fake_bin/fzf"
}

remove_fake_fzf() {
  rm -f "$fake_bin/fzf"
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
  unset ATLAS_TEST_RELEASE_BRANCH_EXISTS
  unset ATLAS_TEST_REMOTE_RELEASE_BRANCH_EXISTS
  unset ATLAS_TEST_EXISTING_RC_TAGS
  unset ATLAS_TEST_STATUS
  unset ATLAS_TEST_PR_EXISTS
  unset ATLAS_TEST_FZF_CHOICE
  unset ATLAS_RELEASE_TEST_PROMPTS
  unset ATLAS_RELEASE_TEST_DISABLE_FZF
  unset ATLAS_RELEASE_TEST_SKIP_EDITOR
}

expect_fail_without_stdin --dry-run
expect_fail_without_stdin --prepare-pr
expect_fail --version v0.1.0 --publish --dry-run
expect_fail --version invalid --publish --dry-run
expect_fail --prepare-pr --publish --version 0.1.0
expect_fail --publish --version
expect_fail --publish --notes-file

: > "$log"
prompt_output=$(printf '3\n' | ATLAS_RELEASE_TEST_PROMPTS=1 run_prepare --dry-run)
printf '%s\n' "$prompt_output" | grep -q 'Dry run: would run checks, create annotated tag v0.1.0' || {
  echo "bare numbered prompt did not select publish mode" >&2
  exit 1
}
reset_flags

: > "$log"
write_fake_fzf
prompt_output=$(ATLAS_TEST_FZF_CHOICE=publish ATLAS_RELEASE_TEST_PROMPTS=1 ATLAS_RELEASE_TEST_DISABLE_FZF=0 run_prepare --dry-run)
printf '%s\n' "$prompt_output" | grep -q 'Dry run: would run checks, create annotated tag v0.1.0' || {
  echo "bare fzf prompt did not select publish mode" >&2
  exit 1
}
reset_flags

write_fake_fzf
ATLAS_TEST_FZF_CHOICE=cancel ATLAS_RELEASE_TEST_PROMPTS=1 ATLAS_RELEASE_TEST_DISABLE_FZF=0 expect_fail --dry-run
reset_flags
remove_fake_fzf

: > "$log"
open_pr_output=$(printf 'y\n' | ATLAS_TEST_BRANCH=release/v0.1.0 run_prepare --open-pr 2>&1) || {
  printf '%s\n' "$open_pr_output" >&2
  cat "$log" >&2
  exit 1
}
grep -q 'scripts/release/validate-release-tooling.sh' "$log" || {
  echo "prepare-release --open-pr did not run release tooling validation" >&2
  exit 1
}
expect_log_absent 'scripts/release/test-prepare-release.sh' "prepare-release --open-pr ran recursive release-script smoke tests"
expect_log_absent 'scripts/git-hooks/test-common.sh' "prepare-release --open-pr ran git-hook smoke tests"
grep -q 'cargo clippy --workspace --all-targets -- -D warnings -D clippy::dbg_macro' "$log" || {
  echo "prepare-release --open-pr did not run broad clippy validation" >&2
  exit 1
}
grep -q 'cargo clippy --workspace --lib --bins -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::unimplemented -D clippy::todo -D clippy::unreachable' "$log" || {
  echo "prepare-release --open-pr did not run strict runtime clippy validation" >&2
  exit 1
}
grep -q 'cargo test --workspace' "$log" || {
  echo "prepare-release --open-pr did not run cargo tests" >&2
  exit 1
}
grep -q 'git add Cargo.lock THIRD-PARTY-NOTICES.md crates/atlas-cli/Cargo.toml docs/releases/v0.1.0.md' "$log" || {
  echo "prepare-release --open-pr did not stage release-prep files" >&2
  exit 1
}
grep -q 'git commit --no-verify -m chore(release): prepare v0.1.0' "$log" || {
  echo "prepare-release --open-pr did not create the release-prep commit" >&2
  exit 1
}
grep -q 'git push --no-verify -u origin release/v0.1.0' "$log" || {
  echo "prepare-release --open-pr did not push the release branch" >&2
  exit 1
}
grep -q 'gh pr create --base main --head release/v0.1.0 --title chore(release): prepare v0.1.0 --body Prepare v0.1.0 for release.' "$log" || {
  echo "prepare-release --open-pr did not create the expected PR" >&2
  exit 1
}
reset_flags

: > "$log"
ATLAS_TEST_BRANCH=release/v0.1.0 run_prepare --open-pr --dry-run >/dev/null
expect_log_absent 'git commit --no-verify -m chore(release): prepare v0.1.0' "prepare-release --open-pr dry-run committed"
expect_log_absent 'git push --no-verify -u origin release/v0.1.0' "prepare-release --open-pr dry-run pushed"
expect_log_absent 'gh pr create' "prepare-release --open-pr dry-run created a PR"
reset_flags

ATLAS_TEST_BRANCH=dev expect_fail --open-pr
reset_flags
ATLAS_TEST_BRANCH=release/v0.1.0 ATLAS_TEST_STATUS=' M README.md' expect_fail --open-pr
reset_flags
ATLAS_TEST_BRANCH=release/v0.1.0 ATLAS_TEST_PR_EXISTS=1 expect_fail --open-pr
reset_flags

ATLAS_TEST_DIRTY=1 expect_fail --prepare-pr --version 0.1.0
reset_flags
ATLAS_TEST_RELEASE_BRANCH_EXISTS=1 expect_fail --prepare-pr --version 0.1.0
reset_flags
ATLAS_TEST_REMOTE_RELEASE_BRANCH_EXISTS=1 expect_fail --prepare-pr --version 0.1.0
reset_flags

: > "$log"
ATLAS_TEST_BRANCH=dev run_prepare --prepare-pr --version 0.1.0 >/dev/null
grep -q 'git switch main' "$log" || {
  echo "prepare-release --prepare-pr did not switch to main before creating the release branch" >&2
  exit 1
}
grep -q 'git pull --ff-only origin main' "$log" || {
  echo "prepare-release --prepare-pr did not fast-forward main before creating the release branch" >&2
  exit 1
}
switch_main_line=$(first_log_line 'git switch main')
pull_line=$(first_log_line 'git pull --ff-only origin main')
release_branch_line=$(first_log_line 'git switch -c release/v0.1.0')
if [ "$switch_main_line" -ge "$pull_line" ] || [ "$pull_line" -ge "$release_branch_line" ]; then
  echo "prepare-release --prepare-pr ran main switch/pull/release-branch creation out of order" >&2
  exit 1
fi
reset_flags

: > "$log"
run_prepare --prepare-pr --version 0.1.0 --dry-run >/dev/null
expect_log_absent 'git switch -c release/v0.1.0' "prepare-release --prepare-pr dry-run created a release branch"
expect_log_absent 'cargo ' "prepare-release --prepare-pr dry-run ran cargo"
expect_log_absent 'dist plan' "prepare-release --prepare-pr dry-run ran dist"
expect_log_absent 'git tag -a' "prepare-release --prepare-pr dry-run created a tag"
expect_log_absent 'git push origin' "prepare-release --prepare-pr dry-run pushed"
expect_log_absent 'gh release create' "prepare-release --prepare-pr dry-run created a release"

run_prepare --prepare-pr --version 0.1.0-rc.1 >/dev/null
grep -q 'git switch -c release/v0.1.0-rc.1' "$log" || {
  echo "prepare-release --prepare-pr did not create the expected release branch" >&2
  exit 1
}
grep -q 'version = "0.1.0-rc.1"' "$work/crates/atlas-cli/Cargo.toml" || {
  echo "prepare-release --prepare-pr did not update the crate version" >&2
  exit 1
}
grep -q 'cargo check -p atlas-cli' "$log" || {
  echo "prepare-release --prepare-pr did not refresh the lockfile through cargo" >&2
  exit 1
}
grep -q 'scripts/release/generate-notices.py' "$log" || {
  echo "prepare-release --prepare-pr did not regenerate third-party notices" >&2
  exit 1
}
[ -f "$work/docs/releases/v0.1.0-rc.1.md" ] || {
  echo "prepare-release --prepare-pr did not scaffold release notes" >&2
  exit 1
}

cat > "$work/crates/atlas-cli/Cargo.toml" <<'EOF_CARGO_RESET'
[package]
name = "atlas-cli"
version = "0.1.0"
EOF_CARGO_RESET
rm -f "$work/docs/releases/v0.1.0-rc.1.md"
reset_flags

: > "$log"
printf '\n' | ATLAS_RELEASE_TEST_PROMPTS=1 run_prepare --prepare-pr >/dev/null
grep -q 'git switch -c release/v0.1.0-rc.1' "$log" || {
  echo "prepare-release --prepare-pr numbered prompt did not default to first RC candidate" >&2
  exit 1
}
grep -q 'version = "0.1.0-rc.1"' "$work/crates/atlas-cli/Cargo.toml" || {
  echo "prepare-release --prepare-pr numbered prompt did not update to the default RC candidate" >&2
  exit 1
}
cat > "$work/crates/atlas-cli/Cargo.toml" <<'EOF_CARGO_RESET'
[package]
name = "atlas-cli"
version = "0.1.0"
EOF_CARGO_RESET
rm -f "$work/docs/releases/v0.1.0-rc.1.md"
reset_flags

: > "$log"
ATLAS_TEST_EXISTING_RC_TAGS=1
export ATLAS_TEST_EXISTING_RC_TAGS
printf '\n' | ATLAS_RELEASE_TEST_PROMPTS=1 run_prepare --prepare-pr >/dev/null
grep -q 'git switch -c release/v0.1.0-rc.3' "$log" || {
  echo "prepare-release --prepare-pr did not advance past existing RC tags" >&2
  exit 1
}
grep -q 'version = "0.1.0-rc.3"' "$work/crates/atlas-cli/Cargo.toml" || {
  echo "prepare-release --prepare-pr did not apply the advanced RC version" >&2
  exit 1
}
cat > "$work/crates/atlas-cli/Cargo.toml" <<'EOF_CARGO_RESET'
[package]
name = "atlas-cli"
version = "0.1.0"
EOF_CARGO_RESET
rm -f "$work/docs/releases/v0.1.0-rc.3.md"
reset_flags

: > "$log"
printf '2\n' | ATLAS_RELEASE_TEST_PROMPTS=1 run_prepare --prepare-pr >/dev/null
grep -q 'git switch -c release/v0.1.1' "$log" || {
  echo "prepare-release --prepare-pr did not apply patch selection" >&2
  exit 1
}
cat > "$work/crates/atlas-cli/Cargo.toml" <<'EOF_CARGO_RESET'
[package]
name = "atlas-cli"
version = "0.1.0"
EOF_CARGO_RESET
rm -f "$work/docs/releases/v0.1.1.md"
reset_flags

: > "$log"
write_fake_fzf
ATLAS_TEST_FZF_CHOICE=0.2.0 ATLAS_RELEASE_TEST_PROMPTS=1 ATLAS_RELEASE_TEST_DISABLE_FZF=0 run_prepare --prepare-pr >/dev/null
grep -q 'git switch -c release/v0.2.0' "$log" || {
  echo "prepare-release --prepare-pr fzf prompt did not apply selected version" >&2
  exit 1
}
cat > "$work/crates/atlas-cli/Cargo.toml" <<'EOF_CARGO_RESET'
[package]
name = "atlas-cli"
version = "0.1.0"
EOF_CARGO_RESET
rm -f "$work/docs/releases/v0.2.0.md"
reset_flags
remove_fake_fzf

: > "$log"
cat > "$fake_bin/test-editor" <<'EOF_EDITOR'
#!/bin/sh
printf 'editor %s\n' "$*" >> "$ATLAS_TEST_COMMAND_LOG"
printf '\nEdited by fake editor.\n' >> "$1"
EOF_EDITOR
chmod +x "$fake_bin/test-editor"
VISUAL= EDITOR=test-editor ATLAS_RELEASE_TEST_PROMPTS=1 ATLAS_RELEASE_TEST_SKIP_EDITOR=0 run_prepare --prepare-pr --version 0.3.0 >/dev/null
grep -q 'editor docs/releases/v0.3.0.md' "$log" || {
  echo "prepare-release --prepare-pr did not open release notes in the configured editor" >&2
  exit 1
}
grep -q 'Edited by fake editor.' "$work/docs/releases/v0.3.0.md" || {
  echo "prepare-release --prepare-pr editor did not receive the release notes path" >&2
  exit 1
}
cat > "$work/crates/atlas-cli/Cargo.toml" <<'EOF_CARGO_RESET'
[package]
name = "atlas-cli"
version = "0.1.0"
EOF_CARGO_RESET
rm -f "$work/docs/releases/v0.3.0.md"
rm -f "$fake_bin/test-editor"
reset_flags

printf '9\n' | ATLAS_RELEASE_TEST_PROMPTS=1 expect_fail --prepare-pr
reset_flags

ATLAS_TEST_DIRTY=1 expect_fail --publish --version 0.1.0
reset_flags
ATLAS_TEST_UNTRACKED_NOTES=1 expect_fail --publish --version 0.1.0
reset_flags
ATLAS_TEST_DIRTY_NOTES=1 expect_fail --publish --version 0.1.0
reset_flags
ATLAS_TEST_LOCAL_TAG_EXISTS=1 expect_fail --publish --version 0.1.0
reset_flags
ATLAS_TEST_REMOTE_TAG_EXISTS=1 expect_fail --publish --version 0.1.0
reset_flags
ATLAS_TEST_RELEASE_EXISTS=1 expect_fail --publish --version 0.1.0
reset_flags
expect_fail --publish --version 0.1.0 --notes-file docs/releases/missing.md

cp "$work/crates/atlas-cli/Cargo.toml" "$tmp/Cargo.toml.good"
sed 's/version = "0.1.0"/version = "0.2.0"/' "$tmp/Cargo.toml.good" > "$work/crates/atlas-cli/Cargo.toml"
expect_fail --publish --version 0.1.0
cp "$tmp/Cargo.toml.good" "$work/crates/atlas-cli/Cargo.toml"

ATLAS_TEST_BRANCH=dev ATLAS_TEST_DIRTY=1 ATLAS_TEST_FETCH_FAIL=1 run_prepare --publish --version 0.1.0 --dry-run >/dev/null 2>&1
expect_log_absent 'cargo fmt' "prepare-release dry-run ran cargo"
expect_log_absent 'dist plan' "prepare-release dry-run ran dist"
expect_log_absent 'git tag -a v0.1.0' "prepare-release dry-run created a tag"
expect_log_absent 'git push origin v0.1.0' "prepare-release dry-run pushed a tag"
expect_log_absent 'gh release create v0.1.0' "prepare-release dry-run created a release"
reset_flags

printf 'y\n' | run_prepare --publish --version 0.1.0 >/dev/null
grep -q 'git tag -a v0.1.0 -m Release v0.1.0' "$log" || {
  echo "prepare-release did not create the expected annotated tag" >&2
  exit 1
}
grep -q 'git push --no-verify origin v0.1.0' "$log" || {
  echo "prepare-release did not push the expected tag" >&2
  exit 1
}
grep -q 'gh release create v0.1.0 --draft --verify-tag --notes-file docs/releases/v0.1.0.md --title v0.1.0' "$log" || {
  echo "prepare-release did not create the expected draft release" >&2
  exit 1
}
grep -q 'dist plan --verbose error --tag v0.1.0 --allow-dirty' "$log" || {
  echo "prepare-release did not run dist plan" >&2
  exit 1
}
dist_line=$(first_log_line 'dist plan --verbose error --tag v0.1.0 --allow-dirty')
post_check_status_line=$(last_log_line 'git status --porcelain')
tag_line=$(first_log_line 'git tag -a v0.1.0')
if [ "$dist_line" -ge "$post_check_status_line" ] || [ "$post_check_status_line" -ge "$tag_line" ]; then
  echo "prepare-release did not re-check worktree cleanliness after validation and before tagging" >&2
  exit 1
fi

: > "$log"
cat > "$work/crates/atlas-cli/Cargo.toml" <<'EOF_CARGO_023'
[package]
name = "atlas-cli"
version = "0.2.3"
EOF_CARGO_023
cat > "$work/docs/releases/v0.2.3.md" <<'EOF_NOTES_023'
# v0.2.3

## Summary

Distinct version fixture.

## Install Notes

None.

## Known Issues

- None known.
EOF_NOTES_023
printf 'y\n' | run_prepare --publish >/dev/null
grep -q 'git tag -a v0.2.3 -m Release v0.2.3' "$log" || {
  echo "prepare-release without --version did not use the crate version" >&2
  exit 1
}
grep -q 'git pull --ff-only origin main' "$log" || {
  echo "prepare-release did not fast-forward main before tagging" >&2
  exit 1
}
cp "$tmp/Cargo.toml.good" "$work/crates/atlas-cli/Cargo.toml"

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
printf 'y\n' | run_prepare --publish --version 0.1.0-rc.1 >/dev/null
grep -q 'gh release create v0.1.0-rc.1 --draft --verify-tag --notes-file docs/releases/v0.1.0-rc.1.md --prerelease --latest=false --title v0.1.0-rc.1' "$log" || {
  echo "prepare-release did not create the expected prerelease draft" >&2
  exit 1
}

printf 'prepare-release smoke tests passed\n'
