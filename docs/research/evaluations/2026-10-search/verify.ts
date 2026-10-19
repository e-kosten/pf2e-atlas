// Offline integrity and point-metric replay. No source corpus/model required.
// node --experimental-strip-types docs/research/evaluations/2026-10-search/verify.ts
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
const base = path.dirname(fileURLToPath(import.meta.url));
const json = (p) => JSON.parse(fs.readFileSync(path.join(base, p), "utf8"));
const lines = (p) =>
  fs
    .readFileSync(path.join(base, p), "utf8")
    .trim()
    .split("\n")
    .map((s) => JSON.parse(s));
const mean = (xs) => xs.reduce((a, b) => a + b, 0) / xs.length;
const dcg = (xs) => xs.reduce((s, x, i) => s + x / Math.log2(i + 2), 0);
let compared = 0;
function close(actual, expected, label) {
  assert.equal(actual === null, expected === null, label);
  if (actual !== null)
    assert(
      Math.abs(actual - expected) < 1e-11,
      `${label}: ${actual} != ${expected}`,
    );
  compared++;
}
const manifest = json("manifest.json");
assert.equal(
  new Set(manifest.outputs.map((f) => f.path)).size,
  manifest.outputs.length,
);
for (const f of manifest.outputs) {
  const bytes = fs.readFileSync(path.join(base, f.path));
  assert.equal(bytes.length, f.bytes, f.path);
  assert.equal(
    crypto.createHash("sha256").update(bytes).digest("hex"),
    f.sha256,
    f.path,
  );
}
const judgments = lines("semantic/judgments.jsonl").map((j) => ({
  ...j,
  final: j.opinions[j.finalOpinion],
}));
for (const split of ["development", "fresh"]) {
  const qs = json(`semantic/${split}-queries.json`).concepts;
  const ranks = lines(`semantic/${split}-rankings.jsonl`);
  const report = json(`semantic/${split}-metrics.json`);
  report.rows = [];
  const all = judgments.filter((j) => j.cohort === split);
  const observations = new Map(
    qs.flatMap((q) => q.observations.map((o) => [o.id, q.id])),
  );
  const seen = new Set();
  for (const r of ranks) {
    const cfg = r.model + "/" + r.representation;
    assert.equal(observations.get(r.id), r.concept);
    assert(!seen.has(r.id + "|" + cfg));
    seen.add(r.id + "|" + cfg);
    const pool = all.filter((j) => j.observations.some((o) => o.id === r.id));
    const byKey = new Map(pool.map((j) => [j.key, j.final]));
    assert.equal(byKey.size, pool.length);
    const top = r.top10.map((x) => {
      assert(byKey.has(x.key), x.key);
      return byKey.get(x.key);
    });
    assert.equal(new Set(r.top10.map((x) => x.key)).size, r.top10.length);
    assert(
      pool.every(
        (j) =>
          Number.isInteger(j.final.grade) &&
          j.final.grade >= 0 &&
          j.final.grade <= 3,
      ),
    );
    const ideal = dcg(
      pool
        .map((j) => 2 ** j.final.grade - 1)
        .sort((a, b) => b - a)
        .slice(0, 10),
    );
    const metrics = {
      ndcg: ideal ? dcg(top.map((g) => 2 ** g.grade - 1)) / ideal : null,
      useful5Bounds: top.slice(0, 5).filter((g) => g.grade >= 2).length / 5,
      anyUseful5Bounds: +top.slice(0, 5).some((g) => g.grade >= 2),
      harmful3Bounds: top.slice(0, 3).filter((g) => g.contradictory === true)
        .length,
      anyHarmful3Bounds: +top.slice(0, 3).some((g) => g.contradictory === true),
    };
    report.rows.push({
      id: r.id,
      concept: r.concept,
      config: cfg,
      ndcgBounds: [metrics.ndcg, metrics.ndcg],
      ...Object.fromEntries(
        Object.entries(metrics).map(([k, v]) => [k, k === "ndcg" ? v : [v, v]]),
      ),
    });
  }
  assert.equal(ranks.length, observations.size * 8);
  for (const [cfg, values] of Object.entries(report.overall))
    for (const [metric, summary] of Object.entries(values)) {
      const conceptMeans = qs.map((q) =>
        mean(
          report.rows
            .filter((r) => r.config === cfg && r.concept === q.id)
            .map((r) => r[metric][0]),
        ),
      );
      for (const bound of summary.bounds)
        close(mean(conceptMeans), bound, `${split}/${cfg}/${metric}`);
    }
}
const pool = lines("lexical/pool.jsonl"),
  grades = lines("lexical/judgments.jsonl"),
  ranks = lines("lexical/rankings.jsonl");
