---
name: pf2e-atlas-cli
description: Use when answering Pathfinder 2e data questions with the local PF2e Atlas CLI, including record lookup, strict name resolution, search, graph context, filter discovery, and artifact readiness checks.
---

# PF2e Atlas CLI

Use the local `atlas` command for Pathfinder 2e record lookup, search, graph context, and filter discovery. If the user names an explicit Atlas binary path, use that exact binary for command examples, smoke tests, and diagnostics. Otherwise assume `atlas` is installed on `PATH`; if it is missing, tell the user to install or expose the Atlas CLI before continuing.

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

If strict resolution reports ambiguity, inspect the returned alternatives and continue with the canonical key for the best match. Fetch the selected key with `record get` when you need full details. Use narrower filters only when you need to rerun resolution or the alternatives are too broad:

```bash
atlas record resolve "Shield Block" --kind feat --alternatives 5 --detail preview --json
atlas record get feats-srd:jM72TjJ965jocBV8 --detail standard --json
```

If strict resolution misses, use `search` with a narrow kind or pack filter instead of treating `record resolve` as fuzzy search. If the query appears misspelled or malformed, do not rely on Atlas typo tolerance; retry with corrected canonical spelling or nearby known PF2E terms when you can infer them safely from context, then use narrow search only if strict resolution still misses.

Use `search` when you need a result set. Text queries run ranked retrieval; filter-only invocations return deterministic lists:

```bash
atlas search "low level healing spell" --kind spell --detail preview
atlas search --kind equipment --rarity uncommon --detail preview
```

Translate clear structured constraints into filters instead of leaving them only in the text query. For example, "low-level" usually means `--max-level`, "level 3" means `--level 3`, "uncommon" means `--rarity uncommon`, "cheap" or a price ceiling means `--max-price`, and known traits should use `--trait` or `--any-trait`. Keep the remaining query text focused on the concept that cannot be expressed structurally:

```bash
atlas search "healing spell" --kind spell --max-level 2 --detail preview --limit 8
atlas search "protect an ally" --kind feat --trait champion --detail preview --limit 8
```

Use `--limit` for exploratory searches, filter-only lists, `similar` research, and backlink/uses graph calls unless the user needs a broad inventory. Small result sets are easier to judge and reduce context noise:

```bash
atlas search "low level healing spell" --kind spell --detail preview --limit 8
atlas search --kind equipment --rarity uncommon --detail preview --limit 20
atlas similar "Dirge of Doom" --kind spell --limit 8 --json
atlas graph uses conditionitems:TBSHQspnbcqxsmjL --limit 5 --json
```

Use `graph links` when you already have a canonical record key or strict resolvable record name and need connected one-hop reference context around that record. This is the right follow-up after `record resolve`, `record get`, or `search` identifies the key. Do not look for a separate rule-context command; the intended workflow is explicit record identification followed by graph context retrieval:

```bash
atlas graph links actionspf2e:1kGNdIIhuglAjIp9 --json
atlas graph links actionspf2e:1kGNdIIhuglAjIp9 --backlinks 4 --json
atlas graph links spells-srd:4koZzrnMXhhosn0D --json
atlas graph uses conditionitems:AJh5ex99aV6VTggg --limit 3 --json
atlas graph links bestiary-ability-glossary-srd:Tkd8sH4pwFIPzqTr --outgoing 0 --backlinks 6 --json
```

By default, graph links include outgoing references and omit backlinks. Use `--backlinks <count>` only when incoming context is useful, because backlinks can be noisy for common rules, actions, conditions, and traits. Use `graph uses` for backlinks-only context. Graph context is retrieval only; it does not synthesize answers.

PF2e actions such as Treat Wounds are Atlas `rule` records in the `actionspf2e` pack, not a separate `action` record kind. Use `--pack-name actionspf2e` or `--kind rule` when narrowing action records; do not use `--kind action`.

