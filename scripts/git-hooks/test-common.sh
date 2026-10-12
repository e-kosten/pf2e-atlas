#!/bin/sh

set -eu

script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd -P)
repo_root=$(CDPATH='' cd -- "$script_dir/../.." && pwd -P)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/atlas-git-hooks-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT HUP INT TERM

fake_bin="$tmp/bin"
log="$tmp/commands.log"
mkdir -p "$fake_bin"

cat > "$fake_bin/cargo" <<'EOF_CARGO'
#!/bin/sh
printf 'cargo %s\n' "$*" >> "$ATLAS_TEST_COMMAND_LOG"
printf 'cargo detail output for %s\n' "$*"
[ "${ATLAS_TEST_FAIL_CARGO:-0}" -eq 0 ] || exit 19
exit 0
EOF_CARGO
chmod +x "$fake_bin/cargo"

cat > "$fake_bin/npm" <<'EOF_NPM'
#!/bin/sh
printf 'npm %s\n' "$*" >> "$ATLAS_TEST_COMMAND_LOG"
printf 'npm detail output for %s\n' "$*"
[ "${ATLAS_TEST_FAIL_NPM:-0}" -eq 0 ] || exit 23
exit 0
EOF_NPM
chmod +x "$fake_bin/npm"

: >"$log"
quiet_output=$(ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" "$repo_root/scripts/verify-changed.sh" --all 2>&1)
if printf '%s\n' "$quiet_output" | grep -Eq '(cargo|npm) detail output'; then
  echo "quiet changed-path verification surfaced successful command detail output" >&2
  exit 1
fi

grep -q 'cargo fmt --check' "$log" || {
  echo "fast changed-path verification did not run cargo fmt" >&2
  exit 1
}
grep -q 'cargo clippy --workspace --all-targets -- -D warnings -D clippy::dbg_macro' "$log" || {
  echo "fast changed-path verification did not run broad clippy validation" >&2
  exit 1
}
grep -q 'cargo clippy --workspace --lib --bins -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::unimplemented -D clippy::todo -D clippy::unreachable' "$log" || {
  echo "fast changed-path verification did not run strict runtime clippy validation" >&2
  exit 1
}
grep -q 'npm --prefix web/atlas-ui run format:check' "$log" || {
  echo "fast changed-path verification did not run web format check" >&2
  exit 1
}
grep -q 'npm --prefix web/atlas-ui run lint' "$log" || {
  echo "fast changed-path verification did not run web lint" >&2
  exit 1
}
grep -q 'npm --prefix web/atlas-ui run typecheck' "$log" || {
  echo "fast changed-path verification did not run web typecheck" >&2
  exit 1
}
if grep -q 'cargo test --workspace' "$log" || grep -q 'cargo build --workspace' "$log"; then
  echo "fast changed-path verification unexpectedly ran full Rust integration checks" >&2
  exit 1
fi
if grep -q 'npm --prefix web/atlas-ui run verify' "$log"; then
  echo "fast changed-path verification unexpectedly ran full web verify" >&2
  exit 1
fi

: >"$log"
verbose_output=$(ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" "$repo_root/scripts/verify-changed.sh" --all --full --verbose 2>&1)
if ! printf '%s\n' "$verbose_output" | grep -Eq '(cargo|npm) detail output'; then
  echo "verbose full changed-path verification did not surface command detail output" >&2
  exit 1
fi

grep -q 'cargo test --workspace' "$log" || {
  echo "full changed-path verification did not run cargo tests" >&2
  exit 1
}
grep -q 'cargo build --workspace' "$log" || {
  echo "full changed-path verification did not run cargo build" >&2
  exit 1
}
grep -q 'npm --prefix web/atlas-ui run verify' "$log" || {
  echo "full changed-path verification did not run web verify" >&2
  exit 1
}

# Exercise the actual hook and selector against real Git history. Every compiler
# and frontend command still resolves to the logging stubs above.
fixture="$tmp/repo"
mkdir -p "$fixture/.githooks" "$fixture/scripts/git-hooks" "$fixture/crates/example/src" "$fixture/web/atlas-ui/src"
cp "$repo_root/.githooks/pre-push" "$fixture/.githooks/pre-push"
cp "$repo_root/.githooks/pre-commit" "$repo_root/.githooks/pre-merge-commit" "$fixture/.githooks/"
cp "$repo_root/scripts/git-hooks/common.sh" "$fixture/scripts/git-hooks/common.sh"
cp "$repo_root/scripts/verify-changed.sh" "$repo_root/scripts/verify.sh" "$fixture/scripts/"
git init --quiet "$fixture"
git -C "$fixture" config user.name 'Atlas hook fixture'
git -C "$fixture" config user.email 'atlas-hook-fixture@example.invalid'
git -C "$fixture" config commit.gpgsign false
git -C "$fixture" config core.hooksPath /dev/null
printf 'reference documentation\n' >"$fixture/README.md"
printf 'pub fn example() {}\n' >"$fixture/crates/example/src/lib.rs"
printf 'export const value = 0;\n' >"$fixture/web/atlas-ui/src/example.ts"
git -C "$fixture" add .
git -C "$fixture" commit --quiet -m 'test: initial fixture'
base="$(git -C "$fixture" rev-parse HEAD)"
git -C "$fixture" update-ref refs/remotes/origin/main "$base"
zero=0000000000000000000000000000000000000000
missing=1111111111111111111111111111111111111111
push_input="$tmp/push-input"
push_output="$tmp/push-output"
push_remote=origin
push_cwd="$fixture"
fail_cargo=0
fail_npm=0
case_count=0

commit_file() {
  relative_path="$1"
  mkdir -p "$(dirname "$fixture/$relative_path")"
  printf '%s\n' "$2" >"$fixture/$relative_path"
  git -C "$fixture" add -- "$relative_path"
  git -C "$fixture" commit --quiet -m 'test: change fixture'
}

reset_fixture() {
  git -C "$fixture" checkout --quiet --detach "$base"
}

push_ref() {
  printf 'refs/heads/test %s refs/heads/test %s\n' "$1" "$2" >"$push_input"
}

expect_push() {
  label="$1"
  expected="$2"
  : >"$log"
  status=0
  (
    cd "$push_cwd"
    ATLAS_TEST_COMMAND_LOG="$log" ATLAS_TEST_FAIL_CARGO="$fail_cargo" ATLAS_TEST_FAIL_NPM="$fail_npm" \
      PATH="$fake_bin:$PATH" sh "$fixture/.githooks/pre-push" "$push_remote" fixture://remote
  ) <"$push_input" >"$push_output" 2>&1 || status="$?"
  case "$expected" in
    blocked | failed)
      if [ "$status" -eq 0 ]; then
        echo "Expected rejected push: $label" >&2
        cat "$push_output" >&2
        exit 1
      fi
      ;;
    *)
      if [ "$status" -ne 0 ]; then
        echo "Unexpected rejected push: $label" >&2
        cat "$push_output" >&2
        exit 1
      fi
      ;;
  esac
  rust_count="$(grep -c '^cargo test --workspace$' "$log" || true)"
  web_count="$(grep -c '^npm --prefix web/atlas-ui run verify$' "$log" || true)"
  case "$expected" in
    none | blocked)
      [ ! -s "$log" ] || { echo "$label unexpectedly executed gates" >&2; cat "$log" >&2; exit 1; }
      ;;
    rust) [ "$rust_count:$web_count" = '1:0' ] || { echo "$label selected wrong gates" >&2; cat "$log" >&2; exit 1; } ;;
    web) [ "$rust_count:$web_count" = '0:1' ] || { echo "$label selected wrong gates" >&2; cat "$log" >&2; exit 1; } ;;
    both) [ "$rust_count:$web_count" = '1:1' ] || { echo "$label selected wrong gates" >&2; cat "$log" >&2; exit 1; } ;;
    failed) [ -s "$log" ] || { echo "$label did not reach its failing gate" >&2; exit 1; } ;;
  esac
  case_count=$((case_count + 1))
  printf 'Passed hook fixture: %s\n' "$label"
}

