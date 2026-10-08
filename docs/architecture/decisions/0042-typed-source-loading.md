# ADR 0042: Typed source loading before normalization

Status: Accepted

## Context

Generated authored DTOs and partial-field admission are available across the
pinned Foundry source portfolio. The artifact build path still reads generic JSON
and immediately constructs Atlas records, including metrics. Source loading needs
an independently callable boundary so normalization and storage can be designed
from the complete source models and observed query needs.

## Decision

`atlas-ingest::load_foundry_documents` returns `LoadedFoundrySource`: manifest
metadata and ordered packs containing generated Actor, Item, JournalEntry, Macro
and RollTable source DTOs. The stage calls the existing admission policy from
ADR 0041. It preserves original file bytes, their SHA-256, ordered raw values,
missing/null/invalid states and contextual diagnostics. Embedded children and
authored references remain inside the generated DTOs, without flattening,
deletion, resolution, runtime preparation or derived mechanical facts.

Manifest and pack discovery have one shared owner under `source::discovery`.
The typed loader does not call Atlas normalization, metric extraction, rich
content parsing, enrichment, embedding execution or database writers. The
existing product artifact build and `source analyze` continue to describe the
current product pipeline. This is an independent pre-normalization source stage,
not a compatibility adapter into the existing normalized records.

Unsupported/ambiguous object roots remain raw-only admitted documents with
diagnostics. Invalid JSON, non-object envelopes and file read failures have
explicit quarantine outcomes. Quarantine retains bytes and hashes whenever a
read succeeded; it does not claim raw retention when a read failed. Missing or
unreadable packs remain reported pack outcomes. Unavailable source roots and
unparseable manifests return errors. Diagnostic record locators use pack name
and source path; they are not invented canonical Atlas keys when identity is
missing or invalid.

The private Rust `atlas-dev source load` command emits loading counts and every
diagnostic/failure. Partial modeled documents are successful loading outcomes.
Raw-only roots, quarantine, unavailable packs or an empty load produce exit 1
with the report still available. Root/manifest failures use the existing command
error exit 2. JSON uses the standard report envelope; its `status: ok` means the
report was produced, not that all inputs were modeled. Rust does not launch Node.
Specific rule interpretation remains independently callable and atomic; loading
preserves Item's generic rule sources without claiming those specific rules were
validated or executed.

## Consequences

The complete source inputs can guide a combined normalization and database design
review before new storage commitments. The review must cover typed family facts,
embedded relationships, raw/diagnostic retention, FTS, semantic retrieval, filters,
indexes and representative query plans. It must reassess the current metric
abstraction rather than expanding it by default. Authored versus derived values,
units, context and unavailable states require explicit decisions.

Production adoption follows the agreed normalization/storage design. The loaded
source API retains the complete corpus in memory; callers own its lifetime.
It is neither a persistent quarantine store nor proof of Foundry runtime admission.
