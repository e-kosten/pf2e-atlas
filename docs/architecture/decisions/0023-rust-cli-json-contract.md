# 0023 Rust CLI JSON Contract

## Status

Accepted

## Context

The Rust CLI is becoming the primary local agent surface for artifact validation, record retrieval, strict record resolution, and search. These commands need one durable machine-readable contract instead of command-local JSON shapes that drift as each surface lands.

The CLI also needs to distinguish stored or presented records from retrieval results. A record is the projection of a persisted presentation document. A result wraps a record with the metadata that explains why it was returned, such as resolution or search match information.

## Decision

All Rust CLI JSON output uses one top-level envelope:

```json
{
  "status": "ok",
  "data": {}
}
```

Command failures use the error envelope:

```json
{
  "status": "error",
  "error": {
    "code": "record_not_found",
    "message": "record not found: actions:no-such-record"
  }
}
```

Successful payloads live under `data`. Errors live under `error`. Commands do not emit a `command` field because the caller already knows the invoked command. Optional fields are omitted when absent or empty. CLI JSON field names are `snake_case`.

Record-facing commands use `summary`, `standard`, and `full` detail levels. All detail levels keep the same record JSON schema and vary only by hydration depth. The Rust CLI does not accept `minimal`, `compact`, or TypeScript-specific detail values. Raw source JSON is exposed only by explicit opt-in flags and appears as top-level `source_json` in the relevant command payload.

The shared record DTO is a projection of the renderer-neutral presentation record. Result DTOs wrap records with search or resolution metadata. Batch record commands return per-item results with per-item errors when some keys or queries miss.

Record sections use stable section and block kinds. Rich `ContentDocument` blocks are rendered to markdown for Phase 5 JSON output. Structured `ContentDocument` JSON and original Foundry rich-text output are future formats, not part of the initial CLI contract.

Validation commands are successful command executions when validation runs, even when the artifact is invalid. `atlas index validate --json` returns `status: "ok"` with `data.valid: false` for invalid artifacts and exits with code `3`. Top-level `status: "error"` is reserved for cases where the command cannot run.

## Consequences

Existing Rust JSON commands must route through the shared envelope helper instead of serializing command-local top-level objects.

The standard process exit classes are:

- `0`: success
- `1`: domain no-result or partial domain misses
- `2`: invalid user input
- `3`: runtime, index, artifact, or environment failure

Search, record retrieval, record resolution, graph, and future discovery commands should reuse the shared record/result DTOs instead of defining command-specific record fragments.

Future work that adds structured content output should extend the content block format deliberately and preserve the current markdown projection as a stable default.
