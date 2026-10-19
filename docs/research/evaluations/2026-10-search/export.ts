// One-shot archive exporter; no product dependency and no inference or grading.
// node --experimental-strip-types export.ts /path/to/original/repository
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
const root = path.resolve(process.argv[2] ?? ".");
const out = path.dirname(fileURLToPath(import.meta.url));
const sem = "scratch/full-corpus-search/embedding-full";
const lex = "scratch/search-fusion-review";
const sources = new Map();
const sha = (bytes) => crypto.createHash("sha256").update(bytes).digest("hex");
function read(rel) {
  const bytes = fs.readFileSync(path.join(root, rel));
  sources.set(rel, { path: rel, sha256: sha(bytes), bytes: bytes.length });
  return bytes;
}
const json = (rel) => JSON.parse(read(rel).toString());
function clean(value) {
  if (Array.isArray(value)) return value.map(clean);
  if (value && typeof value === "object")
    return Object.fromEntries(
      Object.entries(value).flatMap(([k, v]) => {
        if (["quote", "excerpt"].includes(k))
          return [[k + "Sha256", sha(String(v))]];
        if (k === "rationale" && typeof v === "string") {
          const redacted = v
            .replace(
              /\([^)]*<(?:p|div|table)\b[^)]*\)/g,
              "[source excerpt omitted]",
            )
            .replace(/“[^”]*”|"[^"\n]*"/g, "[quotation omitted]");
          if (redacted !== v)
            return [
              [k, clean(redacted)],
              ["originalRationaleSha256", sha(v)],
            ];
        }
        return [[k, clean(v)]];
      }),
    );
  if (typeof value === "string")
    return value
      .replaceAll(root + "/", "")
      .replaceAll(/\/Users\/[^\s"']+?\/pathfinder-2e-foundry-mcp\//g, "");
  return value;
}
const outputs = [];
// Keep tabular arrays one row per line instead of expanding every numeric cell.
function format(value) {
  if (Array.isArray(value) && value.length === 0) return "[]";
  if (Array.isArray(value))
    return (
      "[\n" + value.map((row) => "  " + JSON.stringify(row)).join(",\n") + "\n]"
    );
  if (value && typeof value === "object")
    return (
      "{\n" +
      Object.entries(value)
        .map(
          ([key, entry]) =>
            "  " +
            JSON.stringify(key) +
            ": " +
            (Array.isArray(entry)
              ? format(entry).replaceAll("\n", "\n  ")
              : JSON.stringify(entry)),
        )
        .join(",\n") +
      "\n}"
    );
  return JSON.stringify(value);
}
function write(rel, value, lines = false) {
  assert(!outputs.some((f) => f.path === rel), "Duplicate output " + rel);
  const dest = path.join(out, rel);
  fs.mkdirSync(path.dirname(dest), { recursive: true });
  const bytes = lines
    ? value.map((x) => JSON.stringify(clean(x))).join("\n") + "\n"
    : format(clean(value)) + "\n";
  fs.writeFileSync(dest, bytes);
  outputs.push({
    path: rel,
    sha256: sha(bytes),
    bytes: Buffer.byteLength(bytes),
  });
}
function text(rel, dest) {
  const bytes = read(rel);
  fs.mkdirSync(path.dirname(path.join(out, dest)), { recursive: true });
  fs.writeFileSync(path.join(out, dest), bytes);
  outputs.push({ path: dest, sha256: sha(bytes), bytes: bytes.length });
}
function histories(dirs) {
  const rows = [];
  function walk(dir) {
    for (const e of fs
      .readdirSync(path.join(root, dir), { withFileTypes: true })
      .sort((a, b) => a.name.localeCompare(b.name))) {
      const f = dir + "/" + e.name;
      if (
        e.isDirectory() &&
        ![
          "node_modules",
          "captures",
          "source-context",
          "preview",
          "harness",
        ].includes(e.name)
      )
        walk(f);
      else if (e.isFile() && /grades.*\.json$/.test(e.name)) {
        const candidate = JSON.parse(
          fs.readFileSync(path.join(root, f), "utf8"),
        );
        if (Array.isArray(candidate.grades)) {
          const data = json(f);
          rows.push({ source: f, ...data });
        }
      }
    }
  }
  for (const d of dirs) walk(d);
  return rows;
}
write(
  "semantic/judgments.jsonl",
  json(sem + "/user-review/all-judgments.json").records,
  true,
);
write("semantic/opinion-history.jsonl", histories([sem + "/execution"]), true);
for (const split of ["development", "fresh"]) {
  const dir = sem + "/execution/" + split;
  write(`semantic/${split}-queries.json`, json(dir + "/concepts.json"));
  write(
    `semantic/${split}-rankings.jsonl`,
    json(dir + "/rankings.json").map((r) => ({
      model: r.model,
      representation: r.representation,
      id: r.id,
      concept: r.concept,
      query: r.query,
      args: r.args,
      rankingPolicy: r.rankingPolicy,
      diagnostics: r.diagnostics,
      top10: r.top10.map(({ semantic, ...x }) => x),
      referenceRanks: r.referenceRanks,
    })),
    true,
  );
  write(`semantic/${split}-metrics.json`, json(dir + "/quality-results.json"));
  write(
    `semantic/${split}-adjudications.json`,
    json(dir + "/adjudications.json"),
  );
  write(
    `semantic/${split}-review-assembly.json`,
    json(dir + "/review-assembly.json"),
  );
}
for (const rel of [
  "runtime/effective-contracts.json",
  "evaluation-design/protocol.json",
  "execution/search-performance.json",
  "execution/costs.json",
  "execution/review-reliability.json",
  "execution/plausible-sensitivity.json",
  "execution/opinion-sensitivity.json",
  "execution/expanded-review/coverage-amendment.json",
  "execution/quality-finalization.json",
  "execution/quality-unblinding-ready.json",
  "revised/manifest.json",
  "corpus/manifest.json",
])
  write("semantic/method/" + rel.replaceAll("/", "--"), json(sem + "/" + rel));
for (const [rel, dest] of [
  ["evaluation/GRADING.md", "grading.md"],
  ["evaluation/TASKS.md", "reviewer-tasks.md"],
  ["evaluation-design/protocol.md", "protocol.md"],
  ["context/README.md", "parent-context.md"],
  ["REVIEW-PLAN.md", "review-plan.md"],
])
  text(sem + "/" + rel, "semantic/method/" + dest);
const lexPool = [];
for (const split of ["development-v2", "holdout-v2"]) {
  write(
    `lexical/${split}-queries.json`,
    json(lex + "/" + split + "/queries.json"),
  );
  lexPool.push(
    ...json(lex + "/" + split + "/pool-private.json").map((x) => ({
      ...x,
      split,
    })),
  );
}
const matrix = lex + "/projection-fusion-followup";
lexPool.push(
  ...json(matrix + "/pool-private.json").map((x) => ({
    ...x,
    cluster: x.context.replace(/-[uf]$/, ""),
  })),
);
assert.equal(
  new Set(lexPool.map((x) => x.split + "|" + x.id)).size,
  lexPool.length,
);
write("lexical/pool.jsonl", lexPool, true);
write(
  "lexical/judgments.jsonl",
  json(matrix + "/resolved-utilities.json").grades,
  true,
);
write(
  "lexical/unresolved.json",
  json(matrix + "/resolved-utilities.json").unresolved,
);
write(
  "lexical/opinion-history.jsonl",
  histories([
    lex + "/development-v2",
    lex + "/holdout-v2",
    lex + "/relevance-realignment",
    matrix,
  ]),
  true,
);
write(
  "lexical/rankings.jsonl",
  json(matrix + "/rankings.json").map((c) => ({
    split: c.split,
    query: c.query,
    profile: c.profile,
    policies: Object.fromEntries(
      Object.entries(c.policies).map(([k, v]) => [k, v.slice(0, 10)]),
    ),
    referenceRanks: Object.fromEntries(
      Object.entries(c.policies).map(([k, v]) => [
        k,
        c.query.referenceKeys.map(
          (key) => v.findIndex((x) => x.key === key) + 1 || null,
        ),
      ]),
    ),
  })),
  true,
);
for (const mode of ["mean", "low", "high"])
  write(`lexical/metrics-${mode}.json`, json(matrix + `/metrics-${mode}.json`));
write("lexical/contrasts.json", json(matrix + "/contrasts.json"));
for (const [rel, dest] of [
  ["projection-fusion-followup/GRADING.md", "grading.md"],
  ["projection-fusion-followup/PROTOCOL.md", "protocol.md"],
  ["projection-fusion-followup/AUDIT.md", "audit.md"],
  ["relevance-realignment/PROTOCOL.md", "realignment-protocol.md"],
  ["relevance-realignment/CALIBRATION.md", "calibration.md"],
])
  text(lex + "/" + rel, "lexical/method/" + dest);
write("manifest.json", {
  format: 1,
  scope:
    "Semantic eight-configuration study and final lexical/fusion matrix; not all historical experiments.",
  transformation:
    "JSON retains decisions, rationales and numeric results. Literal evidence quotes become SHA256 digests; source paths become repository-relative. Ranked results retain top10 and reference ranks, not full corpus ranks or passage bodies. Original input hashes bind unmodified local archives.",
  sources: [...sources.values()].sort((a, b) => a.path.localeCompare(b.path)),
  outputs,
});
console.log(
  `${sources.size} source files; ${outputs.length} exports; ${outputs.reduce((n, x) => n + x.bytes, 0)} bytes`,
);
