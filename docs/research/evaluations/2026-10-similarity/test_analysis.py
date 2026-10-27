"""Synthetic analysis regressions; never read ranked experiment reports."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location("similarity_analysis", Path(__file__).with_name("analysis.py"))
a = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(a)


def opinion(value, connection="intrinsic", support="supported"):
    return {"grade": value, "connection_class": connection, "evidence_support": support}


def source(key, unavailable=False):
    return {"identity": {"record_key": key, "title": "Source " + key, "kind": "creature"},
            "availability": "unavailable" if unavailable else "available", "reason": "Missing canonical source" if unavailable else "Canonical source",
            "content": [] if unavailable else [{"label": "Description", "text": "Owned fire ability", "address": {"document_id": key}}],
            "source_pointers": [] if unavailable else ["source.json#/" + key], "source_hashes": [] if unavailable else ["1" * 64]}


class PoolFixture(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.inputs = self.root / "inputs"
        self.inputs.mkdir()
        self.manifest = {"experiment_id": "synthetic-only", "artifact_generation": "synthetic:1", "source_revision": "source-revision",
                         "profiles": ["model-a", "model-b"], "variants": [{"id": "baseline", "options": {"method": "baseline"}},
                             {"id": "passages", "options": {"method": "passages", "candidate_parts": "any"}}],
                         "scenarios": [{"id": "case-1", "seed": {"kind": "record", "record_key": "seed"}, "seed_title": "Seed",
                             "intent": "Find owned fire abilities", "source_binding_id": "seed-binding", "selection": {"kind": "all"},
                             "selection_labels": [], "split": "development", "stratum": "whole-record", "cluster": "fire", "quality_eligible": True}]}
        corpus = {"records": [{"record_key": "seed", "kind": "creature"}]}
        a.write(self.inputs / "manifest.json", self.manifest)
        a.write(self.inputs / "corpus.json", corpus)
        self.freeze = {"state": "frozen-before-ranked-results", "files": {name: a.digest((self.inputs / name).read_bytes()) for name in ["manifest.json", "corpus.json"]}}
        a.write(self.inputs / "freeze.json", self.freeze)
        self.provenance = {"freeze_sha256": a.digest((self.inputs / "freeze.json").read_bytes()),
                           "manifest_sha256": self.freeze["files"]["manifest.json"], "corpus_sha256": self.freeze["files"]["corpus.json"],
                           "artifact_generation": "synthetic:1", "source_revision": "source-revision"}
        self.paths = []
        for profile, production, fixed in [("model-a", ["alpha", "beta"], ["gamma", "alpha"]), ("model-b", ["beta", "delta"], ["epsilon", "delta"])]:
            case = {"scenario_id": "case-1", "profile": profile, "provenance": self.provenance,
                    "production": {"variants": [{"id": v["id"], "status": "ok", "returned_top20": production} for v in self.manifest["variants"]]},
                    "fixed_pool": {"rankings": [{"id": v["id"], "ranking": fixed} for v in self.manifest["variants"]]}}
            path = self.root / (profile + ".json")
            a.write(path, case)
            self.paths.append(path)

    def pool(self, **kwargs):
        return a.build_pool(self.inputs, self.paths, seed=123, **kwargs)

    def frozen_sources(self):
        public, private = self.pool()
        pool_dir = self.root / "pool"
        a.freeze_pool(public, private, pool_dir)
        rows = public["scenarios"][0]["candidates"]
        pieces = {"scenarios": [{"scenario_id": "case-1", "seed": source("seed"),
                                 "candidates": [{"blind_candidate_id": c["blind_candidate_id"], **source(c["record_key"])} for c in rows]}]}
        return public, private, pool_dir, pieces

    def test_deduplicated_all_source_profile_union_and_determinism(self):
        public, private = self.pool()
        again, _ = a.build_pool(self.inputs, reversed(self.paths), 123)
        self.assertEqual(public, again)
        candidates = public["scenarios"][0]["candidates"]
        self.assertEqual({c["record_key"] for c in candidates}, {"alpha", "beta", "gamma", "delta", "epsilon"})
        self.assertEqual(len(candidates), 5)
        self.assertEqual(len(private["scenarios"][0]["rankings"]), 8)
        self.assertNotIn("model-a", json.dumps(public))
        self.assertNotIn("configuration", json.dumps(public))
        self.assertNotIn("scores", json.dumps(public))
        changed_order, _ = a.build_pool(self.inputs, self.paths, 456)
        self.assertEqual({c["record_key"]: c["blind_candidate_id"] for c in candidates},
                         {c["record_key"]: c["blind_candidate_id"] for c in changed_order["scenarios"][0]["candidates"]})

    def test_missing_config_requires_draft_and_draft_cannot_grade(self):
        with self.assertRaisesRegex(ValueError, "missing compared"):
            a.build_pool(self.inputs, self.paths[:1], 123)
        public, private = a.build_pool(self.inputs, self.paths[:1], 123, allow_incomplete=True)
        self.assertFalse(private["complete_configuration_reports"])
        a.freeze_pool(public, private, self.root / "draft")
        with self.assertRaisesRegex(ValueError, "incomplete configuration"):
            a.bind_packets(self.root / "draft", {"scenarios": []}, self.root / "packets")

    def test_hash_mismatch_and_conflicting_rankings_fail(self):
        case = a.load(self.paths[0])
        case["provenance"]["source_revision"] = "wrong"
        wrong = self.root / "wrong.json"
        a.write(wrong, case)
        with self.assertRaisesRegex(ValueError, "mismatched source_revision"):
            a.build_pool(self.inputs, [wrong], 123)
        case = a.load(self.paths[0])
        case["production"]["variants"][0]["returned_top20"] = ["another"]
        conflict = self.root / "conflict.json"
        a.write(conflict, case)
        with self.assertRaisesRegex(ValueError, "conflicting duplicate"):
            a.build_pool(self.inputs, self.paths + [conflict], 123)
        (self.inputs / "corpus.json").write_text("{}")
        with self.assertRaisesRegex(ValueError, "frozen input changed"):
            self.pool()

    def test_packet_binding_exact_root_hashes_and_absence(self):
        public, _, pool_dir, pieces = self.frozen_sources()
        pieces["scenarios"][0]["candidates"][0] = {"blind_candidate_id": public["scenarios"][0]["candidates"][0]["blind_candidate_id"],
                                                  **source(public["scenarios"][0]["candidates"][0]["record_key"], unavailable=True)}
        freeze = a.bind_packets(pool_dir, pieces, self.root / "packets")
        packet = a.load(self.root / "packets" / "case-1.json")
        self.assertEqual(a.digest((self.root / "packets" / "case-1.json").read_bytes()), freeze["packets"]["case-1"]["sha256"])
        self.assertEqual(packet["selection"], {"kind": "all"})
        self.assertEqual(packet["candidates"][0]["availability"], "unavailable")
        self.assertNotIn("rankings", packet)
        with self.assertRaisesRegex(ValueError, "refusing to overwrite"):
            a.bind_packets(pool_dir, pieces, self.root / "packets")
        pieces["scenarios"][0]["candidates"][0]["identity"]["record_key"] = "wrong"
        with self.assertRaisesRegex(ValueError, "wrong root source"):
            a.bind_packets(pool_dir, pieces, self.root / "wrong-packets")

    def test_packet_forbids_retrieval_metadata_and_mutation(self):
        _, _, pool_dir, pieces = self.frozen_sources()
        pieces["scenarios"][0]["seed"]["identity"]["score"] = .99
        with self.assertRaisesRegex(ValueError, "unexpected public identity"):
            a.bind_packets(pool_dir, pieces, self.root / "packets")
        pieces["scenarios"][0]["seed"]["identity"].pop("score")
        pieces["scenarios"][0]["seed"]["content"][0]["model"] = "model-a"
        with self.assertRaisesRegex(ValueError, "unexpected public content"):
            a.bind_packets(pool_dir, pieces, self.root / "packets")
        (pool_dir / "pool.json").write_text("{}")
        with self.assertRaisesRegex(ValueError, "public pool changed"):
            a.bind_packets(pool_dir, pieces, self.root / "packets")

    def test_read_opinion_bound_to_packet_and_prompt_with_no_overwrite(self):
        row = {"scenario_id": "case-1", "blind_candidate_id": "cid", "judge_id": "a", "judge_model": "model-version",
               "judge_prompt_sha256": "2" * 64, "packet_sha256": "3" * 64, "grade": 2, "evidence_support": "supported",
               "connection_class": "intrinsic", "uncertainty": "low", "source_pointers": ["source.json#/x"], "rationale": "Useful owned capability"}
        path = self.root / "opinions.jsonl"
        path.write_text(json.dumps(row) + "\n")
        values, receipts = a.read_opinions([path], {("case-1", "cid")}, {"case-1": {"sha256": "3" * 64}}, ["a", "b"])
        self.assertEqual(values[("case-1", "cid", "a")], row)
        self.assertEqual(receipts[0]["sha256"], a.digest(path.read_bytes()))
        with self.assertRaisesRegex(ValueError, "duplicate opinion"):
            a.read_opinions([path, path], {("case-1", "cid")}, {"case-1": {"sha256": "3" * 64}}, ["a", "b"])
        row["packet_sha256"] = "4" * 64
        path.write_text(json.dumps(row) + "\n")
        with self.assertRaisesRegex(ValueError, "packet digest mismatch"):
            a.read_opinions([path], {("case-1", "cid")}, {"case-1": {"sha256": "3" * 64}}, ["a", "b"])


class Metrics(unittest.TestCase):
    def test_gains_zero_ideal_and_unknown_are_explicit(self):
        self.assertEqual(a.metric(["x", "y"], {"x": 3, "y": 2})["ndcg_at10"], 1)
        self.assertEqual(a.metric(["x", "y"], {"x": 3, "y": 2})["useful_at5"], 2)
        self.assertEqual(a.metric(["y", "x"], {"x": 3, "y": 2})["ndcg_at10"], (3 + 7 / a.math.log2(3)) / (7 + 3 / a.math.log2(3)))
        self.assertIsNone(a.metric(["x"], {"x": 0})["ndcg_at10"])
        self.assertEqual(a.metric(["x"], {"x": 0})["useful_at5"], 0)
        with self.assertRaisesRegex(ValueError, "complete grade"):
            a.metric(["x"], {"x": None})
        for invalid in [True, .5, -1, 4, "2", None]:
            with self.assertRaises(ValueError):
                a.grade(invalid)
        self.assertIsNone(a.grade("unknown"))

    def test_disputes_unknown_until_independent_adjudication(self):
        self.assertEqual(a.resolve(opinion(2), opinion(2)), (2, []))
        self.assertEqual(a.resolve(opinion(2), opinion(3)), (None, ["grade_disagreement"]))
        value, reasons = a.resolve(opinion(2), opinion(2, "associated-only", "unsupported"))
        self.assertIsNone(value)
        self.assertEqual(reasons, ["evidence_support_dispute", "ownership_dispute"])
        self.assertEqual(a.resolve(opinion(2), opinion(3), opinion(1))[0], 1)
        self.assertIsNone(a.resolve(opinion(2), None, opinion(1))[0])
        self.assertIsNone(a.resolve(opinion("unknown"), opinion(3))[0])

    def private(self):
        return {"experiment_id": "synthetic", "input_freeze_sha256": "1" * 64, "scenarios": [
            {"scenario_id": sid, "split": "holdout", "stratum": "whole-record", "kind": kind, "cluster": cluster, "quality_eligible": True,
             "candidates": [{"blind_candidate_id": sid + "-x"}, {"blind_candidate_id": sid + "-y"}],
             "rankings": [{"configuration": config, "source": "production", "matched_seed_scope": True, "status": "ok",
                           "ranking": [sid + "-x", sid + "-y"] if config == "a" else [sid + "-y", sid + "-x"]} for config in ["a", "b"]]}
            for sid, kind, cluster in [("h", "hazard", "fire"), ("c", "creature", "fire"), ("s", "spell", "ice"), ("u", "spell", "ice")]]}

    def opinions(self, private):
        return {(scenario["scenario_id"], c["blind_candidate_id"], judge): opinion(3 if c["blind_candidate_id"].endswith("-x") else 0)
                for scenario in private["scenarios"] for c in scenario["candidates"] for judge in ["j1", "j2"]}

    def test_common_complete_cases_unknown_sensitivity_and_paired_subset(self):
        private = self.private()
        opinions = self.opinions(private)
        opinions[("u", "u-x", "j2")] = opinion(2)
        # One captured method failure excludes this case from every compared primary aggregate.
        private["scenarios"][1]["rankings"][1]["status"] = "budget_exceeded"
        result = a.analyze(private, {}, opinions, ["j1", "j2"])
        combined = [row for row in result["aggregates"] if row["judgment_view"] == "agreement_or_adjudication"]
        self.assertEqual([row["shared_complete_scenarios"] for row in combined], [["h", "s"], ["h", "s"]])
        self.assertTrue(all(row["metrics"]["ndcg_at10"]["kind_macro"] is None for row in combined))
        unknown = next(row for row in result["cases"] if row["scenario_id"] == "u" and row["configuration"] == "a" and row["judgment_view"] == "agreement_or_adjudication")
        self.assertIsNone(unknown["primary"])
        self.assertEqual(unknown["unknown_candidates"], 1)
        self.assertIsNone(unknown["sensitivity"]["all_unknown_grade_0"]["ndcg_at10"])
        self.assertEqual(unknown["sensitivity"]["all_unknown_grade_3"]["ndcg_at10"], 1)
        failure = next(row for row in result["cases"] if row["scenario_id"] == "c" and row["configuration"] == "b")
        self.assertEqual(failure["sensitivity"], {})
        combined_pairs = [row for row in result["pairwise_graded_preferences"] if row["judgment_view"] == "agreement_or_adjudication"]
        self.assertEqual(combined_pairs[0]["scenarios"], ["h", "s"])
        adjudicated = a.analyze(private, {}, opinions, ["j1", "j2"], {("u", "u-x"): opinion(2)})
        self.assertEqual(next(row for row in adjudicated["aggregates"] if row["judgment_view"] == "agreement_or_adjudication")["shared_complete_scenarios"], ["h", "s", "u"])
        self.assertEqual(next(row for row in result["resolved_judgments"] if row["scenario_id"] == "u" and row["blind_candidate_id"] == "u-x")["original_opinions"]["j2"]["grade"], 2)

    def test_kind_cluster_macro_and_unmatched_advanced_scope(self):
        private = self.private()
        # A second spell in the same cluster must not double the kind's weight.
        result = a.analyze(private, {}, self.opinions(private), ["j1", "j2"])
        row = next(row for row in result["aggregates"] if row["configuration"] == "a")
        self.assertEqual(row["metrics"]["ndcg_at10"]["kind_macro"], 1)
        self.assertEqual(row["complete_kind_counts"], {"hazard": 1, "creature": 1, "spell": 2})
        # Replace known scalar metrics to expose the cluster/kind aggregation arithmetic.
        cases = [row for row in result["cases"] if row["judgment_view"] == "agreement_or_adjudication" and row["configuration"] == "a"]
        for row in cases:
            row["primary"]["ndcg_at10"] = {"h": 0, "c": .3, "s": .6, "u": 1}[row["scenario_id"]]
        summary = a.summaries(cases, private["scenarios"])[0]
        self.assertAlmostEqual(summary["metrics"]["ndcg_at10"]["kind_macro"], (0 + .3 + .8) / 3)
        private["scenarios"][0]["stratum"] = "section-focused"
        private["scenarios"][0]["rankings"][0]["matched_seed_scope"] = False
        result = a.analyze(private, {}, self.opinions(private), ["j1", "j2"])
        advanced = [row for row in result["aggregates"] if row["stratum"] == "section-focused"]
        self.assertEqual({row["configuration"] for row in advanced}, {"b"})


if __name__ == "__main__":
    unittest.main()
