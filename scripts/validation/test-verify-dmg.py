"""Exercise macOS image verification errors without mounting a disk image."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


VERIFY = Path(__file__).resolve().parents[2] / "desktop/verify-dmg.sh"
BUSY = "hdiutil: verify failed - Resource temporarily unavailable"
INVALID = "hdiutil: verify failed - image data corrupted"


class VerifyDmgTest(unittest.TestCase):
    def check_verification(self, diagnostics, status, expected_calls):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "responses").write_text("\n".join(diagnostics) + "\n")
            verifier = root / "hdiutil"
            verifier.write_text("""#!/bin/sh
set -eu
[ "$1" = verify ] && [ "$2" = 'test image.dmg' ] || exit 2
echo verify >> calls
response=$(sed -n "$(wc -l < calls | tr -d ' ')p" responses)
[ "$response" = ok ] && exit 0
echo "$response" >&2
exit 73
""")
            verifier.chmod(0o755)
            result = subprocess.run(
                ["sh", VERIFY, "test image.dmg"], cwd=root,
                env={**os.environ, "PATH": tmp + os.pathsep + os.environ["PATH"]},
                capture_output=True, text=True,
            )
            self.assertEqual(result.returncode, status, result.stderr)
            self.assertEqual((root / "calls").read_text().splitlines(), ["verify"] * expected_calls)
            self.assertEqual(result.stderr.splitlines(), [s for s in diagnostics if s != "ok"])

    def test_success(self):
        self.check_verification(["ok"], 0, 1)

    def test_busy_then_success(self):
        self.check_verification([BUSY, "ok"], 0, 2)

    def test_persistent_contention(self):
        self.check_verification([BUSY] * 3, 73, 3)

    def test_invalid_image(self):
        self.check_verification([INVALID], 73, 1)


if __name__ == "__main__":
    unittest.main()
