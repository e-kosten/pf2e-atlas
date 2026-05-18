# Rust Runtime Migration Checklist

Status: active working checklist  
Root worktree: `.worktrees/rust-runtime-root`  
Primary branch: `rust/runtime-root`  
Architecture decision: [ADR 0017](../../architecture/decisions/0017-rust-runtime-cli-first-migration.md)  
Roadmap: [migration-roadmap.md](./migration-roadmap.md)

This checklist is the working handoff for follow-up agents moving PF2e Atlas from the current TypeScript MCP/TUI runtime toward a Rust-owned deterministic core, CLI-first agent surface, Ratatui workbench, and optional MCP compatibility.

The checklist is intentionally larger than one implementation pass. Treat each checked item as a validated, committed increment on `rust/runtime-root` or on a short-lived worktree branched from it.

## Working Rules

- Use `.worktrees/rust-runtime-root` as the persistent Rust migration root.
- Prefer small commits that complete one checklist item or one tightly related group.
- Use Cargo-native validation by default:

```bash
cd rust
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

- Use Cargo's release profile for ingest and query performance comparisons:

```bash
cd rust
cargo run --release -p atlas-cli -- index analyze --source ../vendor/pf2e --json
```

Do not compare TypeScript runtime timings against Rust dev-profile `cargo run` timings.

- Do not run the full Node lint/build/test gate for normal Rust iteration.
- Run the full repo gate only before landing back to `main`, when touching TypeScript runtime behavior, or when changing repo-wide Node tooling.
- Do not make Rust shell out to the TypeScript runtime or Node MCP server.
- Do not add placeholder crates. Add a crate only when its first real slice lands.
- Do not let Rust CLI command names imply new product capability during parity phases. Each command should map to an
  existing MCP tool, current tag/editorial script, or an explicitly documented new product decision.
- Treat the current TypeScript runtime as the behavior baseline, not as proof that its names, table shapes, or type
  boundaries are the best long-term design. When an implementation slice exposes a weak or ambiguous TypeScript
  decision, pause and present the tradeoff, or ask for input, instead of silently copying it.
- Keep runtime artifacts and deterministic ingest moving toward Rust ownership. Python, Node, and TypeScript are acceptable for exploratory analysis, parity harnesses, and temporary migration scripts.
- If a slice exposes a major architecture decision not covered by ADR 0017, add or update an ADR before continuing.

## Current Foundation

- [x] Add `rust/` workspace.
- [x] Add `atlas-domain`, `atlas-index`, `atlas-search`, `atlas-runtime`, and `atlas-cli`.
- [x] Pin Rust toolchain with `rust-toolchain.toml`.
- [x] Track `Cargo.lock`.
- [x] Deny `unsafe_code` in initial crates.
- [x] Use standard Clippy with warnings denied.
- [x] Implement `atlas index validate --index <path> --json`.
- [x] Make current TypeScript-built indexes report `MISSING_ARTIFACT_METADATA`.
- [x] Record ADR 0017.
- [x] Update Rust migration roadmap to Rust-owned deterministic ingest/index direction.

## Phase 1: Artifact Contract And Validation

Goal: define and enforce the runtime artifact boundary before porting lookup/search behavior.

- [x] Move the spike artifact-contract draft into durable repo docs.
- [x] Decide the canonical contract version string for the first Rust artifact.
- [x] Define required `artifact_metadata` keys in `atlas-domain`.
- [x] Expand `atlas-index` validation beyond the first five keys:
  - [x] contract version
  - [x] schema version
  - [x] source kind
  - [x] source signature
  - [x] source record count
  - [x] content hash algorithm
  - [x] embedding provider family
  - [x] embedding model id
  - [x] embedding model revision
  - [x] embedding tokenizer id
  - [x] embedding pooling
  - [x] embedding normalization
  - [x] embedding dimensions
  - [x] embedding dtype
  - [x] embedding distance metric
  - [x] document/query prefixes
  - [x] FTS tokenizer
  - [x] adjacent manifest path
- [x] Add typed validation diagnostics for each incompatible or missing contract family.
- [x] Add fixture indexes for:
  - [x] valid minimal contract
  - [x] missing `artifact_metadata`
  - [x] missing required key
  - [x] stale source signature
  - [x] embedding mismatch
  - [x] unsupported schema version
- [x] Keep validation metadata readable without loading sqlite-vec.
- [x] Add golden JSON tests for `atlas index validate`.
- [x] Add `atlas artifact inspect --json` if validation output starts carrying too much detail. Not needed for phase one; validation output remains focused on metadata and diagnostics.
- [x] Document artifact validation failure codes in `rust/README.md`.

Acceptance:
- `atlas index validate` returns stable JSON for valid and invalid fixtures.
- The current TypeScript index still produces a clear legacy-contract diagnostic.
- No vector-extension load is required for metadata validation.

## Phase 1.5: TypeScript Runtime Inventory And Rust Contract Mapping

Goal: derive the Rust contract surface from the current TypeScript runtime before broad domain types, ingest writers, or search/discovery commands are implemented.

- [x] Inventory current TypeScript SQLite tables and identify which runtime commands or discovery flows depend on each table.
- [x] Inventory current TypeScript indexing stages and map them to planned Rust writer stages.
- [x] Map current TypeScript record, category, metadata, metric, request, filter, search, lookup, rule-graph, and output shapes to planned Rust contracts, including the decision to retire TypeScript subcategory as a Rust contract.
- [x] Classify each mapped contract as:
  - [x] parity requirement
  - [x] deliberate Rust redesign
  - [x] transitional compatibility input
  - [x] retired/non-goal
- [x] Identify JSON blob fields that should remain JSON for parity and fields that should become typed Rust structures or side-table rows.
- [x] Identify legacy, compatibility, or bridge paths that Rust should not preserve as first-class runtime concepts.
- [x] Review foundational TypeScript DB and type design choices and record which ones Rust should adapt before treating them as durable artifact contracts.
- [x] Define crate ownership for each mapped contract across `atlas-domain`, `atlas-index`, future `atlas-ingest`, future `atlas-search`, future `atlas-tags`, and surface crates.
- [x] Define the first parity fixture set for each later capability gate:
  - [x] lookup and record presentation
  - [x] rule graph and rule context
  - [x] schema/filter discovery
  - [x] search and browse
  - [x] derived-tag runtime
  - [x] CLI output contracts
- [x] Update or add a Rust artifact contract document for runtime tables beyond `artifact_metadata` before Phase 3 writer work starts.

Acceptance:
- [x] Every Phase 2 domain type traces to a current TypeScript contract, a deliberate Rust-only contract, or an explicit non-goal.
- [x] Every Phase 3 table/write task traces to a current runtime dependency or an explicit accepted difference.
- [x] Later phases cannot introduce new artifact, table, or output-contract dependencies without updating the contract map.
- [x] The mapping is concrete enough for follow-up agents to implement Phase 2 and Phase 3 slices without rediscovering current TypeScript runtime shape from scratch.

## Phase 2: Rust Domain Model

Goal: define the typed runtime vocabulary that later ingest, index, search, CLI, and TUI crates share.

- [x] Add `RecordKey` with pack and record id parsing.
- [x] Add `record_family` vocabulary for Atlas-derived product grouping.
- [x] Remove subcategory from Rust domain, search filters, record summary, and minimal writer schema; replace useful cases with explicit metadata/filter axes.
- [x] Add rarity, level, action-cost, and source/publication primitives.
- [x] Add canonical record summary type.
- [x] Add Rust CLI detail-level vocabulary:
  - [x] summary
  - [x] standard
  - [x] full
- [x] Add internal text availability/parsing status vocabulary for row loading and diagnostics, not as a new
  user-facing answerability capability:
  - [x] resolved
  - [x] missing
  - [x] localized placeholder
  - [x] unsupported markup
- [x] Add canonical `SearchRequest` model:
  - [x] browse
  - [x] search
  - [x] lookup
- [x] Add canonical filter tree:
  - [x] record family
  - [x] level
  - [x] rarity
  - [x] action cost
  - [x] metadata predicate
  - [x] metric predicate
  - [x] links_to
  - [x] linked_from
  - [x] any_of/all_of/not
- [x] Add rule graph contracts.
- [x] Add metadata-field vocabulary.
- [x] Keep durable CLI output envelope policy out of the domain crate; Phase 5 establishes the CLI-owned JSON envelope in `atlas-cli`.
- [x] Keep domain free of SQLite, CLI, TUI, MCP, and ingest dependencies.

Acceptance:
- Domain types compile independently.
- Search/filter fixtures can round-trip through `serde`.
- Adding a new filter kind requires explicit handling in downstream compilation tests.

## Phase 3: Rust Canonical Ingest And Index Builder

Goal: move deterministic Foundry JSON ingest and SQLite artifact construction toward Rust ownership.

- [x] Add `atlas-ingest` crate once the first real writer slice starts.
- [x] Parse vendored Foundry JSON with tolerant boundary types.
- [x] Normalize canonical record keys.
- [x] Normalize canonical names.
- [x] Map Foundry `document_type` plus record `type` into `record_family`, while preserving `foundry_document_type` and `foundry_record_type` source axes for the fixture writer.
- [x] Parse Foundry-authored description, blurb, public notes, rule-note text, and embedded item/spell text into `ContentDocument`.
- [x] Preserve content references in the AST and derive plain-text/markdown/search projections from `ContentDocument`.
- [x] Extract traits.
- [x] Extract publication/source metadata.
- [x] Report skipped source records with path and reason during ingest.
- [x] Keep `army` actors as a separate `record_family` rather than folding them into `creature`.
- [x] Extract selected direct `system_*` projections.
- [x] Preserve raw price JSON and normalize `price_cp`.
- [x] Normalize activation time separately from effect/spell duration.
- [x] Extract source-backed aliases from remaster journals, migration rename files, and embedded compendium sources.
- [x] Generate derived affliction canonicals and hidden provenance instances from staged action, consumable, and spell records.
- [x] Extract variant family metadata (`variant_group_key`, `variant_base_name`, `variant_label`, `variant_axes`, confidence, and source).
- [x] Extract reference edges from resolved content references without substring false positives.
- [x] Write `artifact_metadata`.
- [x] Split source, generated, and artifact record count metadata.
- [x] Write `packs`.
- [x] Write `records`.
- [x] Write weighted `records_fts` fields from content/presentation projections.
- [x] Write `reference_edges` with source kind and visibility.
- [x] Write `record_content` supplemental content rows.
- [x] Write record traits.
- [x] Write unified `record_metrics` with actor/item metric domains.
- [x] Generate metric key/value catalogs from `record_metrics`.
- [x] Write aliases.
- [x] Write remaster links preserving current premaster-to-remaster bridge behavior.
- [x] Write actor side-data.
- [x] Write item side-data.
- [x] Write spell side-data.
- [x] Generate source signatures from source paths and per-record hashes.
- [x] Group index management commands under `atlas index` without legacy command aliases.
- [x] Add a small fixture ingest test.
- [x] Add a full-corpus analysis command that reports counts without writing the full artifact.
- [x] Compare Rust full-corpus counts against a freshly rebuilt TypeScript index from the same PF2E source revision.

Acceptance:
- Rust can build a minimal valid artifact consumed by `atlas index validate`.
- Full-corpus analyzer has zero JSON parse failures.
- Known count differences from TypeScript are classified as accepted policy differences or open parity defects.

Comparison note:
- Rebuilt the TypeScript index from the same `vendor/pf2e` source with `npm run refresh-index -- --embedding-provider hash --index-path .cache/pf2e-index.ts-parity.sqlite`; it produced 82 packs and 27,424 records.
- Ran Rust full-corpus analysis and built `.cache/pf2e-rust-index.parity.sqlite`; Rust produced 96 packs and 29,674 records, with 0 skipped records, and `atlas index validate --json` returned `OK`.
- The 14-pack Rust surplus is accepted policy: Rust includes the same 82 TypeScript packs plus generated `derived-afflictions` and `derived-affliction-instances`, `action-macros`, `pf2e-macros`, `rollable-tables`, `pathfinder-society-boons`, and PFS bestiary/introduction packs.
- The 2,250-record Rust surplus is classified as accepted policy from broader source coverage and generated provenance: `army` actors (+22), additional actor records from PFS/source packs (`hazard` +425, `npc` +1,324), additional source items (`feat` +157, `effect` +20), generated afflictions/provenance (`affliction` +16, `affliction-instance` +136), and tooling records (`Macro|script` +81, `RollTable|RollTable` +69).

## Phase 4: Embeddings And Vector Artifact

Goal: keep Rust semantic search in a catalog-backed embedding space, preserve explicit MiniLM parity support, and use the accepted default embedding model from ADR 0018.

- [x] Add `atlas-embedding` crate only when query or document embedding implementation starts.
- [x] Add an `atlas-embedding` model catalog that owns the default model decision and all embedding identity fields used by ingest, validation, and query runtime.
- [x] Port MiniLM query embedding from the spike.
- [x] Validate tokenizer, pooling, normalization, dimensions, dtype, and prefixes.
- [x] Add query-vector fixture comparisons against the current TypeScript provider.
- [x] Decide ONNX Runtime packaging strategy.
- [x] Add Rust-owned document embedding input construction and pending input/hash generation for default-visible records.
- [x] Implement document embedding generation for Rust-built artifacts.
- [x] Store embedding identity in normal SQLite metadata.
- [x] Add Rust-owned `document_embedding_cache` table for reusable vector blobs, semantic input hashes, and dimensions.
- [x] Add Rust-owned `record_vector_index` sqlite-vec table contract with rowid plus embedding only.
- [x] Add a shared `atlas-index` compiler that lowers canonical filters into authoritative SQL eligible-record keysets for reuse by vector, browse, and search execution.
- [x] Compile semantic-search filters into authoritative SQL eligible-record keysets and constrain sqlite-vec through eligible `document_embedding_cache.rowid` values.
- [x] Reject semantic search filters that cannot compile to an authoritative SQL keyset for the first Rust baseline.
- [x] Add a minimal `atlas search semantic --json` CLI route for query-string embedding, optional `--filter-json`, and raw vector hit output.
- [x] Route semantic search orchestration through `atlas-search` and keep SQLite connection ownership behind `atlas-index`.
- [x] Emit structured stderr progress logs for long-running Rust index build and vector-index operations without polluting JSON stdout.
- [x] Integrate sqlite-vec loading behind explicit capability checks.
- [x] Add diagnostics for vector extension unavailable.
- [x] Write `document_embedding_cache` reusable vector blobs with semantic input hashes when document embeddings are generated.
- [x] Reuse unchanged `document_embedding_cache` rows by default on Rust artifact rebuilds, with an explicit opt-out.
- [x] Write `record_vector_index` sqlite-vec rows from `document_embedding_cache` when sqlite-vec is available.
- [x] Validate cache and vector-index key coverage against default-visible searchable records.
- [x] Log and report tokenizer truncation telemetry for document embedding inputs.
- [x] Performance-test keyset-prefiltered semantic search before adding vec metadata or partition columns.
- [x] Select BGE small as the default Rust embedding model through ADR 0018, while keeping MiniLM as an explicit parity and older-artifact option.
- [x] Ensure switching the active embedding model is a catalog/config change plus artifact rebuild, not scattered edits to storage, validation, ingest, and search code.
- [ ] Evaluate longer-context and quality-ceiling embedding options after the Rust baseline is measurable, including Nomic and `all-mpnet-base-v2`, against query quality, ingest speed, artifact size, vector dimensions, and runtime packaging.

Acceptance:
- Rust MiniLM query vectors match existing TypeScript vectors for sampled queries when MiniLM is selected.
- The default Rust embedding metadata follows ADR 0018's BGE small contract.
- Runtime fails clearly on embedding identity mismatch.
- Semantic search can be disabled or rejected without corrupting lexical/structured commands.

## Phase 5: Record Retrieval And Structured Search Foundation

Goal: establish the first production-quality Rust retrieval surface using the CLI shape accepted in ADR 0019.

- [x] Implement broad typed row loading from `records` and side tables.
- [x] Expose targeted typed readers for exact record-key lookup and ordered multi-key lookup.
- [x] Add `atlas record get <record-key> --json`.
- [x] Add persisted-record detail projection shared by `record get`, `record resolve`, and `search` results.
- [x] Add `summary`, `standard`, and `full` record output; do not add `minimal`, `compact`, or `answerable` as wire detail values.
- [x] Defer localization resolution from the Phase 5 commit until the localization artifact policy is explicit and testable.
- [x] Add `atlas search` filter-only behavior that routes to deterministic list mode when no text is supplied.
- [x] Reuse canonical `SearchFilterNode` lowering for filter-only search and record-resolution narrowing.
- [x] Add pagination and deterministic sort modes for filter-only `atlas search`.
- [x] Add `atlas record resolve <name> --json` for strict name, normalized-name, verified-alias, and exact full-variant-name resolution.
- [x] Implement controlled alternatives for ambiguous record resolution.
- [x] Preserve safe record-resolution behavior: a strict miss exits nonzero and does not return fuzzy unrelated records.
- [x] Defer fuzzy name matching and variant-family expansion flags from the Phase 5 commit; strict resolution remains the only default behavior.
- [x] Defer variant-aware resolution from `variant_*` metadata from the Phase 5 commit. Do not turn every variant base name into an alias; source-backed aliases remain limited to remaster journals, migration rename files, and embedded compendium sources. Exact full variant names can still resolve through ordinary name matching.
- [x] Add golden output tests for `atlas record get`, `atlas record resolve`, and filter-only `atlas search`.
- [x] Add CLI exit code tests for invalid record keys, strict resolve misses, ambiguity, and invalid filter JSON.
- [x] Compare `Treat Wounds`, `Grabbed`, and `Antidote (Lesser)` against current MCP/TypeScript behavior. Manual comparison used current MCP lookup/get results for `actionspf2e:1kGNdIIhuglAjIp9`, `conditionitems:kWc1fhmv9LBiTuei`, and `equipment-srd:ktjFOp3U0wQD9t0Z` against a Rust artifact built from the same `vendor/pf2e` source at `.cache/pf2e-rust-phase5-compare.sqlite`.

Acceptance:
- `atlas record get` is usable for known canonical record keys.
- `atlas record resolve` is usable for known named records and returns nonzero with concise diagnostics on strict misses.
- `atlas search` with filters and no text returns deterministic filtered lists.
- Output size stays compact by default while using the `summary` detail wire value.

## Phase 6: Search And Browse Runtime

Goal: extend the unified `atlas search` surface with ranked text retrieval using SQLite-centered hybrid retrieval.

- [x] Keep filter-only list behavior aligned with the Phase 5 `atlas search` foundation.
- [x] Implement canonical filter lowering to SQL eligible keysets.
- [x] Wire structured filter SQL into ranked text search execution.
- [x] Implement FTS retrieval.
- [x] Implement query analysis.
- [x] Implement sqlite-vec vector retrieval.
- [x] Implement hybrid candidate fusion.
- [x] Use weighted RRF with equal FTS/vector weights as the default hybrid fusion method.
- [x] Reject explicit unweighted `--fusion rrf` when lane weights are also supplied.
- [x] Boost strong name and verified-alias matches above broader FTS and vector matches in normal `atlas search <text>` results.
- [x] Keep generic Node-style rerank adjustments out of the baseline unless a quality fixture proves they are needed.
- [x] Implement `search.exclude`.
- [x] Implement advanced retrieval-path flags, such as `--retrieval fts|vector|hybrid`, rather than Node-era `lexical|balanced|concept` product profiles.
- [x] Ensure FTS retrieval works with record-only artifacts produced by `atlas setup --no-embeddings`.
- [x] Remove the temporary Phase 4 diagnostic `atlas search semantic --query ...` CLI branch once normal ranked search covers vector retrieval.
- [x] Implement explain output if still useful.
- [x] Defer full live-index top-k quality fixtures to the Rust search quality tuning backlog item.
- [x] Defer TypeScript parity or accepted-difference reporting to the Rust search quality tuning backlog item.
- [x] Keep Tantivy, LanceDB, non-default embedding-model switches, and heavyweight rerankers deferred unless a new quality result justifies them.

Acceptance:
- Representative lookup/search/rule workflows pass against Rust.
- Focused fixture tests cover record lookup/resolve, filter-only search, FTS search, exclude, vector-readiness errors, ranking/fusion units, and record JSON shape.
- Full live-index top-k bakeoff and TypeScript accepted-difference reports are tracked by the Rust search quality tuning backlog item.
- Quality differences are documented rather than accidental.

## Phase 7: Filter And Schema Discovery

Goal: replace MCP’s strongest remaining advantage: dynamic schema/facet discovery.

- [ ] Add a CLI equivalent for `pf2e_get_search_semantics`, such as `atlas schema search-filters --json`.
- [ ] Add a CLI equivalent for `pf2e_list_filter_values`, such as `atlas filters list-values --field <field> --json`.
- [ ] Add category and explicit-axis filtering for value discovery.
- [ ] Add trait discovery.
- [ ] Add item metadata discovery.
- [ ] Add actor metric discovery.
- [ ] Add spell metadata discovery.
- [ ] Add examples in `rust/README.md`.
- [ ] Add production Codex skill snippets once commands stabilize.
- [ ] Add golden tests for schema output.

Acceptance:
- Agents can discover non-obvious filters without MCP.
- The poison-consumables task from the CLI spike can be completed without guessing hidden metadata fields.

## Phase 8: CLI Product Surface And Skill

Goal: make the Rust CLI the default local agent interface.

- [ ] Stabilize command naming.
- [x] Make `atlas setup` the standard first-run install/repair command, with full semantic readiness by default and `--no-embeddings` for record-only setup.
- [x] Add setup `--check`, `--offline`, and `--force-rebuild` behavior with structured action/readiness output.
- [x] Make `atlas index validate` symmetric with setup: full semantic validation by default, `--no-embeddings` for base validation, and `--embeddings-only` for focused vector diagnostics.
- [x] Remove the separate `atlas index validate-vectors` command.
- [x] Stabilize the baseline JSON envelope for Phase 5 record, search, setup, and index commands:
  - [x] use a shared CLI-owned envelope helper in `atlas-cli`
  - [x] keep successful payloads under `data` and command failures under `error`
  - [x] define success, error, pagination, and timing fields for Phase 5 commands that need them
  - [x] add golden tests for representative success and failure commands
- [ ] Stabilize exit codes.
- [ ] Add human-readable output mode if needed.
- [ ] Add `--json` default policy or explicit always-JSON policy.
- [ ] Add `--index` and config lookup.
- [ ] Add source-signature, embedding-mismatch, and incompatible-artifact diagnostics.
- [ ] Add command golden tests.
- [ ] Add a production Codex skill for PF2e Atlas CLI workflows.
- [ ] Include command-choice rules:
  - [ ] `record get` for exact record keys
  - [ ] `record resolve` for strict names and verified aliases
  - [ ] `search` for result sets, including text queries and filter-only listing
  - [ ] `--retrieval` for FTS, vector, or hybrid retrieval tuning
  - [ ] rule-context for rules answers
  - [ ] schema/filters for discovery
- [ ] Re-run the CLI-vs-MCP evaluation tasks.
- [ ] Decide whether CLI plus skill graduates from `adjust` to `keep`.

Acceptance:
- Local agents can complete the standard evaluation tasks through CLI plus skill.
- MCP is no longer needed for ordinary local PF2E lookup/search/rules work.

## Phase 9: Rule Graph And Rule Context

Goal: evaluate the rule graph retrieval node implementation and decide whether it is worth keeping or evolvoling. Consider for instance whether the product surface is really "rule graph/context" or if more generally the utility is about searching for record clusters."

- [ ] Load direct outgoing references.
- [ ] Load backlinks.
- [ ] Add graph get command by record key.
- [ ] Add `rule-context <name>` command.
- [ ] Resolve the bestiary glossary `Grab` record correctly.
- [ ] Add support-record shaping that avoids noisy default backlinks.
- [ ] Add `--include-backlinks`.
- [ ] Preserve current rule-context behavior: return primary/support records and edges without synthesizing answers.
- [ ] Add golden tests for `Grab`.
- [ ] Add tests for localized text.
- [ ] Add tests for ambiguous rule names.

Acceptance:
- `atlas rule-context Grab` returns useful primary and support records for direct rules answers.
- Support records are useful by default and expandable when needed.

## Phase 10: Ratatui Workbench

Goal: replace the Ink TUI after the Rust runtime shell is stable.

- [ ] Add `atlas-tui` crate when the first real TUI slice starts.
- [ ] Define explicit app state.
- [ ] Define route/navigation state.
- [ ] Define rendered-row model.
- [ ] Define list/detail state.
- [ ] Define modal/prompt state.
- [ ] Define pointer/selection state.
- [ ] Add search result list/detail screen.
- [ ] Add record detail screen.
- [ ] Add reference navigation.
- [ ] Add filter/schema explorer.
- [ ] Add text selection and copy behavior.
- [ ] Add link open/copy behavior.
- [ ] Add terminal capability abstraction for clipboard/open.
- [ ] Add Ratatui `TestBackend` render tests.
- [ ] Add manual acceptance checklist for Ghostty, iTerm2, tmux, and non-tmux.
- [ ] Port derived-tag review/editorial workflows only after core search/browse TUI is stable.

Acceptance:
- Ratatui can replace the user-facing search/detail flows.
- Editorial migration is broken into separate validated slices.

## Phase 11: Optional MCP Compatibility

Goal: decide late whether MCP is still worth carrying.

- [ ] Revisit optional MCP spike after CLI plus skill is operational.
- [ ] Identify any real external-client requirement.
- [ ] If kept, add `atlas-mcp` as a thin surface over Rust runtime services.
- [ ] Preserve only useful tool names/schemas.
- [ ] Add no MCP-only backend behavior.
- [ ] Keep MCP validation secondary to CLI/runtime validation.
- [ ] If dropped, update README, docs, and skills to remove MCP as a primary surface.

Acceptance:
- MCP is either retired or clearly isolated as compatibility.

## Phase 12: Derived Tags And Editorial Runtime

Goal: redesign derived tags after the core Rust record, search, discovery, and TUI model is stable.

- [ ] Reconsider the derived-tag product model against `record_family`, explicit source axes, and retired subcategory semantics.
- [ ] Decide which derived-tag concepts remain runtime filter axes, which become authored taxonomy, and which retire.
- [ ] Define Rust derived-tag runtime types only after that model is accepted.
- [ ] Add `atlas-tags` crate only when the accepted runtime/editorial model starts implementation.
- [ ] Load published ontology and assignments if they remain part of the accepted runtime model.
- [ ] Load rules, exemplars, and review registries only if they are still needed by runtime commands.
- [ ] Implement tag filters and discovery commands against the redesigned model.
- [ ] Add parity or accepted-difference tests for retained current derived-tag assignments.
- [ ] Defer high-churn candidate discovery and clustering until runtime tag consumption is stable.

Acceptance:
- The retained derived-tag surface has an accepted Rust design.
- Rust search can filter and present retained derived-tag concepts without depending on the TypeScript tag runtime.

## Phase 13: TypeScript Runtime Retirement

Goal: remove the old runtime only after Rust owns the requested capability set.

- [ ] Freeze new TypeScript runtime feature work once Rust parity is sufficient.
- [ ] Compare Rust and TypeScript against the same vendored source revision.
- [ ] Classify every difference as accepted, fixed, or deferred.
- [ ] Remove or archive Node MCP runtime.
- [ ] Remove or archive Ink TUI runtime.
- [ ] Remove TypeScript index builder after Rust index builder is canonical.
- [ ] Preserve any still-useful exploratory scripts outside the core runtime path.
- [ ] Update architecture overview.
- [ ] Update architecture boundaries.
- [ ] Update search architecture.
- [ ] Update TUI architecture.
- [ ] Update editorial architecture.
- [ ] Update README.
- [ ] Update CONTRIBUTING.
- [ ] Move completed backlog items to history.
- [ ] Remove transitional bridges and compatibility-only paths.

Acceptance:
- No unowned parallel runtime remains.
- Docs describe the current Rust architecture, not the migration history.
- TypeScript is either gone from the core path or explicitly scoped to non-core tools.

## Agent Handoff Template

Use this shape when assigning follow-up agents:

```text
Worktree root: /Users/ekosten/projects/pathfinder-mcp/pathfinder-2e-foundry-mcp/.worktrees/rust-runtime-root
Checklist item(s): <exact items from docs/backlog/rust-cli-runtime/migration-checklist.md>
Allowed scope: <files/crates/docs the agent may edit>
Do not change: <explicit exclusions>
Validation: <Cargo commands or targeted command>
Completion: commit the coherent slice on rust/runtime-root or report blocker with exact failing command/output>
```

For parallel work, create short-lived worktrees from `rust/runtime-root` instead of editing the same files concurrently.

## Lightweight Validation Policy

Use the smallest validation set that proves the slice:

- Rust source changes: Cargo fmt, Clippy, tests, build.
- Rust docs-only changes: content review and `git diff --check`.
- Architecture docs: content review against ADR 0017 and the roadmap.
- TypeScript runtime changes: targeted Node tests plus full Node gate before landing to `main`.
- Main landing: run the repo landing gate and the Cargo gate.

Do not report a checklist item complete if validation is skipped without recording why.
