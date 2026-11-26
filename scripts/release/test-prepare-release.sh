#!/usr/bin/env sh
set -eu

# Fixture timings must not appear as real validation measurements in CI.
unset GITHUB_STEP_SUMMARY

repo_root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/atlas-prepare-release-test.XXXXXX")
cleanup() {
  rm -rf "$tmp"
}
trap cleanup EXIT INT TERM

work="$tmp/work"
fake_bin="$tmp/bin"
log="$tmp/commands.log"
mkdir -p "$work/dev-tools/release" "$work/scripts/release" "$work/scripts/git-hooks" "$work/crates/atlas-cli" "$work/docs/releases" "$fake_bin"
mkdir -p "$work/dev-tools/release/src"
cp "$repo_root/scripts/release/prep-state.mjs" "$repo_root/scripts/release/release-notes.mjs" "$work/scripts/release/"
cp "$repo_root/dev-tools/release/src/package-version.mjs" "$work/dev-tools/release/src/"
cp "$repo_root/scripts/prepare-release.sh" "$work/scripts/prepare-release.sh"
chmod +x "$work/scripts/prepare-release.sh"
mkdir -p "$work/scripts/validation"
cp "$repo_root/scripts/validation/run-check.mjs" "$repo_root/scripts/validation/run-process.mjs" "$repo_root/scripts/validation/verify.mjs" "$repo_root/scripts/validation/plan.mjs" "$repo_root/scripts/validation/inputs.mjs" "$repo_root/scripts/validation/status.mjs" "$repo_root/scripts/validation/tooling.mjs" "$repo_root/scripts/validation/run-check.sh" "$repo_root/scripts/validation/launch.sh" "$work/scripts/validation/"
cp "$repo_root/scripts/verify.sh" "$work/scripts/verify.sh"
chmod +x "$work/scripts/verify.sh"
mkdir -p "$work/scripts/development" "$work/web/atlas-ui"
cp "$repo_root/scripts/development/preflight.mjs" "$work/scripts/development/"
cp "$repo_root/web/atlas-ui/package.json" "$work/web/atlas-ui/"
cp "$repo_root/rust-toolchain.toml" "$work/"
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
cat > "$tmp/base.lock" <<'LOCK'
version = 4
[[package]]
name = "atlas-cli"
version = "0.1.0"
LOCK
cp "$tmp/base.lock" "$work/Cargo.lock"
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
        release-prep/v0.1.0|release-prep/v0.1.0-rc.1)
          [ "${ATLAS_TEST_RELEASE_BRANCH_EXISTS:-0}" = 1 ] && printf 'abc123\n' && exit 0
          exit 1
          ;;
        release-prep/v*)
          exit 1
          ;;
      esac
    fi
    case "$2" in
      HEAD|origin/main) printf 'abc123\n' ;;
      release/v[0-9]*|rc/v[0-9]*)
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
  show)
    case "$2" in
      abc123:crates/atlas-cli/Cargo.toml) cat crates/atlas-cli/Cargo.toml ;;
      *) exit 1 ;;
    esac
    ;;
  --literal-pathspecs)
    [ -f "$6" ] || exit 0
    printf '100644 blob aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\t%s\0' "$6"
    ;;
  cat-file) printf '1\n' ;;
  show-ref) exit 1 ;;
  ls-remote)
    if [ "$2" = "--exit-code" ] && [ "$3" = "--heads" ]; then
      [ "${ATLAS_TEST_REMOTE_RELEASE_BRANCH_EXISTS:-0}" = 1 ] && printf 'abc123\trefs/heads/%s\n' "$5" && exit 0
      exit 2
    fi
    exit 2
    ;;
  tag)
    if [ "$2" = "-l" ]; then
      [ "${ATLAS_TEST_EXISTING_RC_TAGS:-0}" = 1 ] && printf 'rc/v0.1.0-rc.1\nrc/v0.1.0-rc.2\n'
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
  "api --paginate")
    if [ "${ATLAS_TEST_RELEASE_EXISTS:-0}" = 1 ]; then echo release/v0.1.0; exit 0; fi
    case "${ATLAS_TEST_RELEASE_STATE:-missing}" in
      missing) exit 0 ;;
      draft) printf 'true\n' ;;
      published) printf 'false\n' ;;
      *) exit 1 ;;
    esac
    ;;
  "release create")
    while [ "$1" != --notes-file ]; do shift; done
    cp "$2" "$ATLAS_TEST_DRAFT_NOTES"
    [ "${ATLAS_TEST_DRAFT_FAIL:-0}" -eq 0 ] || exit 1
    exit 0
    ;;
