"""Regression checks for baseline routing and pin transitions."""
from pathlib import Path
from contextlib import closing
import hashlib
import gzip
import json
import runpy
import shutil
import sqlite3
import subprocess
import tempfile
import unittest
from unittest.mock import Mock, patch

HERE = Path(__file__).resolve().parent
STACK = runpy.run_path(str(HERE / "embedding-stack.py"))
PROBE = runpy.run_path(str(HERE / "probe-embedding-upgrade.py"))


class Routing(unittest.TestCase):
    def setUp(self):
        directory = self.enterContext(tempfile.TemporaryDirectory())
        self.root = Path(directory)
        for file in ("Cargo.lock", "Cargo.toml"):
            shutil.copyfile(HERE.parents[1] / file, self.root / file)
        self.pin = self.root / STACK["BASELINE_PIN"]
        self.pin.parent.mkdir(parents=True)
        self.pin.write_text("{}")
        self.git("init", "-q")
        self.git("add", ".")
        self.git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                 "commit", "-qm", "baseline")
        self.select = STACK["selected_checks"]
        self.enterContext(patch.dict(self.select.__globals__, ROOT=self.root))

    def git(self, *args):
        subprocess.run(["git", "-C", str(self.root), *args], check=True,
                       stdout=subprocess.DEVNULL)

    def test_unrelated_change_skips_probe(self):
        (self.root / "README.md").write_text("Docs")
        self.git("add", "README.md")
        self.assertEqual(self.select("HEAD", False), {"model": [], "workspace": []})

    def test_embedding_implementation_change_selects_linux(self):
        for name in ("atlas-search/src/similar.rs", "atlas-runtime/src/workspace.rs"):
            with self.subTest(name=name):
                path = self.root / "crates" / name
                path.parent.mkdir(parents=True)
                path.write_text("// changed")
                self.git("add", str(path))
                self.assertEqual(self.select("HEAD", False)["model"], ["ubuntu-24.04"])
                self.git("rm", "-f", str(path))

    def test_native_lockfile_change_runs_matrix_even_with_receipt_update(self):
        lockfile = self.root / "Cargo.lock"
        lockfile.write_text(lockfile.read_text().replace(
            'name = "ort"\nversion = "', 'name = "ort"\nversion = "fixture-',
        ))
        receipt = self.pin.with_name("embedding-stack.json")
        receipt.write_text("{}")
        self.git("add", ".")
        self.assertEqual(self.select("HEAD", False)["model"], STACK["FULL_MATRIX"])

    def test_removing_an_active_pin_fails(self):
        self.pin.unlink()
        with self.assertRaisesRegex(ValueError, "pin is missing"):
            self.select("HEAD", False)

    def test_transitive_tokenizer_update_runs_matrix(self):
        lockfile = self.root / "Cargo.lock"
        lockfile.write_text(lockfile.read_text().replace(
            'name = "unicode-normalization-alignments"\nversion = "',
            'name = "unicode-normalization-alignments"\nversion = "fixture-',
        ))
        self.assertEqual(self.select("HEAD", False)["model"], STACK["FULL_MATRIX"])

    def test_daachorse_only_update_requires_review_and_runs_matrix(self):
        before = STACK["snapshot"]()
        lockfile = self.root / "Cargo.lock"
        original = lockfile.read_text()
        updated = original.replace(
            'name = "daachorse"\nversion = "',
            'name = "daachorse"\nversion = "fixture-', 1,
        )
        self.assertNotEqual(original, updated, "Fixture must contain daachorse")
        lockfile.write_text(updated)
        self.assertEqual(self.select("HEAD", False)["model"], STACK["FULL_MATRIX"])
        after = STACK["snapshot"]()
        self.assertNotEqual(before, after, "Reviewed receipt must detect the update")
        self.assertEqual(
            [row for row in before["packages"] if row["name"] != "daachorse"],
            [row for row in after["packages"] if row["name"] != "daachorse"],
        )
        receipt = self.pin.with_name("embedding-stack.json")
        receipt.write_text(json.dumps(after))
        self.git("add", ".")
        self.assertEqual(self.select("HEAD", False)["model"], STACK["FULL_MATRIX"])

    def test_native_link_settings_run_matrix(self):
        config = self.root / ".cargo/config.toml"
        config.parent.mkdir()
        config.write_text('[target.x86_64-pc-windows-msvc]\nrustflags = ["-C", "opt-level=2"]\n')
        self.git("add", str(config))
        self.assertEqual(self.select("HEAD", False)["model"], STACK["FULL_MATRIX"])

    def test_first_pin_runs_matrix(self):
        self.git("rm", str(self.pin))
        self.git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                 "commit", "-qm", "no baseline yet")
        self.pin.parent.mkdir(parents=True, exist_ok=True)
        self.pin.write_text("{}")
        self.git("add", str(self.pin))
        self.assertEqual(self.select("HEAD", False)["model"], STACK["FULL_MATRIX"])

    def test_schema_cutover_runs_all_four_targets(self):
        cutoff = self.root / STACK["RELEASE_PREDECESSOR_PIN"]
        cutoff.write_text("{}")
        self.git("add", str(cutoff))
        self.assertEqual(self.select("HEAD", False)["model"], STACK["FULL_MATRIX"])


    def test_release_schema_changes_run_offline_checks_on_all_targets(self):
        for name in ("atlas-runtime/src/release_history.rs",
                     "atlas-index/src/release_schema.sql",
                     "atlas-runtime/src/workspace_settings.sql",
                     "atlas-local-state/src/release/encounters.sql",
                     "atlas-runtime/tests/fixtures/schema-42.sqlite.gz"):
            with self.subTest(name=name):
                path = self.root / "crates" / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b"fixture change")
                self.git("add", str(path))
                self.assertEqual(self.select("HEAD", False), {"model": [], "workspace": STACK["FULL_MATRIX"]})
                self.git("rm", "-f", str(path))

    def test_numerical_changes_keep_full_matrix_even_with_schema_changes(self):
        for name in ("atlas-embedding/src/inference.rs", "atlas-ingest/src/native_execution.rs",
                     "atlas-runtime/src/release_history.rs"):
            path = self.root / "crates" / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("// changed")
            self.git("add", str(path))
        self.assertEqual(self.select("HEAD", False), {
            "model": STACK["FULL_MATRIX"], "workspace": STACK["FULL_MATRIX"],
        })

    def test_unknown_base_and_manual_keep_both_checks(self):
        expected = {"model": STACK["FULL_MATRIX"], "workspace": STACK["FULL_MATRIX"]}
        self.assertEqual(self.select("", False), expected)
        self.assertEqual(self.select("HEAD", True), expected)
        self.assertEqual(self.select("no-such-commit", False), expected)
        self.assertEqual(self.select("", False, True), {"model": [], "workspace": []})

    def test_probe_depth_distinguishes_workflow_ingest_and_native_changes(self):
        depth = STACK["probe_depth"]
        for name, expected in (
            (".github/workflows/ci.yml", {"build": False, "near_limit": False}),
            ("crates/atlas-ingest/src/build.rs", {"build": True, "near_limit": False}),
            ("crates/atlas-embedding/src/inference.rs", {"build": False, "near_limit": True}),
            ("crates/atlas-ingest/src/native_execution.rs", {"build": True, "near_limit": True}),
            ("dist-workspace.toml", {"build": False, "near_limit": True}),
            (".github/workflows/release-build-check.yml", {"build": False, "near_limit": True}),
        ):
            with self.subTest(name=name):
                path = self.root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("// changed")
                self.git("add", name)
                self.assertEqual(depth("HEAD"), expected)
                self.git("rm", "-f", name)


