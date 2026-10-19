# atlas-ingest

`atlas-ingest` loads and admits authored Foundry source, preserves developer diagnostics, resolves source references, and prepares a source-backed artifact for `atlas-index` to validate and publish atomically.

The build consumes checked DTOs directly, preparing sanitized HTML and interaction caches once under an explicit indexing locale with source-English fallback and selecting attributed units through `atlas-record`. Requested search profiles run sequentially after shared source preparation; an empty profile collection builds lexical content without native vector tables. Both profiles share BGE tokenizer passage/context/identity budgets and raw assembly. Owned headers omit only the pack label; root headers and identity inputs keep the baseline recipe, without parent context. Each selected native tokenizer then counts/guards the officially prefixed input and binds the exact execution reuse key. Gemma never resplits or expands the shared baseline; Gemma-only builds need the shared tokenizer but no BGE weights.

For each current input, the selected native tokenizer binds its exact execution
reuse key. Ingest reads durable checkpoints verified against complete model
identity, raw input hash, native token count, dimensions and vector checksum.
Inference runs outside database write transactions; completed batches are saved
in short checkpoint transactions so interrupted work can resume. Current source
attribution is regenerated independently of vector reuse. Reports expose
inferred/reused counts per profile. All profiles finish before one maintenance
publication activates the reference update, preserving the previously published
reference and saved user state on failure.

Unchanged source documents reuse checked DTO snapshots. Prepared content is reused
only with exact preparation dependencies and replayed reference-resolution
outcomes, including unresolved references and target-name changes. No old embedding
text reconstruction is required. Offline index validation remains independent of
tokenizer/model assets.

Every relevant pack file, manifest binding, locale catalog and migration declaration participates in the aggregate content fingerprint, including invalid or unaddressable files excluded from product records. Discovery uses a captured file inventory, and every consumed file is checked against its captured hash before parsing. Publication rechecks the complete inventory; transient edits cannot silently change the admitted inputs. Source bytes stay in the source checkout and ingest developer outcomes; the database stores the checked compressed typed snapshot, per-record path/hash, derived query projections and compact prepared caches.

`analyze_foundry_source` reports typed admission and selection counts. `audit_source_paths` samples raw source paths for developer discovery, without claiming those paths have a product consumer. Database schema, validation, SQL and atomic publication belong to `atlas-index`; runtime paths, downloading and CLI presentation belong to their respective owners.

Build progress uses the existing tracing/CLI pipeline. Shared preparation reports
record/unit occurrences; selected-profile native validation, reuse and inference
report their actual denominators, distinguishing deduplicated inference inputs
from source occurrences. Updates are throttled and publication remains atomic.
