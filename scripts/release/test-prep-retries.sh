#!/usr/bin/env sh
# Exercise actual branch, index, commit and remote state. GitHub and build jobs
# are isolated fixture boundaries; no network or real package builds are needed.
set -eu
source_dir=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
fixture=$(mktemp -d "${TMPDIR:-/tmp}/atlas-prep-retry.XXXXXX")
trap 'rm -rf "$fixture"' EXIT
export ATLAS_FIXTURE="$fixture"
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
unset GIT_DIR GIT_INDEX_FILE GIT_WORK_TREE GIT_COMMON_DIR GIT_PREFIX
mkdir -p "$fixture/bin" "$fixture/work"
cd "$fixture/work"
git init -q -b main
git config user.name "Atlas fixture"
git config user.email "fixture@example.org"
git init -q --bare "$fixture/origin.git"
git remote add origin "$fixture/origin.git"
mkdir -p scripts/release dev-tools/release/src crates/atlas-cli docs/releases web/atlas-ui
cp "$source_dir/scripts/prepare-release.sh" scripts/
cp "$source_dir/scripts/release/prep-state.mjs" scripts/release/
cp "$source_dir/dev-tools/release/src/package-version.mjs" dev-tools/release/src/
cp "$source_dir/.gitignore" .
cat > crates/atlas-cli/Cargo.toml <<'TOML'
[package]
name = "atlas-cli"
version = "0.1.0-rc.6"

[dependencies.other]
version = "0.1.0-rc.6"
TOML
cat > Cargo.lock <<'LOCK'
version = 4

[[package]]
name = "atlas-cli"
version = "0.1.0-rc.6"

[[package]]
name = "other"
version = "0.1.0-rc.6"
LOCK
printf '# v0.1.0\n\nPreviously authored notes.\n' > docs/releases/v0.1.0.md
for check in scripts/release/validate-release-tooling.sh scripts/verify.sh; do
  cat > "$check" <<'CHECK'
#!/bin/sh
printf '%s\n' "$0" >> "$ATLAS_FIXTURE/checks"
CHECK
  chmod +x "$check"
done
cat > "$fixture/bin/npm" <<'NPM'
#!/bin/sh
printf 'npm %s\n' "$*" >> "$ATLAS_FIXTURE/checks"
NPM
cat > "$fixture/bin/cargo" <<'CARGO'
#!/bin/sh
echo 'cargo check cannot compile the absent or stale frontend' >&2
exit 17
CARGO
cat > "$fixture/bin/gh" <<'GH'
#!/bin/sh
set -eu
case "$1 $2" in
  'auth status') exit 0 ;;
  'repo view') printf 'https://github.com/fixture/atlas\n' ;;
  'release view'|'pr view') exit 1 ;;
  'api --paginate') [ ! -f "$ATLAS_FIXTURE/release-error" ] ;;
  'pr list')
    [ ! -f "$ATLAS_FIXTURE/pr-error" ] || exit 1
    if [ -f "$ATLAS_FIXTURE/pr" ]; then cat "$ATLAS_FIXTURE/pr"; else echo '[]'; fi ;;
  'pr create')
    shift 2
    while [ "$#" -gt 0 ]; do
      case "$1" in
        --base) base=$2 ;;
        --head) head=$2 ;;
        --repo) repository=$2 ;;
        --title|--body) ;;
        *) exit 22 ;;
      esac
      shift 2
    done
    [ "$base" = main ] && [ "$head" = "$(git branch --show-current)" ] && [ "$repository" = 'https://github.com/fixture/atlas' ] || exit 22
    [ ! -f "$ATLAS_FIXTURE/pr" ] || exit 23
    [ ! -f "$ATLAS_FIXTURE/create-fail" ] || exit 24
    printf '[{"url":"https://github.com/fixture/atlas/pull/1","state":"OPEN","isCrossRepository":false,"baseRefName":"main","headRefName":"%s","headRefOid":"%s"}]\n'  \
      "$(git branch --show-current)" "$(git rev-parse HEAD)" > "$ATLAS_FIXTURE/pr"
    echo created >> "$ATLAS_FIXTURE/created"
    [ ! -f "$ATLAS_FIXTURE/create-lost" ] || exit 25
    echo https://github.com/fixture/atlas/pull/1 ;;
  *) echo "unexpected gh operation: $*" >&2; exit 1 ;;
esac
GH
export ATLAS_FIXTURE_GIT
ATLAS_FIXTURE_GIT=$(command -v git)
cat > "$fixture/bin/git" <<'GIT'
#!/bin/sh
if [ "${1:-} ${2:-}" = 'push --no-verify' ] && [ -f "$ATLAS_FIXTURE/push-lost" ]; then
  "$ATLAS_FIXTURE_GIT" "$@" || exit $?
  exit 26