class PredecessorSearch(unittest.TestCase):
    def test_requires_known_semantic_hit_without_fixing_unrelated_ranks(self):
        compare = PROBE["compare_predecessor_hit"]
        expected = [["known", [["passage", 0.75]]], ["other", [["elsewhere", 0.5]]]]
        compare(expected, [expected[0], ["unrelated", [["different", 0.4]]]], "bge-base")
        for actual in (
            list(reversed(expected)),
            [["other", [["elsewhere", 0.75]]]],
            [["known", [["wrong passage", 0.75]]]],
            [["known", [["passage", 0.6]]]],
        ):
            with self.subTest(actual=actual):
                with self.assertRaisesRegex(RuntimeError, "witness changed"):
                    compare(expected, actual, "bge-base")


class FrozenNumerical(unittest.TestCase):
    def setUp(self):
        directory = self.enterContext(tempfile.TemporaryDirectory())
        self.path = Path(directory) / "samples.json.gz"
        self.profile = "bge-base"
        self.target = "aarch64-apple-darwin"
        self.policy = "test-policy"
        self.payload = {
            "format": 2, "producer": {"target": self.target},
            "profiles": {self.profile: {
                "contract": {"profile_id": self.profile, "execution_policy": self.policy},
                "samples": [{"kind": "identity"}, {"kind": "passage"}],
            }},
        }

    def load(self, *, target=None, digest=None):
        self.path.write_bytes(gzip.compress(json.dumps(self.payload).encode()))
        return PROBE["load_v2_numerical"](
            self.path, digest or PROBE["sha256"](self.path), target or self.target,
            {self.profile: self.policy},
        )

    def test_numerical_evidence_loads_without_a_workspace(self):
        self.assertEqual(self.load(), self.payload)

    def test_checksum_target_policy_and_sample_coverage_are_required(self):
        with self.assertRaisesRegex(RuntimeError, "checksum"):
            self.load(digest="0" * 64)
        with self.assertRaisesRegex(RuntimeError, "target"):
            self.load(target="x86_64-unknown-linux-gnu")
        data = self.payload["profiles"][self.profile]
        data["contract"]["execution_policy"] = "changed"
        with self.assertRaisesRegex(RuntimeError, "policy"):
            self.load()
        data["contract"]["execution_policy"] = self.policy
        data["samples"].pop()
        with self.assertRaisesRegex(RuntimeError, "incomplete"):
            self.load()

    def test_workspace_refresh_keeps_numeric_pin_and_producer(self):
        from copy import deepcopy

        pins = {self.profile: {"execution_policy": self.policy, "asset": "test-bundle"}}
        target = self.target
        row = {kind: {
            "tag": "fixtures/embedding-compatibility/v2",
            "asset": f"{kind}-{target}.{extension}", "commit": "a" * 40,
            "sha256": "a" * 64,
        } for kind, extension in (("workspace", "tar.gz"), ("numerical", "json.gz"))}
        previous = {target: {self.profile: row}}
        current = deepcopy(previous)
        current[target][self.profile]["workspace"]["sha256"] = "b" * 64
        current[target][self.profile]["workspace"]["commit"] = "b" * 40
        PROBE["validate_v2_receipts"](current, previous, {target}, pins, pins)
        current[target][self.profile]["numerical"]["sha256"] = "b" * 64
        with self.assertRaisesRegex(RuntimeError, "rotated without transition"):
            PROBE["validate_v2_receipts"](current, previous, {target}, pins, pins)
        with self.assertRaisesRegex(RuntimeError, "all release targets"):
            PROBE["validate_v2_receipts"](current, previous, {target, "missing"}, pins, pins)


