#!/usr/bin/env node

import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

const options = parseArgs(process.argv.slice(2));
const sourceRunRoot = path.resolve(repoRoot, requiredOption("sourceRun"));
const modelsPath = path.resolve(repoRoot, options.models ?? "scratch/embedding-comparison/models.json");
const queriesPath = path.resolve(repoRoot, options.queries ?? "scratch/embedding-comparison/queries.json");
const outputRoot = path.resolve(
  repoRoot,
  options.output ?? `scratch/embedding-comparison/runs/search-${timestampRunId()}`,
);
const atlasPath = path.resolve(repoRoot, options.atlas ?? "rust/target/release/atlas");
const embeddingCachePath = path.resolve(repoRoot, options.embeddingCachePath ?? ".cache/hf-models");
const atlasLogDetail = options.atlasLogDetail ?? "summary";
const skipAtlasBuild = Boolean(options.skipAtlasBuild);
const quiet = Boolean(options.quiet);

if (!options.atlas && !skipAtlasBuild) {
  buildDefaultAtlasBinary();
}

if (!fs.existsSync(atlasPath)) {
  throw new Error(`atlas binary not found at ${atlasPath}; run: cd rust && cargo build --release -p atlas-cli`);
}
if (!fs.existsSync(sourceRunRoot)) {
  throw new Error(`source run not found at ${sourceRunRoot}`);
}

log(`checking atlas binary: ${path.relative(repoRoot, atlasPath)}`);
const atlasCapabilities = readAtlasCapabilities(atlasPath);
if (!atlasCapabilities.search_semantic_embedding_model) {
  throw new Error(
    `atlas binary at ${atlasPath} does not support search semantic --embedding-model; run: cd rust && cargo build --release -p atlas-cli`,
  );
}

const models = readJson(modelsPath).filter((model) => model.enabled !== false);
const queries = readJson(queriesPath);
if (models.length === 0) {
  throw new Error(`no enabled models found in ${modelsPath}`);
}

log(`starting search-only embedding comparison: ${models.length} model(s), ${queries.length} queries`);
log(`source run: ${path.relative(repoRoot, sourceRunRoot)}`);
log(`output: ${path.relative(repoRoot, outputRoot)}`);

fs.mkdirSync(outputRoot, { recursive: true });
fs.mkdirSync(path.join(outputRoot, "models"), { recursive: true });
fs.mkdirSync(path.join(outputRoot, "review-packets"), { recursive: true });
fs.mkdirSync(path.join(outputRoot, "review-scores"), { recursive: true });
fs.mkdirSync(path.join(outputRoot, "score-templates"), { recursive: true });

const rubricPath = path.join(repoRoot, "scratch/embedding-comparison/scoring-rubric.json");
if (fs.existsSync(rubricPath)) {
  fs.copyFileSync(rubricPath, path.join(outputRoot, "scoring-rubric.json"));
}

const preflight = {
  source_run_exists: fs.existsSync(sourceRunRoot),
  atlas_exists: fs.existsSync(atlasPath),
  atlas_capabilities: atlasCapabilities,
  embedding_cache_path: embeddingCachePath,
  sqlite3: sqlite3Available(),
  query_count: queries.length,
  models: models.map((model) => modelPreflight(model)),
};
writeJson(path.join(outputRoot, "preflight.json"), preflight);

const notReady = preflight.models.filter((model) => !model.ready || !model.source_artifact_exists);
if (notReady.length > 0) {
  throw new Error(
    [
      "search-only comparison cannot start because selected models are not ready:",
      ...notReady.map((model) => {
        const missing = [
          model.ready ? null : "local model assets",
          model.source_artifact_exists ? null : "source artifact",
        ]
          .filter(Boolean)
          .join(", ");
        return `- ${model.model_id} (${model.runtime_model}): missing ${missing}`;
      }),
    ].join("\n"),
  );
}

writeJson(path.join(outputRoot, "run-config.json"), {
  started_at: new Date().toISOString(),
  mode: "search-only",
  source_run: sourceRunRoot,
  models_path: modelsPath,
  queries_path: queriesPath,
  output: outputRoot,
  atlas: atlasPath,
  embedding_cache_path: embeddingCachePath,
  atlas_log_detail: atlasLogDetail,
  skip_atlas_build: skipAtlasBuild,
});

const modelSummaries = [];
const querySummaries = [];
const queryResultsById = new Map(queries.map((query) => [query.id, []]));

