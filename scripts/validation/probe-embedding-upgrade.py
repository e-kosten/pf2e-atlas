#!/usr/bin/env python3
"""Exercise a candidate's compatibility with a genuine predecessor index."""
import argparse
import gzip
import hashlib
import io
import json
import math
import os
import platform
from pathlib import Path
import re
import shutil
import sqlite3
import subprocess
import tarfile
import time
import tempfile
import zipfile
from contextlib import closing

ROOT = Path(__file__).resolve().parents[2]
PROFILES = ("bge-base", "embeddinggemma")
RELEASE_TARGETS = (
    "x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu",
    "aarch64-apple-darwin", "x86_64-pc-windows-msvc",
)
QUERIES = ("frighten an enemy using a threatening display", "restore health to a wounded ally")
SCORE_TOLERANCE = 2e-5
PIN_PATH = "scripts/validation/embedding-baseline.json"
V2_PIN_PATH = "scripts/validation/embedding-fixtures-v2.json"

SOURCE_FIXTURE = "crates/atlas-ingest/tests/fixtures/foundry-source/core-record-types"
RELEASE_PIN = "scripts/validation/schema-42-predecessor.json"
RELEASE_TAG = "v0.1.0-nightly.20261112173917.g449cf4c9315c"
RELEASE_ASSET = "cli-only-release"
RELEASE_DIGEST = "113c7ea605bbc77962f53a1f250c14d9acec351cc714e15babe3c9d22798bcd6"
SCHEMA_29_PIN = {
    "tag": "embedding-baseline-v1",
    "asset": "embedding-compatibility-baseline.tar.gz",
    "sha256": "7a5cedd86c4ae2826a9fa0d73a75ca570749d5ab0349dda70b2bac6912e4a374",
    "execution_policy": "fastembed5.15.0/tokenizers0.22.2/f32/native-specials/no-truncation/right-padding/threads4",
}
SCHEMA_41_PIN = {
    **SCHEMA_29_PIN,
    "tag": "v0.1.0-nightly.20261109050726.g787c00b3b26e",
    "asset": RELEASE_ASSET,
    "sha256": "6cc6cae16566251cf6e3937d786a2672071aff40ee34f6a7ead4fd6bacddc7f5",
}
SCHEMA_42_PIN = {
    **SCHEMA_29_PIN, "tag": RELEASE_TAG, "asset": RELEASE_ASSET, "sha256": RELEASE_DIGEST,
}


class AtlasCommandError(RuntimeError):
    def __init__(self, executable, args, evidence, stderr):
        self.evidence = evidence
        super().__init__(f"{executable.name} {' '.join(args[:2])} failed. See {evidence}: {stderr}")


def invoke(executable, args, evidence):
    started = time.monotonic()
    try:
        result = subprocess.run(
            [str(executable), *map(str, args), "--path-mode", "repo", "--json"],
            capture_output=True, text=True, timeout=240,
        )
    finally:
        write_json(evidence.with_suffix(".timing.json"), {
            "command": [executable.name, *map(str, args[:2])],
            "seconds": round(time.monotonic() - started, 3),
        })
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
    before = workspace_schema(old)
    expected = fresh if isinstance(fresh, int) else workspace_schema(fresh)
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


def materialize_source(root, fixture=None):
    source = root / "source"
    shutil.copytree(fixture if fixture is not None else ROOT / SOURCE_FIXTURE, source)
    for ordinal, path in enumerate(sorted(source.glob("packs/*/*.json")), start=1):
        record = json.loads(path.read_text(encoding="utf-8"))
        record["_id"] = f"{ordinal:016d}"
        write_json(path, record)
    locale = source / "static/lang/en.json"
    locale.parent.mkdir(parents=True, exist_ok=True)
    locale.write_text("{}\n", encoding="utf-8")
    return source


def capture(baseline, cache, root, profiles, provenance, fixture=None, *, compact=False):
    source = materialize_source(root, fixture)

    index = root / "predecessor.sqlite"
    inferred = counts(build(baseline, source, index, cache, root / "baseline-build.json", profiles))
    if any(inferred[profile]["inferred_inputs"] <= 0 for profile in profiles):
        raise RuntimeError("Predecessor lacks real inference evidence")
    invoke(baseline, [
        "lists", "create", "Compatibility sentinel", "--slug", "compatibility",
        "--record", "actions:0000000000000001", "--index", index,
    ], root / "list-create.json")
    saved = invoke(baseline, ["lists", "show", "compatibility", "--index", index], root / "list-before.json")
    expected = {}
    for profile in profiles:
        cases = QUERIES[:1] if compact else QUERIES
        expected[profile] = queries(baseline, index, cache, root, profile, "baseline", cases)
        if not compact:
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