class CompactDocumentSamples(unittest.TestCase):
    def test_equal_input_hash_with_changed_token_count_fails_before_reading(self):
        directory = self.enterContext(tempfile.TemporaryDirectory())
        path = Path(directory) / "fixture.sqlite"
        with closing(sqlite3.connect(path)) as db, db:
            db.executescript("""
                CREATE TABLE semantic_units(input_id, unit_kind, profile_id);
                CREATE TABLE embedding_inputs(input_id, input_hash);
                INSERT INTO semantic_units VALUES(1, 'identity', 'bge-base');
                INSERT INTO embedding_inputs VALUES(1, x'12');
            """)
        before = {"bge-base": {("{}", "12", 6): "old"}}
        after = {"bge-base": {("{}", "12", 7): "new"}}
        with self.assertRaisesRegex(RuntimeError, "token count changed"):
            PROBE["check_document_samples"](None, path, path, "bge-base", before, after, layout_changed=True)

    def test_same_policy_vector_drift_fails(self):
        directory = self.enterContext(tempfile.TemporaryDirectory())
        path = Path(directory) / "fixture.sqlite"
        with closing(sqlite3.connect(path)) as db, db:
            db.executescript("""
                CREATE TABLE semantic_units(input_id, unit_kind, profile_id);
                CREATE TABLE embedding_inputs(input_id, input_hash);
                INSERT INTO semantic_units VALUES(1, 'identity', 'bge-base');
                INSERT INTO embedding_inputs VALUES(1, x'12');
            """)
        receipt = {"bge-base": {("{}", "12", 6): "hash"}}
        vectors = [json.dumps([{"hash": "12", "tokens": 6, "values": [value]}])
                   for value in (0.1, 0.2)]
        with patch.object(subprocess, "check_output", side_effect=vectors):
            with self.assertRaisesRegex(RuntimeError, "vector drift"):
                PROBE["check_document_samples"](None, path, path, "bge-base", receipt, receipt, layout_changed=True)