esac
EOF_GH
chmod +x "$fake_bin/gh"

for command in cargo dist npm rustc; do
  cat > "$fake_bin/$command" <<'EOF_CMD'
#!/bin/sh
printf '%s %s\n' "$(basename "$0")" "$*" >> "$ATLAS_TEST_COMMAND_LOG"
if [ "$(basename "$0")" = rustc ]; then
  pinned=$(sed -n 's/^[[:space:]]*channel[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' rust-toolchain.toml)
  printf 'rustc %s (fixture)\n' "$pinned"
fi
if [ "${1:-}" = metadata ]; then printf '{"target_directory":"%s/target","packages":[]}\n' "$PWD"; fi
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
  if run_prepare "$@" >"$tmp/expected-failure.log" 2>&1; then
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
  matched_line=$(grep -n "$1" "$log" | sed -n '1s/:.*//p')
  [ -n "$matched_line" ] || { echo "Missing expected command: $1" >&2; return 1; }
  printf '%s\n' "$matched_line"
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
printf '%s\n' "$prompt_output" | grep -q 'Dry run: would run checks, create annotated tag release/v0.1.0' || {
  echo "bare numbered prompt did not select publish mode" >&2
  exit 1
}
reset_flags

: > "$log"
write_fake_fzf
prompt_output=$(ATLAS_TEST_FZF_CHOICE=publish ATLAS_RELEASE_TEST_PROMPTS=1 ATLAS_RELEASE_TEST_DISABLE_FZF=0 run_prepare --dry-run)
printf '%s\n' "$prompt_output" | grep -q 'Dry run: would run checks, create annotated tag release/v0.1.0' || {
  echo "bare fzf prompt did not select publish mode" >&2
  exit 1
}
reset_flags

write_fake_fzf
ATLAS_TEST_FZF_CHOICE=cancel ATLAS_RELEASE_TEST_PROMPTS=1 ATLAS_RELEASE_TEST_DISABLE_FZF=0 expect_fail --dry-run
reset_flags
remove_fake_fzf

# Branch and PR recovery use a real local Git repository below.
ATLAS_TEST_BRANCH=dev expect_fail --open-pr
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
release_branch_line=$(first_log_line 'git switch -c release-prep/v0.1.0')
if [ "$switch_main_line" -ge "$pull_line" ] || [ "$pull_line" -ge "$release_branch_line" ]; then
  echo "prepare-release --prepare-pr ran main switch/pull/release-branch creation out of order" >&2
  exit 1
fi
reset_flags

: > "$log"
run_prepare --prepare-pr --version 0.1.0 --dry-run >/dev/null
expect_log_absent 'git switch -c release-prep/v0.1.0' "prepare-release --prepare-pr dry-run created a release branch"
expect_log_absent 'cargo ' "prepare-release --prepare-pr dry-run ran cargo"
expect_log_absent 'dist plan' "prepare-release --prepare-pr dry-run ran dist"
expect_log_absent 'git tag -a' "prepare-release --prepare-pr dry-run created a tag"
expect_log_absent 'git push origin' "prepare-release --prepare-pr dry-run pushed"
expect_log_absent 'gh release create' "prepare-release --prepare-pr dry-run created a release"

