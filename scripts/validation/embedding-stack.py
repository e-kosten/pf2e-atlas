#!/usr/bin/env python3
"""Verify the embedding dependency set recorded for compatibility review."""
import argparse
import json
from pathlib import Path
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[2]
RECEIPT = ROOT / "scripts/validation/embedding-stack.json"
PACKAGES = {
    "fastembed", "tokenizers", "text-splitter", "ort", "ort-sys",
    "onig", "onig_sys", "ndarray", "matrixmultiply",
    "esaxx-rs", "regex", "regex-syntax", "spm_precompiled",
    "unicode-normalization-alignments", "unicode-segmentation", "daachorse",
}


BASELINE_PIN = "scripts/validation/embedding-baseline.json"
RELEASE_PREDECESSOR_PIN = "scripts/validation/schema-41-predecessor.json"
FULL_MATRIX = ["ubuntu-24.04", "ubuntu-24.04-arm", "macos-26", "windows-2022"]


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True)


def snapshot(ref=None):
    def read(path):
        return git("show", f"{ref}:{path}") if ref else (ROOT / path).read_text()

    lock = tomllib.loads(read("Cargo.lock"))
    packages = [entry for entry in lock["package"] if entry["name"] in PACKAGES]
    missing = PACKAGES - {entry["name"] for entry in packages}
    if missing and not ref:
        raise ValueError(f"Review embedding provenance coverage for {', '.join(sorted(missing))}")
    manifest = tomllib.loads(read("Cargo.toml"))
    declared = manifest["workspace"]["dependencies"]
    return {
        "packages": sorted(packages, key=lambda entry: (entry["name"], entry["version"])),
        "workspace_dependencies": {name: declared[name] for name in PACKAGES if name in declared},
    }


def selected_runners(base, manual):
    if not (ROOT / BASELINE_PIN).exists():
        if manual or (base and git("ls-tree", "--name-only", base, BASELINE_PIN).strip()):
            raise ValueError("The embedding baseline pin is missing")
        return []
    if manual:
        return FULL_MATRIX
    if not base:
        return []
    paths = set(git("diff", "--name-only", base).splitlines())
    before, after = snapshot(base), snapshot()
    native = PACKAGES - {"text-splitter"}
    if (paths & {".gitattributes", "rust-toolchain.toml", "crates/atlas-embedding/Cargo.toml"}
            or any(path.startswith((".cargo/", "crates/atlas-sqlite-vec/")) for path in paths)
            or paths & {BASELINE_PIN, RELEASE_PREDECESSOR_PIN,
                        "crates/atlas-runtime/src/release_history.rs",
                        "crates/atlas-index/src/release_schema.sql",
                        "crates/atlas-local-state/src/release.rs"}
            or any(path.startswith(("crates/atlas-local-state/src/release/",
                                    "crates/atlas-runtime/tests/fixtures/schema-41.sqlite"))
                   for path in paths)
            or [row for row in before["packages"] if row["name"] in native]
            != [row for row in after["packages"] if row["name"] in native]
            or any(before["workspace_dependencies"].get(name) != after["workspace_dependencies"].get(name)
                   for name in native)):
        return FULL_MATRIX
    watched = {
        BASELINE_PIN, ".github/workflows/ci.yml", "scripts/validation/embedding-stack.py",
        "scripts/validation/probe-embedding-upgrade.py", "Cargo.lock",
        "scripts/validation/test-embedding-baseline.py",
    }
    if before != after or paths & watched or any(path.startswith((
        "crates/atlas-embedding/", "crates/atlas-index/", "crates/atlas-ingest/",
        "crates/atlas-record/", "crates/atlas-runtime/", "crates/atlas-search/",
        "crates/atlas-storage/",
    )) for path in paths):
        return FULL_MATRIX[:1]
    return []


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--print", action="store_true", help="Print the current dependency set for review")
    mode.add_argument("--select", action="store_true", help="Print the baseline probe runner matrix")
    parser.add_argument("--base", default="")
    parser.add_argument("--manual", action="store_true")
    args = parser.parse_args()
    if args.select:
        print(json.dumps(selected_runners(args.base, args.manual)))
        raise SystemExit(0)
    current = snapshot()
    if args.print:
        print(json.dumps(current, indent=2, sort_keys=True))
    elif current != json.loads(RECEIPT.read_text()):
        parser.exit(1, "Embedding stack changed. Run the compatibility review in docs/development/dependencies.md.\n")
    else:
        print("Embedding stack matches the reviewed dependency set.")
