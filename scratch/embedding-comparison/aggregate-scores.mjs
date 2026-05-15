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
const scoresByModelCategory = new Map();
const queryScores = [];
const queryById = new Map((summary.queries ?? []).map((query) => [query.query_id, query]));
const queryRuntimeByModel = readQueryRuntimeByModel(runRoot);

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
    const quality = scoreQuality(score);
    const entry = {
      query_id: review.query_id,
      review_mode: review.review_mode ?? "numeric",
      query_category: queryById.get(review.query_id)?.query_category ?? "uncategorized",
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
    const categoryKey = `${modelId}\u0000${entry.query_category}`;
    if (!scoresByModelCategory.has(categoryKey)) {
      scoresByModelCategory.set(categoryKey, []);
    }
    scoresByModelCategory.get(categoryKey).push(entry);
  }
}

const deterministicByModel = new Map(summary.models.map((model) => [model.model_id, model]));
const modelScores = [...scoresByModel.entries()]
  .map(([modelId, scores]) => {
    const deterministic = deterministicByModel.get(modelId) ?? {};
    const runtime = queryRuntimeByModel.get(modelId) ?? {};
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
      average_rank: average(scores.map((score) => score.rank)),
      average_rank_score: average(scores.map((score) => score.rank_score)),
      rank_1_count: countRankAtMost(scores, 1),
      rank_top_2_count: countRankAtMost(scores, 2),
      rank_top_3_count: countRankAtMost(scores, 3),
      build_duration_ms: deterministic.build_duration_ms ?? null,
      artifact_bytes: deterministic.artifact_bytes ?? null,
      truncated_document_count: deterministic.truncated_document_count ?? null,
      total_tokens_over_limit: deterministic.total_tokens_over_limit ?? null,
      tokens_over_limit_rate: deterministic.tokens_over_limit_rate ?? null,
      vector_build_status: deterministic.vector_build_status ?? null,
      query_duration_average_ms: runtime.average_ms ?? null,
      query_duration_p95_ms: runtime.p95_ms ?? null,
      query_duration_min_ms: runtime.min_ms ?? null,
      query_duration_max_ms: runtime.max_ms ?? null,
    };
  })
  .sort((left, right) => (right.average_quality ?? -Infinity) - (left.average_quality ?? -Infinity));

const categoryScoresByModel = categoryScores(scoresByModelCategory);
const categoryBalancedModelScores = categoryBalancedScores(modelScores, categoryScoresByModel);
const categoryRankings = rankingsByCategory(categoryScoresByModel);

const aggregate = {
  status: "ok",
  run: runRoot,
  scored_query_count: new Set(queryScores.map((score) => score.query_id)).size,
  model_scores: modelScores,
  category_balanced_model_scores: categoryBalancedModelScores,
  category_rankings: categoryRankings,
  category_scores: categoryScoresByModel,
  query_scores: queryScores,
};

writeJson(path.join(runRoot, "score-summary.json"), aggregate);
fs.writeFileSync(path.join(runRoot, "score-summary.md"), scoreSummaryMarkdown(aggregate));
console.log(`wrote score summary to ${path.join(runRoot, "score-summary.md")}`);

