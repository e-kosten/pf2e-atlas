#!/usr/bin/env python3
"""Require a packaged Atlas to build and query both profiles using public seeds."""

import json
from pathlib import Path
import subprocess
import sys
import tempfile

PROFILES = ("bge-base", "embeddinggemma")


def run(binary, *args):
    result = subprocess.run([binary, *args, "--json"], check=False, stdout=subprocess.PIPE)
    if result.returncode:
        raise RuntimeError(result.stdout.decode(errors="replace"))
    payload = json.loads(result.stdout)
    if payload.get("status") != "ok":
        raise RuntimeError(json.dumps(payload))
    return payload["data"]


def main():
    binary = str(Path(sys.argv[1]).resolve(strict=True))
    with tempfile.TemporaryDirectory(prefix="atlas-release-seeds-") as directory:
        index = str(Path(directory) / "workspace.sqlite")
        args = ["setup", "--path-mode", "global", "--index", index]
        for profile in PROFILES:
            args.extend(["--search-profile", profile])
        setup = run(binary, *args)
        build = setup.get("build")
        if not setup.get("ready") or not build or build["inferred_inputs"] != 0:
            raise RuntimeError(f"clean setup did not complete with zero inference: {setup}")
        coverage = {report["profile_id"]: report for report in build["profiles"]}
        if set(coverage) != set(PROFILES):
            raise RuntimeError(f"clean setup did not cover both profiles: {coverage}")
        for profile, report in coverage.items():
            if report["inferred_inputs"] != 0 or report["reused_inputs"] <= 0:
                raise RuntimeError(f"missing precomputed coverage for {profile}: {report}")
            query = run(
                binary, "search", "fireball explosion", "--retrieval", "vector",
                "--index", index, "--search-profile", profile, "--limit", "3",
            )
            if not query.get("rows"):
                raise RuntimeError(f"semantic search returned no results for {profile}: {query}")
        print(json.dumps(coverage, indent=2))


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, KeyError, ValueError) as error:
        sys.exit(
            f"Published embedding coverage failed. See the embedding asset refresh "
            f"procedure in docs/development/releases.md.\n{error}"
        )
