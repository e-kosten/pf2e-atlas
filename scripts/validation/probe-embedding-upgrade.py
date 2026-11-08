#!/usr/bin/env python3
"""Exercise a candidate's compatibility with a genuine predecessor index."""
import argparse
import gzip
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shutil
import sqlite3
import subprocess
import tarfile
from contextlib import closing

ROOT = Path(__file__).resolve().parents[2]
PROFILES = ("bge-base", "embeddinggemma")
QUERIES = ("frighten an enemy using a threatening display", "restore health to a wounded ally")
SCORE_TOLERANCE = 2e-5
PIN_PATH = "scripts/validation/embedding-baseline.json"


class AtlasCommandError(RuntimeError):
    def __init__(self, executable, args, evidence, stderr):
        self.evidence = evidence
        super().__init__(f"{executable.name} {' '.join(args[:2])} failed. See {evidence}: {stderr}")


def invoke(executable, args, evidence):
    result = subprocess.run(
        [str(executable), *map(str, args), "--path-mode", "repo", "--json"],
        capture_output=True, text=True, timeout=240,
    )
    evidence.write_text(result.stdout)
    if result.returncode:
        raise AtlasCommandError(executable, args, evidence, result.stderr)
    return json.loads(result.stdout)["data"]


def build(executable, source, index, cache, evidence, profiles):
    return invoke(executable, [
        "index", "build", "--source", source, "--output", index,
        "--embedding-cache-path", cache,
        *(arg for profile in profiles for arg in ("--search-profile", profile)),
    ], evidence)


def search(executable, index, cache, profile, query, evidence):
    result = invoke(executable, [
        "search", query, "--index", index, "--embedding-cache-path", cache,
        "--search-profile", profile, "--retrieval", "vector", "--limit", "5",
    ], evidence)
    rows = [[row["record"]["record_key"], [
        [match["navigation"], match["semantic_similarity"]]
        for match in row["matches"] if match["semantic_similarity"] is not None
    ]] for row in result["rows"]]
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


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def profile_contracts(index):
    with closing(sqlite3.connect(index.as_uri() + "?mode=ro", uri=True)) as db:
        contracts = [json.loads(row[0]) for row in db.execute("SELECT identity_json FROM embedding_models")]
    return {contract["profile_id"]: contract for contract in contracts}


def has_trait_seed_chunks(index):
    with closing(sqlite3.connect(index.as_uri() + "?mode=ro", uri=True)) as db:
        return db.execute("SELECT 1 FROM sqlite_schema WHERE type='table' AND name='trait_seed_chunks'").fetchone() is not None


def published_vectors(index, *, retiring_trait_seeds=False):
    """Compare numerical identities across maintenance without depending on row IDs."""
    sql = ("SELECT m.identity_json,i.input_hash,i.token_count,i.published_hash "
           "FROM embedding_inputs i JOIN embedding_models m USING(model_id) "
           "WHERE i.published_hash IS NOT NULL")
    if retiring_trait_seeds:
        sql += (" AND (NOT EXISTS(SELECT 1 FROM trait_seed_chunks t WHERE t.input_id=i.input_id)"
                " OR EXISTS(SELECT 1 FROM semantic_units u WHERE u.input_id=i.input_id))")
    result = {}
    with closing(sqlite3.connect(index.as_uri() + "?mode=ro", uri=True)) as db:
        for identity, input_hash, tokens, vector_hash in db.execute(sql):
            contract = json.loads(identity)
            key = (json.dumps(contract, sort_keys=True), input_hash.hex(), tokens)
            registry = result.setdefault(contract["profile_id"], {})
            if key in registry:
                raise RuntimeError("Duplicate published embedding input identity")
            registry[key] = vector_hash.hex()
    return result


def require_preserved_vectors(profile, before, after, report):
    if report["inferred_inputs"] != 0 or report["reused_inputs"] <= 0:
        raise RuntimeError(f"{profile} did not reuse the predecessor vectors")
    if not before.get(profile) or before[profile] != after.get(profile):
        raise RuntimeError(f"{profile} changed the frozen published vectors")


