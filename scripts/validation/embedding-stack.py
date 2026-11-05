#!/usr/bin/env python3
"""Verify the embedding dependency set recorded for compatibility review."""
import argparse
import json
from pathlib import Path
import tomllib

ROOT = Path(__file__).resolve().parents[2]
RECEIPT = ROOT / "scripts/validation/embedding-stack.json"
PACKAGES = {
    "fastembed", "tokenizers", "text-splitter", "ort", "ort-sys",
    "onig", "onig_sys", "ndarray", "matrixmultiply",
}


def snapshot():
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
    packages = [entry for entry in lock["package"] if entry["name"] in PACKAGES]
    missing = PACKAGES - {entry["name"] for entry in packages}
    if missing:
        raise ValueError(f"Review embedding provenance coverage for {', '.join(sorted(missing))}")
    manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
    declared = manifest["workspace"]["dependencies"]
    return {
        "packages": sorted(packages, key=lambda entry: (entry["name"], entry["version"])),
        "workspace_dependencies": {name: declared[name] for name in PACKAGES if name in declared},
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--print", action="store_true", help="Print the current dependency set for review")
    args = parser.parse_args()
    current = snapshot()
    if args.print:
        print(json.dumps(current, indent=2, sort_keys=True))
    elif current != json.loads(RECEIPT.read_text()):
        parser.exit(1, "Embedding stack changed. Run the compatibility review in docs/development/dependencies.md.\n")
    else:
        print("Embedding stack matches the reviewed dependency set.")