def check_document_samples(reader, old, fresh, profile, before, after, *, layout_changed=False):
    # Layout can legitimately remove inputs. Compare the surviving prepared inputs
    # under both builds, with the native extension reading actual stored values.
    old_counts = {digest: tokens for _, digest, tokens in before.get(profile, {})}
    new_counts = {digest: tokens for _, digest, tokens in after.get(profile, {})}
    with closing(sqlite3.connect(old.as_uri() + "?mode=ro", uri=True)) as db:
        rows = db.execute(
            "SELECT DISTINCT unit_kind, lower(hex(input_hash)) FROM semantic_units "
            "JOIN embedding_inputs USING(input_id) WHERE profile_id = ? ORDER BY 1, 2",
            (profile,),
        ).fetchall()
    hashes = []
    for kind in ("identity", "passage"):
        surviving = [digest for unit, digest in rows if unit == kind and digest in new_counts]
        if not surviving and (kind == "identity" or not layout_changed):
            raise RuntimeError(f"{profile} has no matching {kind} document samples")
        hashes.extend(surviving[:4])
    hashes = list(dict.fromkeys(hashes))
    if any(old_counts[digest] != new_counts[digest] for digest in hashes):
        raise RuntimeError(f"{profile} native token count changed")
    samples = [(digest, old_counts[digest]) for digest in hashes]

    def read(index):
        return json.loads(subprocess.check_output(
            [str(reader), str(index), profile, *hashes], text=True, timeout=60,
        ))

    for (digest, tokens), previous, current in zip(samples, read(old), read(fresh), strict=True):
        if any(row["hash"] != digest or row["tokens"] != tokens for row in (previous, current)):
            raise RuntimeError(f"{profile} sample identity changed: {digest}")
        if not previous["values"] or any(
            not math.isclose(a, b, rel_tol=0, abs_tol=SCORE_TOLERANCE)
            for a, b in zip(previous["values"], current["values"], strict=True)
        ):
            raise RuntimeError(f"{profile} document vector drift: {digest}")
    return len(samples)


