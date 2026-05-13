import { spawnSync } from "node:child_process";
import { performance } from "node:perf_hooks";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { loadPf2eApplicationRuntime } from "../../src/app/runtime.js";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const atlas = path.join(root, "rust", "target", "release", "atlas");
const rustIndex = path.join(root, ".cache", "pf2e-rust-index.batch32.sqlite");
const nodeIndex = path.join(root, ".cache", "pf2e-index.node-minilm.sqlite");
const embeddingCachePath = path.join(root, ".cache", "hf-models");
const highLimit = 4096;

const scope = (category) => ({
  kind: "scope",
  category,
  subcategory: { kind: "any" },
});

const cases = [
  {
    id: "unfiltered-healing",
    query: "healing magic",
    rustFilter: null,
    nodeFilter: undefined,
  },
  {
    id: "spell-primal-healing",
    query: "restore vitality with primal healing",
    rustFilter: {
      kind: "all_of",
      children: [
        { kind: "record_family", value: "spell" },
        {
          kind: "metadata_predicate",
          predicate: { field_type: "set", field: "traditions", op: "includes", value: "primal" },
        },
        {
          kind: "metadata_predicate",
          predicate: { field_type: "set", field: "traits", op: "includes", value: "healing" },
        },
      ],
    },
    nodeFilter: {
      kind: "allOf",
      children: [
        scope("spell"),
        { kind: "metadataPredicate", predicate: { field: "traditions", op: "includes", value: "primal" } },
        { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "healing" } },
      ],
    },
  },
  {
    id: "creature-dragon-rarity",
    query: "rare dragon enemy",
    rustFilter: {
      kind: "all_of",
      children: [
        { kind: "record_family", value: "creature" },
        {
          kind: "metadata_predicate",
          predicate: { field_type: "set", field: "traits", op: "includes", value: "dragon" },
        },
        { kind: "rarity", match: { kind: "in", values: ["rare", "uncommon"] } },
      ],
    },
    nodeFilter: {
      kind: "allOf",
      children: [
        scope("creature"),
        { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "dragon" } },
        { kind: "rarity", match: { kind: "in", values: ["rare", "uncommon"] } },
      ],
    },
  },
  {
    id: "spell-sustained-burst",
    query: "sustained battlefield area effect",
    rustFilter: {
      kind: "all_of",
      children: [
        { kind: "record_family", value: "spell" },
        {
          kind: "metadata_predicate",
          predicate: { field_type: "boolean", field: "sustained", op: "eq", value: true },
        },
        {
          kind: "metadata_predicate",
          predicate: { field_type: "enum_string", field: "area_type", op: "eq", value: "burst" },
        },
      ],
    },
    nodeFilter: {
      kind: "allOf",
      children: [
        scope("spell"),
        { kind: "metadataPredicate", predicate: { field: "sustained", op: "eq", value: true } },
        { kind: "metadataPredicate", predicate: { field: "areaType", op: "eq", value: "burst" } },
      ],
    },
  },
  {
    id: "creature-undead-high-ac",
    query: "hard to hit undead guardian",
    rustFilter: {
      kind: "all_of",
      children: [
        { kind: "record_family", value: "creature" },
        {
          kind: "metadata_predicate",
          predicate: { field_type: "set", field: "traits", op: "includes", value: "undead" },
        },
        { kind: "metric", metric: "ac.value", op: "gte", value: 30 },
      ],
    },
    nodeFilter: {
      kind: "allOf",
      children: [
        scope("creature"),
        { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "undead" } },
        { kind: "metric", metric: "ac.value", op: "gte", value: 30 },
      ],
    },
  },
  {
    id: "equipment-consumable-mid-price-not-unique",
    query: "alchemical consumable moderate item",
    rustFilter: {
      kind: "all_of",
      children: [
        { kind: "record_family", value: "equipment" },
        {
          kind: "metadata_predicate",
          predicate: { field_type: "set", field: "traits", op: "includes", value: "consumable" },
        },
        { kind: "price", match: { kind: "between", min: 1000, max: 10000 } },
        { kind: "not", child: { kind: "rarity", match: { kind: "eq", value: "unique" } } },
      ],
    },
    nodeFilter: {
      kind: "allOf",
      children: [
        scope("equipment"),
        { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "consumable" } },
        { kind: "price", match: { kind: "between", min: 1000, max: 10000 } },
        { kind: "not", child: { kind: "rarity", match: { kind: "eq", value: "unique" } } },
      ],
    },
  },
  {
    id: "spell-mental-or-fear",
    query: "mind-affecting fear magic",
    rustFilter: {
      kind: "all_of",
      children: [
        { kind: "record_family", value: "spell" },
        {
          kind: "any_of",
          children: [
            {
              kind: "metadata_predicate",
              predicate: { field_type: "set", field: "traits", op: "includes", value: "mental" },
            },
            {
              kind: "metadata_predicate",
              predicate: { field_type: "set", field: "traits", op: "includes", value: "fear" },
            },
          ],
        },
      ],
    },
    nodeFilter: {
      kind: "allOf",
      children: [
        scope("spell"),
        {
          kind: "anyOf",
          children: [
            { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "mental" } },
            { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "fear" } },
          ],
        },
      ],
    },
  },
  {
    id: "links-to-frightened",
    query: "fear effect frightened",
    rustFilter: { kind: "links_to", target: "conditionitems:TBSHQspnbcqxsmjL" },
    nodeFilter: { kind: "linksTo", target: "conditionitems:TBSHQspnbcqxsmjL" },
  },
];

