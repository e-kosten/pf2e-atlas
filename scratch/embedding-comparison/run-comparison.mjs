#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

const options = parseArgs(process.argv.slice(2));
const sourceRoot = path.resolve(repoRoot, options.source ?? "vendor/pf2e");
const modelsPath = path.resolve(repoRoot, options.models ?? "scratch/embedding-comparison/models.json");
const queriesPath = path.resolve(repoRoot, options.queries ?? "scratch/embedding-comparison/queries.json");
const outputRoot = path.resolve(
  repoRoot,
  options.output ?? `scratch/embedding-comparison/runs/${timestampRunId()}`,
);
const atlasPath = path.resolve(repoRoot, options.atlas ?? "rust/target/release/atlas");
const embeddingCachePath = path.resolve(repoRoot, options.embeddingCachePath ?? ".cache/hf-models");
const embeddingBatchSize = Number(options.embeddingBatchSize ?? 32);

if (!fs.existsSync(atlasPath)) {
  throw new Error(`atlas binary not found at ${atlasPath}; run: cd rust && cargo build --release -p atlas-cli`);
}

const models = readJson(modelsPath).filter((model) => model.enabled !== false);
const queries = readJson(queriesPath);

if (models.length === 0) {
  throw new Error(`no enabled models found in ${modelsPath}`);
}

fs.mkdirSync(outputRoot, { recursive: true });
fs.mkdirSync(path.join(outputRoot, "models"), { recursive: true });
fs.mkdirSync(path.join(outputRoot, "review-packets"), { recursive: true });
fs.mkdirSync(path.join(outputRoot, "review-scores"), { recursive: true });
fs.mkdirSync(path.join(outputRoot, "score-templates"), { recursive: true });

const preflight = {
  source_exists: fs.existsSync(sourceRoot),
  atlas_exists: fs.existsSync(atlasPath),
  embedding_cache_path: embeddingCachePath,
  sqlite3: commandExists("sqlite3"),
  query_count: queries.length,
  models: models.map((model) => modelPreflight(model)),
};
writeJson(path.join(outputRoot, "preflight.json"), preflight);

writeJson(path.join(outputRoot, "run-config.json"), {
  started_at: new Date().toISOString(),
  source: sourceRoot,
  models_path: modelsPath,
  queries_path: queriesPath,
  output: outputRoot,
  atlas: atlasPath,
  embedding_cache_path: embeddingCachePath,
  embedding_batch_size: embeddingBatchSize,
});

const modelSummaries = [];
const querySummaries = [];
const queryResultsById = new Map(queries.map((query) => [query.id, []]));