def policy_refresh_required(old, fresh):
    def context(index):
        with closing(sqlite3.connect(index.as_uri() + "?mode=ro", uri=True)) as db:
            digest, encoded = db.execute(
                "SELECT context_hash,context_json FROM artifact_context WHERE singleton=1").fetchone()
        if hashlib.sha256(encoded.encode()).hexdigest() != digest:
            raise RuntimeError("Artifact context checksum mismatch")
        return json.loads(encoded)

    before, after = context(old), context(fresh)
    # These are the fields behind the grouped policy-admission diagnostic. Normal
    # maintenance must subsequently succeed; this does not bypass its classifier.
    policies = ("query_projection_version", "query_catalog_version", "lookup_version",
                "lexical_selection_version", "fts_tokenizer", "content_interpretation_version",
                "content_selection_version", "localization_policy", "asset_policy",
                "relationship_policy")
    return (all(before[key] == after[key] for key in
                ("format_version", "snapshot_version", "source_contract"))
            and any(before[key] != after[key] for key in policies))


def expected_transition_rejection(failure, old, fresh):
    reason = failure.get("code"), failure.get("message")
    if reason == ("vector_readiness_required", "stored query model differs from the executable contract"):
        return True
    return (reason == ("artifact_incompatible", "projection/lookup/content/FTS policy identity")
            and policy_refresh_required(old, fresh))


def workspace_schema(index):
    with closing(sqlite3.connect(index.as_uri() + "?mode=ro", uri=True)) as db:
        return db.execute("PRAGMA user_version").fetchone()[0]


def migrate_predecessor(old, fresh, migrator, manifest, root):
    before, expected = workspace_schema(old), workspace_schema(fresh)
    if before == expected:
        return {"from": before, "to": expected}
    if before > expected or migrator is None:
        raise RuntimeError(f"Cannot check frozen vectors across workspace schema {before} → {expected}. "
                           "For an upgrade, provide the candidate's --workspace-migrator.")
    subprocess.run([str(migrator), str(old)], check=True, timeout=240)
    if workspace_schema(old) != expected or profile_contracts(old) != manifest["profiles"]:
        raise RuntimeError("Workspace migration did not preserve the frozen embedding contracts")
    schema = {"from": before, "to": expected}
    write_json(root / "workspace-migration.json", schema)
    return schema


def saved_list_state(view):
    summary = view["list"]
    required = ("list_key", "slug", "name", "tags", "created_at", "updated_at")
    return {
        "list": {**{key: summary[key] for key in required}, "description": summary.get("description")},
        "items": [
            {"record_key": row["record_key"], "position": row["position"],
             "note": row.get("note"), "snapshot": row["snapshot"]}
            for row in view["items"]
        ],
    }


def counts(report):
    return {row["profile_id"]: row for row in report["search_profiles"]}


def queries(executable, index, cache, root, profile, phase, cases=QUERIES):
    return {
        query: search(executable, index, cache, profile, query, root / f"{profile}-{ordinal}-{phase}.json")
        for ordinal, query in enumerate(cases)
    }


def compare_queries(expected, actual, profile, phase):
    if expected.keys() != actual.keys():
        raise RuntimeError(f"Query cases changed for {profile}")
    for query, rows in expected.items():
        compare(rows, actual[query], f"{profile} {phase} ({query})")


