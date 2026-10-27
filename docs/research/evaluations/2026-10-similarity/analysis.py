#!/usr/bin/env python3
"""Build blind Similar judging pools and compute source-grounded relevance metrics.

This tool never scores vectors. It consumes the maintained atlas-dev rankings,
keeps retrieval metadata private, and treats absent/disputed grades as unknown.
"""
import argparse
import collections
import hashlib
import itertools
import json
import math
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parent
GAINS = (0, 1, 3, 7)
SUPPORT = {"supported", "partial", "unsupported", "unknown"}
CONNECTION = {"intrinsic", "owned-capability", "associated-only", "incidental", "none", "unknown"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()


def digest(value):
    return hashlib.sha256(value).hexdigest()


def load(path):
    return json.loads(Path(path).read_text())


def write(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    require(not path.exists(), f"refusing to overwrite evidence: {path}")
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False, sort_keys=True) + "\n")
    return digest(path.read_bytes())


def inputs(directory):
    directory = Path(directory)
    freeze = load(directory / "freeze.json")
    require(freeze["state"] == "frozen-before-ranked-results", "unexpected input freeze state")
    for name, expected in freeze["files"].items():
        require(Path(name).name == name, "unsafe frozen input path")
        require(digest((directory / name).read_bytes()) == expected, f"frozen input changed: {name}")
    return load(directory / "manifest.json"), load(directory / "corpus.json"), freeze, digest((directory / "freeze.json").read_bytes())


def report_data(path):
    value = load(path)
    if "status" in value and "data" in value:
        require(value["status"] == "ok", f"failed command report: {path}")
        value = value["data"]
    require(isinstance(value, dict), f"expected case report object: {path}")
    return value


def available_variants(manifest, scenario):
    return [variant for variant in manifest["variants"] if not (
        scenario["seed"]["kind"] == "trait" and variant["options"].get("candidate_parts") == "corresponding"
    )]


def ranking_rows(case, source):
    block = case.get(source)
    if not block:
        return []
    if source == "production":
        return [{"id": row["id"], "status": row["status"], "keys": row.get("returned_top20", [])[:10]}
                for row in block.get("variants", [])]
    require(source == "fixed_pool", f"unsupported ranking source: {source}")
    return [{"id": row["id"], "status": row.get("status", "ok"), "keys": row.get("ranking", [])[:10]}
            for row in block.get("rankings", [])]


def score_lookup(case, source, variant):
    block = case.get(source) or {}
    exact = block.get("exact") or {}
    for run in exact.get("runs", []):
        if run["id"] == variant:
            return {row["record_key"]: row.get("score") for row in run["rows"]}
    # Actual product capture has scalar scores for its representative variant.
    for capture in block.get("capture_groups", []):
        if capture.get("representative_variant") == variant:
            return {row["record_key"]: row.get("score") for row in capture.get("returned_scores", [])}
    return {}