Use `similar` when you already have a seed record and want records like it. This uses the seed record's stored embedding, applies normal structured filters to candidate records, and adds modest shared-reference and shared-trait evidence:

```bash
atlas similar "Dirge of Doom" --kind spell --json
atlas similar feats-srd:jM72TjJ965jocBV8 --limit 12 --explain
```

For early research, prefer human-readable output with `--detail preview` or `--detail description` instead of JSON. Preview is best for scanning candidate result sets; description is best when the descriptive text is needed to judge fit. After identifying likely records, use `--detail standard --json` when you need the normal structured record context. Use `--detail full --include-raw --json` only when raw source metadata is directly relevant.

Use one Atlas process for a batch when you have multiple exact keys or strict names. `record get` accepts multiple canonical keys, and `record resolve` accepts multiple strict names or verified aliases:

```bash
atlas record get actionspf2e:1kGNdIIhuglAjIp9 equipment-srd:s1vB3HdXjMigYAnY --detail standard --json
atlas record resolve "Treat Wounds" "Trip" --pack-name actionspf2e --detail standard --json
```

## Saved Lists

Use `atlas lists` when the user asks you to persist a curated set of Atlas records for later review or editing. `atlas list` is an alias for the same command. Saved lists are durable local state stored beside the active Atlas artifact; they are not part of the rebuildable index and can be edited by the CLI, the web UI, or other agents.

Create lists with a stable id, human-friendly name, optional description, and optional grouping tags. Repeat `--tag` for multiple tags:

```bash
atlas lists create undead-research --name "Undead Research" --description "Campaign prep" --tag arc-one --tag necromancer
```

Add records by canonical key, strict name, or verified alias. `lists add` must resolve each record to one specific Atlas record before inserting it. If resolution misses or is ambiguous, inspect alternatives with `record resolve` or `search`; do not guess a key. Repeated refs and `--stdin` are supported for batch-style additions. Batch additions are non-atomic: inspect every item outcome and treat any failed item as needing follow-up.

```bash
atlas lists add undead-research "Skeleton Guard" --note "Compare low-level undead options"
atlas lists add undead-research actionspf2e:1kGNdIIhuglAjIp9
atlas lists add undead-research "Skeleton Guard" "Zombie Shambler" "Ghoul"
printf '%s\n' "Skeleton Guard" "Zombie Shambler" | atlas lists add undead-research --stdin
```

View lists with `lists ls` and `lists show`. `lists ls --json` includes item counts and tags. `lists show` preserves list order and reports stale or removed records as `status: "unresolved"` while retaining the saved snapshot, so unresolved items are not automatically deleted. Prefer compact output for verification and agent workflows:

```bash
atlas lists ls --json
atlas lists show undead-research --summary --json
atlas lists show undead-research --keys-only --json
atlas lists show undead-research --detail none --json
```

Edit metadata with `lists edit`. `--tag` replaces the full tag set, and `--clear-tags` removes all tags:

```bash
atlas lists edit undead-research --name "Undead Research" --description "Campaign prep"
atlas lists edit undead-research --tag arc-one --tag necromancer
atlas lists edit undead-research --clear-tags
```

Use raw JSON export/import for portable list snapshots. `export` writes the raw saved-list document, not the standard Atlas JSON envelope. Import fails by default when the target id already exists; pass `--id` to import under a new id or `--replace` to replace the final target id:

```bash
atlas lists export undead-research > undead-research.json
atlas lists import undead-research.json --id undead-research-copy --json
atlas lists import undead-research.json --replace --json
```

Remove an item only when the user asks to edit membership or when the record is clearly not part of the requested list. Delete a list only when the user explicitly wants the durable local state removed:

```bash
atlas lists remove undead-research actionspf2e:1kGNdIIhuglAjIp9
atlas lists delete undead-research
```