class PinTransition(unittest.TestCase):
    def test_exact_schema_cutover_keeps_execution_ids_and_retires_old_fixture(self):
        old = dict.fromkeys(PROBE["PROFILES"], PROBE["SCHEMA_41_PIN"])
        current = dict.fromkeys(PROBE["PROFILES"], PROBE["SCHEMA_42_PIN"])
        checks = PROBE["pinned_checks"](current, old)
        self.assertEqual(len(checks), 1)
        self.assertEqual(next(iter(checks)), (
            PROBE["RELEASE_TAG"], PROBE["RELEASE_ASSET"], PROBE["RELEASE_DIGEST"],
        ))
        changed = dict(current, **{"bge-base": {**current["bge-base"], "execution_policy": "changed"}})
        with self.assertRaisesRegex(RuntimeError, "approved ID transition"):
            PROBE["pinned_checks"](changed, old)

    def test_packaged_pin_still_requires_reviewed_id_for_future_replacement(self):
        profile = "bge-base"
        old = PROBE["SCHEMA_42_PIN"]
        new = {**PROBE["SCHEMA_29_PIN"], "tag": "embedding-baseline-v2",
               "sha256": "b" * 64, "execution_policy": "next"}
        with self.assertRaisesRegex(RuntimeError, "approved ID transition"):
            PROBE["pinned_checks"]({profile: new}, {profile: old})
        checks = PROBE["pinned_checks"]({profile: new}, {profile: {**old, "transition_to": "next"}})
        self.assertEqual(len(checks), 2)

    def test_switch_requires_prior_transition_and_keeps_old_fixture_in_check(self):
        pin = {
            "tag": "embedding-baseline-v1", "asset": "embedding-compatibility-baseline.tar.gz",
            "sha256": "a" * 64, "execution_policy": "old",
        }
        new = {**pin, "tag": "embedding-baseline-v2", "sha256": "b" * 64, "execution_policy": "new"}
        current = {"bge-base": pin, "embeddinggemma": new}
        previous = dict.fromkeys(current, pin)
        with self.assertRaisesRegex(RuntimeError, "approved ID transition"):
            PROBE["pinned_checks"](current, previous)
        previous["embeddinggemma"] = {**pin, "transition_to": "new"}
        old_check, new_check = PROBE["pinned_checks"](current, previous).values()
        self.assertEqual(old_check, {"bge-base": ("old", None), "embeddinggemma": ("old", "new")})
        self.assertEqual(new_check, {"embeddinggemma": ("new", None)})

    def test_identical_bytes_can_move_to_a_new_tag_without_old_download(self):
        pin = {
            "tag": "embedding-baseline-v1", "asset": "embedding-compatibility-baseline.tar.gz",
            "sha256": "a" * 64, "execution_policy": "current",
        }
        restored = {**pin, "tag": "fixtures/embedding-compatibility/v2"}
        checks = PROBE["pinned_checks"]({"bge-base": restored}, {"bge-base": pin})
        self.assertEqual(checks, {
            (restored["tag"], pin["asset"], pin["sha256"]): {"bge-base": ("current", None)},
        })