run_prepare --prepare-pr --version 0.1.0-rc.1 >/dev/null
grep -q 'git switch -c release-prep/v0.1.0-rc.1' "$log" || {
  echo "prepare-release --prepare-pr did not create the expected release branch" >&2
  exit 1
}
grep -q 'version = "0.1.0-rc.1"' "$work/crates/atlas-cli/Cargo.toml" || {
  echo "prepare-release --prepare-pr did not update the crate version" >&2
  exit 1
}
grep -q 'version = "0.1.0-rc.1"' "$work/Cargo.lock"
expect_log_absent 'cargo check' "prepare-release compiled the frontend-dependent CLI"
[ -f "$work/docs/releases/v0.1.0-rc.1.md" ] || {
  echo "prepare-release --prepare-pr did not scaffold release notes" >&2
  exit 1
}

cp "$tmp/base.lock" "$work/Cargo.lock"
cat > "$work/crates/atlas-cli/Cargo.toml" <<'EOF_CARGO_RESET'
[package]
name = "atlas-cli"
version = "0.1.0"
EOF_CARGO_RESET
rm -f "$work/docs/releases/v0.1.0-rc.1.md"
reset_flags

: > "$log"
printf '\n' | ATLAS_RELEASE_TEST_PROMPTS=1 run_prepare --prepare-pr >/dev/null
grep -q 'git switch -c release-prep/v0.1.0-rc.1' "$log" || {
  echo "prepare-release --prepare-pr numbered prompt did not default to first RC candidate" >&2
  exit 1
}
grep -q 'version = "0.1.0-rc.1"' "$work/crates/atlas-cli/Cargo.toml" || {
  echo "prepare-release --prepare-pr numbered prompt did not update to the default RC candidate" >&2
  exit 1
}
cp "$tmp/base.lock" "$work/Cargo.lock"
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
grep -q 'git switch -c release-prep/v0.1.0-rc.3' "$log" || {
  echo "prepare-release --prepare-pr did not advance past existing RC tags" >&2
  exit 1
}
grep -q 'version = "0.1.0-rc.3"' "$work/crates/atlas-cli/Cargo.toml" || {
  echo "prepare-release --prepare-pr did not apply the advanced RC version" >&2
  exit 1
}
cp "$tmp/base.lock" "$work/Cargo.lock"
cat > "$work/crates/atlas-cli/Cargo.toml" <<'EOF_CARGO_RESET'
[package]
name = "atlas-cli"
version = "0.1.0"
EOF_CARGO_RESET
rm -f "$work/docs/releases/v0.1.0-rc.3.md"

# A new clone still advances past retired RC tags using tracked notes.
printf '# v0.1.0-rc.6\n' > "$work/docs/releases/v0.1.0-rc.6.md"
printf '\n' | ATLAS_RELEASE_TEST_PROMPTS=1 ATLAS_RELEASE_TEST_DISABLE_FZF=1 run_prepare --prepare-pr --dry-run > "$work/release-choice.log"
grep -q 'would create release-prep/v0.1.0-rc.7' "$work/release-choice.log"
rm "$work/docs/releases/v0.1.0-rc.6.md"
reset_flags

: > "$log"
printf '2\n' | ATLAS_RELEASE_TEST_PROMPTS=1 run_prepare --prepare-pr >/dev/null
grep -q 'git switch -c release-prep/v0.1.1' "$log" || {
  echo "prepare-release --prepare-pr did not apply patch selection" >&2
  exit 1
}
cp "$tmp/base.lock" "$work/Cargo.lock"
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
grep -q 'git switch -c release-prep/v0.2.0' "$log" || {
  echo "prepare-release --prepare-pr fzf prompt did not apply selected version" >&2
  exit 1
}
cp "$tmp/base.lock" "$work/Cargo.lock"
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
cp "$tmp/base.lock" "$work/Cargo.lock"
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

: > "$log"
ATLAS_TEST_BRANCH=dev ATLAS_TEST_DIRTY=1 ATLAS_TEST_FETCH_FAIL=1 run_prepare --publish --dry-run >/dev/null 2>&1
expect_log_absent 'cargo fmt' 'publish dry-run ran cargo'
expect_log_absent 'dist plan' 'publish dry-run ran dist'
expect_log_absent 'git tag .* -a ' 'publish dry-run created a tag'
expect_log_absent 'git push' 'publish dry-run pushed a tag'
expect_log_absent 'gh release create' 'publish dry-run created a release'
reset_flags