def capture(baseline, cache, root, profiles, provenance):
    source = root / "source"
    shutil.copytree(ROOT / "crates/atlas-ingest/tests/fixtures/foundry-source/core-record-types", source)
    for ordinal, path in enumerate(sorted(source.glob("packs/*/*.json")), start=1):
        record = json.loads(path.read_text())
        record["_id"] = f"{ordinal:016d}"
        write_json(path, record)
    locale = source / "static/lang/en.json"
    locale.parent.mkdir(parents=True)
    locale.write_text("{}\n", encoding="utf-8")

    index = root / "predecessor.sqlite"
    inferred = counts(build(baseline, source, index, cache, root / "baseline-build.json", profiles))
    if any(inferred[profile]["inferred_inputs"] <= 0 for profile in profiles):
        raise RuntimeError("Predecessor lacks real inference evidence")
    invoke(baseline, [
        "lists", "create", "compatibility", "--name", "Compatibility sentinel",
        "--record", "actions:0000000000000001", "--index", index,
    ], root / "list-create.json")
    saved = invoke(baseline, ["lists", "show", "compatibility", "--index", index], root / "list-before.json")
    expected = {}
    for profile in profiles:
        expected[profile] = queries(baseline, index, cache, root, profile, "baseline")
        compare_queries(expected[profile], queries(baseline, index, cache, root, profile, "repeat"),
                        profile, "baseline repeat")
    contracts = profile_contracts(index)
    with closing(sqlite3.connect(index)) as db:
        if db.execute("PRAGMA wal_checkpoint(TRUNCATE)").fetchone() != (0, 0, 0):
            raise RuntimeError("Could not checkpoint the baseline database")
    manifest = {
        "format": 1,
        "producer": {**provenance, "binary_sha256": sha256(baseline)},
        "profiles": contracts,
        "inferred_inputs": {profile: inferred[profile]["inferred_inputs"] for profile in profiles},
        "expected_queries": expected,
        "saved_list": saved,
    }
    write_json(root / "manifest.json", manifest)
    return manifest


def extract_bundle(archive, digest, root):
    actual = sha256(archive)
    if actual != digest:
        raise RuntimeError(f"Baseline archive SHA-256 mismatch: expected {digest}, got {actual}")
    with tarfile.open(archive, "r:gz") as bundle:
        bundle.extractall(root, filter="data")
    manifest = json.loads((root / "manifest.json").read_text(encoding="utf-8"))
    if manifest["format"] != 1 or profile_contracts(root / "predecessor.sqlite") != manifest["profiles"]:
        raise RuntimeError("Baseline database does not match its manifest")
    return manifest


def archive_bundle(root, output):
    def normalize(member):
        member.uid = member.gid = member.mtime = 0
        member.uname = member.gname = ""
        return member

    with output.open("xb") as raw, gzip.GzipFile(fileobj=raw, mode="wb", mtime=0) as compressed:
        with tarfile.open(fileobj=compressed, mode="w") as archive:
            for name in ("manifest.json", "predecessor.sqlite", "source"):
                archive.add(root / name, arcname=name, filter=normalize)