for (const model of models) {
  const modelDir = path.join(outputRoot, "models", model.id);
  fs.mkdirSync(modelDir, { recursive: true });
  fs.mkdirSync(path.join(modelDir, "queries"), { recursive: true });
  writeJson(path.join(modelDir, "model-config.json"), model);

  const runtimeModel = model.runtime_model ?? "default";
  const artifactPath = path.join(modelDir, "artifact.sqlite");
  const buildArgs = [
    "index",
    "build",
    "--source",
    sourceRoot,
    "--output",
    artifactPath,
    "--embedding-model",
    runtimeModel,
    "--embedding-cache-path",
    embeddingCachePath,
    "--embedding-batch-size",
    String(embeddingBatchSize),
    "--no-reuse-embeddings",
    "--json",
  ];
  writeJson(path.join(modelDir, "build-command.json"), { command: atlasPath, args: buildArgs });

  const build = runCommand(atlasPath, buildArgs, repoRoot);
  fs.writeFileSync(path.join(modelDir, "build.stdout.json"), build.stdout);
  fs.writeFileSync(path.join(modelDir, "build.stderr.log"), build.stderr);
  if (build.status !== 0) {
    writeJson(path.join(modelDir, "build-error.json"), build);
    modelSummaries.push({
      model_id: model.id,
      status: "build_failed",
      status_code: build.status,
      preflight: modelPreflight(model),
      stderr_excerpt: excerpt(build.stderr),
      stdout_excerpt: excerpt(build.stdout),
    });
    continue;
  }

  const buildReport = parseJsonOutput(build.stdout, `build output for ${model.id}`);
  const buildVectors = runCommand(
    atlasPath,
    ["index", "build-vectors", "--index", artifactPath, "--json"],
    repoRoot,
  );
  fs.writeFileSync(path.join(modelDir, "build-vectors.stdout.json"), buildVectors.stdout);
  fs.writeFileSync(path.join(modelDir, "build-vectors.stderr.log"), buildVectors.stderr);
  const buildVectorsReport = buildVectors.stdout.trim()
    ? parseJsonOutput(buildVectors.stdout, `build-vectors output for ${model.id}`)
    : { status: "error", stderr: buildVectors.stderr };
  writeJson(path.join(modelDir, "build-vectors-report.json"), buildVectorsReport);

  const embeddingMetrics = embeddingMetricsFor({
    model,
    build,
    buildReport,
    buildVectors,
    buildVectorsReport,
    artifactPath,
    embeddingCachePath,
  });
  writeJson(path.join(modelDir, "build-report.json"), buildReport);
  writeJson(path.join(modelDir, "embedding-metrics.json"), embeddingMetrics);

  const validateVectors = runCommand(
    atlasPath,
    ["index", "validate-vectors", "--index", artifactPath, "--json"],
    repoRoot,
  );
  fs.writeFileSync(path.join(modelDir, "validate-vectors.stdout.json"), validateVectors.stdout);
  fs.writeFileSync(path.join(modelDir, "validate-vectors.stderr.log"), validateVectors.stderr);
  writeJson(
    path.join(modelDir, "validate-vectors-report.json"),
    validateVectors.stdout.trim()
      ? parseJsonOutput(validateVectors.stdout, `validate-vectors output for ${model.id}`)
      : { status: "error", stderr: validateVectors.stderr },
  );

  const modelQuerySummaries = [];
  for (const query of queries) {
    const queryResult = runQuery({ model, modelDir, artifactPath, query });
    modelQuerySummaries.push(queryResult.summary);
    querySummaries.push(queryResult.summary);
    queryResultsById.get(query.id)?.push(queryResult.packetEntry);
  }

  modelSummaries.push({
    model_id: model.id,
    status: "ok",
    build_duration_ms: build.duration_ms,
    artifact_bytes: embeddingMetrics.artifact_bytes,
    generated_document_embedding_count: buildReport.generated_document_embedding_count,
    reused_document_embedding_count: buildReport.reused_document_embedding_count,
    vector_build_status: buildVectorsReport.status,
    truncated_document_count: buildReport.document_embedding_tokenization?.truncated_document_count ?? null,
    query_count: modelQuerySummaries.length,
  });
}

for (const query of queries) {
  const entries = queryResultsById.get(query.id) ?? [];
  writeReviewPacket(outputRoot, query, entries);
}
writeRunReviewInstructions(outputRoot);

const summary = {
  status: "ok",
  output: outputRoot,
  model_count: modelSummaries.length,
  query_count: queries.length,
  models: modelSummaries,
  queries: querySummaries,
};
writeJson(path.join(outputRoot, "summary.json"), summary);
fs.writeFileSync(path.join(outputRoot, "summary.md"), summaryMarkdown(summary));

console.log(`wrote embedding comparison run to ${outputRoot}`);

function runQuery({ model, modelDir, artifactPath, query }) {
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

  const output = runCommand(atlasPath, args, repoRoot);
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
  const result = {
    status: "ok",
    model_id: model.id,
    query_id: query.id,
    query: query.query,
    query_category: query.category ?? null,
    query_length: queryLength(query.query),
    filter: query.filter ?? null,
    review_guidance: query.review_guidance,
    duration_ms: output.duration_ms,
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
      hit_count: enrichedHits.length,
      top_distance: enrichedHits[0]?.distance ?? null,
    },
    packetEntry: {
      model_id: model.id,
      status: "ok",
      duration_ms: output.duration_ms,
      results: enrichedHits,
    },
  };
}

