"""Safety regressions use real files/SQLite; model execution is a separate smoke test."""

import argparse
import contextlib
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import baseline
import snapshots


def fixture_database(path):
    with contextlib.closing(sqlite3.connect(path)) as connection, connection:
        connection.executescript("""
            PRAGMA user_version=6;
            CREATE TABLE workspace(id INTEGER PRIMARY KEY, workspace_key TEXT, reference_revision INTEGER, maintenance TEXT);
            INSERT INTO workspace VALUES(1,'fixture',3,NULL);
            CREATE TABLE artifact_context(singleton INTEGER PRIMARY KEY, context_json TEXT);
            INSERT INTO artifact_context VALUES(1,'{"source_revision":"fixture","source_contract":"fixture"}');
            CREATE TABLE saved_lists(id INTEGER PRIMARY KEY, title TEXT);
            INSERT INTO saved_lists VALUES(1,'Preserve me');
        """)


class BaselineTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.checkout = self.root / "checkout with spaces"
        self.checkout.mkdir()
        self.store = self.root / "store"
        (self.root / "source").mkdir()
        (self.root / "models").mkdir()
        self.database = self.root / "source.sqlite"
        fixture_database(self.database)
        self.binary = self.root / "atlas"
        self.binary.write_text("#!/bin/sh\nprintf 'atlas fixture\\n'\n")
        self.binary.chmod(0o755)
        self.args = argparse.Namespace(binary=self.binary, index=self.database, build_commit="HEAD", build_profile="dev",
                                       toolchain="fixture", source=None, embedding_cache_path=None, baseline=None, port=4727)

    def checks(self, binary, index, assets, cwd):
        return {"readiness": {"ready": True, "paths": {"source": str(self.root / "source"), "embedding_cache": str(self.root / "models")}},
                "deep_validation": {"roots": 1}, "searches": [{"profile": "fixture", "result_count": 1}]}

    def publish(self):
        with patch.object(baseline, "git", return_value="a" * 40), patch.object(baseline.atlas_checks, "validate", side_effect=self.checks):
            return baseline.publish(self.args, self.checkout, self.store)

    def test_live_wal_snapshot_includes_committed_rows(self):
        with contextlib.closing(sqlite3.connect(self.database)) as writer:
            writer.execute("PRAGMA journal_mode=WAL")
            writer.execute("INSERT INTO saved_lists VALUES(2,'Still in WAL')")
            writer.commit()
            self.assertTrue(Path(str(self.database) + "-wal").is_file())
            destination = self.root / "snapshot.sqlite"
            snapshots.snapshot(self.database, destination)
            with contextlib.closing(sqlite3.connect(destination)) as copy:
                self.assertEqual(copy.execute("SELECT COUNT(*) FROM saved_lists").fetchone()[0], 2)
                self.assertEqual(copy.execute("PRAGMA journal_mode").fetchone()[0], "delete")
            self.assertFalse(Path(str(destination) + "-wal").exists())

    def test_active_reference_update_blocks_snapshot(self):
        destination = self.root / "snapshot.sqlite"
        with snapshots.lock(str(self.database) + ".update.lock"):
            with self.assertRaisesRegex(RuntimeError, "Busy"):
                snapshots.snapshot(self.database, destination)
        self.assertFalse(destination.exists())

    def test_failed_publication_preserves_previous_default(self):
        self.publish()
        pointer = (self.store / "current.json").read_bytes()
        with patch.object(baseline, "git", return_value="b" * 40), patch.object(baseline.atlas_checks, "validate", side_effect=RuntimeError("bad model")):
            with self.assertRaisesRegex(RuntimeError, "bad model"):
                baseline.publish(self.args, self.checkout, self.store)
        self.assertEqual((self.store / "current.json").read_bytes(), pointer)
        self.assertEqual(len(list((self.store / "versions").iterdir())), 1)
        self.assertEqual(list(self.store.glob(".publish-*")), [])

    def test_active_publisher_blocks_second_publication(self):
        self.publish()
        pointer = (self.store / "current.json").read_bytes()
        with snapshots.lock(self.store / "publish.lock"):
            with self.assertRaisesRegex(RuntimeError, "Busy"):
                self.publish()
        self.assertEqual((self.store / "current.json").read_bytes(), pointer)

    def test_publication_rejects_assets_in_disposable_worktrees(self):
        def git_fixture(checkout, *args):
            if args[0] == "worktree":
                return "worktree " + str(self.checkout)
            if "--git-common-dir" in args:
                return str(self.root / "primary/.git")
            return "a" * 40

        def checks(*args):
            report = self.checks(*args)
            report["readiness"]["paths"]["source"] = str(self.checkout)
            return report

        with patch.object(baseline, "git", side_effect=git_fixture), patch.object(baseline.atlas_checks, "validate", side_effect=checks):
            with self.assertRaisesRegex(RuntimeError, "outside disposable worktrees"):
                baseline.publish(self.args, self.checkout, self.store)
        self.assertFalse((self.store / "current.json").exists())

    def test_failed_pointer_update_keeps_previous_default(self):
        self.publish()
        pointer = (self.store / "current.json").read_bytes()
        with patch.object(baseline, "replace_json", side_effect=OSError("pointer failure")):
            with self.assertRaisesRegex(OSError, "pointer failure"):
                self.publish()
        self.assertEqual((self.store / "current.json").read_bytes(), pointer)
        self.assertEqual(len(list((self.store / "versions").iterdir())), 2)
        baseline.load_baseline(self.store, verify_data=True)

    def test_publish_copies_saved_state_and_records_actual_metadata(self):
        original = snapshots.sha256(self.database)
        self.publish()
        directory, manifest = baseline.load_baseline(self.store, verify_data=True)
        self.assertEqual(manifest["database"]["schema_version"], 6)
        self.assertEqual(manifest["database"]["workspace_key"], "fixture")
        self.assertEqual(manifest["build"]["commit"], "a" * 40)
        self.assertEqual(snapshots.sha256(self.database), original)
        self.assertEqual((directory / "workspace.sqlite").stat().st_mode & 0o222, 0)

    def test_init_is_independent_and_preserves_existing_changes(self):
        self.publish()
        directory, manifest = baseline.load_baseline(self.store)
        result = baseline.initialize(self.args, self.checkout, self.store)
        self.assertTrue(result["initialized"])
        index = self.checkout / ".atlas/workspace.sqlite"
        self.assertNotEqual(index.stat().st_ino, (directory / "workspace.sqlite").stat().st_ino)
        with contextlib.closing(sqlite3.connect(index)) as connection, connection:
            connection.execute("UPDATE saved_lists SET title='My branch edit'")
        before = index.read_bytes()
        # New default never resets an existing working copy or its seed receipt.
        self.publish()
        self.assertFalse(baseline.initialize(self.args, self.checkout, self.store)["initialized"])
        self.assertEqual(index.read_bytes(), before)
        _, pinned, _ = baseline.workspace_baseline(self.checkout, self.store)
        self.assertEqual(pinned, directory)
        self.assertEqual(snapshots.sha256(directory / "workspace.sqlite"), manifest["database"]["sha256"])

    def test_existing_unmanaged_workspace_is_never_replaced(self):
        index = self.checkout / ".atlas/workspace.sqlite"
        index.parent.mkdir()
        index.write_bytes(b"Existing data, even if not a readable Atlas database")
        result = baseline.initialize(self.args, self.checkout, self.store)
        self.assertFalse(result["initialized"])
        self.assertEqual(index.read_bytes(), b"Existing data, even if not a readable Atlas database")
        self.assertFalse((index.parent / "dev-workspace.json").exists())

    def test_failed_copy_does_not_publish_workspace_or_receipt(self):
        self.publish()
        with patch.object(baseline, "clone_file", side_effect=OSError("disk full")):
            with self.assertRaisesRegex(OSError, "disk full"):
                baseline.initialize(self.args, self.checkout, self.store)
        self.assertFalse((self.checkout / ".atlas/workspace.sqlite").exists())
        self.assertFalse((self.checkout / ".atlas/dev-workspace.json").exists())

    def test_receipt_without_database_is_preserved_for_recovery(self):
        directory = self.checkout / ".atlas"
        directory.mkdir()
        (directory / "dev-workspace.json").write_text('{"baseline":"existing"}')
        result = baseline.initialize(self.args, self.checkout, self.store)
        self.assertFalse(result["initialized"])
        self.assertFalse((directory / "workspace.sqlite").exists())

    def test_orphaned_wal_refuses_initialization(self):
        index = self.checkout / ".atlas/workspace.sqlite"
        index.parent.mkdir()
        Path(str(index) + "-wal").write_bytes(b"recovery data")
        with self.assertRaisesRegex(RuntimeError, "sidecars"):
            baseline.initialize(self.args, self.checkout, self.store)
        self.assertFalse(index.exists())

    def test_corrupted_seed_refuses_initialization(self):
        result = self.publish()
        seed = Path(result["directory"]) / "workspace.sqlite"
        seed.chmod(0o600)
        with open(seed, "ab") as stream:
            stream.write(b"tampered")
        with self.assertRaisesRegex(RuntimeError, "database hash mismatch"):
            baseline.initialize(self.args, self.checkout, self.store)
        self.assertFalse((self.checkout / ".atlas/workspace.sqlite").exists())

    def test_changed_executable_is_detected(self):
        result = self.publish()
        binary = Path(result["directory"]) / "atlas"
        binary.chmod(0o755)
        binary.write_text("different executable")
        with self.assertRaisesRegex(RuntimeError, "executable hash mismatch"):
            baseline.load_baseline(self.store)

    def test_untrusted_baseline_name_cannot_escape_versions(self):
        for value in ("..", "../elsewhere", "/tmp/elsewhere"):
            with self.assertRaisesRegex(RuntimeError, "Invalid baseline"):
                baseline.load_baseline(self.store, value)

    def test_baseline_selection_never_silently_uses_stale_debug_binary(self):
        result = self.publish()
        debug = self.checkout / "target/debug/atlas"
        debug.parent.mkdir(parents=True)
        debug.write_text("stale")
        selected, _ = baseline.candidate_binary(Path(result["directory"]))
        self.assertEqual(selected, Path(result["directory"]) / "atlas")

    def test_shared_database_symlink_is_rejected(self):
        (self.checkout / ".atlas").symlink_to(self.root, target_is_directory=True)
        with self.assertRaisesRegex(RuntimeError, "private directory"):
            baseline.initialize(self.args, self.checkout, self.store)

    def test_existing_database_symlink_is_preserved(self):
        (self.checkout / ".atlas").mkdir()
        (self.checkout / ".atlas/workspace.sqlite").symlink_to(self.database)
        before = snapshots.sha256(self.database)
        self.assertFalse(baseline.initialize(self.args, self.checkout, self.store)["initialized"])
        self.assertEqual(snapshots.sha256(self.database), before)

    def test_status_reports_candidate_failure_without_migrating(self):
        self.publish()
        baseline.initialize(self.args, self.checkout, self.store)
        index = self.checkout / ".atlas/workspace.sqlite"
        before = index.read_bytes()
        failure = {"exit_code": 1, "response": {"status": "error", "message": "unsupported contract"}}
        with patch.object(baseline, "git", return_value="b" * 40), patch.object(baseline.atlas_checks, "invoke", return_value=failure) as invoke:
            report = baseline.status(self.args, self.checkout, self.store)
        self.assertEqual(report["compatibility"], failure)
        self.assertEqual(index.read_bytes(), before)
        for call in invoke.call_args_list:
            command = call.args[1]
            self.assertTrue(command[:2] == ["index", "check"] or (command[0] == "setup" and "--check" in command))

    def test_main_and_linked_worktree_find_same_store(self):
        subprocess.run(["git", "init", "--quiet", str(self.checkout)], check=True)
        for key, value in (("user.name", "Fixture"), ("user.email", "fixture@example.invalid"), ("commit.gpgsign", "false"), ("core.hooksPath", "/dev/null")):
            subprocess.run(["git", "-C", str(self.checkout), "config", key, value], check=True)
        subprocess.run(["git", "-C", str(self.checkout), "commit", "--allow-empty", "-qm", "test: fixture"], check=True)
        worktree = self.root / "parallel"
        subprocess.run(["git", "-C", str(self.checkout), "worktree", "add", "--quiet", "-b", "parallel", str(worktree)], check=True)
        self.assertEqual(baseline.locations(self.checkout)[1], baseline.locations(worktree)[1])