commit_file README.md 'updated documentation'
docs_tip="$(git -C "$fixture" rev-parse HEAD)"
push_ref "$docs_tip" "$base"
expect_push 'existing docs-only branch' none
push_ref "$docs_tip" "$zero"
expect_push 'new docs-only branch from published ancestry' none
push_ref "$docs_tip" "$docs_tip"
expect_push 'unchanged endpoint' none
git -C "$fixture" update-ref refs/remotes/origin/already-published "$docs_tip"
push_ref "$docs_tip" "$zero"
expect_push 'new ref to already-published content' none
git -C "$fixture" update-ref -d refs/remotes/origin/already-published

reset_fixture
commit_file crates/example/src/lib.rs 'pub fn changed() {}'
rust_tip="$(git -C "$fixture" rev-parse HEAD)"
commit_file README.md 'docs after code'
code_then_docs="$(git -C "$fixture" rev-parse HEAD)"
push_ref "$code_then_docs" "$zero"
expect_push 'new branch includes code before final docs commit' rust

reset_fixture
commit_file web/atlas-ui/src/example.ts 'export const value = 1;'
web_tip="$(git -C "$fixture" rev-parse HEAD)"
push_ref "$web_tip" "$zero"
expect_push 'new web-only branch' web
push_ref "$web_tip" "$base"
expect_push 'existing web-only branch' web
fail_npm=1
expect_push 'frontend gate failure blocks push' failed
fail_npm=0

