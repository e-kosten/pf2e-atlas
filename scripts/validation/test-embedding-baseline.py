"""Regression checks for baseline routing and pin transitions."""
from pathlib import Path
import runpy
import shutil
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
        self.select = STACK["selected_runners"]
        self.enterContext(patch.dict(self.select.__globals__, ROOT=self.root))

    def git(self, *args):
        subprocess.run(["git", "-C", str(self.root), *args], check=True,
                       stdout=subprocess.DEVNULL)

    def test_unrelated_change_skips_probe(self):
        (self.root / "README.md").write_text("Docs")
        self.git("add", "README.md")
        self.assertEqual(self.select("HEAD", False), [])

    def test_embedding_implementation_change_selects_linux(self):
        for name in ("atlas-embedding/src/inference.rs", "atlas-ingest/src/native_execution.rs",
                     "atlas-search/src/text.rs", "atlas-runtime/src/workspace.rs"):
            with self.subTest(name=name):
                path = self.root / "crates" / name
                path.parent.mkdir(parents=True)
                path.write_text("// changed")
                self.git("add", str(path))
                self.assertEqual(self.select("HEAD", False), ["ubuntu-24.04"])
                self.git("rm", "-f", str(path))

    def test_native_lockfile_change_runs_matrix_even_with_receipt_update(self):
        lockfile = self.root / "Cargo.lock"
        lockfile.write_text(lockfile.read_text().replace(
            'name = "ort"\nversion = "', 'name = "ort"\nversion = "fixture-',
        ))
        receipt = self.pin.with_name("embedding-stack.json")
        receipt.write_text("{}")
        self.git("add", ".")
        self.assertEqual(self.select("HEAD", False), STACK["FULL_MATRIX"])

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
        self.assertEqual(self.select("HEAD", False), STACK["FULL_MATRIX"])

    def test_native_link_settings_run_matrix(self):
        config = self.root / ".cargo/config.toml"
        config.parent.mkdir()
        config.write_text('[target.x86_64-pc-windows-msvc]\nrustflags = ["-C", "opt-level=2"]\n')
        self.git("add", str(config))
        self.assertEqual(self.select("HEAD", False), STACK["FULL_MATRIX"])

    def test_first_pin_runs_matrix(self):
        self.git("rm", str(self.pin))
        self.git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                 "commit", "-qm", "no baseline yet")
        self.pin.parent.mkdir(parents=True, exist_ok=True)
        self.pin.write_text("{}")
        self.git("add", str(self.pin))
        self.assertEqual(self.select("HEAD", False), STACK["FULL_MATRIX"])


class PinTransition(unittest.TestCase):
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
        restored = {**pin, "tag": "embedding-baseline-v2"}
        checks = PROBE["pinned_checks"]({"bge-base": restored}, {"bge-base": pin})
        self.assertEqual(checks, {
            (restored["tag"], pin["asset"], pin["sha256"]): {"bge-base": ("current", None)},
        })


class ProbeInputs(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