def build_pool(directory, report_paths, seed, sources=("production", "fixed_pool"), scenarios=None, allow_incomplete=False):
    manifest, corpus, freeze, freeze_sha = inputs(directory)
    scenario_map = {scenario["id"]: scenario for scenario in manifest["scenarios"]}
    catalogue = {record["record_key"]: record for record in corpus["records"]}
    cases = {}
    input_files = []
    for path in sorted({Path(path).resolve() for path in report_paths}, key=str):
        case = report_data(path)
        require(case["scenario_id"] in scenario_map, f"unknown scenario report: {case['scenario_id']}")
        provenance = case["provenance"]
        for key, expected in [("freeze_sha256", freeze_sha), ("manifest_sha256", freeze["files"]["manifest.json"]),
                              ("corpus_sha256", freeze["files"]["corpus.json"]), ("artifact_generation", manifest["artifact_generation"]),
                              ("source_revision", manifest["source_revision"])]:
            require(provenance.get(key) == expected, f"report {path} has mismatched {key}")
        profile = case["profile"]
        require(profile in manifest["profiles"], f"unexpected report profile: {profile}")
        if scenarios and case["scenario_id"] not in scenarios:
            continue
        input_files.append({"path": str(path), "sha256": digest(path.read_bytes()), "provenance": provenance})
        for source in sources:
            for row in ranking_rows(case, source):
                require(row["id"] in {v["id"] for v in available_variants(manifest, scenario_map[case["scenario_id"]])}, "unknown or inapplicable variant in report")
                require(len(row["keys"]) == len(set(row["keys"])), "duplicate root in a ranking")
                key = (case["scenario_id"], source, profile, row["id"])
                scores = score_lookup(case, source, row["id"])
                record = {**row, "scores": [scores.get(root) for root in row["keys"]]}
                require(key not in cases or cases[key] == record, f"conflicting duplicate primary ranking: {key}")
                cases[key] = record
    selected = set(scenarios or scenario_map)
    require(selected and selected <= set(scenario_map), "no selected scenarios or unknown scenario ID")
    require(set(sources) <= {"production", "fixed_pool"} and len(sources) == len(set(sources)), "invalid ranking sources")
    public = {"version": 1, "experiment_id": manifest["experiment_id"], "input_freeze_sha256": freeze_sha,
              "randomization_seed": seed, "source_packets_pending": True, "scenarios": []}
    private = {"version": 1, "experiment_id": manifest["experiment_id"], "input_freeze_sha256": freeze_sha,
               "randomization_seed": seed, "ranking_sources": list(sources), "inputs": input_files, "scenarios": []}
    missing = []
    for scenario in manifest["scenarios"]:
        sid = scenario["id"]
        if sid not in selected:
            continue
        records, rankings = set(), []
        for source, profile, variant in itertools.product(sources, manifest["profiles"], available_variants(manifest, scenario)):
            key = sid, source, profile, variant["id"]
            row = cases.get(key)
            if row is None:
                missing.append(key)
                row = {"status": "missing_report", "keys": [], "scores": []}
            records.update(row["keys"])
            rankings.append({"configuration": f"{source}/{profile}/{variant['id']}", "source": source, "profile": profile,
                             "variant": variant["id"], "status": row["status"], "record_keys": row["keys"], "scores": row["scores"],
                             "matched_seed_scope": not (scenario["selection"]["kind"] == "explicit" and variant["options"]["method"] == "baseline")})
        ordered = sorted(records, key=lambda root: digest(canonical([seed, sid, root])))
        ids = {root: "c-" + digest(canonical([manifest["experiment_id"], sid, root]))[:16] for root in ordered}
        require(len(ids) == len(set(ids.values())), "blind ID collision")
        kind = "trait" if scenario["seed"]["kind"] == "trait" else catalogue[scenario["seed"]["record_key"]]["kind"]
        public["scenarios"].append({"scenario_id": sid, "intent": scenario["intent"], "task_scope": scenario["selection"]["kind"],
                                    "selection": scenario["selection"],
                                    "seed_identity": {**scenario["seed"], "title": scenario.get("seed_title")},
                                    "source_binding_id": scenario["source_binding_id"], "selected_sections": scenario.get("selection_labels", []),
                                    "candidates": [{"blind_candidate_id": ids[root], "record_key": root} for root in ordered]})
        for ranking in rankings:
            ranking["ranking"] = [ids[root] for root in ranking.pop("record_keys")]
        private["scenarios"].append({"scenario_id": sid, "split": scenario["split"], "stratum": scenario["stratum"],
                                      "kind": kind, "cluster": scenario["cluster"], "quality_eligible": scenario.get("quality_eligible", True),
                                      "candidates": [{"blind_candidate_id": ids[root], "record_key": root} for root in ordered], "rankings": rankings})
    require(allow_incomplete or not missing, f"missing compared configuration reports (use --allow-incomplete only for a non-grading draft): {missing[:5]}")
    private["complete_configuration_reports"] = not missing
    private["missing_reports"] = missing
    return public, private


