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
RELEASE_PREDECESSOR_PIN = "scripts/validation/schema-42-predecessor.json"
RELEASE_TARGETS = {
    "x86_64-unknown-linux-gnu": "ubuntu-24.04",
    "aarch64-unknown-linux-gnu": "ubuntu-24.04-arm",
    "aarch64-apple-darwin": "macos-26",
    "x86_64-pc-windows-msvc": "windows-2022",
}
FULL_MATRIX = list(RELEASE_TARGETS.values())


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


SCHEMA_PATHS = {
    "crates/atlas-runtime/src/release_history.rs",
    "crates/atlas-index/src/release_schema.sql",
    "crates/atlas-runtime/src/workspace_settings.sql",
    "crates/atlas-runtime/src/release_migration_tests.rs",
    "scripts/validation/check-workspace-upgrade.py",
}
SCHEMA_PREFIXES = (
    "crates/atlas-local-state/src/release/",
    "crates/atlas-runtime/tests/fixtures/schema-",
)


def schema_path(path):
    return path in SCHEMA_PATHS or path.startswith(SCHEMA_PREFIXES)


def valid_base(base):
    return subprocess.run(
        ["git", "cat-file", "-e", f"{base}^{{commit}}"], cwd=ROOT,
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    ).returncode == 0


def probe_depth(base):
    if not base or not valid_base(base):
        return {"build": True, "near_limit": True}
    paths = set(git("diff", "--no-renames", "--name-only", base).splitlines())
    before, after = snapshot(base), snapshot()
    return {
        "build": any(
            path.startswith(("crates/atlas-ingest/src/", "crates/atlas-record/src/",
                             "crates/atlas-index/src/", "crates/atlas-storage/src/"))
            and not schema_path(path)
            for path in paths
        ),
        "near_limit": before != after or any(
            path.startswith(("crates/atlas-embedding/src/", ".cargo/"))
            or path.endswith("/Cargo.toml")
            or path in {"Cargo.toml", "rust-toolchain.toml", "dist-workspace.toml",
                        ".github/workflows/release-build-check.yml",
                        "crates/atlas-ingest/src/native_execution.rs"}
            or path.startswith(("crates/atlas-cli/src/commands/setup",
                                "crates/atlas-runtime/src/setup"))
            for path in paths
        ),
    }


def selected_checks(base, manual, reusable=False, packaged=False):
    if not (ROOT / BASELINE_PIN).exists():
        raise ValueError("The embedding baseline pin is missing")
    if reusable and not manual:
        return {"model": [], "workspace": []}
    if manual or not base or not valid_base(base):
        return {"model": FULL_MATRIX, "workspace": FULL_MATRIX}
    paths = set(git("diff", "--no-renames", "--name-only", base).splitlines())
    schema = any(map(schema_path, paths))
    before, after = snapshot(base), snapshot()
    native = PACKAGES - {"text-splitter"}
    watched = {
        BASELINE_PIN, RELEASE_PREDECESSOR_PIN, "scripts/validation/embedding-fixtures-v2.json", ".github/workflows/ci.yml",
        ".github/workflows/release-build-check.yml", "scripts/validation/embedding-stack.py",
        "scripts/validation/probe-embedding-upgrade.py", "scripts/validation/test-embedding-baseline.py",
        "crates/atlas-runtime/examples/read-fixture-vectors.rs",
        "crates/atlas-runtime/examples/check-fixture-inference.rs",
        "crates/atlas-runtime/examples/migrate-fixture-workspace.rs", ".gitattributes", "rust-toolchain.toml",
        "dist-workspace.toml", "crates/atlas-search/src/text.rs",
        "crates/atlas-search/src/service.rs",
    }
    if packaged:
        watched = {".gitattributes", "rust-toolchain.toml", "dist-workspace.toml",
                   "crates/atlas-search/src/text.rs", "crates/atlas-search/src/service.rs"}
    model_paths = paths - {path for path in paths if schema_path(path)}
    full = (model_paths & watched
            or any(path.startswith((".cargo/", "crates/atlas-sqlite-vec/", "crates/atlas-embedding/"))
                   or path.endswith("/Cargo.toml") or path == "Cargo.toml"
                   or path == "crates/atlas-ingest/src/native_execution.rs" for path in model_paths)
            or [row for row in before["packages"] if row["name"] in native]
            != [row for row in after["packages"] if row["name"] in native]
            or any(before["workspace_dependencies"].get(name) != after["workspace_dependencies"].get(name)
                   for name in native))
    integration = before != after or any(path.startswith((
        "crates/atlas-index/", "crates/atlas-ingest/", "crates/atlas-record/",
        "crates/atlas-runtime/", "crates/atlas-search/", "crates/atlas-storage/",
    )) for path in model_paths)
    setup = any(path.startswith(("crates/atlas-ingest/tests/fixtures/foundry-source/core-record-types/",
                                 "crates/atlas-cli/src/commands/setup",
                                 "crates/atlas-runtime/src/setup")) for path in paths)
    return {
        "model": FULL_MATRIX if full else FULL_MATRIX[:1] if integration else [],
        "workspace": FULL_MATRIX if schema or setup else [],
    }


