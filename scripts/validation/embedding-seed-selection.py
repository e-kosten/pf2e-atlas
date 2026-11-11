#!/usr/bin/env python3
"""Select public embedding coverage for changes affecting managed search setup."""

import subprocess
import sys


def affects_coverage(path):
    return path.startswith((
        ".cargo/", "crates/atlas-embedding/", "crates/atlas-ingest/",
        "crates/atlas-index/", "crates/atlas-runtime/", "crates/atlas-cli/",
        "crates/atlas-domain/", "crates/atlas-storage/", "crates/atlas-record/",
        "crates/atlas-sqlite-vec/", "crates/atlas-foundry-model/", "crates/atlas-search/",
        "crates/atlas-app-model/", "crates/atlas-app-service/", "crates/atlas-cli-support/",
        "crates/atlas-local-state/", "dev-tools/source-contracts/", "dev-tools/release/src/",
        "scripts/validation/embedding-seed-",
    )) or path in {
        ".github/workflows/ci.yml", ".github/workflows/release-build-check.yml",
        ".github/workflows/release.yml", ".github/workflows/nightly.yml",
        "Cargo.lock", "Cargo.toml", "rust-toolchain.toml",
        "scripts/release/check-embedding-seeds.py", "scripts/validation/ci-results.mjs",
        "scripts/validation/embedding-stack.py", "scripts/validation/embedding-baseline.json",
    }


if __name__ == "__main__":
    base, head = sys.argv[1:]
    paths = subprocess.check_output(
        ["git", "diff", "--name-only", "-z", "--no-renames", f"{base}...{head}"],
        text=True,
    ).split("\0")
    print("true" if any(map(affects_coverage, paths)) else "false")