def freeze_pool(public, private, destination):
    destination = Path(destination)
    public_sha = write(destination / "pool.json", public)
    private["public_pool_sha256"] = public_sha
    private_sha = write(destination / "private-rankings.json", private)
    write(destination / "pool.freeze.json", {"version": 1, "state": "pool-constructed-awaiting-source-packets",
          "public_pool_sha256": public_sha, "private_rankings_sha256": private_sha,
          "input_freeze_sha256": public["input_freeze_sha256"], "randomization_seed": public["randomization_seed"]})


def validate_source_piece(piece, label):
    require(isinstance(piece, dict), f"{label}: source piece must be an object")
    require(set(piece) <= {"identity", "availability", "reason", "content", "source_pointers", "source_hashes"}, f"{label}: unexpected public source metadata")
    require(piece.get("availability") in {"available", "label_only", "bounded", "unavailable"}, f"{label}: source availability required")
    identity = piece.get("identity", {})
    require(isinstance(identity, dict) and identity.get("title"), f"{label}: title required")
    require(set(identity) <= {"record_key", "trait_key", "title", "kind"}, f"{label}: unexpected public identity metadata")
    hashes = piece.get("source_hashes", [])
    require(isinstance(hashes, list) and all(isinstance(x, str) and re.fullmatch(r"[a-f0-9]{64}", x) for x in hashes), f"{label}: invalid source hash")
    require(isinstance(piece.get("content", []), list) and isinstance(piece.get("source_pointers", []), list), f"{label}: content and pointers must be lists")
    for content in piece.get("content", []):
        require(isinstance(content, dict) and set(content) <= {"label", "text", "html", "address", "source_sha256", "availability", "reason"}, f"{label}: unexpected public content metadata")
        require(content.get("text") or content.get("html") or content.get("reason"), f"{label}: content needs text, html or an explicit absence reason")
    if piece["availability"] == "unavailable":
        require(piece.get("reason"), f"{label}: unavailable source needs a reason")
    else:
        require(hashes and piece.get("source_pointers") and piece.get("content"), f"{label}: source content, hashes and pointers required")


def bind_packets(pool_directory, source_packets, destination):
    pool_directory, destination = Path(pool_directory), Path(destination)
    pool, private, freeze = [load(pool_directory / name) for name in ["pool.json", "private-rankings.json", "pool.freeze.json"]]
    require(freeze["state"] == "pool-constructed-awaiting-source-packets", "unexpected pool freeze state")
    require(private["complete_configuration_reports"], "incomplete configuration draft cannot be frozen for grading")
    require(freeze["public_pool_sha256"] == digest((pool_directory / "pool.json").read_bytes()), "public pool changed")
    require(freeze["private_rankings_sha256"] == digest((pool_directory / "private-rankings.json").read_bytes()), "private mapping changed")
    supplied = source_packets["scenarios"]
    require(len({row["scenario_id"] for row in supplied}) == len(supplied), "duplicate source-packet scenario")
    by_scenario = {row["scenario_id"]: row for row in supplied}
    require(set(by_scenario) == {row["scenario_id"] for row in pool["scenarios"]}, "source-packet scenario coverage mismatch")
    packet_hashes = {}
    for scenario in pool["scenarios"]:
        sid = scenario["scenario_id"]
        require(re.fullmatch(r"[A-Za-z0-9_-]+", sid), "unsafe scenario packet filename")
        supplied = by_scenario[sid]
        require(set(supplied) <= {"scenario_id", "seed", "candidates"}, "unexpected source-packet metadata")
        validate_source_piece(supplied["seed"], sid + " seed")
        source_seed = supplied["seed"]["identity"]
        seed = scenario["seed_identity"]
        source_key = seed.get("record_key", seed.get("trait_key"))
        require(source_seed.get("record_key", source_seed.get("trait_key")) == source_key, f"{sid}: seed identity mismatch")
        rows = supplied["candidates"]
        require(len({row["blind_candidate_id"] for row in rows}) == len(rows), f"{sid}: duplicate candidate source")
        candidates = {row["blind_candidate_id"]: row for row in rows}
        require(set(candidates) == {row["blind_candidate_id"] for row in scenario["candidates"]}, f"{sid}: candidate source coverage mismatch")
        public_candidates = []
        for candidate in scenario["candidates"]:
            cid = candidate["blind_candidate_id"]
            piece = dict(candidates[cid])
            piece.pop("blind_candidate_id")
            validate_source_piece(piece, sid + "/" + cid)
            require(piece["identity"].get("record_key") == candidate["record_key"], f"{sid}/{cid}: wrong root source")
            public_candidates.append({"blind_candidate_id": cid, **piece})
        packet = {"version": 1, "experiment_id": pool["experiment_id"], "scenario_id": sid,
                  "intent": scenario["intent"], "task_scope": scenario["task_scope"], "selected_sections": scenario["selected_sections"],
                  "selection": scenario["selection"],
                  "seed": supplied["seed"], "candidates": public_candidates}
        packet_hashes[sid] = {"file": f"{sid}.json", "sha256": write(destination / f"{sid}.json", packet)}
    return {"version": 1, "state": "grading-packets-frozen-before-opinions", "input_freeze_sha256": pool["input_freeze_sha256"],
            "public_pool_sha256": freeze["public_pool_sha256"], "private_rankings_sha256": freeze["private_rankings_sha256"],
            "randomization_seed": pool["randomization_seed"], "source_packet_input_sha256": digest(canonical(source_packets)), "packets": packet_hashes}


