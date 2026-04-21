import { performance } from "node:perf_hooks";

import { loadPf2eApplicationRuntime } from "../src/app/runtime.js";
import { createPf2eTerminalAppServices, type Pf2eTerminalAppServices } from "../src/tui/app-services.js";

type TimingSummary = {
  label: string;
  coldMs: number;
  warmMs: number;
};

function parseArgs(argv: string[]): { forwardedArgv: string[]; samples: number } {
  const forwardedArgv: string[] = [];
  let samples = 3;

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--samples") {
      const next = argv[index + 1];
      if (!next) {
        throw new Error("Expected a positive integer after --samples.");
      }
      const parsed = Number.parseInt(next, 10);
      if (!Number.isInteger(parsed) || parsed < 1) {
        throw new Error(`Invalid --samples value "${next}".`);
      }
      samples = parsed;
      index += 1;
      continue;
    }
    forwardedArgv.push(arg);
  }

  return { forwardedArgv, samples };
}

function formatMs(durationMs: number): string {
  return `${durationMs.toFixed(2)} ms`;
}

function median(values: number[]): number {
  const sorted = [...values].sort((left, right) => left - right);
  return sorted[Math.floor(sorted.length / 2)] ?? 0;
}

async function measureCall(task: () => PromiseLike<void> | void): Promise<number> {
  const start = performance.now();
  await task();
  return performance.now() - start;
}

async function sampleTimings(
  argv: string[],
  samples: number,
  label: string,
  task: (
    runtime: Awaited<ReturnType<typeof loadPf2eApplicationRuntime>>,
    services: Pf2eTerminalAppServices,
  ) => PromiseLike<void> | void,
): Promise<TimingSummary> {
  const coldRuns: number[] = [];
  const warmRuns: number[] = [];

  for (let index = 0; index < samples; index += 1) {
    const runtime = await loadPf2eApplicationRuntime(argv);
    const services = createPf2eTerminalAppServices(runtime);
    try {
      coldRuns.push(await measureCall(() => task(runtime, services)));
      warmRuns.push(await measureCall(() => task(runtime, services)));
    } finally {
      services.close();
    }
  }

  return {
    label,
    coldMs: median(coldRuns),
    warmMs: median(warmRuns),
  };
}

function printTable(results: TimingSummary[]): void {
  const labelWidth = Math.max(...results.map((result) => result.label.length), "Path".length);
  const coldWidth = Math.max(...results.map((result) => formatMs(result.coldMs).length), "Cold".length);
  const warmWidth = Math.max(...results.map((result) => formatMs(result.warmMs).length), "Warm".length);
  const speedupWidth = "Warmup".length;

  const divider = `${"-".repeat(labelWidth)}  ${"-".repeat(coldWidth)}  ${"-".repeat(warmWidth)}  ${"-".repeat(
    speedupWidth,
  )}`;
  console.log(
    `${"Path".padEnd(labelWidth)}  ${"Cold".padStart(coldWidth)}  ${"Warm".padStart(warmWidth)}  ${"Warmup".padStart(speedupWidth)}`,
  );
  console.log(divider);
  for (const result of results) {
    const warmup = result.coldMs > 0 ? `${(result.coldMs / Math.max(result.warmMs, 0.01)).toFixed(2)}x` : "n/a";
    console.log(
      `${result.label.padEnd(labelWidth)}  ${formatMs(result.coldMs).padStart(coldWidth)}  ${formatMs(
        result.warmMs,
      ).padStart(warmWidth)}  ${warmup.padStart(speedupWidth)}`,
    );
  }
}

function printColdRatios(results: TimingSummary[]): void {
  const fullVocabulary = results.find((result) => result.label === "Full search vocabulary");
  if (!fullVocabulary || fullVocabulary.coldMs <= 0) {
    return;
  }

  console.log("");
  console.log("Cold-path ratios vs full search vocabulary:");
  for (const result of results) {
    if (result === fullVocabulary) {
      continue;
    }
    console.log(`- ${result.label}: ${(result.coldMs / fullVocabulary.coldMs).toFixed(2)}x`);
  }
}

async function main(): Promise<void> {
  const { forwardedArgv, samples } = parseArgs(process.argv.slice(2));
  const results: TimingSummary[] = [];
  results.push(
    await sampleTimings(forwardedArgv, samples, "Category picker options", (_runtime, services) =>
      services.user.search.getCategoryOptions(),
    ),
  );
  results.push(
    await sampleTimings(forwardedArgv, samples, "Search-semantics bootstrap summary", (runtime) =>
      runtime.dataService.getSearchSemanticsBootstrapSummary(),
    ),
  );
  results.push(
    await sampleTimings(forwardedArgv, samples, "Full search vocabulary", (runtime) =>
      runtime.dataService.getSearchVocabulary(),
    ),
  );
  results.push(
    await sampleTimings(forwardedArgv, samples, "Ontology Browser domain load", (_runtime, services) =>
      services.user.ontology.loadSearchSemanticsDomain(),
    ),
  );

  console.log("PF2E search-semantics startup timings");
  console.log(`Samples per path: ${samples}`);
  console.log("");
  printTable(results);
  printColdRatios(results);
}

void main().catch((error) => {
  console.error((error as Error).message);
  process.exitCode = 1;
});