# Publication uses actual tags and commits against a disposable bare origin.
ATLAS_TEST_REAL_GIT=$(command -v git)
ATLAS_TEST_DRAFT_NOTES="$tmp/draft.md"
export ATLAS_TEST_REAL_GIT ATLAS_TEST_DRAFT_NOTES
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR GIT_PREFIX
cat > "$fake_bin/git" <<'EOF_REAL_GIT'
#!/bin/sh
printf 'git %s\n' "$*" >> "$ATLAS_TEST_COMMAND_LOG"
if [ "${ATLAS_TEST_LOST_PUSH:-0}" = 1 ] && [ "$1" = push ]; then
  "$ATLAS_TEST_REAL_GIT" "$@" && exit 1
fi
exec "$ATLAS_TEST_REAL_GIT" "$@"
EOF_REAL_GIT
cd "$work"
git init -q -b main
git config user.name Fixture
git config user.email fixture@example.invalid
git init -q --bare -b main "$tmp/origin.git"
git remote add origin "$tmp/origin.git"
cp "$repo_root/.gitignore" .
git add .
git commit -qm initial
git push -qu origin main
stable=release/v0.1.0
approved=$(git rev-parse HEAD)
canonical=docs/releases/v0.1.0.md
# Inject changes at the end of gates, after the displayed candidate was approved.
cat > "$fake_bin/dist" <<'EOF_DIST'
#!/bin/sh
printf 'dist %s\n' "$*" >> "$ATLAS_TEST_COMMAND_LOG"
case "${ATLAS_TEST_GATE_CHANGE:-}" in
  fail) exit 1 ;;
  branch) git switch -q other ;;
  advance) git merge -q --ff-only next ;;
  origin) git push -q origin next:refs/heads/main ;;
  dirty) echo dirty >> "$ATLAS_TEST_GATE_FILE" ;;
esac
EOF_DIST
git branch other
git switch -qc next
git commit -q --allow-empty -m next
git switch -q main
expect_fail --publish --version 0.2.0 --yes
expect_fail --publish --notes-file absent.md --yes
ATLAS_TEST_RELEASE_EXISTS=1 expect_fail --publish --yes
reset_flags
printf 'temporary\n' > untracked
expect_fail --publish --yes
rm untracked
git tag "$stable" next
stale=$(git rev-parse "$stable")
expect_fail --publish --yes
for change in fail branch advance origin dirty; do
  : > "$log"
  printf 'y\n' | ATLAS_RELEASE_TEST_PROMPTS=1 ATLAS_TEST_GATE_CHANGE="$change" ATLAS_TEST_GATE_FILE="$canonical" expect_fail --publish --yes
  grep -Fq 'dist plan' "$log" || { cat "$tmp/expected-failure.log" >&2; exit 1; }
  expect_log_absent 'gh release create' 'draft created after candidate change'
  [ "$(git rev-parse "$stable")" = "$stale" ]
  [ -z "$(git ls-remote origin "refs/tags/$stable")" ]
  case "$change" in
    branch) git switch -q main ;;
    advance) git reset -q --hard "$approved" ;;
    origin) git --git-dir="$tmp/origin.git" update-ref refs/heads/main "$approved" "$stale" ;;
    dirty) git restore -- "$canonical" ;;
  esac
done
: > "$log"
printf 'y\ny\n' | ATLAS_RELEASE_TEST_PROMPTS=1 ATLAS_TEST_LOST_PUSH=1 run_prepare --publish --version 0.1.0 >/dev/null
expect_log_absent 'scripts/release/test-prepare-release.sh' "release checks invoked recursive smoke tests"
expect_log_absent 'scripts/git-hooks/test-common.sh' "release checks invoked git-hook smoke tests"
for check in \
  'cargo clippy --workspace --all-targets --locked -- -D warnings -D clippy::dbg_macro' \
  'cargo clippy --workspace --lib --bins --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::unimplemented -D clippy::todo -D clippy::unreachable' \
  'cargo nextest run --workspace --locked' \
  'cargo test --workspace --doc --locked'; do
  grep -Fxq "$check" "$log" || { echo "missing release check: $check" >&2; exit 1; }
