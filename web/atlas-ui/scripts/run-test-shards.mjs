/* global URL, console, process */

import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const vitest = path.join(packageRoot, "node_modules", ".bin", "vitest");
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

const reportRoot = mkdtempSync(path.join(tmpdir(), "atlas-vitest-shards-"));
const executedFiles = [];
let executedTests = 0;
try {
  console.log(`Running ${inventory.length} Vitest files in isolated processes`);
  for (const [index, testFile] of inventory.entries()) {
    const reportPath = path.join(reportRoot, `shard-${index + 1}.json`);
    console.log(
      `Vitest file ${index + 1}/${inventory.length}: ${path.relative(packageRoot, testFile)}`,
    );
    const result = spawnSync(
      vitest,
      [
        "run",
        "--maxWorkers=1",
        "--reporter=default",
        "--reporter=json",
        `--outputFile.json=${reportPath}`,
        testFile,
      ],
      { cwd: packageRoot, stdio: "inherit" },
    );
    if (result.status !== 0) {
      process.exitCode = result.status ?? 1;
      break;
    }
    const report = JSON.parse(readFileSync(reportPath, "utf8"));
    const reportedFiles = report.testResults.map((testResult) =>
      path.resolve(testResult.name),
    );
    if (reportedFiles.length !== 1 || reportedFiles[0] !== testFile) {
      console.error(
        `Vitest file filter did not isolate ${path.relative(packageRoot, testFile)}`,
      );
      process.exitCode = 1;
      break;
    }
    executedTests += report.numTotalTests;
    executedFiles.push(...reportedFiles);
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