reset_fixture
commit_file crates/example/src/lib.rs 'pub fn changed() {}'
commit_file web/atlas-ui/src/example.ts 'export const value = 1;'
both_tip="$(git -C "$fixture" rev-parse HEAD)"
printf 'refs/heads/one %s refs/heads/one %s\nrefs/heads/two %s refs/heads/two %s\n' \
  "$both_tip" "$web_tip" "$both_tip" "$rust_tip" >"$push_input"
expect_push 'all pushed refs union Rust then web, once each' both
printf 'refs/heads/one %s refs/heads/one %s\nrefs/heads/two %s refs/heads/two %s\n' \
  "$both_tip" "$rust_tip" "$both_tip" "$web_tip" >"$push_input"
expect_push 'all pushed refs union web then Rust, once each' both
printf 'refs/heads/old-docs %s refs/heads/old-docs %s\nrefs/heads/current %s refs/heads/current %s\n' \
  "$docs_tip" "$base" "$both_tip" "$web_tip" >"$push_input"
expect_push 'other docs-only tip does not hide later current code' rust
printf '(delete) %s refs/heads/old %s\nrefs/heads/current %s refs/heads/current %s\n' \
  "$zero" "$base" "$both_tip" "$rust_tip" >"$push_input"
expect_push 'deletion does not hide later code' web
push_ref "$rust_tip" "$base"
expect_push 'different code tip cannot use current checkout gates' blocked
grep -q 'own checkout' "$push_output" || { echo 'missing candidate guidance' >&2; exit 1; }
push_ref "$both_tip" "$missing"
expect_push 'missing advertised object conservatively runs both gates' both
push_remote=unknown-destination
push_ref "$both_tip" "$zero"
expect_push 'new branch with no destination history runs both gates' both
push_remote=origin
fail_cargo=1
push_ref "$both_tip" "$base"
expect_push 'Rust gate failure blocks push' failed
fail_cargo=0

reset_fixture
git -C "$fixture" checkout --quiet --detach "$docs_tip"
push_ref "$docs_tip" "$rust_tip"
expect_push 'diverged ref includes removal of remote Rust change' rust
push_remote=other
push_ref "$docs_tip" "$zero"
expect_push 'another remote cannot borrow origin ancestry' both
push_remote=origin
git -C "$fixture" tag -a fixture-tag -m fixture "$docs_tip"
tag_oid="$(git -C "$fixture" rev-parse fixture-tag)"
push_ref "$tag_oid" "$zero"
expect_push 'annotated commit tag uses the same candidate comparison' none

git -C "$fixture" checkout --quiet --orphan unrelated-fixture
git -C "$fixture" add .
git -C "$fixture" commit --quiet -m 'test: unrelated history'
push_ref "$(git -C "$fixture" rev-parse HEAD)" "$zero"
expect_push 'new branch with existing but unrelated remote history runs both gates' both

# Both merge parents are known remote tips, so choosing either alone is unsafe.
reset_fixture
commit_file docs/left.md left
left="$(git -C "$fixture" rev-parse HEAD)"
reset_fixture
commit_file docs/right.md right
right="$(git -C "$fixture" rev-parse HEAD)"
git -C "$fixture" merge --quiet --no-ff -m 'test: merge fixture' "$left"
merged="$(git -C "$fixture" rev-parse HEAD)"
git -C "$fixture" update-ref refs/remotes/origin/left "$left"
git -C "$fixture" update-ref refs/remotes/origin/right "$right"
push_ref "$merged" "$zero"
expect_push 'ambiguous new-branch boundaries run both gates' both
git -C "$fixture" update-ref -d refs/remotes/origin/left
git -C "$fixture" update-ref -d refs/remotes/origin/right

for unknown_path in .cargo/config.toml justfile .github/workflows/check.yml dev-tools/probe.ts scripts/probe.ps1 scratch/plans/probe.sh; do
  reset_fixture
  commit_file "$unknown_path" fixture
  push_ref "$(git -C "$fixture" rev-parse HEAD)" "$base"
  expect_push "unknown executable/config path $unknown_path" both
done
reset_fixture
commit_file crates/example/tests/fixtures/example.md 'a validated fixture'
push_ref "$(git -C "$fixture" rev-parse HEAD)" "$base"
expect_push 'Markdown fixture in Rust code is not docs-only' rust
reset_fixture
commit_file scratch/plans/proposal.md 'a documented proposal'
push_ref "$(git -C "$fixture" rev-parse HEAD)" "$base"
expect_push 'known Markdown proposal is docs-only' none

