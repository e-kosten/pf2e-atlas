#!/usr/bin/env python3
import hashlib
import json
import sys
from pathlib import Path


TARGETS = {
    "aarch64-apple-darwin": ("macos", "aarch64"),
    "x86_64-unknown-linux-gnu": ("linux", "x86_64"),
    "aarch64-unknown-linux-gnu": ("linux", "aarch64"),
    "x86_64-pc-windows-msvc": ("windows", "x86_64"),
}


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: generate-release-manifest.py <tag> <dist-dir>", file=sys.stderr)
        return 2
    tag = sys.argv[1]
    dist = Path(sys.argv[2])
    assets = []
    sums = []
    for path in sorted(dist.iterdir()):
        if not path.is_file():
            continue
        name = path.name
        if not (name.endswith(".tar.xz") or name.endswith(".zip")):
            continue
        target = None
        for candidate in TARGETS:
            if candidate in name:
                target = candidate
                break
        if target is None:
            continue
        digest = sha256(path)
        os_name, arch = TARGETS[target]
        assets.append(
            {
                "target": target,
                "os": os_name,
                "arch": arch,
                "name": name,
                "sha256": digest,
            }
        )
        sums.append(f"{digest}  {name}")

    manifest = {"version": tag, "assets": assets}
    (dist / "atlas-release-manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    for extra in ["atlas-installer.sh", "atlas-installer.ps1", "atlas-release-manifest.json", "THIRD-PARTY-NOTICES.md"]:
        extra_path = dist / extra
        if extra_path.exists():
            sums.append(f"{sha256(extra_path)}  {extra}")
    (dist / "SHA256SUMS").write_text("\n".join(sorted(sums)) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    sys.exit(main())
