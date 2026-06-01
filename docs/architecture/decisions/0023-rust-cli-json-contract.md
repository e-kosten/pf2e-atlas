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

JSON payloads are written to stdout. Routine progress is not part of the JSON contract and is controlled separately from payload output. By default, progress renders only for human terminal sessions and is suppressed for JSON or non-terminal automation; users can force or suppress progress with the global `--progress` option or the `ATLAS_PROGRESS` environment variable.

Record-facing commands use `summary`, `preview`, `description`, `standard`, and `full` detail levels. `summary` carries identity and summary facts. `preview` carries summary facts, compact family-specific scan sections such as creature defense or movement when the presentation recipe has them, and a truncated description; it still excludes the full description and details/source-heavy sections. `description` adds the full description section without the extra standard-detail sections, `standard` carries the normal section breakdown, and `full` adds full source metadata. All detail levels keep the same record JSON schema and vary only by hydration depth. The Rust CLI does not accept `minimal`, `compact`, or TypeScript-specific detail values. Raw source JSON is exposed only by explicit opt-in flags and appears as top-level `source_json` in the relevant command payload.

The shared record DTO is a projection of the renderer-neutral presentation record. Result DTOs wrap records with search or resolution metadata. Batch record commands return per-item results with per-item errors when some keys or queries miss.

Record sections use stable section and block kinds. Rich `ContentDocument` blocks are rendered to markdown for Phase 5 JSON output. Structured `ContentDocument` JSON and original Foundry rich-text output are future formats, not part of the initial CLI contract. Batch record payloads include `data.partial` alongside per-item errors so automation can distinguish complete success from partial domain misses without relying only on the process exit code.

Ambiguous strict record-reference errors should include structured alternatives when the command has resolved candidate records. `record resolve`, graph seed resolution, and similar-record seed resolution report `error.code: "record_resolution_ambiguous"` and place parseable candidate records under `error.data.result.alternatives`; callers should not need to scrape candidate names or keys from the human-readable `message`.

Readiness and validation commands are successful command executions when checks run, even when the artifact is invalid. `atlas index check --json` and `atlas index validate --json` return `status: "ok"` with `data.valid: false` for invalid artifacts and exit with code `3`. Top-level `status: "error"` is reserved for cases where the command cannot run.

Setup reports separate read-only diagnostics from mutating or planned repair work. `data.checks` contains read-only work such as source analysis and artifact readiness checks. `data.actions` contains mutating operations and repair decisions such as source fetch, embedding model preparation, and index build. `atlas setup --check` may still perform entries in `checks`, but it must not perform mutating `actions`.

When setup is not ready, JSON output may include `data.not_ready_reasons` as a concise list derived from planned, blocked, or failed checks/actions. Each entry carries a diagnostic `code`, the user-facing `message`, and the related action/status so automation can distinguish common cases such as `artifact_stale_source_signature` without reinterpreting free-form action reasons.

## Consequences

Existing Rust JSON commands must route through the shared envelope helper instead of serializing command-local top-level objects.

The standard process exit classes are:

- `0`: success
- `1`: domain no-result or partial domain misses
- `2`: invalid user input
- `3`: runtime, index, artifact, or environment failure

Search, record retrieval, record resolution, graph, and future discovery commands should reuse the shared record/result DTOs instead of defining command-specific record fragments.

Future work that adds structured content output should extend the content block format deliberately and preserve the current markdown projection as a stable default.