const byID = new Map(grades.map((g) => [g.id, g]));
assert.equal(byID.size, grades.length);
for (const g of grades) {
  const opinions = g.adjudication ? [g.adjudication] : g.opinions;
  const values = opinions.map((o) => o.grade);
  assert(values.every((v) => Number.isInteger(v) && v >= 0 && v <= 3));
  close(mean(values.map((v) => 2 ** v - 1)), g.gain, g.id + "/gain");
  close(mean(values.map((v) => +(v >= 2))), g.useful, g.id + "/useful");
  close(
    mean(opinions.map((o) => +o.contradictory)),
    g.opposite,
    g.id + "/opposite",
  );
  const alternatives = (g.adjudication?.alternativeGrades ?? [])
    .filter((x) => Number.isInteger(x.grade))
    .map((x) => x.grade);
  const range = alternatives.length
    ? [g.adjudication.grade, ...alternatives]
    : values;
  for (const [field, compute] of [
    ["low", Math.min],
    ["high", Math.max],
  ]) {
    close(
      2 ** compute(...range) - 1,
      g[field + "Gain"],
      g.id + "/" + field + "Gain",
    );
    close(
      +(compute(...range) >= 2),
      g[field + "Useful"],
      g.id + "/" + field + "Useful",
    );
  }
}
const queries = ["development-v2", "holdout-v2"].flatMap((split) =>
  json(`lexical/${split}-queries.json`).map((q) => ({ ...q, split })),
);
assert.equal(
  new Set(ranks.map((r) => [r.split, r.query.id, r.profile].join("|"))).size,
  ranks.length,
);
for (const mode of ["mean", "low", "high"]) {
  const report = json(`lexical/metrics-${mode}.json`);
  report.rows = [];
  const gain = mode === "mean" ? "gain" : mode + "Gain",
    useful = mode === "mean" ? "useful" : mode + "Useful";
  for (const exclusion of report.excluded) {
    const actual = [
      ...new Set(
        pool
          .filter((p) => p.split === exclusion.split && !byID.has(p.id))
          .map((p) => p.cluster),
      ),
    ].sort();
    assert.deepEqual(actual, [...exclusion.clusters].sort());
  }
  let count = 0;
  for (const r of ranks) {
    const q = queries.find((q) => q.split === r.split && q.id === r.query.id);
    assert(q);
    if (
      report.excluded
        .find((e) => e.split === r.split)
        .clusters.includes(q.cluster)
    )
      continue;
    const scoped = pool.filter(
      (p) => p.split === r.split && p.context === q.id,
    );
    const gs = new Map(scoped.map((p) => [p.key, byID.get(p.id)]));
    assert([...gs.values()].every(Boolean));
    const ideal = dcg(
      [...gs.values()]
        .map((g) => g[gain])
        .sort((a, b) => b - a)
        .slice(0, 10),
    );
    for (const [policy, top] of Object.entries(r.policies)) {
      const selected = top.map((x) => {
        assert(gs.has(x.key), x.key);
        return gs.get(x.key);
      });
      const computed = {
        ndcg10: ideal ? dcg(selected.map((g) => g[gain])) / ideal : null,
        usefulCountAt5: selected.slice(0, 5).reduce((n, g) => n + g[useful], 0),
        successAt5Low: +selected.slice(0, 5).some((g) => g.lowUseful > 0),
        successAt5High: +selected.slice(0, 5).some((g) => g.highUseful > 0),
        oppositeFractionAt10: mean(selected.map((g) => g.opposite)),
        referenceTop1: +q.referenceKeys.includes(top[0]?.key),
      };
      report.rows.push({
        split: r.split,
        context: q.id,
        cluster: q.cluster,
        category: q.category,
        profile: r.profile,
        policy,
        ...computed,
      });
      count++;
    }
  }
  assert.equal(count, report.rows.length);
  for (const summary of report.summaries) {
    const rows = report.rows.filter(
      (r) =>
        r.split === summary.split &&
        r.profile === summary.profile &&
        r.policy === summary.policy &&
        (summary.category === "all" || r.category === summary.category),
    );
    const groups = new Map();
    for (const r of rows) {
      if (!groups.has(r.cluster)) groups.set(r.cluster, []);
      groups.get(r.cluster).push(r);
    }
    assert.equal(groups.size, summary.clusters);
    for (const k of [
      "ndcg10",
      "usefulCountAt5",
      "successAt5Low",
      "successAt5High",
      "oppositeFractionAt10",
      "referenceTop1",
    ])
      close(
        mean([...groups.values()].map((xs) => mean(xs.map((x) => x[k])))),
        summary[k],
        `lexical summary/${mode}/${k}`,
      );
  }
}
console.log(
  `Verified ${manifest.outputs.length} exported files and ${compared} point-metric values. Bootstrap intervals and source interpretations are preserved, not independently rerun here.`,
);
