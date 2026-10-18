# 0023 Rust CLI JSON Contract

## Status

Partially superseded by ADR 0046. Shared JSON envelopes, output separation and
exit classes remain; source-backed app DTOs replace the historical record/content
payload and hydration model below. Raw source inspection belongs to atlas-dev.

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

Whole-command failures use `error`. Data-bearing responses use `data`; record batches retain it even when some or all inputs fail. Commands do not emit a `command` field because the caller already knows the invoked command. Optional fields are omitted when absent or empty. CLI JSON field names are `snake_case`.

JSON payloads are written to stdout. Routine progress is not part of the JSON contract and is controlled separately from payload output. By default, progress renders only for human terminal sessions and is suppressed for JSON or non-terminal automation; users can force or suppress progress with the global `--progress` option or the `ATLAS_PROGRESS` environment variable.

### Current record batch contract

`record get` and `record resolve` emit exactly one JSON document, including for a
single input. The top-level `status` summarizes the batch: `ok` means every input
succeeded, `partial` means some succeeded and some failed or were unprocessed,
and `error` means none succeeded. `data.results` preserves input order and
duplicates. There are no aggregate counts or redundant partial flag.

Each result contains the original `input` and a `status` discriminator:

- `ok`: `value` contains the source-backed CLI reading response or strict
  resolution result (`total` and bounded `alternatives`).
- `error`: `error` contains `code`, `message`, and optional `details`, `retryable`
  and `recoverable_action`, preserving the app error contract.
- `unprocessed`: `reason: "prior_runtime_failure"` indicates that execution
  stopped after an earlier runtime failure; it does not mean the input was missing.

Malformed keys, missing records and ambiguous names are per-input failures;
other inputs still run. Runtime failures stop the batch and retain prior results.
Command grammar/filter errors and workspace admission failures use the top-level
error envelope before batch work starts. Each read retains its own checked
artifact generation; the CLI does not hold one transaction across the batch.

The process exit is the highest failure class encountered: runtime (3), invalid
input (2), domain miss/ambiguity (1), success (0). Terminal and JSON modes use the
same classification. The JSON status describes completeness, not the exit class.
Top-level and per-input app errors use the same `details` field; graph ambiguity
provides `details.candidates`, while resolution provides `details.total` and
`details.alternatives`. Recovery hints are preserved when supplied by app-service.

CLI record values reuse app-service semantic facts with an explicit reading layout:
`identity` describes the selected content (including effective rank-preview facts),
`parent` is included only for embedded selections, and `selected` retains the
exact navigation address. `content` holds `classification`, prepared `fields`,
structured `body`, and `owned` navigation. Root identity is serialized once;
there are no `record` or `presentation` aliases. Artifact generation and
relationship evidence remain alongside these sections. This is CLI serialization
policy, not a second source model; the web's internal reading DTO is unchanged.
The `owned`/`owners` terminology is tracked separately under CF-04.

JSON record values are complete selected reference responses from app-service;
explicit terminal `--detail` conflicts with `--json`. Human reading supports
summary, standard and full. Source inspection uses `record source`; it is separate
from the batch detail payload. ADR 0046 supersedes the historical normalized
record and rich-content projection contracts.

Readiness and validation commands are successful command executions when checks run, even when the artifact is invalid. `atlas index check --json` and `atlas index validate --json` return `status: "ok"` with `data.valid: false` for invalid artifacts and exit with code `3`. These diagnostic reports describe successful check execution; record batch statuses instead summarize their per-input outcomes.

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

Batch presentation and exit policy belong to atlas-cli; shared envelope serialization belongs to atlas-cli-support. App-service retains ownership of record values and typed service errors.
