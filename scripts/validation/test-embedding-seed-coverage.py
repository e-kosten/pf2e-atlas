#!/usr/bin/env python3
"""Exercise fail-closed advisory coverage routing."""

from pathlib import Path
import runpy
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
CHECK = runpy.run_path(str(ROOT / "scripts/release/check-embedding-seeds.py"))


class CoverageTest(unittest.TestCase):
    def test_only_verified_gaps_are_advisory(self):
        profile = "bge-base"
        pin = {"tag": "embedding-seed/bge-base/v1"}
        missing = {"kind": "missing_input", "profile": profile, "first_hash": "a" * 64}
        data = {"ready": False, "embedding_coverage_gap": missing}
        classify = CHECK["classify_gap"]
        self.assertEqual(classify(profile, pin, 3, data, {})["status"], "gap")
        for unexpected in ({"ready": False}, {"ready": True, "embedding_coverage_gap": missing},
                           {"ready": False, "embedding_coverage_gap": {**missing, "first_hash": "bad"}}):
            with self.assertRaises(RuntimeError):
                classify(profile, pin, 3, unexpected, {})
        with self.assertRaises(RuntimeError):
            classify(profile, pin, 1, data, {})

    def test_reviewed_transition_is_bound_to_pin_and_both_model_identities(self):
        profile = "embeddinggemma"
        pin = {"tag": "embedding-seed/embeddinggemma/v1"}
        old, expected = {"execution_policy": "old"}, {"execution_policy": "new"}
        data = {"ready": False, "embedding_coverage_gap": {
            "kind": "incompatible_model", "profile": profile, "previous": old, "expected": expected,
        }}
        transition = {"pin": pin, "previous": old, "expected": expected}
        classify = CHECK["classify_gap"]
        self.assertEqual(classify(profile, pin, 3, data, {profile: transition})["reason"],
                         "reviewed_model_transition")
        for evidence in ({}, {profile: {**transition, "pin": {"tag": "wrong"}}},
                         {profile: {**transition, "expected": old}}):
            with self.assertRaises(RuntimeError):
                classify(profile, pin, 3, data, evidence)

    def test_a_gap_does_not_mask_the_other_profile_failure(self):
        pins = dict.fromkeys(CHECK["PROFILES"], {"tag": "seed"})
        gap = {"kind": "missing_input", "profile": "bge-base", "first_hash": "a" * 64}
        setup = CHECK["check_advisory"].__globals__
        calls = [
            (3, {"ready": False, "embedding_seed_pins": pins, "embedding_coverage_gap": gap}),
            (3, {"ready": False, "embedding_seed_pins": pins, "error": "invalid hash"}),
        ]
        with tempfile.TemporaryDirectory() as directory:
            with patch.dict(setup, {"setup": lambda *args, **kwargs: calls.pop(0)}):
                with self.assertRaisesRegex(RuntimeError, "embeddinggemma"):
                    CHECK["check_advisory"]("atlas", Path(directory), pins, {})
        self.assertFalse(calls)


if __name__ == "__main__":
    unittest.main()