function scoreSummaryMarkdown(aggregate) {
  const lines = ["# Embedding Comparison Score Summary", ""];
  const hasNumericScores = aggregate.query_scores.some(
    (score) =>
      typeof score.top_1_quality === "number" ||
      typeof score.top_3_quality === "number" ||
      typeof score.top_10_quality === "number",
  );
  const hasRankScores = aggregate.model_scores.some((score) => typeof score.average_rank === "number");
  lines.push(`Run: ${aggregate.run}`, "");
  if (hasNumericScores) {
    lines.push("## Numeric Quality Scores");
    lines.push("");
    lines.push("| Model | Avg quality | Queries | Avg top 1 | Avg top 3 | Avg top 10 | Irrelevant | Overbroad | Relationship noise | Build time | Artifact bytes |");
    lines.push("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |");
    for (const score of aggregate.model_scores) {
      lines.push(
        `| ${score.model_id} | ${format(score.average_quality)} | ${score.scored_query_count} | ${format(score.average_top_1_quality)} | ${format(score.average_top_3_quality)} | ${format(score.average_top_10_quality)} | ${score.total_irrelevant_count} | ${score.total_overbroad_count} | ${score.total_relationship_noise_count} | ${formatDuration(score.build_duration_ms)} | ${score.artifact_bytes ?? ""} |`,
      );
    }
    lines.push("");
  }
  if (hasRankScores) {
    lines.push("## Rank Scores");
    lines.push("");
    lines.push("| Model | Avg rank | Avg category rank | Avg rank score | Avg category rank score | Rank-1 | Top-2 | Top-3 | Queries | Build time | Query avg ms | Query p95 ms | Artifact bytes |");
    lines.push("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |");
    for (const score of aggregate.model_scores) {
      const balanced = aggregate.category_balanced_model_scores.find((entry) => entry.model_id === score.model_id) ?? {};
      lines.push(
        `| ${score.model_id} | ${format(score.average_rank)} | ${format(balanced.average_category_rank)} | ${format(score.average_rank_score)} | ${format(balanced.average_category_rank_score)} | ${score.rank_1_count ?? ""}/${score.scored_query_count} | ${score.rank_top_2_count ?? ""}/${score.scored_query_count} | ${score.rank_top_3_count ?? ""}/${score.scored_query_count} | ${score.scored_query_count} | ${formatDuration(score.build_duration_ms)} | ${format(score.query_duration_average_ms)} | ${format(score.query_duration_p95_ms)} | ${score.artifact_bytes ?? ""} |`,
      );
    }
    lines.push("");
    lines.push("Avg rank is query-weighted. Avg category rank gives each query category equal weight.");
    lines.push("");
    lines.push("## Category-Balanced Rank Scores");
    lines.push("");
    lines.push("| Model | Avg category rank | Avg category rank score | Rank-1 categories | Top-2 categories | Top-3 categories | Categories |");
    lines.push("| --- | ---: | ---: | ---: | ---: | ---: | ---: |");
    for (const score of aggregate.category_balanced_model_scores) {
      lines.push(
        `| ${score.model_id} | ${format(score.average_category_rank)} | ${format(score.average_category_rank_score)} | ${score.rank_1_count} | ${score.rank_top_2_count} | ${score.rank_top_3_count} | ${score.scored_category_count} |`,
      );
    }
    lines.push("");
    lines.push("## Category Winners");
    lines.push("");
    lines.push("| Category | Best model(s) | Best avg rank | Spread | Queries |");
    lines.push("| --- | --- | ---: | ---: | ---: |");
    for (const ranking of aggregate.category_rankings) {
      lines.push(
        `| ${ranking.query_category} | ${ranking.best_model_ids.join("<br>")} | ${format(ranking.best_average_rank)} | ${format(ranking.rank_spread)} | ${ranking.scored_query_count} |`,
      );
    }
    lines.push("");
  }
  lines.push("## Category Scores");
  lines.push("");
  if (hasRankScores && !hasNumericScores) {
    lines.push("| Model | Category | Avg rank | Avg rank score | Queries |");
    lines.push("| --- | --- | ---: | ---: | ---: |");
    for (const score of aggregate.category_scores) {
      lines.push(
        `| ${score.model_id} | ${score.query_category} | ${format(score.average_rank)} | ${format(score.average_rank_score)} | ${score.scored_query_count} |`,
      );
    }
  } else {
    lines.push("| Model | Category | Avg quality | Avg rank | Queries | Irrelevant | Overbroad | Relationship noise |");
    lines.push("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |");
    for (const score of aggregate.category_scores) {
      lines.push(
        `| ${score.model_id} | ${score.query_category} | ${format(score.average_quality)} | ${format(score.average_rank)} | ${score.scored_query_count} | ${score.total_irrelevant_count} | ${score.total_overbroad_count} | ${score.total_relationship_noise_count} |`,
      );
    }
  }
  lines.push("");
  lines.push("Quality formula: numeric reviews use top_1 + top_3 + top_10 - irrelevant - overbroad - relationship_noise; rank-order reviews use rank_score where higher is better.");
  return `${lines.join("\n")}\n`;
}

function categoryScores(scoresByModelCategory) {
  return [...scoresByModelCategory.entries()]
    .map(([key, scores]) => {
      const [modelId, queryCategory] = key.split("\u0000");
      const totalQuality = sum(scores.map((score) => score.quality));
      return {
        model_id: modelId,
        query_category: queryCategory,
        scored_query_count: scores.length,
        total_quality: totalQuality,
        average_quality: scores.length > 0 ? totalQuality / scores.length : null,
        total_irrelevant_count: sum(scores.map((score) => score.irrelevant_count)),
        total_overbroad_count: sum(scores.map((score) => score.overbroad_count)),
        total_relationship_noise_count: sum(scores.map((score) => score.relationship_noise_count)),
        average_rank: average(scores.map((score) => score.rank)),
        average_rank_score: average(scores.map((score) => score.rank_score)),
      };
    })
    .sort((left, right) => {
      const model = left.model_id.localeCompare(right.model_id);
      if (model !== 0) {
        return model;
      }
      return left.query_category.localeCompare(right.query_category);
    });
}

