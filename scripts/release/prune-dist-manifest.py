#!/usr/bin/env python3
import json
import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: prune-dist-manifest.py <manifest> <dist-dir>", file=sys.stderr)
        return 2

    manifest_path = Path(sys.argv[1])
    dist = Path(sys.argv[2])
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    artifacts = manifest.get("artifacts", {})
    kept = {name: artifact for name, artifact in artifacts.items() if (dist / name).is_file()}
    removed = set(artifacts) - set(kept)
    manifest["artifacts"] = kept

    for release in manifest.get("releases", []):
        release["artifacts"] = [
            name
            for name in release.get("artifacts", [])
            if name in kept and name not in removed
        ]

    manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    sys.exit(main())
