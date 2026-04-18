#!/usr/bin/env node

import { spawn } from "node:child_process";

type RefreshStep = {
  script: "refresh-data" | "refresh-embeddings" | "refresh-index";
  label: string;
};

const STEPS: RefreshStep[] = [
  { script: "refresh-data", label: "PF2E data checkout" },
  { script: "refresh-embeddings", label: "embedding assets" },
  { script: "refresh-index", label: "SQLite index" },
];

function formatDuration(durationMs: number): string {
  const totalSeconds = Math.max(0, Math.round(durationMs / 1000));
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;

  if (minutes === 0) {
    return `${seconds}s`;
  }

  return `${minutes}m ${seconds}s`;
}

function npmCommand(): string {
  return process.platform === "win32" ? "npm.cmd" : "npm";
}

async function runStep(step: RefreshStep, index: number, total: number): Promise<void> {
  const startTime = Date.now();
  console.error(`[${index}/${total}] Starting ${step.label} with \`npm run ${step.script}\`.`);

  await new Promise<void>((resolve, reject) => {
    const child = spawn(npmCommand(), ["run", step.script], {
      stdio: "inherit",
      env: process.env,
    });

    child.on("error", reject);
    child.on("exit", (code, signal) => {
      if (code === 0) {
        resolve();
        return;
      }

      reject(
        new Error(
          signal
            ? `${step.label} exited due to signal ${signal}.`
            : `${step.label} exited with code ${code ?? "unknown"}.`,
        ),
      );
    });
  });

  console.error(`[${index}/${total}] Finished ${step.label} in ${formatDuration(Date.now() - startTime)}.`);
}

async function main(): Promise<void> {
  const startTime = Date.now();
  console.error(
    "Refreshing external assets. The first run can take several minutes while model assets download and the index is rebuilt.",
  );

  for (const [index, step] of STEPS.entries()) {
    await runStep(step, index + 1, STEPS.length);
  }

  console.error(`Refresh complete in ${formatDuration(Date.now() - startTime)}.`);
}

main().catch((error) => {
  console.error(`External refresh failed: ${(error as Error).message}`);
  process.exit(1);
});
