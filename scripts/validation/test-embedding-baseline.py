"""Regression checks for baseline routing and pin transitions."""
from pathlib import Path
import runpy
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch

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
        path = self.root / "crates/atlas-embedding/src/inference.rs"
        path.parent.mkdir(parents=True)
        path.write_text("// changed")
        self.git("add", str(path))
        self.assertEqual(self.select("HEAD", False), ["ubuntu-24.04"])

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


if __name__ == "__main__":
    unittest.main()
