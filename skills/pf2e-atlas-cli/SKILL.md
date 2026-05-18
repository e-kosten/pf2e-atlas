---
name: pf2e-atlas-cli
description: Use when answering Pathfinder 2e data questions with the local PF2e Atlas CLI, including record lookup, strict name resolution, search, filter discovery, and artifact readiness checks.
---

# PF2e Atlas CLI

Use the local `atlas` command for Pathfinder 2e record lookup, search, and filter discovery. This skill assumes `atlas` is installed on `PATH`; if it is missing, tell the user to install or expose the Atlas CLI before continuing.

## Readiness

When artifact readiness is uncertain or an Atlas command reports a missing or incompatible index, check setup first:

```bash
atlas setup --check --json
```

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
atlas record get actionspf2e:1kGNdIIhuglAjIp9 --json
```

Use `record resolve` when you need one record from a strict name or verified alias:

```bash
atlas record resolve "Treat Wounds" --family rule --json
```

Use `search` when you need a result set. Text queries run ranked retrieval; filter-only invocations return deterministic lists:

```bash
atlas search "low level healing spell" --family spell --json
atlas search --family equipment --rarity uncommon --json
```

Use `--retrieval fts`, `--retrieval vector`, or `--retrieval hybrid` only when the user explicitly asks to diagnose or tune retrieval behavior. Do not silently fall back to FTS to work around missing embeddings for normal content questions.

## Filter Discovery

Use discovery commands before constructing precise filters whose fields or values are not already known:

```bash
atlas filters fields
atlas filters fields --family spell
atlas filters values --field traits --family spell
atlas filters values --field metric --family creature --metric-prefix save.
```

The Rust query model is based on record families, metadata fields, traits, references, metrics, and canonical filter JSON. Use discovered field ids and values with `atlas search`, `atlas record resolve`, or `--filter-json`.

## JSON Output

Atlas JSON output uses a shared envelope. Successful command payloads are under `data`; command failures are under `error`. Parse the JSON envelope instead of scraping human output.
