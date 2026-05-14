#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const options = parseArgs(process.argv.slice(2));

if (!options.run) {
  throw new Error("usage: node scratch/embedding-comparison/merge-ai-review-scores.mjs --run <run-dir>");
}

const runRoot = path.resolve(repoRoot, options.run);
const manifest = readJson(path.join(runRoot, "ai-review-jobs", "manifest.json"));
const scoreTemplatesDir = path.join(runRoot, "score-templates");
const reviewScoresDir = path.join(runRoot, "review-scores");
const expectedByQuery = expectedScoresByQuery(manifest);
const actualByQuery = new Map();
const errors = [];

for (const job of manifest.jobs ?? []) {
  const scorePath = path.resolve(repoRoot, job.output_path);
  if (!scorePath.startsWith(path.join(runRoot, "ai-review-scores"))) {
    errors.push(`${job.candidate_set} ${job.chunk_id}: output path escapes ai-review-scores`);
    continue;
  }
  if (!fs.existsSync(scorePath)) {
    errors.push(`${job.candidate_set} ${job.chunk_id}: missing ${relative(scorePath)}`);
    continue;
  }

  const review = readJson(scorePath);
  if (review.candidate_set !== job.candidate_set) {
    errors.push(
      `${relative(scorePath)}: candidate_set is ${JSON.stringify(review.candidate_set)}, expected ${job.candidate_set}`,
    );
    continue;
  }
  const scoreByQuery = new Map((review.scores ?? []).map((score) => [score.query_id, score]));
  for (const queryId of job.query_ids ?? []) {
    const score = scoreByQuery.get(queryId);
    if (!score) {
      errors.push(`${relative(scorePath)}: missing score for ${queryId}`);
      continue;
    }
    const normalized = normalizeScore(score, job.candidate_set, queryId, relative(scorePath), errors);
    if (!normalized) {
      continue;
    }
    const key = `${queryId}\u0000${job.candidate_set}`;
    if (actualByQuery.has(key)) {
      errors.push(`${relative(scorePath)}: duplicate score for ${queryId} candidate ${job.candidate_set}`);
      continue;
    }
    actualByQuery.set(key, normalized);
  }
  for (const queryId of scoreByQuery.keys()) {
    if (!(job.query_ids ?? []).includes(queryId)) {
      errors.push(`${relative(scorePath)}: unexpected score for ${queryId}`);
    }
  }
}

for (const [queryId, candidateSets] of expectedByQuery.entries()) {
  for (const candidateSet of candidateSets) {
    if (!actualByQuery.has(`${queryId}\u0000${candidateSet}`)) {
      errors.push(`missing merged score for ${queryId} candidate ${candidateSet}`);
    }
  }
}

if (errors.length > 0) {
  throw new Error(["AI review scores are incomplete or invalid:", ...errors.map((error) => `- ${error}`)].join("\n"));
}

fs.mkdirSync(reviewScoresDir, { recursive: true });
for (const [queryId, candidateSets] of expectedByQuery.entries()) {
  const template = readJson(path.join(scoreTemplatesDir, `${queryId}.json`));
  const scores = template.scores
    .filter((score) => candidateSets.includes(score.candidate_set))
    .map((score) => actualByQuery.get(`${queryId}\u0000${score.candidate_set}`));
  writeJson(path.join(reviewScoresDir, `${queryId}.json`), {
    query_id: queryId,
    scores,
  });
}

console.log(`merged AI review scores for ${expectedByQuery.size} query file(s) into ${relative(reviewScoresDir)}`);

function expectedScoresByQuery(manifest) {
  const expected = new Map();
  for (const job of manifest.jobs ?? []) {
    for (const queryId of job.query_ids ?? []) {
      if (!expected.has(queryId)) {
        expected.set(queryId, []);
      }
      expected.get(queryId).push(job.candidate_set);
    }
  }
  for (const [queryId, candidateSets] of expected.entries()) {
    const templatePath = path.join(scoreTemplatesDir, `${queryId}.json`);
    if (!fs.existsSync(templatePath)) {
      throw new Error(`manifest references missing score template ${relative(templatePath)}`);
    }
    const template = readJson(templatePath);
    const templateCandidates = new Set(template.scores.map((score) => score.candidate_set));
    for (const candidateSet of candidateSets) {
      if (!templateCandidates.has(candidateSet)) {
        throw new Error(`manifest references candidate ${candidateSet} missing from ${queryId} score template`);
      }
    }
  }
  return expected;
}

function normalizeScore(score, candidateSet, queryId, source, errors) {
  const normalized = {
    candidate_set: candidateSet,
    top_1_quality: boundedInteger(score.top_1_quality, 0, 3, "top_1_quality", source, queryId, errors),
    top_3_quality: boundedInteger(score.top_3_quality, 0, 5, "top_3_quality", source, queryId, errors),
    top_10_quality: boundedInteger(score.top_10_quality, 0, 10, "top_10_quality", source, queryId, errors),
    irrelevant_count: nonNegativeInteger(score.irrelevant_count, "irrelevant_count", source, queryId, errors),
    overbroad_count: nonNegativeInteger(score.overbroad_count, "overbroad_count", source, queryId, errors),
    relationship_noise_count: nonNegativeInteger(
      score.relationship_noise_count,
      "relationship_noise_count",
      source,
      queryId,
      errors,
    ),
    missing_obvious_result_notes: Array.isArray(score.missing_obvious_result_notes)
      ? score.missing_obvious_result_notes.map(String)
      : [],
    notes: typeof score.notes === "string" ? score.notes : "",
  };
  return errors.some((error) => error.startsWith(`${source}: ${queryId}`)) ? null : normalized;
}

function boundedInteger(value, min, max, field, source, queryId, errors) {
  if (!Number.isInteger(value) || value < min || value > max) {
    errors.push(`${source}: ${queryId} ${field} must be an integer from ${min} to ${max}`);
    return 0;
  }
  return value;
}

function nonNegativeInteger(value, field, source, queryId, errors) {
  if (!Number.isInteger(value) || value < 0) {
    errors.push(`${source}: ${queryId} ${field} must be a non-negative integer`);
    return 0;
  }
  return value;
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

function relative(filePath) {
  return path.relative(repoRoot, filePath);
}