for (const [modelIndex, model] of models.entries()) {
  const modelDir = path.join(outputRoot, "models", model.id);
  const queryDir = path.join(modelDir, "queries");
  const sourceArtifactPath = sourceArtifactFor(model);
  fs.mkdirSync(queryDir, { recursive: true });
  writeJson(path.join(modelDir, "model-config.json"), model);
  writeJson(path.join(modelDir, "artifact-source.json"), {
    source_run: sourceRunRoot,
    artifact: sourceArtifactPath,
  });

  const modelQuerySummaries = [];
  logModelStep(modelIndex, models.length, model.id, `running ${queries.length} queries`);
  for (const [queryIndex, query] of queries.entries()) {
    logQueryStep(modelIndex, models.length, model.id, queryIndex, queries.length, query.id);
    const queryResult = await runQuery({ model, modelDir, artifactPath: sourceArtifactPath, query });
    modelQuerySummaries.push(queryResult.summary);
    querySummaries.push(queryResult.summary);
    queryResultsById.get(query.id)?.push(queryResult.packetEntry);
  }

  modelSummaries.push({
    model_id: model.id,
    status: "ok",
    source_run: sourceRunRoot,
    artifact_source: sourceArtifactPath,
    artifact_bytes: fileSizeOrNull(sourceArtifactPath),
    query_count: modelQuerySummaries.length,
  });
  logModelStep(modelIndex, models.length, model.id, "model complete");
}

log("writing review packets");
for (const query of queries) {
  const entries = queryResultsById.get(query.id) ?? [];
  writeReviewPacket(outputRoot, query, entries);
}
writeRunReviewInstructions(outputRoot);

const summary = {
  status: "ok",
  mode: "search-only",
  source_run: sourceRunRoot,
  output: outputRoot,
  model_count: modelSummaries.length,
  query_count: queries.length,
  models: modelSummaries,
  queries: querySummaries,
};
writeJson(path.join(outputRoot, "summary.json"), summary);
fs.writeFileSync(path.join(outputRoot, "summary.md"), summaryMarkdown(summary));

console.log(`wrote search-only embedding comparison run to ${outputRoot}`);

async function runQuery({ model, modelDir, artifactPath, query }) {
  const queryDir = path.join(modelDir, "queries");
  const args = [
    "search",
    "semantic",
    "--index",
    artifactPath,
    "--embedding-cache-path",
    embeddingCachePath,
    "--embedding-model",
    model.runtime_model ?? "default",
    "--query",
    query.query,
    "--limit",
    String(query.limit ?? 10),
    "--json",
  ];
  if (query.filter) {
    args.push("--filter-json", JSON.stringify(query.filter));
  }

  const output = await runCommand(atlasPath, args, repoRoot, {
    logPrefix: `[atlas ${model.id} query ${query.id}]`,
    streamStderr: atlasLogDetail === "full",
  });
  fs.writeFileSync(path.join(queryDir, `${query.id}.stdout.json`), output.stdout);
  fs.writeFileSync(path.join(queryDir, `${query.id}.stderr.log`), output.stderr);

  if (output.status !== 0) {
    const failed = {
      status: "error",
      model_id: model.id,
      query_id: query.id,
      duration_ms: output.duration_ms,
      status_code: output.status,
      stderr: output.stderr,
      stderr_excerpt: excerpt(output.stderr),
    };
    writeJson(path.join(queryDir, `${query.id}.json`), failed);
    return {
      summary: failed,
      packetEntry: {
        model_id: model.id,
        status: "error",
        results: [],
      },
    };
  }

  const searchReport = parseJsonOutput(output.stdout, `query output for ${model.id}/${query.id}`);
  const enrichedHits = enrichHits(artifactPath, searchReport.hits ?? []);
  const timing = searchReport.timing ?? {};
  const result = {
    status: "ok",
    model_id: model.id,
    query_id: query.id,
    query: query.query,
    query_category: query.category ?? null,
    query_length: queryLength(query.query),
    filter: query.filter ?? null,
    review_guidance: query.review_guidance,
    source_artifact: artifactPath,
    duration_ms: output.duration_ms,
    rust_timing: timing,
    hit_count: enrichedHits.length,
    hits: enrichedHits,
  };
  writeJson(path.join(queryDir, `${query.id}.json`), result);

  return {
    summary: {
      status: "ok",
      model_id: model.id,
      query_id: query.id,
      query_category: query.category ?? null,
      query_length: queryLength(query.query),
      duration_ms: output.duration_ms,
      rust_timing: timing,
      hit_count: enrichedHits.length,
      top_distance: enrichedHits[0]?.distance ?? null,
      top_rank_distance: enrichedHits[0]?.rank_distance ?? null,
    },
    packetEntry: {
      model_id: model.id,
      status: "ok",
      duration_ms: output.duration_ms,
      rust_timing: timing,
      results: enrichedHits,
    },
  };
}

