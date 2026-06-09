#!/usr/bin/env python3
import json
import subprocess
import sys
from pathlib import Path


def main() -> int:
    check = sys.argv[1:] == ["--check"]
    if sys.argv[1:] and not check:
        print("usage: generate-notices.py [--check]", file=sys.stderr)
        return 2

    root = Path(__file__).resolve().parents[2]
    metadata = subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1", "--locked"],
        cwd=root,
        encoding="utf-8",
        text=True,
    )
    data = json.loads(metadata)
    packages = sorted(
        data["packages"],
        key=lambda package: (package["name"].lower(), package["version"]),
    )

    lines = [
        "# Third-Party Notices",
        "",
        "This file is generated for PF2e Atlas release artifacts.",
        "It is intended as release hygiene and is not legal advice.",
        "",
        "## Project",
        "",
        "- PF2e Atlas: MIT",
        "",
        "## Rust Dependencies",
        "",
    ]
    for package in packages:
        license_expr = package.get("license") or "UNKNOWN"
        source = package.get("source") or "workspace"
        lines.append(f"- {package['name']} {package['version']}: {license_expr} ({source})")

    lines.extend(
        [
            "",
            "## Bundled Native Libraries",
            "",
            "PF2e Atlas uses the `ort` Rust crate, which may copy ONNX Runtime native",
            "libraries into release build outputs. ONNX Runtime is published by Microsoft",
            "under the MIT License.",
            "",
            "- ONNX Runtime: MIT",
            "  - Project: https://github.com/microsoft/onnxruntime",
            "  - License: https://github.com/microsoft/onnxruntime/blob/main/LICENSE",
            "  - Third-party notices: https://github.com/microsoft/onnxruntime/blob/main/ThirdPartyNotices.txt",
            "",
            "If future release artifacts bundle additional native libraries, update this",
            "section before publishing the release.",
            "",
        ]
    )

    output = root / "THIRD-PARTY-NOTICES.md"
    content = "\n".join(lines)
    if check:
        current = output.read_text(encoding="utf-8") if output.exists() else ""
        if current != content:
            print(f"{output} is out of date; run scripts/release/generate-notices.py", file=sys.stderr)
            return 1
        print(f"{output} is current")
        return 0
    output.write_text(content, encoding="utf-8")
    print(output)
    return 0


if __name__ == "__main__":
    sys.exit(main())