function runRust(caseDef, limit) {
  const args = [
    "search",
    "semantic",
    "--index",
    rustIndex,
    "--embedding-cache-path",
    embeddingCachePath,
    "--query",
    caseDef.query,
    "--limit",
    String(limit),
    "--json",
  ];
  if (caseDef.rustFilter) {
    args.push("--filter-json", JSON.stringify(caseDef.rustFilter));
  }

  const started = performance.now();
  const output = spawnSync(atlas, args, { cwd: root, encoding: "utf8" });
  const durationMs = performance.now() - started;
  if (output.status !== 0) {
    throw new Error(`Rust search failed for ${caseDef.id}: ${output.stderr || output.stdout}`);
  }
  const json = JSON.parse(output.stdout);
  return {
    durationMs,
    keys: json.hits.map((hit) => hit.record_key),
    distances: json.hits.map((hit) => hit.distance),
  };
}

async function runNode(runtime, caseDef) {
  const request = {
    mode: "search",
    search: { query: caseDef.query, profile: "concept" },
    filter: caseDef.nodeFilter,
    limit: 10,
    explain: true,
  };
  const started = performance.now();
  const result = await runtime.dataService.search(request);
  const durationMs = performance.now() - started;
  return {
    durationMs,
    keys: result.records.map((record) => record.recordKey),
  };
}

function intersectionCount(left, right) {
  const rightSet = new Set(right);
  return left.filter((value) => rightSet.has(value)).length;
}

const runtime = await loadPf2eApplicationRuntime([
  "--data-path",
  path.join(root, "vendor", "pf2e"),
  "--index-path",
  nodeIndex,
  "--embedding-provider",
  "hf-local",
  "--embedding-cache-path",
  embeddingCachePath,
]);

try {
  const results = [];
  for (const caseDef of cases) {
    const rust10 = runRust(caseDef, 10);
    const rustHigh = caseDef.rustFilter ? runRust(caseDef, highLimit) : null;
    const node = await runNode(runtime, caseDef);
    const highTop10 = rustHigh?.keys.slice(0, 10) ?? rust10.keys;
    results.push({
      id: caseDef.id,
      query: caseDef.query,
      rust_duration_ms: Number(rust10.durationMs.toFixed(1)),
      rust_hit_count: rust10.keys.length,
      rust_top10: rust10.keys,
      rust_high_limit: rustHigh ? highLimit : null,
      rust_high_duration_ms: rustHigh ? Number(rustHigh.durationMs.toFixed(1)) : null,
      rust_high_hit_count: rustHigh?.keys.length ?? null,
      rust_top10_matches_high_limit_top10: JSON.stringify(rust10.keys) === JSON.stringify(highTop10),
      node_duration_ms: Number(node.durationMs.toFixed(1)),
      node_top10: node.keys,
      top10_overlap_count: intersectionCount(rust10.keys, node.keys),
    });
  }
  console.log(JSON.stringify({ high_limit: highLimit, results }, null, 2));
} finally {
  runtime.close();
}