def check(candidate, cache, root, profiles, manifest, transitions, migrator=None, reader=None):
    missing = set(profiles) - manifest["profiles"].keys()
    if missing:
        raise RuntimeError(f"Profiles missing from baseline: {', '.join(sorted(missing))}")
    source = root / "source"
    old = root / "predecessor.sqlite"
    fresh = root / "fresh.sqlite"
    build(candidate, source, fresh, cache, root / "candidate-fresh.json", profiles)
    retiring_trait_seeds = has_trait_seed_chunks(old) and not has_trait_seed_chunks(fresh)
    frozen = published_vectors(old, retiring_trait_seeds=retiring_trait_seeds)
    current = profile_contracts(fresh)
    layout_fields = {"segmentation_tokenizer_sha256", "unit_policy_version", "input_recipe"}
    fresh_vectors = published_vectors(fresh) if reader else {}
    document_samples = {}
    for profile in profiles:
        if reader and profile not in transitions:
            document_samples[profile] = check_document_samples(
                reader, old, fresh, profile, frozen, fresh_vectors,
                layout_changed=any(manifest["profiles"][profile].get(key) != current[profile].get(key)
                                   for key in layout_fields),
            )
    schema = migrate_predecessor(old, fresh, migrator, manifest, root)
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
        if not reader or profile in migrating or profile in changed_layout:
            cases = tuple(manifest["expected_queries"][profile])
            compare_queries(queries(candidate, fresh, cache, root, profile, "fresh", cases),
                            queries(candidate, old, cache, root, profile, "reused", cases),
                            profile, "maintenance versus fresh")

    mixed = {}
    if not reader:
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
        "document_samples": document_samples,
        "profiles": {
            profile: {
                "status": "migration" if profile in migrating else "layout" if profile in changed_layout else "compatible",
                "maintenance": updated[profile],
                "edited": mixed.get(profile),
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
    # The workspace cutoff replaces only the exact old pair of pins. All other
    # fixture or execution-policy changes continue through the ordinary guard.
    cutoff = previous == dict.fromkeys(PROFILES, SCHEMA_41_PIN) and current == dict.fromkeys(PROFILES, SCHEMA_42_PIN)

    def add(profile, pin, transition):
        tag, asset, digest = (pin[key] for key in ("tag", "asset", "sha256"))
        packaged = (tag, asset, digest) == (RELEASE_TAG, RELEASE_ASSET, RELEASE_DIGEST)
        bundle = (re.fullmatch(r"(?:fixtures/embedding-compatibility/|embedding-baseline-)v[1-9][0-9]*", tag)
                  and asset == "embedding-compatibility-baseline.tar.gz"
                  and re.fullmatch(r"[0-9a-f]{64}", digest))
        if ((not packaged and not bundle)
                or (packaged and pin["execution_policy"] != SCHEMA_42_PIN["execution_policy"])
                or transition == pin["execution_policy"]):
            raise RuntimeError(f"Invalid baseline pin for {profile}")
        checks.setdefault((tag, asset, digest), {})[profile] = (pin["execution_policy"], transition)

    for profile, pin in current.items():
        old = previous.get(profile, pin)
        same_fixture = all(old[key] == pin[key] for key in ("sha256", "execution_policy"))
        if not same_fixture and not cutoff:
            if old.get("transition_to") != pin["execution_policy"] or pin.get("transition_to"):
                raise RuntimeError(f"{profile} baseline pin can advance only after an approved ID transition")
            add(profile, old, pin["execution_policy"])
        add(profile, pin, pin.get("transition_to"))
    return checks


def release_target():
    arch = {"arm64": "aarch64", "amd64": "x86_64"}.get(platform.machine().lower(), platform.machine().lower())
    suffix = {"Darwin": "apple-darwin", "Linux": "unknown-linux-gnu", "Windows": "pc-windows-msvc"}
    return f"{arch}-{suffix[platform.system()]}"


def predecessor_ledger(index):
    digest = hashlib.sha256(b"atlas-predecessor-ledger/v1\0")
    with closing(sqlite3.connect(index.as_uri() + "?mode=ro", uri=True)) as db:
        rows = db.execute("SELECT version,name,checksum FROM schema_migrations ORDER BY version")
        for version, name, checksum in rows:
            digest.update(version.to_bytes(4, "big"))
            for field in (name, checksum):
                encoded = field.encode("utf-8")
                digest.update(len(encoded).to_bytes(8, "big"))
                digest.update(encoded)
    return digest.hexdigest()


def extract_release_executable(archive, destination):
    filename = destination.name
    if archive.suffix == ".zip":
        with zipfile.ZipFile(archive) as package:
            members = [item for item in package.infolist()
                       if not item.is_dir() and Path(item.filename).name == filename]
            if len(members) != 1:
                raise RuntimeError("Expected one Atlas executable in the pinned archive")
            with package.open(members[0]) as source, destination.open("wb") as output:
                shutil.copyfileobj(source, output)
    else:
        with tarfile.open(archive, "r:xz") as package:
            members = [item for item in package.getmembers()
                       if item.isfile() and Path(item.name).name == filename]
            if len(members) != 1:
                raise RuntimeError("Expected one Atlas executable in the pinned archive")
            with package.extractfile(members[0]) as source, destination.open("wb") as output:
                shutil.copyfileobj(source, output)
    destination.chmod(0o700)


def producer_receipt(path=None):
    receipt = (ROOT / (path or RELEASE_PIN)).resolve()
    relative = receipt.relative_to(ROOT)
    if path is not None:
        subprocess.run(["git", "ls-files", "--error-unmatch", "--", relative.as_posix()],
                       cwd=ROOT, check=True, stdout=subprocess.DEVNULL, timeout=30)
    data = receipt.read_bytes().replace(b"\r\n", b"\n")
    digest = hashlib.sha256(data).hexdigest()
    release = json.loads(data)
    if path is None and (digest != RELEASE_DIGEST or release["tag"] != RELEASE_TAG or release["schema"] != 42):
        raise RuntimeError("Schema 42 producer receipt differs from the cutover pin")
    if (not re.fullmatch(r"v[0-9][0-9A-Za-z.-]*", release["tag"])
            or not re.fullmatch(r"[0-9a-f]{40}", release["commit"])
            or not re.fullmatch(r"[0-9a-f]{40}", release["source_tree"])
            or not re.fullmatch(r"[0-9a-f]{64}", release["ledger_sha256"])
            or type(release["schema"]) is not int or release["schema"] <= 0
            or tuple(release["queries"]) != QUERIES
            or set(release["assets"]) != set(RELEASE_TARGETS)):
        raise RuntimeError("Invalid packaged producer receipt")
    for target, pin in release["assets"].items():
        extension = "zip" if target.endswith("windows-msvc") else "tar.xz"
        if (pin["asset"] != f"atlas-cli-only-{target}.{extension}"
                or not re.fullmatch(r"[0-9a-f]{64}", pin["sha256"])):
            raise RuntimeError(f"Invalid packaged producer archive for {target}")
    commit = subprocess.check_output(
        ["git", "rev-parse", "--verify", f"refs/tags/{release['tag']}^{{commit}}"],
        cwd=ROOT, text=True, timeout=30,
    ).strip()
    if commit != release["commit"]:
        raise RuntimeError("Producer release tag does not match the receipt commit")
    return release, digest


def capture_release_predecessor(repository, download_env, cache, evidence, profiles, *,
                                compact=False, receipt=None):
    release, receipt_digest = producer_receipt(receipt)
    target = release_target()
    pin = release["assets"][target]
    archive = evidence / pin["asset"]
    subprocess.run(
        ["gh", "release", "download", release["tag"], "--repo", repository,
         "--pattern", pin["asset"], "--output", str(archive)],
        env=download_env, check=True, timeout=120,
    )
    if sha256(archive) != pin["sha256"]:
        raise RuntimeError("Producer archive SHA-256 mismatch")
    baseline = evidence / ("atlas.exe" if archive.suffix == ".zip" else "atlas")
    extract_release_executable(archive, baseline)
    version = subprocess.check_output([str(baseline), "--version"], text=True, timeout=20).strip()
    if version != f"atlas {release['tag'][1:]}":
        raise RuntimeError("Unexpected version in the producer archive")

    tree = subprocess.check_output(
        ["git", "rev-parse", f"{release['commit']}:{SOURCE_FIXTURE}"], cwd=ROOT, text=True,
    ).strip()
    if tree != release["source_tree"]:
        raise RuntimeError("Producer fixture source tree does not match the receipt")
    source = subprocess.check_output(
        ["git", "archive", "--format=tar", release["commit"], SOURCE_FIXTURE], cwd=ROOT,
    )
    fixture_dir = evidence / "frozen-input"
    with tarfile.open(fileobj=io.BytesIO(source), mode="r:") as package:
        package.extractall(fixture_dir, filter="data")
    manifest = capture(baseline, cache, evidence, tuple(profiles), {
        "commit": release["commit"], "target": target, "build": release["tag"],
    }, fixture_dir / SOURCE_FIXTURE, compact=compact)
    index = evidence / "predecessor.sqlite"
    if workspace_schema(index) != release["schema"] or predecessor_ledger(index) != release["ledger_sha256"]:
        raise RuntimeError("Producer workspace schema or ledger does not match the receipt")
    manifest["producer"].update(
        release_receipt_sha256=receipt_digest, release_archive_sha256=pin["sha256"],
        ledger_sha256=release["ledger_sha256"], source_tree=release["source_tree"],
        workspace_schema=release["schema"],
    )
    return manifest


def capture_numerical_payload(root, cache, manifest, reader):
    producer = manifest["producer"]
    helper = ROOT / "scripts/validation/fixture-input-capture.rs"
    inputs_path = root / "prepared-inputs.json"
    with tempfile.TemporaryDirectory(prefix="atlas-fixture-producer-") as directory:
        checkout = Path(directory)
        archive = checkout / "producer.tar"
        subprocess.run(["git", "archive", "-o", str(archive), producer["commit"]],
                       cwd=ROOT, check=True, timeout=120)
        with tarfile.open(archive) as bundle:
            bundle.extractall(checkout, filter="data")
        tests = checkout / "crates/atlas-ingest/src/product_preparation/tests.rs"
        with tests.open("ab") as stream:
            stream.write(b"\n" + helper.read_bytes())
        env = dict(os.environ, ATLAS_FIXTURE_SOURCE=str(root / "source"),
                   ATLAS_FIXTURE_MODELS=str(cache), ATLAS_FIXTURE_INPUTS=str(inputs_path))
        subprocess.run(["cargo", "test", "--locked", "-p", "atlas-ingest", "--lib",
                        "capture_compatibility_inputs"], cwd=checkout, env=env,
                       check=True, timeout=1200)
    prepared = json.loads(inputs_path.read_text(encoding="utf-8"))
    profiles = {}
    for profile, contract in manifest["profiles"].items():
        inputs = prepared[profile]
        if {row["kind"] for row in inputs} != {"identity", "passage"}:
            raise RuntimeError(f"{profile} capture requires both identity and passage inputs")
        rows = json.loads(subprocess.check_output(
            [str(reader), str(root / "predecessor.sqlite"), profile,
             *(row["hash"] for row in inputs)], text=True, timeout=60,
        ))
        for item, stored in zip(inputs, rows, strict=True):
            if (item["hash"], item["tokens"]) != (stored["hash"], stored["tokens"]):
                raise RuntimeError(f"{profile} prepared input does not match the published row")
            if hashlib.sha256(item["text"].encode()).hexdigest() != item["hash"]:
                raise RuntimeError(f"{profile} prepared input checksum mismatch")
            item["values"] = stored["values"]
        profiles[profile] = {"contract": contract, "samples": inputs}
    payload = root / f"numerical-{producer['target']}.json.gz"
    data = {"format": 2, "producer": producer, "helper_sha256": sha256(helper),
            "profiles": profiles}
    payload.write_bytes(gzip.compress(json.dumps(data, sort_keys=True).encode(), mtime=0))
    return payload


def produce_v2(repository, cache, root, reader, *, receipt=None, workspace_only=False):
    download_env = os.environ.copy()
    os.environ.pop("GH_TOKEN", None)
    manifest = capture_release_predecessor(repository, download_env, cache, root, PROFILES,
                                           compact=workspace_only, receipt=receipt)
    write_json(root / "manifest.json", manifest)
    workspace = root / f"workspace-{release_target()}.tar.gz"
    archive_bundle(root, workspace)
    assets = [workspace]
    if not workspace_only:
        assets.append(capture_numerical_payload(root, cache, manifest, reader))
    write_json(root / "fixture-digests.json", {
        "target": release_target(),
        "assets": {path.name: sha256(path) for path in assets},
    })


def load_v2_numerical(path, digest, target, policies):
    if sha256(path) != digest:
        raise RuntimeError("Numerical payload checksum mismatch")
    with gzip.open(path, "rt", encoding="utf-8") as stream:
        payload = json.load(stream)
    if payload["format"] != 2 or payload["producer"]["target"] != target:
        raise RuntimeError("Numerical payload format or target mismatch")
    for profile, policy in policies.items():
        item = payload["profiles"][profile]
        if (item["contract"]["profile_id"] != profile
                or item["contract"]["execution_policy"] != policy):
            raise RuntimeError(f"{profile} numerical policy mismatch")
        if {row["kind"] for row in item["samples"]} != {"identity", "passage"}:
            raise RuntimeError(f"{profile} incomplete numerical samples")
    return payload


def compare_predecessor_hit(expected, actual, profile):
    # Protect a known record and its best matching passage. Unrelated near ties
    # in the fixture are not part of the numerical contract.
    if not expected or not expected[0][1]:
        raise RuntimeError(f"{profile} missing predecessor search witness")
    record = expected[0][0]
    passage, score = expected[0][1][0]
    if actual and actual[0][0] == record and actual[0][1]:
        location, value = actual[0][1][0]
        if location == passage and math.isclose(value, score, abs_tol=SCORE_TOLERANCE, rel_tol=0):
            return
    raise RuntimeError(f"{profile} predecessor search witness changed: {record}")


def check_compatible_workspace(candidate, cache, root, profiles, manifest, migrator, schema):
    old = root / "predecessor.sqlite"
    before = published_vectors(old)
    migration = migrate_predecessor(old, schema, migrator, manifest, root)
    updated = counts(build(candidate, root / "source", old, cache, root / "candidate-reuse.json", profiles))
    after = published_vectors(old)
    invoke(candidate, ["index", "validate", "--no-embeddings", "--index", old],
           root / "candidate-validated.json")
    saved = invoke(candidate, ["lists", "show", "compatibility", "--index", old], root / "list-after.json")
    if saved_list_state(saved) != saved_list_state(manifest["saved_list"]):
        raise RuntimeError("Saved list changed across maintenance")
    for profile in profiles:
        require_preserved_vectors(profile, before, after, updated[profile])
        query, expected = next(iter(manifest["expected_queries"][profile].items()))
        actual = search(candidate, old, cache, profile, query, root / f"{profile}-existing.json")
        compare_predecessor_hit(expected, actual, profile)
    write_json(root / "result.json", {
        "candidate_sha256": sha256(candidate),
        "baseline_sha256": manifest["producer"]["binary_sha256"],
        "workspace_schema": migration,
        "profiles": updated,
        "check": "frozen-documents-and-predecessor-search",
    })
    print(f"Embedding compatibility passed. Evidence is in {root}.")


def check_v2_payloads(candidate, cache, root, migrator, reader, inference, workspace,
                      numerical, policies, transitions, producers=None, *, check_build=False, near_limit=False):
    target = release_target()
    payload = load_v2_numerical(*numerical, target, policies)
    if producers and payload["producer"]["commit"] != producers["numerical"]:
        raise RuntimeError("Numerical payload producer mismatch")
    for profile in policies.keys() - transitions.keys():
        samples = root / f"{profile}-numerical-inputs.json"
        write_json(samples, payload["profiles"][profile]["samples"])
        command = [str(inference), str(cache), profile, str(samples)]
        if near_limit:
            command.append("--near-limit")
        subprocess.run(command, check=True, timeout=360)
    manifest = extract_bundle(*workspace, root)
    if manifest["producer"]["target"] != target:
        raise RuntimeError("Workspace payload target mismatch")
    if (workspace_schema(root / "predecessor.sqlite") != manifest["producer"]["workspace_schema"]
            or predecessor_ledger(root / "predecessor.sqlite") != manifest["producer"]["ledger_sha256"]):
        raise RuntimeError("Workspace payload schema or ledger mismatch")
    if producers and manifest["producer"]["commit"] != producers["workspace"]:
        raise RuntimeError("Workspace payload producer mismatch")
    if (producers and "release_receipt_sha256" in producers
            and manifest["producer"].get("release_receipt_sha256") != producers["release_receipt_sha256"]):
        raise RuntimeError("Workspace payload producer receipt mismatch")
    for profile, policy in policies.items():
        contract = manifest["profiles"][profile]
        frozen_contract = payload["profiles"][profile]["contract"]
        layout = {"segmentation_tokenizer_sha256", "unit_policy_version", "input_recipe"}
        if (contract["execution_policy"] != policy
                or {k: v for k, v in contract.items() if k not in layout}
                != {k: v for k, v in frozen_contract.items() if k not in layout}):
            raise RuntimeError(f"{profile} workspace numerical contract mismatch")
        first = QUERIES[0]
        manifest["expected_queries"][profile] = {first: manifest["expected_queries"][profile][first]}
    description = json.loads(subprocess.check_output([str(migrator), "--describe"], text=True, timeout=30))
    contracts = {contract["profile_id"]: contract for contract in description["profiles"]}
    if (check_build or transitions
            or any(contracts.get(profile) != manifest["profiles"][profile] for profile in policies)):
        check(candidate, cache, root, tuple(policies), manifest, transitions, migrator, reader)
    else:
        check_compatible_workspace(candidate, cache, root, policies, manifest, migrator, description["schema"])


def v2_receipts(base):
    path = ROOT / V2_PIN_PATH
    previous = {}
    if base and subprocess.check_output(
        ["git", "ls-tree", "--name-only", base, V2_PIN_PATH], cwd=ROOT, text=True,
    ).strip():
        previous = json.loads(subprocess.check_output(
            ["git", "show", f"{base}:{V2_PIN_PATH}"], cwd=ROOT, text=True,
        ))
    current = json.loads(path.read_text(encoding="utf-8")) if path.exists() else {}
    if (path.exists() or previous) and not current:
        raise RuntimeError("Published v2 fixture receipts cannot be removed")
    if not current:
        return {}, {}
    targets = set(RELEASE_TARGETS)
    pins, old_pins = read_pins(base)
    validate_v2_receipts(current, previous, targets, pins, old_pins,
                         compare_history=bool(base))
    return current[release_target()], previous.get(release_target(), {})


def validate_v2_receipts(current, previous, targets, pins, old_pins, *, compare_history=True):
    if current.keys() != targets:
        raise RuntimeError("Published v2 fixtures must cover all release targets")
    for target, profiles in current.items():
        if profiles.keys() != pins.keys():
            raise RuntimeError(f"{target} v2 receipts must cover both profiles")
        for profile, receipt in profiles.items():
            for kind, extension in (("workspace", "tar.gz"), ("numerical", "json.gz")):
                payload = receipt[kind]
                if (not re.fullmatch(r"fixtures/embedding-compatibility/v[1-9][0-9]*", payload["tag"])
                        or not re.fullmatch(rf"{kind}-{re.escape(target)}(?:-{profile})?\.{re.escape(extension)}",
                                            payload["asset"])
                        or not re.fullmatch(r"[0-9a-f]{64}", payload["sha256"])
                        or not re.fullmatch(r"[0-9a-f]{40}", payload["commit"])):
                    raise RuntimeError(f"Invalid {target} {profile} {kind} receipt")
            old = previous.get(target, {}).get(profile)
            if compare_history and not old and pins[profile]["asset"] == RELEASE_ASSET:
                producer = json.loads((ROOT / RELEASE_PIN).read_text(encoding="utf-8"))["commit"]
                if any(receipt[kind]["commit"] != producer for kind in ("workspace", "numerical")):
                    raise RuntimeError(f"{target} {profile} initial v2 producer differs from pinned predecessor")
            if old and old["numerical"] != receipt["numerical"]:
                old_pin = old_pins.get(profile, {})
                policy = pins[profile]["execution_policy"]
                if (old_pin.get("execution_policy") == policy
                        or old_pin.get("transition_to") != policy):
                    raise RuntimeError(f"{target} {profile} numerical evidence rotated without transition")


def check_published_v2(candidate, cache, root, base, repository, migrator, reader, inference,
                       receipts, previous_receipts, *, check_build=False, near_limit=False,
                       replacement=None):
    if not reader or not inference:
        raise RuntimeError("Published v2 requires native vector and inference helpers")
    current, previous = read_pins(base)
    if not previous_receipts and replacement is None:
        pinned_checks(current, previous)
    old_transitions = {}
    for profile, previous_receipt in previous_receipts.items():
        old, new = previous[profile], current[profile]
        changed = old["execution_policy"] != new["execution_policy"]
        rotated = previous_receipt["numerical"] != receipts[profile]["numerical"]
        if changed != rotated:
            raise RuntimeError(f"{profile} execution policy and numerical evidence must advance together")
        if changed:
            if old.get("transition_to") != new["execution_policy"] or new.get("transition_to"):
                raise RuntimeError(f"{profile} execution policy advanced without a reviewed transition")
            old_transitions[profile] = new["execution_policy"]

    download_env = os.environ.copy()
    os.environ.pop("GH_TOKEN", None)
    downloaded = {}
    groups = {}
    replay = {}

    def add(profile, receipt, policy, transition):
        key = tuple(tuple(sorted(receipt[kind].items())) for kind in ("workspace", "numerical"))
        profiles = groups.setdefault(key, {})
        if profile in profiles:
            raise RuntimeError(f"Duplicate fixture qualification for {profile}")
        profiles[profile] = (policy, transition)

    for profile, receipt in receipts.items():
        pin = current[profile]
        if replacement is not None:
            if pin.get("transition_to") or profile in old_transitions:
                raise RuntimeError(f"{profile} requires a separate execution-policy transition")
            receipt = {**receipt, "workspace": replacement[1]}
        add(profile, receipt, pin["execution_policy"], pin.get("transition_to"))
    for profile, transition in old_transitions.items():
        add(profile, previous_receipts[profile], previous[profile]["execution_policy"], transition)
    for ordinal, (receipts_key, profiles) in enumerate(groups.items()):
        directory = root / f"bundle-{ordinal}"
        directory.mkdir()
        payloads = {}
        producers = {}
        for kind, fields in zip(("workspace", "numerical"), receipts_key, strict=True):
            pin = dict(fields)
            if replacement is not None and kind == "workspace":
                path = replacement[0]
            elif fields in downloaded:
                path = downloaded[fields]
            else:
                path = root / f"download-{len(downloaded)}-{pin['asset']}"
                subprocess.run(["gh", "release", "download", pin["tag"], "--repo", repository,
                                "--pattern", pin["asset"], "--output", str(path)],
                               env=download_env, check=True, timeout=120)
                downloaded[fields] = path
            payloads[kind] = (path, pin["sha256"])
            producers[kind] = pin["commit"]
        if replacement is not None:
            producers["release_receipt_sha256"] = replacement[1]["release_receipt_sha256"]
        policies = {profile: policy for profile, (policy, _) in profiles.items()}
        transitions = {profile: transition for profile, (_, transition) in profiles.items() if transition}
        check_v2_payloads(candidate, cache, directory, migrator, reader, inference,
                          payloads["workspace"], payloads["numerical"], policies, transitions, producers,
                          check_build=check_build, near_limit=near_limit)
        if replacement is not None:
            result = json.loads((directory / "result.json").read_bytes())
            for profile in profiles:
                replay[profile] = {"numerical": receipts[profile]["numerical"],
                                   "workspace_schema": result["workspace_schema"]}
    return replay


def check_replacement_workspace(candidate, cache, root, base, repository, migrator, reader,
                                inference, workspace, receipt):
    receipts, previous = v2_receipts(base)
    if not receipts:
        raise RuntimeError("Workspace-only replay requires published numerical pins")
    release, receipt_digest = producer_receipt(receipt)
    workspace_pin = {"commit": release["commit"], "sha256": workspace[1],
                     "release_receipt_sha256": receipt_digest}
    replay = check_published_v2(candidate, cache, root, base, repository, migrator, reader,
                                inference, receipts, previous,
                                replacement=(workspace[0], workspace_pin))
    write_json(root / "fixture-replay.json", {
        "target": release_target(),
        "candidate_commit": subprocess.check_output(["git", "rev-parse", "HEAD"],
                                                     cwd=ROOT, text=True, timeout=30).strip(),
        "candidate_sha256": sha256(candidate),
        "workspace": workspace_pin,
        "profiles": replay,
    })


def check_pins(candidate, cache, root, base, repository, migrator=None, reader=None, inference=None, *, check_build=False, near_limit=False):
    if not re.fullmatch(r"[\w.-]+/[\w.-]+", repository):
        raise RuntimeError("Expected a GitHub repository in owner/name format")
    root.mkdir(parents=True, exist_ok=True)
    receipts, previous_receipts = v2_receipts(base) if reader else ({}, {})
    if receipts:
        check_published_v2(candidate, cache, root, base, repository, migrator, reader, inference,
                           receipts, previous_receipts, check_build=check_build, near_limit=near_limit)
        return
    download_env = os.environ.copy()
    # Keep the CI credential out of candidate and migrator subprocesses.
    os.environ.pop("GH_TOKEN", None)
    for ordinal, ((tag, asset, digest), profiles) in enumerate(pinned_checks(*read_pins(base)).items()):
        evidence = root / f"bundle-{ordinal}"
        evidence.mkdir()
        if asset == RELEASE_ASSET:
            manifest = capture_release_predecessor(
                repository, download_env, cache, evidence, profiles, compact=reader is not None,
            )
        else:
            archive = root / f"baseline-{ordinal}.tar.gz"
            subprocess.run(
                ["gh", "release", "download", tag, "--repo", repository,
                 "--pattern", asset, "--output", str(archive)],
                env=download_env, check=True, timeout=120,
            )
            manifest = extract_bundle(archive, digest, evidence)
        transitions = {}
        for profile, (policy, transition) in profiles.items():
            if manifest["profiles"][profile]["execution_policy"] != policy:
                raise RuntimeError(f"{profile} baseline execution ID does not match the Git pin")
            if transition:
                transitions[profile] = transition
        write_json(evidence / "pin.json", {"tag": tag, "asset": asset, "sha256": digest})
        check(candidate, cache, evidence, tuple(profiles), manifest, transitions, migrator, reader)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--baseline", type=Path)
    source.add_argument("--bundle", type=Path)
    source.add_argument("--v2-workspace", type=Path, help="Check unpublished fixture payloads")
    source.add_argument("--produce-v2", action="store_true", help="Capture unpublished fixture payloads")
    source.add_argument("--pins", action="store_true", help="Download the pinned release fixtures with authenticated gh")
    parser.add_argument("--base-ref")
    parser.add_argument("--repository")
    parser.add_argument("--sha256")
    parser.add_argument("--candidate", type=Path)
    parser.add_argument("--numerical", type=Path)
    parser.add_argument("--numerical-sha256")
    parser.add_argument("--producer-receipt", type=Path, help="Checked-in packaged producer receipt")
    parser.add_argument("--workspace-only", action="store_true",
                        help="Capture only the workspace or replay it with published numerical pins")
    parser.add_argument("--fixture-inference", type=Path)
    parser.add_argument("--workspace-migrator", type=Path, help="Candidate helper for schema-only workspace upgrades")
    parser.add_argument("--compact-vector-reader", type=Path,
                        help="Use compact checks with this native fixture-vector reader")
    parser.add_argument("--check-build", action="store_true",
                        help="Compare freshly built and predecessor indexes for ingest changes")
    parser.add_argument("--near-limit", action="store_true",
                        help="Check mixed batches at the native token limit")
    parser.add_argument("--produce", type=Path, help="Write a new baseline archive")
    parser.add_argument("--producer-commit")
    parser.add_argument("--producer-target")
    parser.add_argument("--producer-build")
    parser.add_argument("--profile", action="append", choices=PROFILES)
    parser.add_argument("--model-cache", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if (args.workspace_only or args.producer_receipt) and not (args.produce_v2 or args.v2_workspace):
        parser.error("producer receipts and workspace-only mode require split fixture capture or replay")
    if args.producer_receipt and args.v2_workspace and not args.workspace_only:
        parser.error("a producer receipt for replay requires workspace-only mode")
    if args.repository and not re.fullmatch(r"[\w.-]+/[\w.-]+", args.repository):
        parser.error("expected a GitHub repository in owner/name format")
    if args.produce_v2:
        if (not args.repository or (not args.workspace_only and not args.compact_vector_reader)
                or args.produce or args.candidate or args.profile or args.numerical or args.numerical_sha256):
            parser.error("--produce-v2 requires a repository and, for full capture, a fixture vector reader")
    elif args.v2_workspace:
        if not all((args.candidate, args.sha256, args.fixture_inference, args.compact_vector_reader,
                    args.workspace_migrator)):
            parser.error("--v2-workspace requires a candidate, workspace checksum and candidate helpers")
        if args.workspace_only:
            if not args.repository or args.numerical or args.numerical_sha256:
                parser.error("workspace-only replay requires a repository and uses published numerical pins")
        elif not (args.numerical and args.numerical_sha256):
            parser.error("full replay requires a local numerical payload and its checksum")
    elif args.produce:
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
    reader = args.compact_vector_reader.resolve() if args.compact_vector_reader else None
    inference = args.fixture_inference.resolve() if args.fixture_inference else None
    if args.v2_workspace:
        workspace = (args.v2_workspace.resolve(), args.sha256)
        if args.workspace_only:
            check_replacement_workspace(args.candidate.resolve(), cache, root, args.base_ref,
                                        args.repository, migrator, reader, inference, workspace,
                                        args.producer_receipt)
            return
        pins, _ = read_pins(args.base_ref)
        if any(pin.get("transition_to") for pin in pins.values()):
            raise RuntimeError("Full capture replay requires compatible execution policies")
        policies = {profile: pin["execution_policy"] for profile, pin in pins.items()}
        check_v2_payloads(
            args.candidate.resolve(), cache, root, migrator, reader, inference,
            workspace, (args.numerical, args.numerical_sha256),
            policies, {}, check_build=args.check_build, near_limit=args.near_limit,
        )
        return
    if args.produce_v2:
        produce_v2(args.repository, cache, root, reader, receipt=args.producer_receipt,
                   workspace_only=args.workspace_only)
        return
    if args.pins:
        check_pins(args.candidate.resolve(), cache, root, args.base_ref, args.repository, migrator, reader, inference,
                   check_build=args.check_build, near_limit=args.near_limit)
        return
    if args.bundle:
        manifest = extract_bundle(args.bundle, args.sha256, root)
        profiles = tuple(args.profile or manifest["profiles"])
    else:
        provenance = {"commit": args.producer_commit, "target": args.producer_target, "build": args.producer_build}
        manifest = capture(args.baseline.resolve(), cache, root, profiles, provenance, compact=reader is not None)
    if args.produce:
        archive_bundle(root, args.produce)
        print(f"{sha256(args.produce)}  {args.produce}")
    else:
        check(args.candidate.resolve(), cache, root, profiles, manifest, {}, migrator, reader)


if __name__ == "__main__":
    main()