done

object=$(git rev-parse "refs/tags/$stable")
[ "$(git cat-file -t "$object")" = tag ]
[ "$(git rev-parse "$stable^{}")" = "$approved" ]
[ "$(git ls-remote origin "refs/tags/$stable" | cut -f1)" = "$object" ]
grep -Fq "git push --no-verify origin $object:refs/tags/$stable" "$log"
git cat-file tag "$object" | grep -Fxq "Atlas-Release-Notes: $canonical"
cmp "$canonical" "$ATLAS_TEST_DRAFT_NOTES"
expect_fail --publish --yes
# Recovery is independent of moved HEAD or replaced working files.
custom='docs/releases/alternate notes.md'
printf 'Alternate notes\n\n' > "$custom"
cp "$canonical" "$tmp/stable-original"
git add "$custom"
git commit -qm alternate
rm "$canonical"
ln -s 'alternate notes.md' "$canonical"
: > "$log"
run_prepare --recover-draft "$stable" "$approved" --dry-run >/dev/null
expect_log_absent 'gh release create' 'dry-run recovery created a draft'
run_prepare --recover-draft "$stable" "$approved" --yes >/dev/null
cmp "$tmp/stable-original" "$ATLAS_TEST_DRAFT_NOTES"
expect_log_absent 'git pull' 'recovery changed main'
expect_log_absent 'git push' 'recovery pushed a tag'
: > "$log"
(ATLAS_TEST_RELEASE_STATE=draft run_prepare --recover-draft "$stable" "$approved" --yes >/dev/null)
(ATLAS_TEST_RELEASE_STATE=published expect_fail --recover-draft "$stable" "$approved" --yes)
(ATLAS_TEST_RELEASE_STATE=unreachable expect_fail --recover-draft "$stable" "$approved" --yes)
expect_fail --recover-draft "$stable" "$(git rev-parse HEAD)" --yes
expect_fail --recover-draft rc/v0.1.0 "$approved" --yes
expect_fail --recover-draft "$stable" "$approved" --notes-file "$custom" --yes
expect_log_absent 'gh release create' 'recovery created a draft after a failed check or an existing draft'
git restore -- "$canonical"
# Select a custom-only path on an RC. Lose the draft response; retry the same
# exact bytes from another checkout. The workflow fetches its tag object too.
rc=rc/v0.1.0-rc.1
sed 's/version = "0.1.0"/version = "0.1.0-rc.1"/' "$tmp/base.lock" > Cargo.lock
sed 's/version = "0.1.0"/version = "0.1.0-rc.1"/' crates/atlas-cli/Cargo.toml > "$tmp/rc-manifest"
cp "$tmp/rc-manifest" crates/atlas-cli/Cargo.toml
git add Cargo.lock crates/atlas-cli/Cargo.toml
git commit -qm rc
git push -q origin main
rc_sha=$(git rev-parse HEAD)
if output=$(ATLAS_TEST_DRAFT_FAIL=1 run_prepare --publish --notes-file "./$custom" --yes 2>&1); then
  echo "expected interrupted draft creation" >&2
  exit 1
