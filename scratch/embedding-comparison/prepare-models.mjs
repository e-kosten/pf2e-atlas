#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const options = parseArgs(process.argv.slice(2));
const modelsPath = path.resolve(repoRoot, options.models ?? "scratch/embedding-comparison/models.json");
const embeddingCachePath = path.resolve(repoRoot, options.embeddingCachePath ?? ".cache/hf-models");
const force = Boolean(options.force);
const quiet = Boolean(options.quiet);

const models = readJson(modelsPath).filter((model) => model.enabled !== false);
if (models.length === 0) {
  throw new Error(`no enabled models found in ${modelsPath}`);
}

fs.mkdirSync(embeddingCachePath, { recursive: true });

log(`preparing ${models.length} embedding model(s)`);
log(`models: ${path.relative(repoRoot, modelsPath)}`);
log(`cache: ${path.relative(repoRoot, embeddingCachePath)}`);

const transformers = await import("@huggingface/transformers");
transformers.env.allowLocalModels = true;
transformers.env.allowRemoteModels = true;
transformers.env.cacheDir = embeddingCachePath;

const results = [];
for (const [index, model] of models.entries()) {
  const before = modelPreflight(model);
  if (before.ready && !force) {
    log(`[${index + 1}/${models.length}] ${model.id}: already prepared`);
    results.push({ ...before, status: "ready" });
    continue;
  }

  const started = performance.now();
  log(`[${index + 1}/${models.length}] ${model.id}: downloading/warming ${model.model_id}`);
  try {
    const extractor = await transformers.pipeline("feature-extraction", model.model_id, {
      dtype: "fp32",
      model_file_name: "model",
      revision: model.model_revision ?? "main",
      progress_callback: (progress) => {
        if (progress?.file && progress.status !== "progress") {
          log(`[${index + 1}/${models.length}] ${model.id}: ${progress.status ?? "progress"} ${progress.file}`);
        }
      },
    });
    const warmupText = `${model.query_prefix ?? ""}pf2e semantic search warmup`;
    const warmup = await extractor(warmupText, {
      pooling: "mean",
      normalize: true,
    });
    const after = modelPreflight(model);
    if (!after.ready) {
      throw new Error(
        `download completed but Rust-required files are still missing: ${missingAssetNames(after).join(", ")}`,
      );
    }
    results.push({
      ...after,
      status: "ready",
      duration_ms: performance.now() - started,
      warmup_dimensions: Number(warmup.dims?.at(-1) ?? warmup.data?.length ?? 0),
    });
    log(`[${index + 1}/${models.length}] ${model.id}: ready in ${formatDuration(performance.now() - started)}`);
  } catch (error) {
    const after = modelPreflight(model);
    results.push({
      ...after,
      status: "failed",
      duration_ms: performance.now() - started,
      error: error instanceof Error ? error.message : String(error),
    });
    log(`[${index + 1}/${models.length}] ${model.id}: failed: ${error instanceof Error ? error.message : String(error)}`);
  }
}

const failed = results.filter((result) => result.status !== "ready");
const summary = {
  status: failed.length === 0 ? "ok" : "failed",
  models_path: modelsPath,
  embedding_cache_path: embeddingCachePath,
  model_count: models.length,
  ready_count: results.length - failed.length,
  failed_count: failed.length,
  results,
};

const summaryPath = path.join(repoRoot, "scratch/embedding-comparison/model-prep-summary.json");
writeJson(summaryPath, summary);
log(`wrote model prep summary to ${path.relative(repoRoot, summaryPath)}`);

if (failed.length > 0) {
  console.error("failed to prepare all selected models:");
  for (const failure of failed) {
    console.error(`- ${failure.model_id} (${failure.runtime_model}): ${failure.error ?? missingAssetNames(failure).join(", ")}`);
  }
  process.exit(1);
}

function modelPreflight(model) {
  const modelCachePath = path.join(embeddingCachePath, model.model_id ?? "");
  const tokenizerPath = path.join(modelCachePath, "tokenizer.json");
  const onnxPath = path.join(modelCachePath, "onnx", "model.onnx");
  return {
    model_id: model.id,
    runtime_model: model.runtime_model ?? "default",
    hf_model_id: model.model_id,
    model_cache_path: modelCachePath,
    model_cache_exists: fs.existsSync(modelCachePath),
    tokenizer_json_exists: fs.existsSync(tokenizerPath),
    onnx_model_exists: fs.existsSync(onnxPath),
    ready: fs.existsSync(modelCachePath) && fs.existsSync(tokenizerPath) && fs.existsSync(onnxPath),
    model_cache_bytes: directorySizeOrNull(modelCachePath),
  };
}

function missingAssetNames(model) {
  return [
    model.model_cache_exists ? null : "model cache directory",
    model.tokenizer_json_exists ? null : "tokenizer.json",
    model.onnx_model_exists ? null : "onnx/model.onnx",
  ].filter(Boolean);
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

function formatDuration(durationMs) {
  if (durationMs < 1000) {
    return `${Math.round(durationMs)}ms`;
  }
  return `${(durationMs / 1000).toFixed(1)}s`;
}

function log(message) {
  if (!quiet) {
    console.error(`[embedding-model-prep] ${new Date().toISOString()} ${message}`);
  }
}