function enrichHits(artifactPath, hits) {
  if (hits.length === 0) {
    return [];
  }
  const values = hits
    .map(
      (hit, index) =>
        `(${index + 1}, ${sqlString(hit.embedding_unit_key)}, ${Number(hit.distance)}, ${Number(hit.rank_distance ?? hit.distance)})`,
    )
    .join(", ");
  const sql = `
WITH hits(rank, embedding_unit_key, distance, rank_distance) AS (VALUES ${values})
SELECT
  hits.rank,
  hits.distance,
  hits.rank_distance,
  hits.embedding_unit_key,
  document_embedding_cache.unit_kind,
  document_embedding_cache.label AS unit_label,
  records.record_key,
  records.name,
  records.record_family,
  records.level,
  records.rarity,
  records.traits_json,
  substr(coalesce(records.description_text, ''), 1, 700) AS description_excerpt
FROM hits
LEFT JOIN document_embedding_cache ON document_embedding_cache.embedding_unit_key = hits.embedding_unit_key
LEFT JOIN records ON records.record_key = document_embedding_cache.record_key
ORDER BY hits.rank;
`;

  const output = spawnSync("sqlite3", ["-json", artifactPath, sql], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  if (output.status !== 0) {
    return hits.map((hit, index) => ({ rank: index + 1, ...hit, enrichment_error: output.stderr }));
  }
  return JSON.parse(output.stdout || "[]").map((row) => ({
    rank: row.rank,
    embedding_unit_key: row.embedding_unit_key,
    unit_kind: row.unit_kind,
    unit_label: row.unit_label,
    record_key: row.record_key,
    distance: row.distance,
    rank_distance: row.rank_distance,
    name: row.name,
    record_family: row.record_family,
    level: row.level,
    rarity: row.rarity,
    traits: parseJsonOrNull(row.traits_json) ?? [],
    description_excerpt: row.description_excerpt,
  }));
}

function writeReviewPacket(root, query, entries) {
  const labels = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".split("");
  const mapping = entries.map((entry, index) => ({
    candidate_set: labels[index],
    model_id: entry.model_id,
    status: entry.status,
  }));
  writeJson(path.join(root, "review-packets", `${query.id}.mapping.json`), mapping);

  const lines = [
    `# ${query.id}`,
    "",
    `Query: ${query.query}`,
    "",
    `Category: ${query.category ?? "uncategorized"}`,
    `Query length: ${queryLength(query.query).word_count} words / ${query.query.length} characters`,
    "",
    "Review guidance:",
    query.review_guidance ?? "",
    "",
    "Score each candidate set on top-1, top-3, and top-10 usefulness. Penalize irrelevant, overbroad, or relationship-noise results. Do not assume there is a predefined answer key.",
    "",
  ];

  entries.forEach((entry, index) => {
    lines.push(`## Candidate Set ${labels[index]}`, "");
    if (entry.status !== "ok") {
      lines.push(`Status: ${entry.status}`, "");
      return;
    }
    entry.results.forEach((hit) => {
      lines.push(`${hit.rank}. ${hit.name ?? hit.record_key} (${hit.record_family ?? "unknown"})`);
      lines.push(`   Record key: ${hit.record_key}`);
      if (hit.embedding_unit_key) {
        lines.push(
          `   Matched unit: ${hit.embedding_unit_key} (${hit.unit_kind ?? "unknown"}${
            hit.unit_label ? `: ${hit.unit_label}` : ""
          })`,
        );
      }
      lines.push(`   Distance: ${hit.distance}`);
      if (hit.rank_distance !== undefined && hit.rank_distance !== hit.distance) {
        lines.push(`   Rank distance: ${hit.rank_distance}`);
      }
      if (hit.traits?.length) {
        lines.push(`   Traits: ${hit.traits.join(", ")}`);
      }
      if (hit.level !== null || hit.rarity) {
        lines.push(`   Level/Rarity: ${hit.level ?? "n/a"} / ${hit.rarity ?? "n/a"}`);
      }
      if (hit.description_excerpt) {
        lines.push(`   Excerpt: ${hit.description_excerpt.replace(/\s+/g, " ")}`);
      }
      lines.push("");
    });
  });

  fs.writeFileSync(path.join(root, "review-packets", `${query.id}.md`), `${lines.join("\n")}\n`);
  writeJson(path.join(root, "score-templates", `${query.id}.json`), {
    query_id: query.id,
    scores: mapping.map((entry) => ({
      candidate_set: entry.candidate_set,
      top_1_quality: 0,
      top_3_quality: 0,
      top_10_quality: 0,
      irrelevant_count: 0,
      overbroad_count: 0,
      relationship_noise_count: 0,
      missing_obvious_result_notes: [],
      notes: "",
    })),
  });
}

function writeRunReviewInstructions(root) {
  const relativeRoot = path.relative(repoRoot, root);
  const lines = [
    "# Review Instructions",
    "",
    "1. Open each `review-packets/<query-id>.md` file.",
    "2. Score each anonymized candidate set using `scoring-rubric.json`.",
    "3. Write completed score JSON files under `review-scores/<query-id>.json`.",
    "4. Run:",
    "",
    "```bash",
    `node scratch/embedding-comparison/aggregate-scores.mjs --run ${relativeRoot}`,
    "```",
    "",
    "Use the score templates in `score-templates/` as starting points. Do not edit `review-packets/*.mapping.json`; those files de-anonymize candidate sets for aggregation.",
    "",
  ];
  fs.writeFileSync(path.join(root, "REVIEW.md"), lines.join("\n"));
}

function modelPreflight(model) {
  const modelCachePath = path.join(embeddingCachePath, model.model_id ?? "");
  const tokenizerPath = path.join(modelCachePath, "tokenizer.json");
  const onnxPath = path.join(modelCachePath, "onnx", "model.onnx");
  const sourceArtifactPath = sourceArtifactFor(model);
  return {
    model_id: model.id,
    runtime_model: model.runtime_model ?? "default",
    source_artifact: sourceArtifactPath,
    source_artifact_exists: fs.existsSync(sourceArtifactPath),
    model_cache_path: modelCachePath,
    model_cache_exists: fs.existsSync(modelCachePath),
    tokenizer_json_exists: fs.existsSync(tokenizerPath),
    onnx_model_exists: fs.existsSync(onnxPath),
    ready: fs.existsSync(modelCachePath) && fs.existsSync(tokenizerPath) && fs.existsSync(onnxPath),
    model_cache_bytes: directorySizeOrNull(modelCachePath),
  };
}

function sourceArtifactFor(model) {
  return path.join(sourceRunRoot, "models", model.id, "artifact.sqlite");
}

function runCommand(command, args, cwd, options = {}) {
  const started = performance.now();
  const child = spawn(command, args, {
    cwd,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const stdoutChunks = [];
  const stderrChunks = [];
  const streamStderr = options.streamStderr !== false;
  const stderrStreamer = createLineStreamer(options.logPrefix);

  child.stdout.on("data", (chunk) => {
    stdoutChunks.push(chunk);
  });
  child.stderr.on("data", (chunk) => {
    stderrChunks.push(chunk);
    if (streamStderr) {
      stderrStreamer.write(chunk);
    }
  });

  return new Promise((resolve, reject) => {
    child.on("error", reject);
    child.on("close", (status, signal) => {
      if (streamStderr) {
        stderrStreamer.flush();
      }
      resolve({
        status,
        signal,
        duration_ms: performance.now() - started,
        stdout: Buffer.concat(stdoutChunks).toString("utf8"),
        stderr: Buffer.concat(stderrChunks).toString("utf8"),
      });
    });
  });
}

function createLineStreamer(prefix = "[atlas]") {
  let buffered = "";
  const streamLine = (line) => {
    if (!shouldStreamAtlasLine(line)) {
      return;
    }
    console.error(`${prefix} ${line}`);
  };
  return {
    write(chunk) {
      if (quiet) {
        return;
      }
      buffered += chunk.toString("utf8");
      const lines = buffered.split(/\r?\n/);
      buffered = lines.pop() ?? "";
      for (const line of lines) {
        streamLine(line);
      }
    },
    flush() {
      if (!quiet) {
        streamLine(buffered);
      }
      buffered = "";
    },
  };
}

function shouldStreamAtlasLine(line) {
  const trimmed = line.trim();
  if (trimmed.length === 0) {
    return false;
  }
  if (atlasLogDetail === "full") {
    return true;
  }
  return !/\b(?:Scanning|Loading|Finished) source pack:/.test(trimmed);
}

function queryLength(query) {
  return {
    character_count: query.length,
    word_count: query.trim().split(/\s+/).filter(Boolean).length,
  };
}

function log(message) {
  if (!quiet) {
    console.error(`[embedding-comparison] ${new Date().toISOString()} ${message}`);
  }
}

function logModelStep(modelIndex, modelCount, modelId, message) {
  log(`[model ${modelIndex + 1}/${modelCount}] ${modelId}: ${message}`);
}

function logQueryStep(modelIndex, modelCount, modelId, queryIndex, queryCount, queryId) {
  log(`[model ${modelIndex + 1}/${modelCount}] ${modelId}: query ${queryIndex + 1}/${queryCount} ${queryId}`);
}

function readAtlasCapabilities(command) {
  const searchSemanticHelp = runCommandSync(command, ["search", "semantic", "--help"], repoRoot);
  return {
    search_semantic_help_status: searchSemanticHelp.status,
    search_semantic_embedding_model:
      searchSemanticHelp.status === 0 && searchSemanticHelp.stdout.includes("--embedding-model"),
  };
}

function buildDefaultAtlasBinary() {
  log("building default atlas release binary: cargo build --release -p atlas-cli");
  const result = spawnSync("cargo", ["build", "--release", "-p", "atlas-cli"], {
    cwd: path.join(repoRoot, "rust"),
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 200,
  });
  if (result.status !== 0) {
    throw new Error(
      [
        "failed to build default atlas release binary",
        result.stdout?.trim(),
        result.stderr?.trim(),
      ]
        .filter(Boolean)
        .join("\n"),
    );
  }
}

function runCommandSync(command, args, cwd) {
  const started = performance.now();
  const output = spawnSync(command, args, {
    cwd,
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 200,
  });
  return {
    status: output.status,
    signal: output.signal,
    duration_ms: performance.now() - started,
    stdout: output.stdout ?? "",
    stderr: output.stderr ?? "",
  };
}

function sqlite3Available() {
  const output = spawnSync("sqlite3", ["-version"], {
    encoding: "utf8",
  });
  return output.status === 0;
}

function parseArgs(args) {
  const parsed = {};
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (!arg.startsWith("--")) {
      throw new Error(`unexpected positional argument: ${arg}`);
    }
    const key = arg.slice(2).replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
    const next = args[index + 1];
    if (!next || next.startsWith("--")) {
      parsed[key] = true;
    } else {
      parsed[key] = next;
      index += 1;
    }
  }
  return parsed;
}

function requiredOption(key) {
  if (!options[key]) {
    throw new Error(`missing required option --${key.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`)}`);
  }
  return options[key];
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function writeJson(filePath, value) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

function parseJsonOutput(output, label) {
  try {
    return JSON.parse(output);
  } catch (error) {
    throw new Error(`failed to parse ${label}: ${error.message}`);
  }
}

function parseJsonOrNull(value) {
  if (typeof value !== "string" || value.length === 0) {
    return null;
  }
  try {
    return JSON.parse(value);
  } catch {
    return null;
  }
}

function sqlString(value) {
  return `'${String(value).replaceAll("'", "''")}'`;
}

function fileSizeOrNull(filePath) {
  try {
    return fs.statSync(filePath).size;
  } catch {
    return null;
  }
}

function directorySizeOrNull(directoryPath) {
  try {
    let total = 0;
    for (const entry of fs.readdirSync(directoryPath, { withFileTypes: true })) {
      const child = path.join(directoryPath, entry.name);
      if (entry.isDirectory()) {
        total += directorySizeOrNull(child) ?? 0;
      } else if (entry.isFile()) {
        total += fs.statSync(child).size;
      }
    }
    return total;
  } catch {
    return null;
  }
}

function excerpt(value, limit = 2000) {
  if (!value) {
    return "";
  }
  return value.length <= limit ? value : `${value.slice(0, limit)}...`;
}

function timestampRunId() {
  return new Date().toISOString().replace(/[:.]/g, "-");
}

function summaryMarkdown(summary) {
  const lines = ["# Search-Only Embedding Comparison Summary", ""];
  lines.push(`Source run: ${summary.source_run}`, "");
  lines.push(`Output: ${summary.output}`, "");
  lines.push("| Model | Status | Artifact bytes | Queries |");
  lines.push("| --- | --- | ---: | ---: |");
  for (const model of summary.models) {
    lines.push(
      `| ${model.model_id} | ${model.status} | ${model.artifact_bytes ?? ""} | ${model.query_count ?? ""} |`,
    );
  }
  lines.push("");
  lines.push("Review packets are under `review-packets/`. Put subjective scoring JSON under `review-scores/` before the final decision pass.");
  return `${lines.join("\n")}\n`;
}
