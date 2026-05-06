import { performance } from "node:perf_hooks";

import { loadConfig } from "../dist/app/config.js";
import { Pf2eDataService } from "../dist/data/service.js";
import { RankingConfigStore } from "../dist/search/ranking-config.js";
import { createPf2eTerminalAppServices } from "../dist/tui/app-services.js";
import { buildScopeFilter } from "../dist/domain/search-request-types.js";

function parseArgs(argv) {
  const forwardedArgv = [];
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

function formatMs(durationMs) {
  return `${durationMs.toFixed(2)} ms`;
}

function median(values) {
  const sorted = [...values].sort((left, right) => left - right);
  return sorted[Math.floor(sorted.length / 2)] ?? 0;
}

async function measureCall(task) {
  const start = performance.now();
  await task();
  return performance.now() - start;
}

async function sampleTimings(samples, label, task, services) {
  const timings = [];
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

function printTable(results) {
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

const creatureCatalogRequest = {
  mode: "browse",
  filter: buildScopeFilter("creature"),
  limit: 20,
};

const dragonSearchRequest = {
  mode: "search",
  search: { query: "dragon", profile: "lexical" },
  filter: buildScopeFilter("creature"),
  limit: 20,
};

async function main() {
  const { forwardedArgv, samples } = parseArgs(process.argv.slice(2));
  const config = await loadConfig(forwardedArgv);
  const rankingConfigStore = await RankingConfigStore.create(config.ranking.configPath, { watch: false });
  let dataService;
  try {
    dataService = await Pf2eDataService.load(config.rootPath, config.manifestPath, {
      indexPath: config.indexPath,
      embedding: config.embeddings,
      rankingConfigStore,
    });
  } catch (error) {
    rankingConfigStore.close();
    throw error;
  }
  const runtime = {
    config,
    dataService,
    startupWarnings: dataService.warnings,
    stats: dataService.getStats(),
    close: () => dataService.close(),
  };
  const services = createPf2eTerminalAppServices(runtime);
  try {
    const results = [];
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
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
});
