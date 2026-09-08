/* global URL, console, process */

import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const vitest = path.join(packageRoot, "node_modules", ".bin", "vitest");
const shardCount = 4;

const inventoryResult = spawnSync(vitest, ["list", "--filesOnly"], {
  cwd: packageRoot,
  encoding: "utf8",
  stdio: ["inherit", "pipe", "inherit"],
});
if (inventoryResult.status !== 0) {
  process.exit(inventoryResult.status ?? 1);
}

const inventory = inventoryResult.stdout
  .split("\n")
  .map((file) => file.trim())
  .filter(Boolean)
  .map((file) => path.resolve(packageRoot, file));
if (inventory.length === 0) {
  console.error("Vitest did not discover any test files");
  process.exit(1);
}
if (new Set(inventory).size !== inventory.length) {
  console.error("Vitest returned duplicate test files");
  process.exit(1);
}

const actualShardCount = Math.min(shardCount, inventory.length);
const reportRoot = mkdtempSync(path.join(tmpdir(), "atlas-vitest-shards-"));
const executedFiles = [];
let executedTests = 0;
try {
  console.log(
    `Running ${inventory.length} Vitest files once across ${actualShardCount} isolated shards`,
  );
  for (let index = 1; index <= actualShardCount; index += 1) {
    const reportPath = path.join(reportRoot, `shard-${index}.json`);
    console.log(`Vitest shard ${index}/${actualShardCount}`);
    const result = spawnSync(
      vitest,
      [
        "run",
        "--maxWorkers=1",
        `--shard=${index}/${actualShardCount}`,
        "--reporter=default",
        "--reporter=json",
        `--outputFile.json=${reportPath}`,
      ],
      { cwd: packageRoot, stdio: "inherit" },
    );
    if (result.status !== 0) {
      process.exitCode = result.status ?? 1;
      break;
    }
    const report = JSON.parse(readFileSync(reportPath, "utf8"));
    executedTests += report.numTotalTests;
    executedFiles.push(
      ...report.testResults.map((testResult) => path.resolve(testResult.name)),
    );
  }

  if (process.exitCode === undefined) {
    const executed = new Set(executedFiles);
    const expected = new Set(inventory);
    const missing = inventory.filter((file) => !executed.has(file));
    const unexpected = executedFiles.filter((file) => !expected.has(file));
    if (
      executedFiles.length !== inventory.length ||
      executed.size !== inventory.length ||
      missing.length > 0 ||
      unexpected.length > 0
    ) {
      console.error("Vitest shards did not execute every discovered file exactly once");
      console.error(
        JSON.stringify(
          {
            discovered: inventory.length,
            executed: executedFiles.length,
            missing,
            unexpected,
          },
          null,
          2,
        ),
      );
      process.exitCode = 1;
    } else {
      console.log(
        `Verified ${executedFiles.length} Vitest files and ${executedTests} tests exactly once`,
      );
    }
  }
} finally {
  rmSync(reportRoot, { recursive: true, force: true });
}