function enrichHits(artifactPath, hits) {
  if (hits.length === 0) {
    return [];
  }
  const values = hits
    .map((hit, index) => `(${index + 1}, ${sqlString(hit.record_key)}, ${Number(hit.distance)})`)
    .join(", ");
  const sql = `
WITH hits(rank, record_key, distance) AS (VALUES ${values})
SELECT
  hits.rank,
  hits.distance,
  records.record_key,
  records.name,
  records.record_family,
  records.level,
  records.rarity,
  records.traits_json,
  substr(coalesce(records.description_text, ''), 1, 700) AS description_excerpt
FROM hits
LEFT JOIN records ON records.record_key = hits.record_key
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
    record_key: row.record_key,
    distance: row.distance,
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
      lines.push(`   Distance: ${hit.distance}`);
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

function embeddingMetricsFor({
  model,
  build,
  buildReport,
  buildVectors,
  buildVectorsReport,
  artifactPath,
  embeddingCachePath,
}) {
  const tokenization = buildReport.document_embedding_tokenization ?? {};
  const documentCount = tokenization.document_count ?? 0;
  const truncatedDocumentCount = tokenization.truncated_document_count ?? 0;
  const generated = buildReport.generated_document_embedding_count ?? 0;
  const durationSeconds = build.duration_ms / 1000;
  return {
    model_id: model.id,
    runtime_model: model.runtime_model ?? "default",
    model_cache_path: path.join(embeddingCachePath, model.model_id ?? ""),
    model_cache_exists: fs.existsSync(path.join(embeddingCachePath, model.model_id ?? "")),
    onnx_model_exists: fs.existsSync(path.join(embeddingCachePath, model.model_id ?? "", "onnx", "model.onnx")),
    tokenizer_json_exists: fs.existsSync(path.join(embeddingCachePath, model.model_id ?? "", "tokenizer.json")),
    build_duration_ms: buildReport.build_duration_ms,
    orchestrator_build_duration_ms: build.duration_ms,
    pending_document_embedding_count: buildReport.pending_document_embedding_count,
    document_embedding_count: buildReport.document_embedding_count,
    generated_document_embedding_count: generated,
    reused_document_embedding_count: buildReport.reused_document_embedding_count,
    generated_docs_per_second_coarse: durationSeconds > 0 ? generated / durationSeconds : null,
    artifact_bytes: fileSizeOrNull(artifactPath),
    model_cache_bytes: directorySizeOrNull(path.join(embeddingCachePath, model.model_id ?? "")),
    vector_index: {
      status: buildVectorsReport.status ?? null,
      code: buildVectorsReport.code ?? null,
      duration_ms: buildVectors.duration_ms,
    },
    tokenization: {
      document_count: documentCount,
      max_token_count: tokenization.max_token_count ?? null,
      max_observed_token_count: tokenization.max_observed_token_count ?? null,
      truncated_document_count: truncatedDocumentCount,
      truncation_rate: documentCount > 0 ? truncatedDocumentCount / documentCount : null,
      truncated_examples: tokenization.truncated_examples ?? [],
    },
    unavailable_metrics: [
      "separate_model_load_duration_ms",
      "separate_tokenizer_load_duration_ms",
      "embedding_generation_duration_ms",
      "per_batch_duration_ms",
      "peak_rss_bytes",
    ],
  };
}

function modelPreflight(model) {
  const modelCachePath = path.join(embeddingCachePath, model.model_id ?? "");
  const tokenizerPath = path.join(modelCachePath, "tokenizer.json");
  const onnxPath = path.join(modelCachePath, "onnx", "model.onnx");
  return {
    model_id: model.id,
    runtime_model: model.runtime_model ?? "default",
    model_cache_path: modelCachePath,
    model_cache_exists: fs.existsSync(modelCachePath),
    tokenizer_json_exists: fs.existsSync(tokenizerPath),
    onnx_model_exists: fs.existsSync(onnxPath),
    model_cache_bytes: directorySizeOrNull(modelCachePath),
  };
}

function queryLength(query) {
  return {
    character_count: query.length,
    word_count: query.trim().split(/\s+/).filter(Boolean).length,
  };
}

function runCommand(command, args, cwd) {
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

function commandExists(command) {
  const output = spawnSync("command", ["-v", command], {
    shell: true,
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
  const lines = ["# Embedding Comparison Summary", ""];
  lines.push(`Output: ${summary.output}`, "");
  lines.push("| Model | Status | Build ms | Artifact bytes | Truncated docs | Queries |");
  lines.push("| --- | --- | ---: | ---: | ---: | ---: |");
  for (const model of summary.models) {
    lines.push(
      `| ${model.model_id} | ${model.status} | ${model.build_duration_ms ?? ""} | ${model.artifact_bytes ?? ""} | ${model.truncated_document_count ?? ""} | ${model.query_count ?? ""} |`,
    );
  }
  lines.push("");
  lines.push("Review packets are under `review-packets/`. Put subjective scoring JSON under `review-scores/` before the final decision pass.");
  return `${lines.join("\n")}\n`;
}