function categoryBalancedScores(modelScores, categoryScores) {
  const categoriesByModel = new Map();
  for (const score of categoryScores) {
    if (!categoriesByModel.has(score.model_id)) {
      categoriesByModel.set(score.model_id, []);
    }
    categoriesByModel.get(score.model_id).push(score);
  }
  return modelScores
    .map((modelScore) => {
      const scores = categoriesByModel.get(modelScore.model_id) ?? [];
      return {
        model_id: modelScore.model_id,
        scored_category_count: scores.length,
        average_category_quality: average(scores.map((score) => score.average_quality)),
        average_category_rank: average(scores.map((score) => score.average_rank)),
        average_category_rank_score: average(scores.map((score) => score.average_rank_score)),
        rank_1_count: scores.filter((score) => score.average_rank === 1).length,
        rank_top_2_count: countRankAtMost(scores, 2),
        rank_top_3_count: countRankAtMost(scores, 3),
      };
    })
    .sort((left, right) => {
      const quality = (right.average_category_quality ?? -Infinity) - (left.average_category_quality ?? -Infinity);
      if (quality !== 0) {
        return quality;
      }
      return left.model_id.localeCompare(right.model_id);
    });
}

function rankingsByCategory(categoryScores) {
  const scoresByCategory = new Map();
  for (const score of categoryScores) {
    if (!scoresByCategory.has(score.query_category)) {
      scoresByCategory.set(score.query_category, []);
    }
    scoresByCategory.get(score.query_category).push(score);
  }
  return [...scoresByCategory.entries()]
    .map(([queryCategory, scores]) => {
      const ranked = [...scores].sort((left, right) => {
        const rank = (left.average_rank ?? Infinity) - (right.average_rank ?? Infinity);
        if (rank !== 0) {
          return rank;
        }
        return left.model_id.localeCompare(right.model_id);
      });
      const bestAverageRank = ranked[0]?.average_rank ?? null;
      const worstAverageRank = ranked.at(-1)?.average_rank ?? null;
      return {
        query_category: queryCategory,
        scored_query_count: ranked[0]?.scored_query_count ?? 0,
        best_average_rank: bestAverageRank,
        best_model_ids: ranked
          .filter((score) => score.average_rank === bestAverageRank)
          .map((score) => score.model_id),
        rank_spread:
          typeof bestAverageRank === "number" && typeof worstAverageRank === "number"
            ? worstAverageRank - bestAverageRank
            : null,
        rankings: ranked.map((score) => ({
          model_id: score.model_id,
          average_rank: score.average_rank,
          average_rank_score: score.average_rank_score,
          average_quality: score.average_quality,
        })),
      };
    })
    .sort((left, right) => left.query_category.localeCompare(right.query_category));
}

function readQueryRuntimeByModel(runRoot) {
  const modelsRoot = path.join(runRoot, "models");
  const runtimes = new Map();
  if (!fs.existsSync(modelsRoot)) {
    return runtimes;
  }
  for (const baseModelId of fs.readdirSync(modelsRoot).sort()) {
    const queryDir = path.join(modelsRoot, baseModelId, "queries");
    if (!fs.existsSync(queryDir)) {
      continue;
    }
    for (const mode of ["parent-only", "chunks", "weighted-chunks"]) {
      const durations = [];
      for (const fileName of fs.readdirSync(queryDir)) {
        if (!fileName.endsWith(`.${mode}.json`)) {
          continue;
        }
        const result = readJson(path.join(queryDir, fileName));
        if (typeof result.duration_ms === "number") {
          durations.push(result.duration_ms);
        }
      }
      if (durations.length === 0) {
        continue;
      }
      durations.sort((left, right) => left - right);
      const modelId = `${baseModelId}__${mode}`;
      runtimes.set(modelId, {
        average_ms: average(durations),
        p95_ms: percentile(durations, 0.95),
        min_ms: durations[0],
        max_ms: durations.at(-1),
      });
    }
  }
  return runtimes;
}

function scoreQuality(score) {
  if (typeof score.rank_score === "number") {
    return score.rank_score;
  }
  return (
    number(score.top_1_quality) +
    number(score.top_3_quality) +
    number(score.top_10_quality) -
    number(score.irrelevant_count) -
    number(score.overbroad_count) -
    number(score.relationship_noise_count)
  );
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

function countRankAtMost(scores, maxRank) {
  return scores.filter((score) => {
    const rank = typeof score.rank === "number" ? score.rank : score.average_rank;
    return typeof rank === "number" && rank <= maxRank;
  }).length;
}

function percentile(sortedValues, percentileValue) {
  if (sortedValues.length === 0) {
    return null;
  }
  const index = Math.min(sortedValues.length - 1, Math.ceil(sortedValues.length * percentileValue) - 1);
  return sortedValues[index];
}

function format(value) {
  return typeof value === "number" ? value.toFixed(2) : "";
}

function formatDuration(value) {
  if (typeof value !== "number") {
    return "";
  }
  const totalSeconds = Math.round(value / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}m ${seconds}s`;
}