fi
printf '%s\n' "$output" | grep -Fq "retry with --recover-draft $rc $rc_sha"
cmp "$custom" "$ATLAS_TEST_DRAFT_NOTES"
cp "$custom" "$tmp/rc-original"
echo changed > "$custom"
: > "$log"
run_prepare --recover-draft "$rc" "$rc_sha" --yes >/dev/null
cmp "$tmp/rc-original" "$ATLAS_TEST_DRAFT_NOTES"
grep -Fq -- "--prerelease --latest=false --title $rc" "$log"
git clone -q --depth=1 --no-tags "file://$tmp/origin.git" "$tmp/shallow"
(
  cd "$tmp/shallow"
  [ "$(git rev-parse --is-shallow-repository)" = true ]
  git fetch -q --depth=1 --no-tags origin "refs/tags/$rc"
  object=$(git rev-parse FETCH_HEAD)
  blob=$(node scripts/release/prep-state.mjs remote-notes "$rc" "$rc_sha" "$object")
  git cat-file blob "$blob" > "$tmp/shallow-notes"
  cmp "$tmp/rc-original" "$tmp/shallow-notes"
  if node scripts/release/prep-state.mjs remote-notes "$rc" "$rc_sha" "$rc_sha" >/dev/null 2>&1; then
    echo "expected remote annotation identity mismatch" >&2
    exit 1
  fi
)
git restore -- "$custom"
# Historical tags used either lightweight or unmarked annotations. Guard
# malformed new markers and invalid tree entries before contacting gh.
git switch -qc historical "$approved"
printf 'Different historical notes\n\n' > "$custom"
ln -s v0.1.0.md docs/releases/link.md
: > docs/releases/empty.md
printf ' \t\n' > docs/releases/blank.md
git add docs/releases
git commit -qm historical
legacy=$(git rev-parse HEAD)
git --git-dir="$tmp/origin.git" update-ref -d "refs/tags/$stable"
git tag -d "$stable" >/dev/null
for kind in lightweight annotated; do
  case "$kind" in
    lightweight) git tag "$stable" ;;
    annotated) git tag -a "$stable" -m 'Historical release' ;;
  esac
  git push -q origin "refs/tags/$stable"
  if node scripts/release/prep-state.mjs remote-notes "$stable" "$legacy" "$(git rev-parse "$stable")" >/dev/null 2>&1; then
    echo 'workflow validation accepted missing tag metadata' >&2
    exit 1
  fi
  run_prepare --recover-draft "$stable" "$legacy" --yes >/dev/null
  cmp "$canonical" "$ATLAS_TEST_DRAFT_NOTES"
  run_prepare --recover-draft "$stable" "$legacy" --notes-file "$custom" --yes >/dev/null
  cmp "$custom" "$ATLAS_TEST_DRAFT_NOTES"
  : > "$log"
  for path in docs/releases/link.md docs/releases/empty.md docs/releases/blank.md docs/releases/missing.md ../outside; do
    expect_fail --recover-draft "$stable" "$legacy" --notes-file "$path" --yes
  done
  expect_log_absent 'gh release create' 'recovery accepted an invalid historical path'
  git --git-dir="$tmp/origin.git" update-ref -d "refs/tags/$stable"
  git tag -d "$stable" >/dev/null
done
# A marked tag selects the custom blob even when canonical notes also exist.
git tag -a "$stable" -m "Atlas-Release-Notes: $custom"
git push -q origin "refs/tags/$stable"
run_prepare --recover-draft "$stable" "$legacy" --yes >/dev/null
cmp "$custom" "$ATLAS_TEST_DRAFT_NOTES"
git --git-dir="$tmp/origin.git" update-ref -d "refs/tags/$stable"
git tag -d "$stable" >/dev/null
for marker in 'Atlas-Release-Notes: ' 'Atlas-Release-Notes: ../outside' 'Atlas-Release-Notes: docs/releases/empty.md' 'Atlas-Release-Notes: docs/releases/link.md' 'Atlas-Release-Notes bad' "$(printf 'Atlas-Release-Notes: %s\nAtlas-Release-Notes: %s' "$canonical" "$custom")"; do
  git tag --cleanup=verbatim -a "$stable" -m "$marker"
  git push -q origin "refs/tags/$stable"
  : > "$log"
  expect_fail --recover-draft "$stable" "$legacy" --yes
  expect_log_absent 'gh release create' 'recovery accepted invalid tag metadata'
  git --git-dir="$tmp/origin.git" update-ref -d "refs/tags/$stable"
  git tag -d "$stable" >/dev/null
done
sh "$repo_root/scripts/release/test-prep-retries.sh"
printf 'prepare-release smoke tests passed\n'