class ProbeInputs(unittest.TestCase):
    def test_layout_upgrade_checks_frozen_queries_before_replacing_passages(self):
        root = Path(self.enterContext(tempfile.TemporaryDirectory()))
        original = {"execution_policy": "same", "input_recipe": "old"}
        manifest = {"profiles": {"bge-base": original}, "expected_queries": {
            "bge-base": {"question": [["original record", []]]}}}
        check = PROBE["check"]
        build = Mock(side_effect=[{}, AssertionError("Maintenance ran before the frozen query check")])
        with patch.dict(check.__globals__, build=build, published_vectors=Mock(return_value={}),
                        has_trait_seed_chunks=Mock(return_value=False),
                        migrate_predecessor=Mock(return_value={}),
                        profile_contracts=Mock(return_value={"bge-base": {
                            **original, "input_recipe": "new"}}),
                        queries=Mock(return_value={"question": [["different record", []]]})):
            with self.assertRaisesRegex(RuntimeError, "Ranked records changed"):
                check(Path("atlas"), Path("cache"), root, ("bge-base",), manifest, {})

    def test_standalone_single_profile_bundle_uses_manifest_profiles(self):
        output = Path(self.enterContext(tempfile.TemporaryDirectory())) / "out"
        argv = ["probe", "--bundle", "archive", "--sha256", "digest",
                "--candidate", "atlas", "--model-cache", "cache", "--output", str(output)]
        consumer = Mock()
        manifest = {"profiles": {"embeddinggemma": {}}}
        main = PROBE["main"]
        with patch("sys.argv", argv), patch.dict(main.__globals__, check=consumer,
                                                extract_bundle=Mock(return_value=manifest)):
            main()
        self.assertEqual(consumer.call_args.args[3], ("embeddinggemma",))

    def test_saved_state_ignores_hydrated_presentation_but_detects_item_changes(self):
        baseline = {
            "list": {"list_key": "key", "slug": "list", "name": "List", "tags": [],
                     "created_at": "created", "updated_at": "updated"},
            "items": [{"record_key": "record", "position": 1, "snapshot": {"title": "Title"},
                       "record": {"kind_label": "Old"}}],
        }
        project = PROBE["saved_list_state"]
        original = project(baseline)
        baseline["items"][0]["record"]["kind_label"] = "New"
        self.assertEqual(project(baseline), original)
        baseline["items"][0]["note"] = "Different"
        self.assertNotEqual(project(baseline), original)


class FrozenVectors(unittest.TestCase):
    def setUp(self):
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory()))
        self.contract = {"profile_id": "bge-base", "execution_policy": "old"}
        self.report = {"inferred_inputs": 0, "reused_inputs": 1}

    def registry(self, name, model_id=1, input_id=2, checksum=b"v" * 32):
        index = self.root / name
        with closing(sqlite3.connect(index)) as db, db:
            db.executescript("CREATE TABLE embedding_models(model_id INTEGER,identity_json TEXT);"
                             "CREATE TABLE embedding_inputs(input_id INTEGER,model_id INTEGER,"
                             "input_hash BLOB,token_count INTEGER,published_hash BLOB);")
            db.execute("INSERT INTO embedding_models VALUES(?,?)",
                       (model_id, json.dumps(self.contract, sort_keys=bool(model_id % 2))))
            db.execute("INSERT INTO embedding_inputs VALUES(?,?,?,?,?)",
                       (input_id, model_id, b"i" * 32, 4, checksum))
        return PROBE["published_vectors"](index)

    def test_maintenance_may_renumber_rows_without_changing_frozen_vectors(self):
        before = self.registry("before.sqlite")
        after = self.registry("after.sqlite", model_id=8, input_id=90)
        PROBE["require_preserved_vectors"]("bge-base", before, after, self.report)

    def test_trait_retirement_excludes_only_vectors_exclusive_to_trait_seeds(self):
        index = self.root / "old.sqlite"
        self.registry(index.name)
        with closing(sqlite3.connect(index)) as db, db:
            db.executescript("CREATE TABLE semantic_units(input_id INTEGER);"
                             "CREATE TABLE trait_seed_chunks(input_id INTEGER);"
                             "INSERT INTO semantic_units VALUES(2);"
                             "INSERT INTO trait_seed_chunks VALUES(2),(3);")
            db.executemany("INSERT INTO embedding_inputs VALUES(?,?,?,?,?)", [
                (input_id, 1, bytes([input_id]) * 32, 4, b"v" * 32) for input_id in (3, 4)
            ])

        self.assertTrue(PROBE["has_trait_seed_chunks"](index))
        published = PROBE["published_vectors"]
        all_vectors = published(index)["bge-base"]
        preserved = published(index, retiring_trait_seeds=True)["bge-base"]
        self.assertEqual(len(all_vectors), 3)
        self.assertEqual(preserved, {
            key: checksum for key, checksum in all_vectors.items() if key[1] != (b"\x03" * 32).hex()
        })

    def test_replaced_or_missing_vectors_fail_even_when_reuse_is_reported(self):
        before = self.registry("before.sqlite")
        changed = self.registry("after.sqlite", checksum=b"x" * 32)
        for after in (changed, {}):
            with self.subTest(after=bool(after)), self.assertRaisesRegex(RuntimeError, "frozen"):
                PROBE["require_preserved_vectors"]("bge-base", before, after, self.report)

    def test_inference_is_rejected_even_if_the_resulting_checksum_matches(self):
        registry = self.registry("index.sqlite")
        with self.assertRaisesRegex(RuntimeError, "did not reuse"):
            PROBE["require_preserved_vectors"](
                "bge-base", registry, registry, {"inferred_inputs": 1, "reused_inputs": 1})