def grade(value):
    require(value == "unknown" or (type(value) is int and 0 <= value <= 3), "grade must be 0, 1, 2, 3 or unknown")
    return None if value == "unknown" else value


def read_opinions(paths, candidates, packet_hashes, judges, adjudication=False):
    opinions, receipts = {}, []
    for path in paths:
        path = Path(path)
        receipts.append({"path": str(path), "sha256": digest(path.read_bytes())})
        for number, line in enumerate(path.read_text().splitlines(), 1):
            if not line.strip():
                continue
            opinion = json.loads(line)
            sid, cid, judge = opinion["scenario_id"], opinion["blind_candidate_id"], opinion["judge_id"]
            require((sid, cid) in candidates, f"{path}:{number}: opinion outside frozen pool")
            require((judge not in judges) if adjudication else (judge in judges), "wrong judge role")
            require(opinion.get("judge_model") and opinion.get("rationale"), "judge model and rationale required")
            require(isinstance(opinion.get("judge_prompt_sha256"), str) and re.fullmatch(r"[a-f0-9]{64}", opinion["judge_prompt_sha256"]), "judge prompt digest required")
            require(opinion.get("packet_sha256") == packet_hashes[sid]["sha256"], "opinion packet digest mismatch")
            value = grade(opinion["grade"])
            require(opinion.get("evidence_support") in SUPPORT, "invalid evidence-support grade")
            require(opinion.get("connection_class") in CONNECTION, "invalid connection class")
            require(opinion.get("uncertainty") in {"low", "medium", "high", "unknown"}, "uncertainty required")
            require(isinstance(opinion.get("source_pointers"), list), "source pointers required")
            require(value is None or opinion["source_pointers"], "known usefulness grade requires source pointers")
            key = (sid, cid) if adjudication else (sid, cid, judge)
            require(key not in opinions, "duplicate opinion; preserve revisions in separate analysis runs")
            opinions[key] = opinion
    return opinions, receipts


def resolve(first, second, adjudication=None):
    if first is None or second is None:
        return None, ["missing_independent_opinion"]
    a, b = grade(first["grade"]), grade(second["grade"])
    reasons = []
    if a is None or b is None:
        reasons.append("unknown_opinion")
    elif a != b:
        reasons.append("grade_disagreement")
    if {first["evidence_support"], second["evidence_support"]} == {"supported", "unsupported"}:
        reasons.append("evidence_support_dispute")
    connections = {first["connection_class"], second["connection_class"]}
    if "associated-only" in connections and connections & {"intrinsic", "owned-capability"}:
        reasons.append("ownership_dispute")
    if adjudication is not None:
        return grade(adjudication["grade"]), reasons
    return (None if reasons else a), reasons


