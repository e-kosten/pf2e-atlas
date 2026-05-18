---
name: pf2e-atlas-cli
description: Use when answering Pathfinder 2e data questions with the local PF2e Atlas CLI, including record lookup, strict name resolution, search, filter discovery, and artifact readiness checks.
---

# PF2e Atlas CLI

Use the local `atlas` command for Pathfinder 2e record lookup, search, and filter discovery. This skill assumes `atlas` is installed on `PATH`; if it is missing, tell the user to install or expose the Atlas CLI before continuing.

## Readiness

Do not run readiness checks before normal lookup, resolution, search, or filter discovery. Assume the installed Atlas runtime is ready unless a command fails, returns a readiness/index error, or the user explicitly asks to check setup or diagnose the artifact.

When an Atlas command reports a missing or incompatible index, check setup first. Use JSON for readiness and diagnostics because those commands have machine-readable validity fields and exit classes:

```bash
atlas setup --check --json
```

For direct artifact diagnostics after a failure, use the fast index check:

```bash
atlas index check --json
```

Use record-only readiness only when semantic search is not needed:

```bash
atlas index check --no-embeddings --json
```

Do not run deep validation by default. `atlas index validate --json` is a diagnostic command for debugging an artifact problem after setup/check/search reports a failure or when the user explicitly asks for validation.

Text search expects the full Atlas runtime, including embeddings. If semantic readiness is missing or incompatible, report the readiness error and tell the user to run full setup:

```bash
atlas setup --json
```

Use record-only setup only when the user is preparing an environment for exact record lookup and explicitly does not need semantic search:

```bash
atlas setup --no-embeddings --json
```

## Command Selection

Use `record get` when you already have a canonical `pack:id` record key:

```bash
atlas record get actionspf2e:1kGNdIIhuglAjIp9 --detail description
```

Use `record resolve` when you need one record from a strict name or verified alias. A verified alias is an alias already confirmed by the user, returned by a previous Atlas result, or known from canonical PF2E naming/remaster context; do not guess aliases just to make strict resolution pass:

```bash
atlas record resolve "Treat Wounds" --pack-name actionspf2e --detail description
```

If strict resolution reports ambiguity, inspect the returned alternatives before broadening to search. Rerun with an explicit alternative count or different detail level when needed:

```bash
atlas record resolve "Treat Wounds" --pack-name actionspf2e --alternatives 5 --detail preview
```

If strict resolution misses, use `search` with a narrow family or pack filter instead of treating `record resolve` as fuzzy search. If the query appears misspelled or malformed, do not rely on Atlas typo tolerance; retry with corrected canonical spelling or nearby known PF2E terms when you can infer them safely from context, then use narrow search only if strict resolution still misses.

Use `search` when you need a result set. Text queries run ranked retrieval; filter-only invocations return deterministic lists:

```bash
atlas search "low level healing spell" --family spell --detail preview
atlas search --family equipment --rarity uncommon --detail preview
```

Use `--limit` for exploratory searches and filter-only lists unless the user needs a broad inventory. Small result sets are easier to judge and reduce context noise:

```bash
atlas search "low level healing spell" --family spell --detail preview --limit 8
atlas search --family equipment --rarity uncommon --detail preview --limit 20
```

For early research, prefer human-readable output with `--detail preview` or `--detail description` instead of JSON. Preview is best for scanning candidate result sets; description is best when the descriptive text is needed to judge fit. After identifying likely records, use `--detail standard --json` when you need the normal structured record context. Use `--detail full --include-raw --json` only when raw source metadata is directly relevant.

Use one Atlas process for a batch when you have multiple exact keys or strict names. `record get` accepts multiple canonical keys, and `record resolve` accepts multiple strict names or verified aliases:

```bash
atlas record get actionspf2e:1kGNdIIhuglAjIp9 equipment-srd:s1vB3HdXjMigYAnY --detail standard --json
atlas record resolve "Treat Wounds" "Trip" --pack-name actionspf2e --detail standard --json
```

Use `--json` when the task requires structured parsing, batch result handling, exact field extraction, or diagnostics. Atlas JSON output uses a shared envelope: successful command payloads are under `data`, and top-level command, runtime, or input failures are under `error`. Ambiguous strict resolution returns `status: "error"` with `error.code: "record_resolution_ambiguous"` and structured alternatives under `error.data.result.alternatives`. Record-level or batch failures can appear inside `data.result.error` or `data.results[].error` for successful batch-style commands. Parse the JSON envelope instead of scraping human output: first check top-level `error`, then inspect `error.data` when present, then inspect `data.result.error` for single-record commands or each `data.results[].error` for batch commands before trusting record fields. For readiness and validation commands, an invalid artifact can still produce a successful JSON envelope with `status: "ok"` and `data.valid: false`; do not treat the top-level status alone as artifact readiness.

Use `--retrieval fts`, `--retrieval vector`, or `--retrieval hybrid` only when the user explicitly asks to diagnose or tune retrieval behavior. Do not silently fall back to FTS to work around missing embeddings for normal content questions. Add `--explain` when comparing retrieval behavior or investigating why a ranked result set looks wrong.

## Filter Discovery

Use discovery commands before constructing precise filters whose fields or values are not already known:

```bash
atlas filters fields
atlas filters fields --family spell
atlas filters values --field traits --family spell
atlas filters values --field metric --family creature --metric-label save
```

Filter discovery defaults to human-readable output for scanning. Add `--json` when you need structured field/value payloads, exact counts, or canonical filter data.

The Rust query model is based on record families, metadata fields, traits, references, metrics, and canonical filter JSON. Use discovered field ids and values with `atlas search`, `atlas record resolve`, or `--filter-json`.

Prefer convenience filters when they express the query clearly: `--family`, `--pack-name`, `--pack-label`, `--rarity`, `--publication-title`, `--level`, `--min-level`, `--max-level`, `--price`, `--min-price`, `--max-price`, `--trait`, `--any-trait`, `--references`, `--referenced-by`, and `--metric`. Use `--filter-json` for canonical filter trees that cannot be expressed cleanly with convenience flags, and do not combine `--filter-json` with convenience filters in the same command.

Discover metric keys before constructing metric predicates unless the exact key is already known:

```bash
atlas filters values --field metric --family creature --metric-label "Armor Class" --json
atlas filters values --field metric --family creature --metric-prefix speed. --json
```

Metric predicates often contain shell metacharacters, so quote them:

```bash
atlas search --family creature --metric 'ac.value>=25' --detail preview --limit 8
atlas search --family creature --metric 'speed.fly.value>=100' --metric 'ac.value>=30' --detail preview --limit 8
```