def check(candidate, cache, root, profiles, manifest, transitions, migrator=None):
    missing = set(profiles) - manifest["profiles"].keys()
    if missing:
        raise RuntimeError(f"Profiles missing from baseline: {', '.join(sorted(missing))}")
    source = root / "source"
    old = root / "predecessor.sqlite"
    fresh = root / "fresh.sqlite"
    build(candidate, source, fresh, cache, root / "candidate-fresh.json", profiles)
    retiring_trait_seeds = has_trait_seed_chunks(old) and not has_trait_seed_chunks(fresh)
    frozen = published_vectors(old, retiring_trait_seeds=retiring_trait_seeds)
    schema = migrate_predecessor(old, fresh, migrator, manifest, root)
    current = profile_contracts(fresh)
    layout_fields = {"segmentation_tokenizer_sha256", "unit_policy_version", "input_recipe"}
    changed_layout = set()
    migrating = set()
    rejections = {}
    for profile in profiles:
        original = manifest["profiles"][profile]
        replacement = current[profile]
        changed = {key for key in original.keys() | replacement.keys()
                   if original.get(key) != replacement.get(key)}
        if changed - layout_fields:
            if (profile not in transitions or replacement["execution_policy"] != transitions[profile]
                    or "execution_policy" not in changed):
                raise RuntimeError(f"{profile} execution contract changed without a reviewed ID transition: {sorted(changed)}")
            migrating.add(profile)
            try:
                queries(candidate, old, cache, root, profile, "rejected")
            except AtlasCommandError as error:
                failure = json.loads(error.evidence.read_text())["error"]
                if not expected_transition_rejection(failure, old, fresh):
                    raise
                rejections[profile] = failure
            else:
                raise RuntimeError(f"{profile} unexpectedly accepted the old execution contract")
        else:
            if changed:
                expected = manifest["expected_queries"][profile]
                compare_queries(expected,
                                queries(candidate, old, cache, root, profile, "existing", expected),
                                profile, "new query against old vectors before passage replacement")
                changed_layout.add(profile)

    if migrating != transitions.keys():
        raise RuntimeError("Declared embedding ID transition did not change the execution contract")
    updated = counts(build(candidate, source, old, cache, root / "candidate-reuse.json", profiles))
    preserved = published_vectors(old)
    invoke(candidate, ["index", "validate", "--no-embeddings", "--index", old],
           root / "candidate-validated.json")
    saved = invoke(candidate, ["lists", "show", "compatibility", "--index", old], root / "list-after.json")
    if saved_list_state(saved) != saved_list_state(manifest["saved_list"]):
        raise RuntimeError("Saved list changed across maintenance")
    for profile in profiles:
        row = updated[profile]
        if profile in migrating:
            if row["inferred_inputs"] <= 0 or row["reused_inputs"] != 0:
                raise RuntimeError(f"{profile} did not rebuild with the new execution contract")
        elif profile not in changed_layout:
            require_preserved_vectors(profile, frozen, preserved, row)
            expected = manifest["expected_queries"][profile]
            compare_queries(expected,
                            queries(candidate, old, cache, root, profile, "existing", expected),
                            profile, "new query against frozen vectors after maintenance")
        compare_queries(queries(candidate, fresh, cache, root, profile, "fresh"),
                        queries(candidate, old, cache, root, profile, "reused"),
                        profile, "maintenance versus fresh")

    path = source / "packs/actions/demoralize.json"
    record = json.loads(path.read_text())
    record["system"]["description"]["value"] = "<p>Threaten an opponent to make it frightened.</p>"
    write_json(path, record)
    mixed = counts(build(candidate, source, old, cache, root / "candidate-mixed.json", profiles))
    build(candidate, source, root / "edited.sqlite", cache, root / "candidate-edited-fresh.json", profiles)
    for profile in profiles:
        row = mixed[profile]
        if row["inferred_inputs"] <= 0 or row["reused_inputs"] <= 0:
            raise RuntimeError(f"{profile} incremental source change did not select both reuse and inference")
        compare_queries(queries(candidate, root / "edited.sqlite", cache, root, profile, "edited-fresh"),
                        queries(candidate, old, cache, root, profile, "mixed"),
                        profile, "mixed versus freshly inferred")
    result = {
        "baseline_sha256": manifest["producer"]["binary_sha256"],
        "candidate_sha256": sha256(candidate),
        "workspace_schema": schema,
        "pre_maintenance_rejections": rejections,
        "score_tolerance": SCORE_TOLERANCE,
        "profiles": {
            profile: {
                "status": "migration" if profile in migrating else "layout" if profile in changed_layout else "compatible",
                "maintenance": updated[profile],
                "edited": mixed[profile],
            } for profile in profiles
        },
    }
    write_json(root / "result.json", result)
    print(f"Embedding upgrade probe passed. Evidence is in {root}.")


def read_pins(base):
    current = json.loads((ROOT / PIN_PATH).read_text(encoding="utf-8"))
    previous = {}
    if base:
        def git(*args):
            return subprocess.check_output(["git", *args], cwd=ROOT, text=True)

        if git("ls-tree", "--name-only", base, PIN_PATH).strip():
            previous = json.loads(git("show", f"{base}:{PIN_PATH}"))
    if set(current) != set(PROFILES) or (previous and set(previous) != set(PROFILES)):
        raise RuntimeError("Baseline pins must name both supported profiles")
    return current, previous