class AtlasCheckTests(unittest.TestCase):
    def test_not_ready_does_not_proceed_to_validation_or_search(self):
        report = {"exit_code": 0, "response": {"status": "ok", "data": {"ready": False}}}
        with patch.object(baseline.atlas_checks, "invoke", return_value=report) as invoke:
            with self.assertRaisesRegex(RuntimeError, "needs ready"):
                baseline.atlas_checks.validate("binary", "index", {}, "/tmp")
        self.assertEqual(invoke.call_count, 1)

    def test_failing_deep_validation_rejects_publication(self):
        ready = {"exit_code": 0, "response": {"status": "ok", "data": {
            "ready": True, "search_profiles": [{"profile_id": "bge-base", "ready": True}],
            "paths": {"embedding_cache": "/models"}}}}
        error = {"exit_code": 1, "response": {"status": "error", "message": "bad snapshot"}}
        with patch.object(baseline.atlas_checks, "invoke", side_effect=[ready, error]):
            with self.assertRaisesRegex(RuntimeError, "bad snapshot"):
                baseline.atlas_checks.validate("binary", "index", {}, "/tmp")

    def test_every_profile_requires_a_real_query_result(self):
        profiles = [{"profile_id": "bge-base", "ready": True}, {"profile_id": "embeddinggemma", "ready": True}]
        ready = {"exit_code": 0, "response": {"status": "ok", "data": {
            "ready": True, "search_profiles": profiles, "paths": {"embedding_cache": "/models"}}}}
        deep = {"exit_code": 0, "response": {"status": "ok", "data": {"roots": 1}}}
        hit = {"exit_code": 0, "response": {"status": "ok", "data": {"results": [{"record_key": "item:one"}]}}}
        empty = {"exit_code": 0, "response": {"status": "ok", "data": {"results": []}}}
        with patch.object(baseline.atlas_checks, "invoke", side_effect=[ready, deep, hit, empty]):
            with self.assertRaisesRegex(RuntimeError, "embeddinggemma"):
                baseline.atlas_checks.validate("binary", "index", {}, "/tmp")


if __name__ == "__main__":
    unittest.main()
