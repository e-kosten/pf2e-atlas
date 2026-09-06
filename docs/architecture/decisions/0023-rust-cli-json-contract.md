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

Successful payloads live under `data`. Errors live under `error`. Commands do not emit a `command` field because the caller already knows the invoked command. Optional fields are omitted when absent. Fields, objects, and arrays that the requested detail level does not hydrate are also omitted rather than serialized as empty placeholders; an empty value is retained only inside an included section when known-empty is intentional product meaning. CLI JSON field names are `snake_case`.

JSON payloads are written to stdout. Routine progress is not part of the JSON contract and is controlled separately from payload output. By default, progress renders only for human terminal sessions and is suppressed for JSON or non-terminal automation; users can force or suppress progress with the global `--progress` option or the `ATLAS_PROGRESS` environment variable.

Record-facing commands use `summary`, `preview`, `description`, `standard`, and `full` detail levels. All detail levels keep the same discriminator and common base while varying entity-field presence by hydration depth. `summary` is identity-oriented and omits the creature mechanics body, `preview` adds compact typed scan concepts and truncated prose while omitting activity detail, `description` emphasizes complete descriptive content and omits creature mechanics, `standard` is the normal typed entity view, and `full` adds full source metadata and supplementary content. The Rust CLI does not accept `minimal`, `compact`, or TypeScript-specific detail values. Raw source JSON is exposed only by explicit `--include-raw` opt-in, appears as top-level `source_json` in the record payload, and is independent of detail level.

Human creature output follows the same five selections without exposing placement internals. Spellcasting renders as entry, then rank, then ordered spell rows; ordinary profiles omit opaque occurrence IDs and prepared-slot locators. Description and Full place typed rich content beneath natural domain headings while preserving exact-once ownership internally. Full retains concise source and edition context, but field-level provenance, occurrence identity, ownership, and exact reference diagnostics are exposed separately by `atlas record provenance <canonical-key> [--json]`. That command is a typed diagnostic projection and never a raw-source view; `--include-raw` behavior is unchanged.

`atlas-record::RecordJson` is one CLI/agent presentation model: a flattened `RecordJsonBase` plus a flattened `RecordPresentationJson` tagged by mandatory `presentation_type`. The base contains only genuinely shared identity/classification/source facts, explicit raw opt-in, and structured supplementary rich content and relationships. Entity bodies expose family concepts directly. The creature body owns direct defenses, perception, languages, skills/lore, movement, resources, strikes, actions, and separate ordered spellcasting entries and spells. Creature mechanics are not label/value facts, generic target strings, or blocks inside ordered sections, and are not duplicated in supplementary content.

The `unmigrated` presentation is a temporary, explicit registry boundary for families that do not yet own a typed creature, hazard, or standalone-spell presentation. It is not a silent catch-all or final product state. Each remaining family replaces that boundary with its own optimized variant; the unreleased CLI contract uses direct replacement without compatibility aliases, shims, feature switches, or dual serializers. A creature, hazard, or spell missing its required canonical body fails closed instead of entering this registry.

`RecordJson` is distinct from the canonical `AtlasRecord`, app/web DTOs, storage records, and search/embedding projections. Result DTOs wrap the same tagged record with search, graph, similar, list, or resolution metadata. Batch record commands return per-item results with per-item errors when some keys or queries miss.

`atlas-record::RetrievedRecord { record: AtlasRecord, body: Option<RecordBody> }` is the sole storage-neutral record-bearing retrieval aggregate. Index hydration supplies it, search preserves it, and local CLI paths pass it through app-service. For migrated creatures, `RecordJson` reads only `RecordBody::Creature`, including IWR amounts and exceptions, skill variants and notes, resources, action costs and frequencies, and spell entry and occurrence context. It never recovers migrated values from `AtlasRecord.mechanics`, raw source, prose, storage rows, or an app DTO.

Supplementary and temporary unmigrated sections use stable section and block kinds. Rich content is exposed through structured presentation-content blocks and inline spans rather than raw `RichDocument` nodes or markdown-like strings. Terminal output renders `RecordJson` directly, while web output uses a web-specific projection rather than parsing terminal text or reusing the CLI DTO. Original Foundry rich-text output is not part of the CLI contract. Batch record payloads include `data.partial` alongside per-item errors so automation can distinguish complete success from partial domain misses without relying only on the process exit code.

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