def dcg(values):
    return sum(GAINS[value] / math.log2(index + 2) for index, value in enumerate(values[:10]))


def metric(ranking, grades):
    require(all(value is not None for value in grades.values()), "metric requires a complete grade assignment")
    ideal = dcg(sorted(grades.values(), reverse=True))
    return {"ndcg_at10": dcg([grades[cid] for cid in ranking[:10]]) / ideal if ideal else None,
            "useful_at5": sum(grades[cid] >= 2 for cid in ranking[:5]), "ideal_dcg_at10": ideal}


def mean(values):
    values = [value for value in values if value is not None]
    return sum(values) / len(values) if values else None


def summaries(case_results, manifest_scenarios):
    aggregates = []
    group_keys = sorted({(case["source"], case["split"], case["stratum"], case["judgment_view"]) for case in case_results})
    scenario_meta = {s["scenario_id"]: s for s in manifest_scenarios}
    for source, split, stratum, view in group_keys:
        rows = [row for row in case_results if (row["source"], row["split"], row["stratum"], row["judgment_view"]) == (source, split, stratum, view) and row["matched_seed_scope"]]
        configs = sorted({row["configuration"] for row in rows})
        case_ids = sorted({row["scenario_id"] for row in rows if scenario_meta[row["scenario_id"]]["quality_eligible"]})
        complete = [sid for sid in case_ids if all(any(row["scenario_id"] == sid and row["configuration"] == config and row["primary_eligible"] for row in rows) for config in configs)]
        expected_kinds = {"hazard", "creature", "spell"} if stratum == "whole-record" else {scenario_meta[sid]["kind"] for sid in case_ids}
        for config in configs:
            selected = [row for row in rows if row["configuration"] == config and row["scenario_id"] in complete]
            values = {}
            for field in ["ndcg_at10", "useful_at5"]:
                clusters = collections.defaultdict(list)
                for row in selected:
                    value = row["primary"][field]
                    if value is not None:
                        clusters[(row["kind"], row["cluster"])].append(value)
                per_kind = collections.defaultdict(list)
                for (kind, _), observations in clusters.items():
                    per_kind[kind].append(mean(observations))
                values[field] = {"equal_scenario_mean": mean([row["primary"][field] for row in selected]),
                                 "per_kind_cluster_mean": {kind: mean(observations) for kind, observations in sorted(per_kind.items())},
                                 "kind_macro": mean([mean(per_kind[kind]) for kind in sorted(expected_kinds)]) if expected_kinds and all(per_kind[kind] for kind in expected_kinds) else None,
                                 "missing_kinds": sorted(kind for kind in expected_kinds if not per_kind[kind])}
            aggregates.append({"source": source, "split": split, "stratum": stratum, "judgment_view": view, "configuration": config,
                               "shared_complete_scenarios": complete, "planned_quality_scenarios": case_ids,
                               "planned_kind_counts": dict(collections.Counter(scenario_meta[sid]["kind"] for sid in case_ids)),
                               "complete_kind_counts": dict(collections.Counter(row["kind"] for row in selected)),
                               "zero_ideal_scenarios": [row["scenario_id"] for row in selected if row["primary"]["ndcg_at10"] is None], "metrics": values})
    return aggregates


def pairwise(case_results, aggregates):
    complete = {(row["source"], row["split"], row["stratum"], row["judgment_view"]): set(row["shared_complete_scenarios"]) for row in aggregates}
    groups = collections.defaultdict(list)
    for row in case_results:
        key = row["source"], row["split"], row["stratum"], row["judgment_view"]
        if row["matched_seed_scope"] and row["primary_eligible"] and row["scenario_id"] in complete.get(key, set()):
            groups[(row["source"], row["split"], row["stratum"], row["judgment_view"], row["scenario_id"])].append(row)
    pairs = collections.defaultdict(list)
    for key, rows in groups.items():
        for left, right in itertools.combinations(sorted(rows, key=lambda row: row["configuration"]), 2):
            a, b = left["primary"]["ndcg_at10"], right["primary"]["ndcg_at10"]
            if a is None or b is None:
                continue
            pairs[(*key[:4], left["configuration"], right["configuration"])].append((key[4], a - b))
    return [{"source": key[0], "split": key[1], "stratum": key[2], "judgment_view": key[3], "left": key[4], "right": key[5],
             "metric": "pool-relative ndcg_at10 from matched source grades", "scenarios": [sid for sid, _ in values],
             "left_wins": sum(delta > 1e-12 for _, delta in values), "ties": sum(abs(delta) <= 1e-12 for _, delta in values),
             "right_wins": sum(delta < -1e-12 for _, delta in values), "mean_difference": mean([delta for _, delta in values])}
            for key, values in sorted(pairs.items())]