def pinned_checks(current, previous):
    checks = {}

    def add(profile, pin, transition):
        tag, asset, digest = (pin[key] for key in ("tag", "asset", "sha256"))
        if (not re.fullmatch(r"embedding-baseline-v[1-9][0-9]*", tag)
                or asset != "embedding-compatibility-baseline.tar.gz"
                or not re.fullmatch(r"[0-9a-f]{64}", digest)
                or transition == pin["execution_policy"]):
            raise RuntimeError(f"Invalid baseline pin for {profile}")
        checks.setdefault((tag, asset, digest), {})[profile] = (pin["execution_policy"], transition)

    for profile, pin in current.items():
        old = previous.get(profile, pin)
        same_fixture = all(old[key] == pin[key] for key in ("sha256", "execution_policy"))
        if not same_fixture:
            if old.get("transition_to") != pin["execution_policy"] or pin.get("transition_to"):
                raise RuntimeError(f"{profile} baseline pin can advance only after an approved ID transition")
            add(profile, old, pin["execution_policy"])
        add(profile, pin, pin.get("transition_to"))
    return checks


def check_pins(candidate, cache, root, base, repository, migrator=None):
    if not re.fullmatch(r"[\w.-]+/[\w.-]+", repository):
        raise RuntimeError("Expected a GitHub repository in owner/name format")
    download_env = os.environ.copy()
    # Keep the CI credential out of candidate and migrator subprocesses.
    os.environ.pop("GH_TOKEN", None)
    for ordinal, ((tag, asset, digest), profiles) in enumerate(pinned_checks(*read_pins(base)).items()):
        archive = root / f"baseline-{ordinal}.tar.gz"
        subprocess.run(
            ["gh", "release", "download", tag, "--repo", repository,
             "--pattern", asset, "--output", str(archive)],
            env=download_env, check=True, timeout=120,
        )
        evidence = root / f"bundle-{ordinal}"
        evidence.mkdir()
        manifest = extract_bundle(archive, digest, evidence)
        transitions = {}
        for profile, (policy, transition) in profiles.items():
            if manifest["profiles"][profile]["execution_policy"] != policy:
                raise RuntimeError(f"{profile} baseline execution ID does not match the Git pin")
            if transition:
                transitions[profile] = transition
        write_json(evidence / "pin.json", {"tag": tag, "asset": asset, "sha256": digest})
        check(candidate, cache, evidence, tuple(profiles), manifest, transitions, migrator)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--baseline", type=Path)
    source.add_argument("--bundle", type=Path)
    source.add_argument("--pins", action="store_true", help="Download the pinned release fixtures with authenticated gh")
    parser.add_argument("--base-ref")
    parser.add_argument("--repository")
    parser.add_argument("--sha256")
    parser.add_argument("--candidate", type=Path)
    parser.add_argument("--workspace-migrator", type=Path, help="Candidate helper for schema-only workspace upgrades")
    parser.add_argument("--produce", type=Path, help="Write a new baseline archive")
    parser.add_argument("--producer-commit")
    parser.add_argument("--producer-target")
    parser.add_argument("--producer-build")
    parser.add_argument("--profile", action="append", choices=PROFILES)
    parser.add_argument("--model-cache", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.produce:
        if not (args.baseline and args.producer_commit and args.producer_target and args.producer_build):
            parser.error("production requires a baseline binary, commit, target and packaging run or tag")
        if args.candidate:
            parser.error("choose production or comparison")
    elif not args.candidate or (args.bundle and not args.sha256):
        parser.error("comparison requires a candidate and a SHA-256 for a downloaded bundle")
    if args.pins and not args.repository:
        parser.error("--pins requires --repository")
    root = args.output.resolve()
    root.mkdir(parents=True)
    profiles = tuple(args.profile or PROFILES)
    migrator = args.workspace_migrator.resolve() if args.workspace_migrator else None
    cache = args.model_cache.resolve()
    if args.pins:
        check_pins(args.candidate.resolve(), cache, root, args.base_ref, args.repository, migrator)
        return
    if args.bundle:
        manifest = extract_bundle(args.bundle, args.sha256, root)
        profiles = tuple(args.profile or manifest["profiles"])
    else:
        provenance = {"commit": args.producer_commit, "target": args.producer_target, "build": args.producer_build}
        manifest = capture(args.baseline.resolve(), cache, root, profiles, provenance)
    if args.produce:
        archive_bundle(root, args.produce)
        print(f"{sha256(args.produce)}  {args.produce}")
    else:
        check(args.candidate.resolve(), cache, root, profiles, manifest, {}, migrator)


if __name__ == "__main__":
    main()