def ancestor(base):
    return subprocess.run(
        ["git", "merge-base", "--is-ancestor", base, "HEAD"], cwd=ROOT,
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    ).returncode == 0


def packaged_build_inputs_changed(base):
    path = ".github/workflows/release-build-check.yml"
    before = git("show", f"{base}:{path}")
    after = (ROOT / path).read_text(encoding="utf-8")

    def build_steps(workflow):
        # The assembly and qualification steps do not affect the shipped bits.
        # Compare the build job through the archive construction steps.
        start = workflow.index("      - name: Download verified cargo-dist archive")
        end = workflow.index("      - name: Stage CLI-only artifacts", start)
        return workflow[start:end]

    try:
        return build_steps(before) != build_steps(after)
    except ValueError:
        return True


def release_targets():
    receipt = json.loads((ROOT / RELEASE_PREDECESSOR_PIN).read_text())
    producer = receipt["commit"]
    pin = json.loads((ROOT / BASELINE_PIN).read_text())
    assets = list(RELEASE_TARGETS)
    v2 = ROOT / "scripts/validation/embedding-fixtures-v2.json"
    if v2.exists():
        receipts = json.loads(v2.read_text())
        if (receipts.keys() != RELEASE_TARGETS.keys()
                or any(profiles.keys() != pin.keys() for profiles in receipts.values())):
            raise ValueError("v2 fixtures must cover all release targets")
        producers = {payload["commit"] for profiles in receipts.values()
                     for row in profiles.values() for payload in row.values()}
        if not producers:
            raise ValueError("v2 fixture producer receipts are missing")
        return assets if any(
            not ancestor(base) or packaged_build_inputs_changed(base)
            or selected_checks(base, False, packaged=True)["model"] == FULL_MATRIX
            for base in producers
        ) else []
    if receipt["assets"].keys() != RELEASE_TARGETS.keys():
        raise ValueError("Predecessor receipt must cover all release targets")
    if (any(row["tag"] != receipt["tag"] or row["asset"] != "cli-only-release"
            for row in pin.values())
            or not ancestor(producer)):
        return assets
    runners = selected_checks(producer, False, packaged=True)["model"]
    # A retrieval-only change keeps its normal PR integration check. Release
    # qualification covers every packaged target when numerical selection fires.
    return assets if runners == FULL_MATRIX or packaged_build_inputs_changed(producer) else []


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--print", action="store_true", help="Print the current dependency set for review")
    mode.add_argument("--release-targets", action="store_true")
    mode.add_argument("--select", action="store_true", help="Print the baseline probe runner matrix")
    parser.add_argument("--base", default="")
    parser.add_argument("--manual", action="store_true")
    parser.add_argument("--reusable", action="store_true")
    args = parser.parse_args()
    if args.release_targets:
        print(json.dumps(release_targets()))
        raise SystemExit(0)
    if args.select:
        result = selected_checks(args.base, args.manual, args.reusable)
        result.update(probe_depth(args.base) if result["model"] and not args.manual
                      else {"build": False, "near_limit": False})
        print(json.dumps(result))
        raise SystemExit(0)
    current = snapshot()
    if args.print:
        print(json.dumps(current, indent=2, sort_keys=True))
    elif current != json.loads(RECEIPT.read_text()):
        parser.exit(1, "Embedding stack changed. Run the compatibility review in docs/development/dependencies.md.\n")
    else:
        print("Embedding stack matches the reviewed dependency set.")
