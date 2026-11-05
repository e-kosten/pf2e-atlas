#!/usr/bin/env python3
"""Exercise a candidate's compatibility with a genuine predecessor index."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[2]
PROFILES = ("bge-base", "embeddinggemma")
QUERIES = ("frighten an enemy using a threatening display", "restore health to a wounded ally")
SCORE_TOLERANCE = 2e-5


def invoke(executable, args, evidence):
    result = subprocess.run(
        [str(executable), *map(str, args), "--path-mode", "repo", "--json"],
        capture_output=True, text=True, timeout=240,
    )
    evidence.write_text(result.stdout)
    if result.returncode:
        raise RuntimeError(f"{executable.name} {' '.join(args[:2])} failed. See {evidence}: {result.stderr}")
    return json.loads(result.stdout)["data"]


def build(executable, source, index, cache, evidence):
    return invoke(executable, [
        "index", "build", "--source", source, "--output", index,
        "--embedding-cache-path", cache,
        "--search-profile", PROFILES[0], "--search-profile", PROFILES[1],
    ], evidence)


def search(executable, index, cache, profile, query, evidence):
    result = invoke(executable, [
        "search", query, "--index", index, "--embedding-cache-path", cache,
        "--search-profile", profile, "--retrieval", "vector", "--limit", "5",
    ], evidence)
    rows = [(row["record"]["record_key"], [
        (match["navigation"], match["semantic_similarity"])
        for match in row["matches"] if match["semantic_similarity"] is not None
    ]) for row in result["rows"]]
    if len(rows) < 2 or not any(matches for _, matches in rows):
        raise RuntimeError(f"Insufficient semantic coverage in {evidence}")
    return rows


def compare(expected, actual, name):
    if [key for key, _ in actual] != [key for key, _ in expected]:
        raise RuntimeError(f"Ranked records changed for {name}")
    for (_, before), (_, after) in zip(expected, actual, strict=True):
        if [location for location, _ in after] != [location for location, _ in before]:
            raise RuntimeError(f"Matching passages changed for {name}")
        for (_, old), (_, new) in zip(before, after, strict=True):
            if not math.isclose(old, new, rel_tol=0, abs_tol=SCORE_TOLERANCE):
                raise RuntimeError(f"Semantic score changed for {name}: {old} -> {new}")


def sha256(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for argument in ("baseline", "candidate", "model-cache", "output"):
        parser.add_argument("--" + argument, required=True, type=lambda value: Path(value).resolve())
    args = parser.parse_args()
    root = args.output
    root.mkdir(parents=True)
    source = root / "source"
    shutil.copytree(ROOT / "crates/atlas-ingest/tests/fixtures/foundry-source/core-record-types", source)
    for ordinal, path in enumerate(sorted(source.glob("packs/*/*.json")), start=1):
        record = json.loads(path.read_text())
        record["_id"] = f"{ordinal:016d}"
        path.write_text(json.dumps(record))
    locale = source / "static/lang/en.json"
    locale.parent.mkdir(parents=True)
    locale.write_text("{}\n")

    old = root / "predecessor.sqlite"
    fresh = root / "fresh.sqlite"
    original = build(args.baseline, source, old, args.model_cache, root / "baseline-build.json")
    if original["inferred_inputs"] <= 0:
        raise RuntimeError("Predecessor lacks real inference evidence")
    invoke(args.baseline, [
        "lists", "create", "compatibility", "--name", "Compatibility sentinel",
        "--record", "actions:0000000000000001", "--index", old,
    ], root / "list-create.json")
    saved_before = invoke(args.baseline, ["lists", "show", "compatibility", "--index", old],
                          root / "list-before.json")
    for profile in PROFILES:
        for ordinal, query in enumerate(QUERIES):
            prefix = f"{profile}-{ordinal}"
            baseline = search(args.baseline, old, args.model_cache, profile, query, root / f"{prefix}-baseline.json")
            repeat = search(args.baseline, old, args.model_cache, profile, query, root / f"{prefix}-repeat.json")
            compare(baseline, repeat, prefix + " baseline repeat")
            upgraded = search(args.candidate, old, args.model_cache, profile, query, root / f"{prefix}-existing.json")
            compare(baseline, upgraded, prefix + " new query against old vectors")

    updated = build(args.candidate, source, old, args.model_cache, root / "candidate-reuse.json")
    if updated["inferred_inputs"] != 0 or updated["reused_inputs"] <= 0:
        raise RuntimeError("Candidate did not reuse the predecessor vectors")
    saved_after = invoke(args.candidate, ["lists", "show", "compatibility", "--index", old],
                         root / "list-after.json")
    if saved_before != saved_after:
        raise RuntimeError("Saved list changed across compatible maintenance")

    build(args.candidate, source, fresh, args.model_cache, root / "candidate-fresh.json")
    for profile in PROFILES:
        for ordinal, query in enumerate(QUERIES):
            prefix = f"{profile}-{ordinal}"
            reused = search(args.candidate, old, args.model_cache, profile, query, root / f"{prefix}-reused.json")
            rebuilt = search(args.candidate, fresh, args.model_cache, profile, query, root / f"{prefix}-fresh.json")
            compare(reused, rebuilt, prefix + " reused and freshly inferred vectors")

    path = source / "packs/actions/demoralize.json"
    record = json.loads(path.read_text())
    record["system"]["description"]["value"] = "<p>Threaten an opponent to make it frightened.</p>"
    path.write_text(json.dumps(record))
    mixed = build(args.candidate, source, old, args.model_cache, root / "candidate-mixed.json")
    if not (0 < mixed["inferred_inputs"] < original["inferred_inputs"] and mixed["reused_inputs"] > 0):
        raise RuntimeError("Incremental source change did not select both reuse and inference")

    result = {
        "baseline_sha256": sha256(args.baseline),
        "candidate_sha256": sha256(args.candidate),
        "profiles": PROFILES,
        "score_tolerance": SCORE_TOLERANCE,
        "reused_inputs": updated["reused_inputs"],
        "inferred_inputs": updated["inferred_inputs"],
        "mixed_inferred_inputs": mixed["inferred_inputs"],
    }
    (root / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(f"Embedding upgrade probe passed. Evidence is in {root}.")


if __name__ == "__main__":
    main()