Use `--json` when the task requires structured parsing, batch result handling, exact field extraction, or diagnostics. Atlas JSON output uses a shared envelope: successful command payloads are under `data`, and top-level command, runtime, or input failures are under `error`. Ambiguous strict resolution returns `status: "error"` with `error.code: "record_resolution_ambiguous"` and structured alternatives under `error.data.result.alternatives`. Record-level or batch failures can appear inside `data.result.error` or `data.results[].error` for successful batch-style commands. Batch `record get` and `record resolve` payloads also include `data.partial`; when it is `true`, inspect each `data.results[].error` before trusting the batch. Parse the JSON envelope instead of scraping human output: first check top-level `error`, then inspect `error.data` when present, then inspect `data.result.error` for single-record commands or each `data.results[].error` for batch commands before trusting record fields. For readiness and validation commands, an invalid artifact can still produce a successful JSON envelope with `status: "ok"` and `data.valid: false`; do not treat the top-level status alone as artifact readiness.

Use `--retrieval fts`, `--retrieval vector`, or `--retrieval hybrid` only when the user explicitly asks to diagnose or tune retrieval behavior. Do not silently fall back to FTS to work around missing embeddings for normal content questions. Add `--explain --json` when comparing retrieval behavior or investigating why a ranked result set looks wrong; JSON output carries the useful rank, score, and lane details.

Relationship filters require canonical record keys. Resolve or fetch a record first when correctness matters, then pass the verified key to `--references` or `--referenced-by`. A syntactically valid but nonexistent key can produce an empty result set, so do not treat zero results as proof of no relationship if the key was guessed:

```bash
atlas record resolve "Battle Medicine" --kind feat --detail standard --json
atlas search --referenced-by feats-srd:wYerMk6F1RZb0Fwt --detail preview --limit 10
atlas search --references actionspf2e:1kGNdIIhuglAjIp9 --detail preview --limit 10
```

Read the relationship direction from the candidate record's perspective: `--referenced-by KEY` finds records linked from `KEY`, while `--references KEY` finds records that link to `KEY`.

## Filter Discovery

Use discovery commands before constructing precise filters whose fields or values are not already known:

```bash
atlas filters fields
atlas filters fields --kind spell
atlas filters values --field traits --kind spell
atlas filters values --field metric --kind creature --metric-query save
```

Filter discovery defaults to human-readable output for scanning. Add `--json` when you need structured field/value payloads, exact counts, or canonical filter data. For sampled text fields, use `--sample-limit` or its alias `--limit`; ordinary enumerable value lists do not need a limit.

The Rust query model is based on record kinds, metadata fields, traits, references, metrics, and canonical filter JSON. Use discovered field ids and values with `atlas search`, `atlas record resolve`, or `--filter-json`.

Prefer convenience filters when they express the query clearly: `--kind`, `--pack-name`, `--pack-label`, `--rarity`, `--publication-title`, `--level`, `--min-level`, `--max-level`, `--price`, `--min-price`, `--max-price`, `--trait`, `--any-trait`, `--references`, `--referenced-by`, and `--metric`. Use `--filter-json` for canonical filter trees that cannot be expressed cleanly with convenience flags, and do not combine `--filter-json` with convenience filters in the same command.

Discover metric keys before constructing metric predicates unless the exact key is already known. Prefer `--metric-query` for natural metric terms like "save", "armor", "speed", "perception", or "hp"; use `--metric-label`, `--metric-prefix`, or `--metric` when you already know the label, prefix, or key:

```bash
atlas filters values --field metric --kind creature --metric-query save --json
atlas filters values --field metric --kind creature --metric-label "Armor Class" --json
atlas filters values --field metric --kind creature --metric-prefix speed. --json
```

For numeric or stat-like questions, metric predicates are usually better than natural-language search text. Metric predicates often contain shell metacharacters, so quote them:

```bash
atlas search --kind creature --metric 'ac.value>=25' --detail preview --limit 8
atlas search --kind creature --metric 'speed.fly.value>=100' --metric 'ac.value>=30' --detail preview --limit 8
```
