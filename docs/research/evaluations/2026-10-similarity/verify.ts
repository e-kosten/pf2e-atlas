// Repository research-gate entrypoint. Python owns this dataset's offline replay.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const base = dirname(fileURLToPath(import.meta.url));
for (const args of [
  [join(base, "verify.py")],
  ["-m", "unittest", "discover", "-s", base, "-p", "test_*.py"],
  [join(base, "verify_results.py")],
]) {
  const result = spawnSync("python3", args, { stdio: "inherit" });
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `Offline research check failed: ${args.join(" ")}`);
}