reset_fixture
mkdir -p "$fixture/docs"
git -C "$fixture" mv crates/example/src/lib.rs docs/renamed.md
git -C "$fixture" commit --quiet -m 'test: rename Rust into docs'
push_ref "$(git -C "$fixture" rev-parse HEAD)" "$base"
expect_push 'renamed source retains deleted Rust path' rust
push_cwd="$fixture/web/atlas-ui"
expect_push 'hook from subdirectory sees outside source deletion' rust
push_cwd="$fixture"

reset_fixture
chmod +x "$fixture/README.md"
git -C "$fixture" add README.md
git -C "$fixture" commit --quiet -m 'test: executable documentation'
push_ref "$(git -C "$fixture" rev-parse HEAD)" "$base"
expect_push 'executable Markdown cannot skip gates' both
reset_fixture
rm "$fixture/README.md"
ln -s crates/example/src/lib.rs "$fixture/README.md"
git -C "$fixture" add README.md
git -C "$fixture" commit --quiet -m 'test: symlink documentation'
push_ref "$(git -C "$fixture" rev-parse HEAD)" "$base"
expect_push 'symlink Markdown cannot skip gates' both

reset_fixture
commit_file 'docs/space name.md' documentation
push_ref "$(git -C "$fixture" rev-parse HEAD)" "$base"
expect_push 'space in recognized documentation path' none
reset_fixture
unusual_path="$(printf 'docs/new\nline.md')"
commit_file "$unusual_path" fixture
push_ref "$(git -C "$fixture" rev-parse HEAD)" "$base"
expect_push 'quoted unusual filename conservatively validates' both

printf '(delete) %s refs/heads/old %s\n' "$zero" "$base" >"$push_input"
expect_push 'deletion-only push' none
: >"$push_input"
expect_push 'missing ref input runs full gates' both
printf 'refs/heads/test %s refs/heads/test\n' "$base" >"$push_input"
expect_push 'malformed input is rejected' blocked
blob="$(git -C "$fixture" rev-parse HEAD:README.md)"
push_ref "$blob" "$zero"
expect_push 'non-commit pushed object is rejected' blocked
push_ref "$missing" "$zero"
expect_push 'missing local candidate is rejected' blocked
printf 'refs/heads/test %s refs/heads/test %s extra\n' "$base" "$base" >"$push_input"
expect_push 'extra input fields are rejected' blocked

# The same classifier applies to staged commits and range consumers.
reset_fixture
printf 'staged documentation\n' >"$fixture/README.md"
git -C "$fixture" add README.md
for staged_hook in pre-commit pre-merge-commit; do
  : >"$log"
  (
    cd "$fixture/web/atlas-ui"
    ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" sh "$fixture/.githooks/$staged_hook"
  ) >"$push_output" 2>&1
  [ ! -s "$log" ] || { echo "$staged_hook did not recognize staged docs" >&2; exit 1; }
  case_count=$((case_count + 1))
  printf 'Passed hook fixture: staged docs skip %s\n' "$staged_hook"
done
mkdir -p "$fixture/scratch/plans"
printf 'executable fixture\n' >"$fixture/scratch/plans/probe.sh"
git -C "$fixture" add scratch/plans/probe.sh
: >"$log"
(
  cd "$fixture/web/atlas-ui"
  ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" sh "$fixture/.githooks/pre-commit"
) >"$push_output" 2>&1
grep -q '^cargo fmt --check$' "$log" || { echo 'staged unknown skipped Rust gate' >&2; exit 1; }
grep -q '^npm --prefix web/atlas-ui run typecheck$' "$log" || { echo 'staged unknown skipped web gate' >&2; exit 1; }
case_count=$((case_count + 1))
printf 'Passed hook fixture: staged unknown uses both fast gates\n'

# Literal path input must not discard a final line without a newline.
printf 'unknown-tooling-file' >"$fixture/paths-input"
: >"$log"
(
  cd "$fixture"
  ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" scripts/verify-changed.sh --paths-file paths-input --full
) >"$push_output" 2>&1
grep -q '^cargo test --workspace$' "$log" || { echo 'unterminated path skipped Rust gate' >&2; exit 1; }
grep -q '^npm --prefix web/atlas-ui run verify$' "$log" || { echo 'unterminated path skipped web gate' >&2; exit 1; }
case_count=$((case_count + 1))
printf 'Passed hook fixture: literal final path without newline is classified\n'

: >"$log"
if (
  cd "$fixture"
  ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" scripts/verify-changed.sh --range does-not-exist --full
) >"$push_output" 2>&1; then
  echo 'invalid diff range incorrectly succeeded' >&2
  exit 1
fi
[ ! -s "$log" ] || { echo 'invalid range executed gates' >&2; exit 1; }
case_count=$((case_count + 1))
printf 'Passed hook fixture: invalid diff range fails before gates\n'

printf 'git-hook common smoke tests and %s fixture cases passed\n' "$case_count"