def analyze(private, judging_freeze, opinions, judges, adjudications=None):
    adjudications = adjudications or {}
    require(len(judges) == 2 and len(set(judges)) == 2, "two distinct independent judge IDs required")
    case_results, resolved, overlaps = [], [], []
    for scenario in private["scenarios"]:
        sid = scenario["scenario_id"]
        ids = [candidate["blind_candidate_id"] for candidate in scenario["candidates"]]
        views = {judge: {} for judge in judges}
        views["agreement_or_adjudication"] = {}
        for cid in ids:
            first, second = [opinions.get((sid, cid, judge)) for judge in judges]
            for judge, opinion in zip(judges, [first, second]):
                views[judge][cid] = grade(opinion["grade"]) if opinion else None
            value, reasons = resolve(first, second, adjudications.get((sid, cid)))
            views["agreement_or_adjudication"][cid] = value
            resolved.append({"scenario_id": sid, "blind_candidate_id": cid, "grade": value if value is not None else "unknown",
                             "original_opinions": {judge: opinions.get((sid, cid, judge)) for judge in judges},
                             "adjudication": adjudications.get((sid, cid)), "disagreements": reasons,
                             "adjudication_required": bool(reasons) and (sid, cid) not in adjudications})
        for ranking in scenario["rankings"]:
            for name, grades in views.items():
                missing = {cid for cid, value in grades.items() if value is None}
                available = ranking["status"] == "ok"
                complete = available and not missing and scenario["quality_eligible"]
                row = {"scenario_id": sid, "split": scenario["split"], "stratum": scenario["stratum"], "kind": scenario["kind"], "cluster": scenario["cluster"],
                       "configuration": ranking["configuration"], "source": ranking["source"], "matched_seed_scope": ranking["matched_seed_scope"],
                       "judgment_view": name, "status": ranking["status"], "quality_eligible": scenario["quality_eligible"],
                       "judged_candidates": len(ids) - len(missing), "unknown_candidates": len(missing),
                       "unknown_top10_positions": sum(cid in missing for cid in ranking["ranking"][:10]),
                       "primary_eligible": complete, "primary": metric(ranking["ranking"], grades) if complete else None,
                       "sensitivity": {}}
                if available and scenario["quality_eligible"]:
                    for bound in [0, 3]:
                        row["sensitivity"][f"all_unknown_grade_{bound}"] = metric(ranking["ranking"], {cid: bound if value is None else value for cid, value in grades.items()})
                case_results.append(row)
        for left, right in itertools.combinations(scenario["rankings"], 2):
            if left["source"] != right["source"] or left["status"] != "ok" or right["status"] != "ok":
                continue
            a, b = set(left["ranking"][:10]), set(right["ranking"][:10])
            overlaps.append({"scenario_id": sid, "left": left["configuration"], "right": right["configuration"], "shared_roots": len(a & b),
                             "union_roots": len(a | b), "jaccard": len(a & b) / len(a | b) if a | b else None})
    aggregates = summaries(case_results, private["scenarios"])
    return {"version": 1, "experiment_id": private["experiment_id"], "input_freeze_sha256": private["input_freeze_sha256"],
            "judging_freeze": judging_freeze, "judge_ids": judges,
            "consensus_policy": "exact grade agreement or independent adjudication; material evidence/ownership disputes require adjudication",
            "sensitivity_note": "Uniform unknown-grade assignments are sensitivity scenarios, not guaranteed mathematical nDCG bounds or confidence intervals.",
            "metric_scope": "nDCG is relative to the entire judged union pool, not a corpus-wide ideal; candidate recall is separate and not computed here.",
            "resolved_judgments": resolved, "cases": case_results, "aggregates": aggregates,
            "pairwise_graded_preferences": pairwise(case_results, aggregates), "method_overlap": overlaps}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    pool = sub.add_parser("pool", help="construct public blind pool and private rankings; does not open source bodies or grades")
    pool.add_argument("--input-dir", type=Path, default=ROOT)
    pool.add_argument("--report", type=Path, action="extend", nargs="+", required=True)
    pool.add_argument("--scenario", action="append")
    pool.add_argument("--seed", type=int, default=20261026)
    pool.add_argument("--ranking-source", choices=["production", "fixed_pool"], action="append")
    pool.add_argument("--allow-incomplete", action="store_true")
    pool.add_argument("--output-dir", type=Path, required=True)
    bind = sub.add_parser("bind-packets", help="validate public source packets and freeze before opinions")
    bind.add_argument("--pool-dir", type=Path, required=True)
    bind.add_argument("--source-packets", type=Path, required=True)
    bind.add_argument("--output-dir", type=Path, required=True)
    metrics = sub.add_parser("metrics", help="analyze two independent opinions and optional independent adjudications")
    metrics.add_argument("--pool-dir", type=Path, required=True)
    metrics.add_argument("--judging-freeze", type=Path, required=True)
    metrics.add_argument("--judgments", type=Path, action="append", required=True)
    metrics.add_argument("--adjudications", type=Path, action="append", default=[])
    metrics.add_argument("--judge", action="append", required=True)
    metrics.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.command == "pool":
            public, private = build_pool(args.input_dir, args.report, args.seed, args.ranking_source or ["production", "fixed_pool"], args.scenario, args.allow_incomplete)
            freeze_pool(public, private, args.output_dir)
            result = {"scenarios": len(public["scenarios"]), "candidates": sum(len(row["candidates"]) for row in public["scenarios"]), "complete_configuration_reports": private["complete_configuration_reports"]}
        elif args.command == "bind-packets":
            freeze = bind_packets(args.pool_dir, load(args.source_packets), args.output_dir)
            write(args.output_dir / "judging.freeze.json", freeze)
            result = {"packets": len(freeze["packets"]), "judging_freeze_sha256": digest((args.output_dir / "judging.freeze.json").read_bytes())}
        else:
            private_path = args.pool_dir / "private-rankings.json"
            private, freeze = load(private_path), load(args.judging_freeze)
            require(freeze["state"] == "grading-packets-frozen-before-opinions", "unexpected judging freeze state")
            require(freeze["private_rankings_sha256"] == digest(private_path.read_bytes()), "private rankings changed after packet freeze")
            for packet in freeze["packets"].values():
                require(Path(packet["file"]).name == packet["file"], "unsafe packet filename")
                require(digest((args.judging_freeze.parent / packet["file"]).read_bytes()) == packet["sha256"], "grading packet changed")
            candidates = {(row["scenario_id"], c["blind_candidate_id"]) for row in private["scenarios"] for c in row["candidates"]}
            opinions, receipts = read_opinions(args.judgments, candidates, freeze["packets"], args.judge)
            adjudications, adjudication_receipts = read_opinions(args.adjudications, candidates, freeze["packets"], args.judge, True)
            report = analyze(private, freeze, opinions, args.judge, adjudications)
            report["opinion_files"] = receipts
            report["adjudication_files"] = adjudication_receipts
            write(args.output, report)
            result = {"case_metrics": len(report["cases"]), "unresolved": sum(row["grade"] == "unknown" for row in report["resolved_judgments"])}
        print(json.dumps({"status": "ok", **result}, sort_keys=True))
    except (ValueError, KeyError, OSError, TypeError, json.JSONDecodeError) as error:
        parser.exit(2, f"Analysis failed: {error}\n")


if __name__ == "__main__":
    main()
