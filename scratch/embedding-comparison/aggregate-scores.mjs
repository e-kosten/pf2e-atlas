#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const options = parseArgs(process.argv.slice(2));
const runRoot = path.resolve(repoRoot, options.run ?? "");

if (!options.run) {
  throw new Error("usage: node scratch/embedding-comparison/aggregate-scores.mjs --run <run-dir>");
}

const summary = readJson(path.join(runRoot, "summary.json"));
const scoreDir = path.join(runRoot, "review-scores");
const packetsDir = path.join(runRoot, "review-packets");
const scoresByModel = new Map();
const queryScores = [];

for (const fileName of fs.readdirSync(scoreDir).filter((file) => file.endsWith(".json")).sort()) {
  const scoreFile = path.join(scoreDir, fileName);
  const review = readJson(scoreFile);
  const mappingPath = path.join(packetsDir, `${review.query_id}.mapping.json`);
  const mapping = readJson(mappingPath);
  const modelByCandidate = new Map(mapping.map((entry) => [entry.candidate_set, entry.model_id]));

  for (const score of review.scores ?? []) {
    const modelId = modelByCandidate.get(score.candidate_set);
    if (!modelId) {
      continue;
    }
    const quality =
      number(score.top_1_quality) +
      number(score.top_3_quality) +
      number(score.top_10_quality) -
      number(score.irrelevant_count) -
      number(score.overbroad_count) -
      number(score.relationship_noise_count);
    const entry = {
      query_id: review.query_id,
      candidate_set: score.candidate_set,
      model_id: modelId,
      quality,
      ...score,
    };
    queryScores.push(entry);
    if (!scoresByModel.has(modelId)) {
      scoresByModel.set(modelId, []);
    }
    scoresByModel.get(modelId).push(entry);
  }
}

const deterministicByModel = new Map(summary.models.map((model) => [model.model_id, model]));
const modelScores = [...scoresByModel.entries()]
  .map(([modelId, scores]) => {
    const deterministic = deterministicByModel.get(modelId) ?? {};
    const totalQuality = sum(scores.map((score) => score.quality));
    return {
      model_id: modelId,
      scored_query_count: scores.length,
      total_quality: totalQuality,
      average_quality: scores.length > 0 ? totalQuality / scores.length : null,
      average_top_1_quality: average(scores.map((score) => score.top_1_quality)),
      average_top_3_quality: average(scores.map((score) => score.top_3_quality)),
      average_top_10_quality: average(scores.map((score) => score.top_10_quality)),
      total_irrelevant_count: sum(scores.map((score) => score.irrelevant_count)),
      total_overbroad_count: sum(scores.map((score) => score.overbroad_count)),
      total_relationship_noise_count: sum(scores.map((score) => score.relationship_noise_count)),
      build_duration_ms: deterministic.build_duration_ms ?? null,
      artifact_bytes: deterministic.artifact_bytes ?? null,
      truncated_document_count: deterministic.truncated_document_count ?? null,
      vector_build_status: deterministic.vector_build_status ?? null,
    };
  })
  .sort((left, right) => (right.average_quality ?? -Infinity) - (left.average_quality ?? -Infinity));

const aggregate = {
  status: "ok",
  run: runRoot,
  scored_query_count: new Set(queryScores.map((score) => score.query_id)).size,
  model_scores: modelScores,
  query_scores: queryScores,
};

writeJson(path.join(runRoot, "score-summary.json"), aggregate);
fs.writeFileSync(path.join(runRoot, "score-summary.md"), scoreSummaryMarkdown(aggregate));
console.log(`wrote score summary to ${path.join(runRoot, "score-summary.md")}`);

function scoreSummaryMarkdown(aggregate) {
  const lines = ["# Embedding Comparison Score Summary", ""];
  lines.push(`Run: ${aggregate.run}`, "");
  lines.push("| Model | Avg quality | Queries | Avg top 1 | Avg top 3 | Avg top 10 | Irrelevant | Overbroad | Relationship noise | Build ms | Artifact bytes |");
  lines.push("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |");
  for (const score of aggregate.model_scores) {
    lines.push(
      `| ${score.model_id} | ${format(score.average_quality)} | ${score.scored_query_count} | ${format(score.average_top_1_quality)} | ${format(score.average_top_3_quality)} | ${format(score.average_top_10_quality)} | ${score.total_irrelevant_count} | ${score.total_overbroad_count} | ${score.total_relationship_noise_count} | ${score.build_duration_ms ?? ""} | ${score.artifact_bytes ?? ""} |`,
    );
  }
  lines.push("");
  lines.push("Quality formula: top_1 + top_3 + top_10 - irrelevant - overbroad - relationship_noise.");
  return `${lines.join("\n")}\n`;
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
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

function number(value) {
  return typeof value === "number" ? value : 0;
}

function sum(values) {
  return values.reduce((total, value) => total + number(value), 0);
}

function average(values) {
  return values.length > 0 ? sum(values) / values.length : null;
}

function format(value) {
  return typeof value === "number" ? value.toFixed(2) : "";
}
