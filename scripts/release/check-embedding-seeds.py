#!/usr/bin/env python3
"""Check public embedding coverage with the candidate Atlas executable."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile

PROFILES = ("bge-base", "embeddinggemma")
ROOT = Path(__file__).resolve().parents[2]
PINS = ROOT / "crates/atlas-runtime/embedding-seeds.json"
TRANSITIONS = ROOT / "scripts/validation/embedding-seed-transitions.json"


def run(binary, *args):
    result = subprocess.run([binary, *args, "--json"], check=False, stdout=subprocess.PIPE)
    payload = json.loads(result.stdout)
    if payload.get("status") != "ok":
        raise RuntimeError(json.dumps(payload))
    return result.returncode, payload["data"]


def setup(binary, index, profiles, *, audit=False):
    args = ["setup", "--path-mode", "global", "--index", str(index)]
    for profile in profiles:
        args.extend(["--search-profile", profile])
    if audit:
        args.append("--audit-embedding-coverage")
    return run(binary, *args)


def covered(data, profiles):
    build = data.get("build")
    if not data.get("ready") or not build or build["inferred_inputs"] != 0:
        raise RuntimeError(f"clean setup did not complete with zero inference: {data}")
    coverage = {report["profile_id"]: report for report in build["profiles"]}
    if set(coverage) != set(profiles) or len(build["profiles"]) != len(profiles):
        raise RuntimeError(f"clean setup did not cover the requested profiles: {coverage}")
    for profile, report in coverage.items():
        if report["inferred_inputs"] != 0 or report["reused_inputs"] <= 0:
            raise RuntimeError(f"missing precomputed coverage for {profile}: {report}")
    return coverage


def search(binary, index, profile):
    code, query = run(
        binary, "search", "fireball explosion", "--retrieval", "vector",
        "--index", str(index), "--search-profile", profile, "--limit", "3",
    )
    if code or not query.get("rows"):
        raise RuntimeError(f"semantic search failed for {profile}: {query}")


def classify_gap(profile, pin, code, data, transitions):
    gap = data.get("embedding_coverage_gap")
    if code != 3 or data.get("ready") is not False or "build" in data or not isinstance(gap, dict):
        raise RuntimeError(f"unexpected embedding setup failure for {profile}: {data}")
    if gap.get("profile") != profile:
        raise RuntimeError(f"coverage gap names a different profile: {gap}")
    kind = gap.get("kind")
    if (kind == "missing_input" and set(gap) == {"kind", "profile", "first_hash"}
            and re.fullmatch(r"[0-9a-f]{64}", gap.get("first_hash", ""))):
        return {"status": "gap", "reason": "missing_input", "first_hash": gap["first_hash"]}
    if kind == "incompatible_model" and set(gap) == {"kind", "profile", "previous", "expected"}:
        expected = {"pin": pin, "previous": gap["previous"], "expected": gap["expected"]}
        if transitions.get(profile) == expected:
            return {"status": "gap", "reason": "reviewed_model_transition"}
    raise RuntimeError(f"unreviewed or invalid embedding coverage gap for {profile}: {gap}")


def check_strict(binary, directory):
    index = directory / "workspace.sqlite"
    code, data = setup(binary, index, PROFILES)
    if code:
        raise RuntimeError(f"clean setup failed: {data}")
    coverage = covered(data, PROFILES)
    for profile in PROFILES:
        search(binary, index, profile)
    return coverage


def check_advisory(binary, directory, pins, transitions):
    result = {}
    for profile in PROFILES:
        with tempfile.TemporaryDirectory(prefix=f"{profile}-", dir=directory) as workspace:
            index = Path(workspace) / "workspace.sqlite"
            code, data = setup(binary, index, (profile,), audit=True)
            if data.get("embedding_seed_pins") != pins:
                raise RuntimeError("candidate executable's embedding seed pins differ from the checkout")
            if code:
                result[profile] = classify_gap(profile, pins[profile], code, data, transitions)
                continue
            coverage = covered(data, (profile,))[profile]
            search(binary, index, profile)
            result[profile] = {"status": "covered", "reused_inputs": coverage["reused_inputs"],
                               "inferred_inputs": coverage["inferred_inputs"]}
    return result


def file_digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--advisory", action="store_true")
    parser.add_argument("--report", type=Path)
    parser.add_argument("--source-sha")
    parser.add_argument("--tag")
    parser.add_argument("--archive", type=Path)
    args = parser.parse_args()
    if args.report and (not args.advisory or not re.fullmatch(r"[0-9a-f]{40}", args.source_sha or "")
                        or not args.tag or not args.archive):
        parser.error("--report requires advisory mode and the exact source, tag, and archive")
    if args.advisory:
        actual_sha = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
        if args.report and actual_sha != args.source_sha:
            raise RuntimeError("packaged coverage source SHA differs from the checkout")
    binary = str(args.binary.resolve(strict=True))
    with tempfile.TemporaryDirectory(prefix="atlas-release-seeds-") as directory:
        directory = Path(directory)
        if args.advisory:
            pins = json.loads(PINS.read_text(encoding="utf-8"))
            transitions = json.loads(TRANSITIONS.read_text(encoding="utf-8"))
            if set(pins) != set(PROFILES) or not set(transitions).issubset(PROFILES):
                raise RuntimeError("invalid embedding seed pins or transition profiles")
            coverage = check_advisory(binary, directory, pins, transitions)
            if args.report:
                report = {
                    "version": 1,
                    "source_sha": args.source_sha,
                    "tag": args.tag,
                    "archive_sha256": file_digest(args.archive),
                    "pins": pins,
                    "profiles": coverage,
                }
                args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
            coverage = {"source_sha": actual_sha, "pins": pins, "profiles": coverage}
        else:
            coverage = check_strict(binary, directory)
        print(json.dumps(coverage, indent=2))


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, KeyError, ValueError, OSError, TypeError) as error:
        sys.exit(
            f"Published embedding coverage failed. See the embedding asset refresh "
            f"procedure in docs/development/releases.md.\n{error}"
        )
