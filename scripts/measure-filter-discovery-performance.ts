import { performance } from "node:perf_hooks";

import { loadPf2eApplicationRuntime } from "../src/app/runtime.js";
import { createPf2eTerminalAppServices, type Pf2eTerminalAppServices } from "../src/tui/app-services.js";
import { buildScopeFilter, type SearchRequest } from "../src/domain/search-request-types.js";

type TimingSummary = {
  label: string;
  medianMs: number;
  minMs: number;
  maxMs: number;
};

function parseArgs(argv: string[]): { forwardedArgv: string[]; samples: number } {
  const forwardedArgv: string[] = [];
  let samples = 5;

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
  samples: number,
  label: string,
  task: (services: Pf2eTerminalAppServices) => PromiseLike<void> | void,
  services: Pf2eTerminalAppServices,
): Promise<TimingSummary> {
  const timings: number[] = [];
  for (let index = 0; index < samples; index += 1) {
    timings.push(await measureCall(() => task(services)));
  }

  return {
    label,
    medianMs: median(timings),
    minMs: Math.min(...timings),
    maxMs: Math.max(...timings),
  };
}

function printTable(results: TimingSummary[]): void {
  const labelWidth = Math.max(...results.map((result) => result.label.length), "Path".length);
  const medianWidth = Math.max(...results.map((result) => formatMs(result.medianMs).length), "Median".length);
  const minWidth = Math.max(...results.map((result) => formatMs(result.minMs).length), "Min".length);
  const maxWidth = Math.max(...results.map((result) => formatMs(result.maxMs).length), "Max".length);
  console.log(
    `${"Path".padEnd(labelWidth)}  ${"Median".padStart(medianWidth)}  ${"Min".padStart(minWidth)}  ${"Max".padStart(maxWidth)}`,
  );
  console.log(
    `${"-".repeat(labelWidth)}  ${"-".repeat(medianWidth)}  ${"-".repeat(minWidth)}  ${"-".repeat(maxWidth)}`,
  );
  for (const result of results) {
    console.log(
      `${result.label.padEnd(labelWidth)}  ${formatMs(result.medianMs).padStart(medianWidth)}  ${formatMs(
        result.minMs,
      ).padStart(minWidth)}  ${formatMs(result.maxMs).padStart(maxWidth)}`,
    );
  }
}

const creatureCatalogRequest: SearchRequest = {
  mode: "browse",
  filter: buildScopeFilter("creature"),
  limit: 20,
};

const dragonSearchRequest: SearchRequest = {
  mode: "search",
  search: { query: "dragon", profile: "lexical" },
  filter: buildScopeFilter("creature"),
  limit: 20,
};

async function main(): Promise<void> {
  const { forwardedArgv, samples } = parseArgs(process.argv.slice(2));
  const runtime = await loadPf2eApplicationRuntime(forwardedArgv);
  const services = createPf2eTerminalAppServices(runtime);
  try {
    const results: TimingSummary[] = [];
    results.push(
      await sampleTimings(
        samples,
        "Catalog trait values",
        () => {
          runtime.dataService.listFilterValues({ field: "traits", category: "creature" });
        },
        services,
      ),
    );
    results.push(
      await sampleTimings(
        samples,
        "Matching dragon trait values",
        () => runtime.dataService.discoverFilterValues({ field: "traits", category: "creature" }, dragonSearchRequest),
        services,
      ),
    );
    results.push(
      await sampleTimings(
        samples,
        "Creature Statistics catalog domain",
        () =>
          services.user.ontology.loadSearchFilterExplorerDomain({
            request: creatureCatalogRequest,
            discoveryMode: "catalog",
          }),
        services,
      ),
    );
    results.push(
      await sampleTimings(
        samples,
        "Creature Statistics matching domain",
        () =>
          services.user.ontology.loadSearchFilterExplorerDomain({
            request: dragonSearchRequest,
            discoveryMode: "matching",
          }),
        services,
      ),
    );

    console.log("PF2E filter-discovery timings");
    console.log(`Samples per path: ${samples}`);
    console.log("");
    printTable(results);
  } finally {
    services.close();
  }
}

void main().catch((error) => {
  console.error((error as Error).message);
  process.exitCode = 1;
});