class PolicyAdmission(unittest.TestCase):
    def setUp(self):
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory()))
        self.context = dict.fromkeys((
            "format_version", "snapshot_version", "source_contract", "query_projection_version",
            "query_catalog_version", "lookup_version", "lexical_selection_version", "fts_tokenizer",
            "content_interpretation_version", "content_selection_version", "localization_policy",
            "asset_policy", "relationship_policy"), "unchanged")

    def index(self, name, **changes):
        index = self.root / name
        encoded = json.dumps({**self.context, **changes})
        with closing(sqlite3.connect(index)) as db, db:
            db.execute("CREATE TABLE artifact_context(singleton INTEGER,context_hash TEXT,context_json TEXT)")
            db.execute("INSERT INTO artifact_context VALUES(1,?,?)",
                       (hashlib.sha256(encoded.encode()).hexdigest(), encoded))
        return index

    def test_content_and_catalog_refresh_explains_policy_admission_failure(self):
        old = self.index("old.sqlite")
        new = self.index("new.sqlite", content_interpretation_version="new", query_catalog_version=9)
        self.assertTrue(PROBE["policy_refresh_required"](old, new))

    def test_unchanged_policy_or_changed_source_contract_does_not_explain_failure(self):
        old = self.index("old.sqlite")
        for name, changes in (("same", {}), ("source", {
                "source_contract": "different", "content_interpretation_version": "new"})):
            with self.subTest(name=name):
                self.assertFalse(PROBE["policy_refresh_required"](
                    old, self.index(name + ".sqlite", **changes)))

    def test_tampered_policy_metadata_is_not_accepted(self):
        old, new = self.index("old.sqlite"), self.index("new.sqlite")
        with closing(sqlite3.connect(old)) as db, db:
            db.execute("UPDATE artifact_context SET context_json='{}'")
        with self.assertRaisesRegex(RuntimeError, "checksum mismatch"):
            PROBE["policy_refresh_required"](old, new)

    def test_transition_requires_the_exact_admission_code_and_reason(self):
        old = self.index("old.sqlite")
        new = self.index("new.sqlite", content_interpretation_version="new")
        accepted = PROBE["expected_transition_rejection"]
        for code, message in (
                ("vector_readiness_required", "stored query model differs from the executable contract"),
                ("artifact_incompatible", "projection/lookup/content/FTS policy identity")):
            self.assertTrue(accepted({"code": code, "message": message}, old, new))
            self.assertFalse(accepted({"code": "unexpected", "message": message}, old, new))
        self.assertFalse(accepted({"code": "artifact_incompatible", "message": "source/artifact contract identity"}, old, new))
        self.assertFalse(accepted({"code": "artifact_incompatible", "message": "projection/lookup/content/FTS policy identity"}, old, old))


if __name__ == "__main__":
    unittest.main()
