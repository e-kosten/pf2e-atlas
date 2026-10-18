#!/bin/sh
set -eu
root=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd -P)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/atlas-push-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT HUP INT TERM
mkdir -p "$tmp/repo" "$tmp/bin"
cat > "$tmp/bin/npm" <<'MOCK'
#!/bin/sh
[ "${CARGO_NET_OFFLINE:-}" = true ] || exit 71
[ "$*" = '--prefix dev-tools/release run notices -- --check' ] || exit 72
printf 'notices\n' >> "$PUSH_TEST_LOG"
exit "${PUSH_TEST_FAILURE:-0}"
MOCK
chmod +x "$tmp/bin/npm"
export PATH="$tmp/bin:$PATH" PUSH_TEST_LOG="$tmp/log"
cd "$tmp/repo"
git init -q
git config user.name 'Atlas hook fixture'
git config user.email 'atlas-hook-fixture@example.invalid'
git config commit.gpgsign false
git config core.hooksPath /dev/null
printf 'target/\nnode_modules/\n' > .gitignore
printf '[workspace]\n' > Cargo.toml
git add .
git commit -qm 'test: base'
base=$(git rev-parse HEAD)
zero=0000000000000000000000000000000000000000
printf 'docs\n' > README.md
git add .
git commit -qm 'test: docs'
docs=$(git rev-parse HEAD)
run_hook() {
  printf 'refs/heads/test %s refs/heads/test %s\n' "$1" "$2" | sh "$root/scripts/git-hooks/check-push-notices.sh" > "$tmp/output" 2>&1
}
fail_with() {
  if run_hook "$2" "$3"; then echo "Expected failure: $1" >&2; exit 1; fi
  grep -q "$1" "$tmp/output"
}
run_hook "$docs" "$base"
[ ! -e "$PUSH_TEST_LOG" ]
run_hook "$zero" "$base"
fail_with 'release tooling dependencies' "$docs" "$zero"
mkdir -p dev-tools/release/node_modules/typescript/bin
touch dev-tools/release/node_modules/typescript/bin/tsc
run_hook "$docs" "$zero"
[ "$(wc -l < "$PUSH_TEST_LOG" | tr -d ' ')" = 1 ]
printf '# changed\n' >> Cargo.toml
git add Cargo.toml
git commit -qm 'test: dependency'
dependency=$(git rev-parse HEAD)
printf 'more docs\n' >> README.md
git add README.md
git commit -qm 'test: later docs'
head=$(git rev-parse HEAD)
# Finds dependency changes earlier in the outgoing range; checks multiple refs once.
: > "$PUSH_TEST_LOG"
printf 'refs/heads/a %s refs/heads/a %s\nrefs/heads/b %s refs/heads/b %s\n' "$head" "$base" "$head" "$base" | sh "$root/scripts/git-hooks/check-push-notices.sh"
[ "$(wc -l < "$PUSH_TEST_LOG" | tr -d ' ')" = 1 ]
printf 'unrelated dirt\n' >> README.md
run_hook "$head" "$base"
printf '# dirty\n' >> Cargo.toml
fail_with 'commit or stash' "$head" "$base"
git restore Cargo.toml
fail_with 'from a checkout' "$dependency" "$base"
PUSH_TEST_FAILURE=9
export PUSH_TEST_FAILURE
fail_with 'Notices check failed' "$head" "$base"
unset PUSH_TEST_FAILURE
# Deleting an input is still relevant.
git rm -q Cargo.toml
git commit -qm 'test: removed manifest'
run_hook "$(git rev-parse HEAD)" "$head"
echo 'Pre-push notices tests passed.'
