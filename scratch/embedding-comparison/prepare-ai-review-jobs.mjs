#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const options = parseArgs(process.argv.slice(2));

if (!options.run) {
  throw new Error(
    "usage: node scratch/embedding-comparison/prepare-ai-review-jobs.mjs --run <run-dir> [--chunk-size 10] [--exclude-query <query-id>]",
  );
}

const runRoot = path.resolve(repoRoot, options.run);
const reviewPacketsDir = path.join(runRoot, "review-packets");
const scoreTemplatesDir = path.join(runRoot, "score-templates");
const jobsRoot = path.join(runRoot, "ai-review-jobs");
const scoresRoot = path.join(runRoot, "ai-review-scores");
const chunkSize = positiveInteger(options.chunkSize ?? "10", "--chunk-size");
const excludedQueries = new Set(arrayOption(options.excludeQuery));

const rubric = readJson(path.join(runRoot, "scoring-rubric.json"));
const templates = fs
  .readdirSync(scoreTemplatesDir)
  .filter((fileName) => fileName.endsWith(".json"))
  .map((fileName) => readJson(path.join(scoreTemplatesDir, fileName)))
  .filter((template) => !excludedQueries.has(template.query_id))
  .sort((left, right) => left.query_id.localeCompare(right.query_id));

if (templates.length === 0) {
  throw new Error("no score templates remain after applying excludes");
}

const candidateLetters = candidateLettersFromTemplates(templates);
fs.mkdirSync(jobsRoot, { recursive: true });
fs.mkdirSync(scoresRoot, { recursive: true });

const jobs = [];
for (const candidateSet of candidateLetters) {
  const candidateJobDir = path.join(jobsRoot, `candidate-${candidateSet}`);
  const candidateScoreDir = path.join(scoresRoot, `candidate-${candidateSet}`);
  fs.mkdirSync(candidateJobDir, { recursive: true });
  fs.mkdirSync(candidateScoreDir, { recursive: true });

  const chunks = chunk(templates, chunkSize);
  for (const [chunkIndex, chunkTemplates] of chunks.entries()) {
    const chunkId = `chunk-${String(chunkIndex + 1).padStart(3, "0")}`;
    const outputPath = path.join(candidateScoreDir, `${chunkId}.json`);
    const jobPath = path.join(candidateJobDir, `${chunkId}.md`);
    const queryIds = chunkTemplates.map((template) => template.query_id);
    fs.writeFileSync(
      jobPath,
      reviewJobMarkdown({
        candidateSet,
        chunkId,
        outputPath,
        queryIds,
        rubric,
        packets: queryIds.map((queryId) => candidatePacketSection(queryId, candidateSet)),
      }),
    );
    jobs.push({
      candidate_set: candidateSet,
      chunk_id: chunkId,
      job_path: relative(jobPath),
      output_path: relative(outputPath),
      query_ids: queryIds,
    });
  }
}

const manifest = {
  version: 1,
  run: relative(runRoot),
  generated_at: new Date().toISOString(),
  chunk_size: chunkSize,
  excluded_queries: [...excludedQueries].sort(),
  candidate_sets: candidateLetters,
  query_count: templates.length,
  job_count: jobs.length,
  jobs,
};
writeJson(path.join(jobsRoot, "manifest.json"), manifest);
console.log(`wrote ${jobs.length} AI review job(s) to ${relative(jobsRoot)}`);

function candidatePacketSection(queryId, candidateSet) {
  const packetPath = path.join(reviewPacketsDir, `${queryId}.md`);
  const packet = fs.readFileSync(packetPath, "utf8");
  const candidateHeading = `## Candidate Set ${candidateSet}`;
  const candidateStart = packet.indexOf(candidateHeading);
  if (candidateStart < 0) {
    throw new Error(`packet ${relative(packetPath)} does not contain ${candidateHeading}`);
  }
  const nextCandidateStart = packet.indexOf("\n## Candidate Set ", candidateStart + candidateHeading.length);
  const headerEnd = packet.indexOf("\n## Candidate Set ");
  if (headerEnd < 0) {
    throw new Error(`packet ${relative(packetPath)} does not contain candidate sections`);
  }
  const header = packet.slice(0, headerEnd).trimEnd();
  const candidateSection = packet
    .slice(candidateStart, nextCandidateStart < 0 ? packet.length : nextCandidateStart)
    .trimEnd();
  return {
    query_id: queryId,
    markdown: `${header}\n\n${candidateSection}\n`,
  };
}

function reviewJobMarkdown({ candidateSet, chunkId, outputPath, queryIds, rubric, packets }) {
  const lines = [
    `# AI Review Job ${candidateSet} ${chunkId}`,
    "",
    "You are scoring search result quality for one anonymized candidate set.",
    "",
    "Rules:",
    "- Score only the candidate set shown in this file.",
    "- Do not infer or ask for model identities.",
    "- Do not open mapping files or summary files.",
    "- Score against the query and review guidance only.",
    "- Penalize irrelevant, overbroad, or relationship-noise results.",
    "- Do not assume there is a predefined answer key.",
    "",
    `Candidate set: ${candidateSet}`,
    `Queries in this job: ${queryIds.join(", ")}`,
    "",
    "Write exactly one JSON file to:",
    "",
    "```text",
    relative(outputPath),
    "```",
    "",
    "Output JSON shape:",
    "",
    "```json",
    JSON.stringify(
      {
        candidate_set: candidateSet,
        scores: queryIds.map((queryId) => ({
          query_id: queryId,
          top_1_quality: 0,
          top_3_quality: 0,
          top_10_quality: 0,
          irrelevant_count: 0,
          overbroad_count: 0,
          relationship_noise_count: 0,
          missing_obvious_result_notes: [],
          notes: "",
        })),
      },
      null,
      2,
    ),
    "```",
    "",
    "Rubric:",
    "",
    "```json",
    JSON.stringify(rubric, null, 2),
    "```",
    "",
  ];

  for (const packet of packets) {
    lines.push("---", "", packet.markdown.trimEnd(), "");
  }

  return `${lines.join("\n")}\n`;
}

function candidateLettersFromTemplates(templates) {
  const first = templates[0].scores.map((score) => score.candidate_set);
  for (const template of templates) {
    const letters = template.scores.map((score) => score.candidate_set);
    if (letters.join("\u0000") !== first.join("\u0000")) {
      throw new Error(`score template ${template.query_id} has inconsistent candidate sets`);
    }
  }
  return first;
}

function chunk(values, size) {
  const chunks = [];
  for (let index = 0; index < values.length; index += size) {
    chunks.push(values.slice(index, index + size));
  }
  return chunks;
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
    } else if (parsed[key] === undefined) {
      parsed[key] = next;
      index += 1;
    } else if (Array.isArray(parsed[key])) {
      parsed[key].push(next);
      index += 1;
    } else {
      parsed[key] = [parsed[key], next];
      index += 1;
    }
  }
  return parsed;
}

function arrayOption(value) {
  if (value === undefined) {
    return [];
  }
  return Array.isArray(value) ? value : [value];
}

function positiveInteger(value, optionName) {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    throw new Error(`${optionName} must be a positive integer`);
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
