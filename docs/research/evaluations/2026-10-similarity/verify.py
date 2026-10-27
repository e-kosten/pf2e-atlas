#!/usr/bin/env python3
"""Verify frozen Similar evaluation inputs without opening rankings or a database."""
import argparse
import collections
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent


def sha(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def load(name):
    return json.loads((ROOT / name).read_text())


def verify(source_root=None):
    freeze = load("freeze.json")
    require(freeze["state"] == "frozen-before-ranked-results", "unexpected freeze state")
    for name, digest in freeze["files"].items():
        require(Path(name).name == name, "freeze paths must be local leaf names")
        require(sha((ROOT / name).read_bytes()) == digest, f"digest changed: {name}")
    manifest, corpus, sources = [load(n) for n in ["manifest.json", "corpus.json", "seeds.json"]]
    generation = manifest["artifact_generation"]
    require(corpus["artifact_generation"] == sources["artifact_generation"] == generation, "generation mismatch")
    require(corpus["source_revision"] == sources["source_revision"] == manifest["source_revision"], "source revision mismatch")
    require(manifest["profiles"] == ["bge-base", "embeddinggemma"], "unexpected profile matrix")
    scenarios = manifest["scenarios"]
    require(len(scenarios) == 24, "expected 24 scenarios")
    require(len({s["id"] for s in scenarios}) == 24, "duplicate scenario IDs")
    require(collections.Counter(s["split"] for s in scenarios) == {"development": 12, "holdout": 12}, "split counts")
    require(collections.Counter(s["stratum"] for s in scenarios) == {"whole-record": 12, "section-focused": 6, "trait-default": 6}, "comparison strata")
    clusters, seeds = collections.defaultdict(set), collections.defaultdict(set)
    for scenario in scenarios:
        clusters[scenario["cluster"]].add(scenario["split"])
        seeds[canonical(scenario["seed"])].add(scenario["split"])
    require(all(len(v) == 1 for v in clusters.values()), "theme cluster leaks across split")
    require(all(len(v) == 1 for v in seeds.values()), "seed leaks across split")
    require(len({s["cluster"] for s in scenarios if s["split"] == "holdout"}) == 3, "holdout theme count")
    bindings = {b["id"]: b for b in sources["bindings"]}
    require(len(bindings) == 22, "expected 22 unique source seeds")
    families = collections.Counter()
    cells = 0
    for scenario in scenarios:
        binding = bindings[scenario["source_binding_id"]]
        require(binding["seed"] == scenario["seed"], f"seed binding mismatch: {scenario['id']}")
        kind = scenario["seed"]["kind"]
        family = "trait" if kind == "trait" else next(r["kind"] for r in corpus["records"] if r["record_key"] == scenario["seed"]["record_key"])
        families[family, scenario["split"]] += 1
        require(scenario["compare_corresponding"] == (kind == "record"), "trait correspondence is unavailable")
        require(scenario["candidate_filter"] is None, "v1 corpus comparisons use no additional filter")
        cells += len(manifest["profiles"]) * sum(1 for v in manifest["variants"] if kind == "record" or v["options"].get("candidate_parts") != "corresponding")
        selection = scenario["selection"]
        require(selection["kind"] in ["all", "explicit"], "unknown selection kind")
        if selection["kind"] == "explicit":
            require(selection["artifact_generation"] == generation, "explicit selection generation")
            require(selection["selectors"] and len(set(selection["selectors"])) == len(selection["selectors"]), "empty/duplicate selected leaves")
            require(scenario["baseline_comparison"] == "context-only-unmatched-selection", "baseline selection scope must be explicit")
        for profile in manifest["profiles"]:
            inventory = binding["inventories"][profile]
            parts = inventory["parts"]
            original_inventory = {"artifact_generation": generation, "search_profile": profile, "seed": binding["seed"], "parts": parts, "role_recipe": binding["role_recipe"], "supports_corresponding_parts": inventory["supports_corresponding_parts"]}
            require(sha(canonical(original_inventory)) == inventory["sha256"], f"inventory binding changed: {binding['id']}/{profile}")
            require(len({p["selector"] for p in parts}) == len(parts), "duplicate inventory selectors")
            if selection["kind"] == "explicit":
                eligible = {p["selector"] for p in parts if p["has_vectors"]}
                require(set(selection["selectors"]) <= eligible, f"selected leaves unavailable: {scenario['id']}/{profile}")
    require(families == {(family, split): 3 for family in ["hazard", "creature", "spell", "trait"] for split in ["development", "holdout"]}, "family balance")
    require(cells == 552, "unexpected applicable variant cell count")
    amp = next(s for s in scenarios if s["seed"] == {"kind": "trait", "trait_key": "trait:amp"})
    require(not amp["quality_eligible"], "label-only Amp is diagnostic-only")
    amp_binding = bindings[amp["source_binding_id"]]
    require(amp_binding["source"]["catalogs"] and not amp_binding["source"]["definition_available"], "Amp must be known catalog label without definition")
    require(all(p["label_only"] and p["has_vectors"] for i in amp_binding["inventories"].values() for p in i["parts"]), "Amp label-only vectors")
    records = corpus["records"]
    require(len(records) == len({r["record_key"] for r in records}) == 278, "corpus identity count")
    sample = [r for r in records if "identity-sample" in r["origins"]]
    require(collections.Counter(r["stratum"] for r in sample) == {k: 60 for k in ["hazard", "creature", "spell", "other"]}, "stratified sample counts")
    require(corpus["selection"]["seed"] == 20261026, "identity sample seed")
    require(corpus["selection"]["prefer_remaster"] is False and manifest["policy"]["prefer_remaster"] is False, "controlled universe edition policy")
    for source in [b["source"] for b in bindings.values() if b["seed"]["kind"] == "record"] + [r["source"] for r in records if "source" in r]:
        require(re.fullmatch(r"[0-9a-f]{64}", source["sha256"]) is not None, "source digest format")
        path = Path(source["path"])
        require(not path.is_absolute() and ".." not in path.parts, "unsafe source path")
        if source_root:
            require(sha((source_root / path).read_bytes()) == source["sha256"], f"original source hash changed: {path}")
    performance = manifest["performance"]
    require(performance["warm_repetitions"] == 20 and performance["cold_sessions"] == 1, "performance sample contract")
    require(performance["additional_cases"] == [{"scenario_id": "dev-c02", "variant_id": "passages-corresponding-0p25", "profiles": manifest["profiles"]}], "complex corresponding timing case")
    print(json.dumps({"status": "ok", "experiment_id": manifest["experiment_id"], "scenarios": 24, "unique_seeds": 22, "corpus_keys": 278, "applicable_cells": cells, "freeze_sha256": sha((ROOT / "freeze.json").read_bytes())}, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-root", type=Path, help="optionally verify original source files under the pinned checkout")
    args = parser.parse_args()
    try:
        verify(args.source_root)
    except (ValueError, KeyError, OSError, json.JSONDecodeError, StopIteration) as error:
        parser.exit(1, f"Input verification failed: {error}\n")