fi
exec "$ATLAS_FIXTURE_GIT" "$@"
GIT
chmod +x "$fixture/bin/"*
export PATH="$fixture/bin:$PATH"
git add .
git commit -qm base
git push -qu origin main
prep() { scripts/prepare-release.sh --prepare-pr --version "$1" </dev/null; }
open_pr() { scripts/prepare-release.sh --open-pr --yes "$@" </dev/null; }
reject() {
  if "$@" > "$fixture/rejection" 2>&1; then echo "unexpected success: $*" >&2; exit 1; fi
}
branch=release-prep/v0.1.0
reject scripts/prepare-release.sh --prepare-pr --version 0.1.0 --notes-file ../invalid.md
[ "$(git branch --show-current)" = main ]
prep 0.1.0
git diff --exit-code -- docs/releases/v0.1.0.md
[ "$(grep -c 'version = "0.1.0-rc.6"' Cargo.lock)" = 1 ]
[ "$(grep -c 'version = "0.1.0-rc.6"' crates/atlas-cli/Cargo.toml)" = 1 ]
open_pr --dry-run
[ "$(git rev-parse HEAD)" = "$(git rev-parse main)" ]
[ -z "$(git ls-remote --heads origin "refs/heads/$branch")" ]
printf 'extra\n' >> Cargo.lock
reject open_pr
node -e 'const fs=require("node:fs");fs.writeFileSync("Cargo.lock",fs.readFileSync("Cargo.lock","utf8").replace(/extra\n$/,""));'
touch unexpected
reject open_pr
rm unexpected
touch "$fixture/release-error"
reject open_pr
rm "$fixture/release-error"
touch "$fixture/pr-error"
reject open_pr
rm "$fixture/pr-error"
[ "$(git rev-parse HEAD)" = "$(git rev-parse main)" ]

# A failed push leaves one valid local commit. Advance main independently.
cat > "$fixture/origin.git/hooks/pre-receive" <<'HOOK'
#!/bin/sh
exit 1
HOOK
chmod +x "$fixture/origin.git/hooks/pre-receive"
reject open_pr
candidate=$(git rev-parse HEAD)
[ "$(git rev-list --count main..HEAD)" = 1 ]
rm "$fixture/origin.git/hooks/pre-receive"
git switch -q main
printf 'unrelated main work\n' > advance
git add advance
git commit -qm advance
git push -q origin main
git switch -q "$branch"

# Refuse an unrelated committed edit and a different remote branch head.
touch other
git add other
git commit -qm unrelated
reject open_pr
git reset -q --hard "$candidate"
git push -q origin "main:refs/heads/$branch"
reject open_pr
git push -q origin --delete "$branch"

# Lost push acknowledgement means the retry sees this same commit upstream.
touch "$fixture/push-lost"
reject open_pr
[ ! -e "$fixture/pr" ]
rm "$fixture/push-lost"
[ "$(git ls-remote --heads origin "refs/heads/$branch" | cut -f1)" = "$candidate" ]
touch "$fixture/create-fail"
reject open_pr
[ ! -e "$fixture/pr" ]
rm "$fixture/create-fail"
touch "$fixture/create-lost"
reject open_pr
[ -s "$fixture/pr" ]
cp "$fixture/pr" "$fixture/good-pr"
for pair in 'main wrong' 'release-prep/v0.1.0 different' 'OPEN CLOSED' 'false true' "$candidate deadbeef"; do
  set -- $pair
  sed "s|$1|$2|" "$fixture/good-pr" > "$fixture/pr"
  reject open_pr
done
cp "$fixture/good-pr" "$fixture/pr"
open_pr | grep -F 'https://github.com/fixture/atlas/pull/1'
[ "$(git rev-parse HEAD)" = "$candidate" ]
[ "$(wc -l < "$fixture/created" | tr -d ' ')" = 1 ]
for check in scripts/release/validate-release-tooling.sh 'npm --prefix web/atlas-ui ci' 'npm --prefix web/atlas-ui run build' scripts/verify.sh; do
  grep -Fxq "$check" "$fixture/checks"
done

git switch -q main
mkdir -p web/atlas-ui/dist
echo stale > web/atlas-ui/dist/index.html
prep 0.2.0
grep -q 'version = "0.2.0"' Cargo.lock
grep -q 'version = "0.2.0"' crates/atlas-cli/Cargo.toml
[ "$(cat web/atlas-ui/dist/index.html)" = stale ]
# An interrupted prepare left one finished file. Retry from its prep branch.
git restore -- Cargo.lock
scripts/prepare-release.sh --prepare-pr --version 0.2.0 --dry-run </dev/null
git diff --exit-code -- Cargo.lock
prep 0.2.0
grep -q 'version = "0.2.0"' Cargo.lock
printf '# v0.2.0\n\nAuthored notes.\n' > docs/releases/v0.2.0.md
rm "$fixture/pr" "$fixture/create-lost"
if [ "$(git config --type=bool --default true --get core.filemode)" = true ]; then
  chmod +x docs/releases/v0.2.0.md
  reject open_pr
  chmod -x docs/releases/v0.2.0.md
  [ "$(git rev-parse HEAD)" = "$(git rev-parse main)" ]
fi
open_pr
[ "$(git rev-list --count main..HEAD)" = 1 ]
[ "$(wc -l < "$fixture/created" | tr -d ' ')" = 2 ]
git switch -q main
sed 's/version = "0.1.0-rc.6"/version = "9.9.9"/g' Cargo.lock > Cargo.lock.changed
mv Cargo.lock.changed Cargo.lock
git add Cargo.lock
git commit -qm 'inconsistent fixture'
git push -q origin main
reject prep 0.3.0
git diff --exit-code
[ "$(git branch --show-current)" = main ]
git restore --source=HEAD^ -- Cargo.lock
git add Cargo.lock
git commit -qm 'correct inconsistent fixture'
git push -q origin main
prep 0.3.0
grep -q 'version = "0.3.0"' Cargo.lock
grep -q 'version = "0.3.0"' crates/atlas-cli/Cargo.toml
echo 'Release preparation and PR retry fixtures passed'
