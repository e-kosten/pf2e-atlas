#!/usr/bin/env sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)

sh -n \
  "$repo_root/scripts/prepare-release.sh" \
  "$repo_root/scripts/install/atlas-installer.sh" \
  "$repo_root/scripts/release/validate-release-assets.sh" \
  "$repo_root/scripts/release/test-release-tools.sh" \
  "$repo_root/scripts/release/test-prepare-release.sh"

PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile \
  "$repo_root/scripts/release/generate-notices.py" \
  "$repo_root/scripts/release/generate-release-manifest.py" \
  "$repo_root/scripts/release/prune-dist-manifest.py"

python3 "$repo_root/scripts/release/generate-notices.py" --check >/dev/null

ruby -e 'require "yaml"; YAML.load_file(ARGV.fetch(0))' "$repo_root/.github/workflows/release.yml"

if command -v pwsh >/dev/null 2>&1; then
  pwsh -NoProfile -Command "\$null = [System.Management.Automation.Language.Parser]::ParseFile('$repo_root/scripts/install/atlas-installer.ps1', [ref]\$null, [ref]\$errors); if (\$errors.Count -gt 0) { \$errors | ForEach-Object { Write-Error \$_ }; exit 1 }"
  pwsh -NoProfile -Command "\$null = [System.Management.Automation.Language.Parser]::ParseFile('$repo_root/scripts/release/test-installer.ps1', [ref]\$null, [ref]\$errors); if (\$errors.Count -gt 0) { \$errors | ForEach-Object { Write-Error \$_ }; exit 1 }"
else
  printf 'warning: pwsh not found; PowerShell parser validation is skipped locally\n' >&2
fi

if command -v actionlint >/dev/null 2>&1; then
  actionlint "$repo_root/.github/workflows/release.yml"
else
  printf 'warning: actionlint not found; workflow semantic lint is skipped locally\n' >&2
fi

printf 'release tooling static validation passed\n'
