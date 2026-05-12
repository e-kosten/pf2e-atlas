# Rust CLI Runtime Migration Roadmap

Status: in_progress  
Design state: staged implementation foundation  
Priority: active  
Owner: unassigned  
Last reviewed: 2026-05-12

## Problem

PF2e Atlas currently centers its local agent-facing product surface on a Node/TypeScript MCP server plus an Ink terminal workbench. That shape made sense while MCP was the primary integration target and while the repo benefited from TypeScript's low-friction schema and tool-surface iteration.

The actual operating model is narrower and more local:

- the tool runs on the developer's machine
- there are no important external AI clients to support
- agents need reliable local lookup/search/rules/tagging workflows more than cross-client protocol interoperability
- many workflows are better expressed as procedure plus compact command output than as broad MCP tool schemas
- the current guardrails still allow agents to land incomplete or transitional TypeScript changes and call them done

The spike set has answered the durable direction question: the long-term core should be a Rust local toolchain, with CLI and TUI as the primary surfaces, deterministic ingest/index building moving to Rust, exploratory analysis kept adjacent, and MCP demoted to an optional compatibility mode.

## Desired Outcome

Move toward a Rust-centered local runtime architecture:

- Rust owns the runtime core:
  - domain contracts
  - canonical search request and filter model
  - SQLite index access
  - lexical/vector/hybrid search
  - rule graph traversal
  - derived-tag runtime/editorial state machines
  - CLI
  - TUI
  - optional MCP mode
- Rust also owns deterministic offline preparation once parity work catches up:
  - Foundry PF2E ingest
  - normalized record construction
  - source signatures
  - SQLite table writes
  - embedding identity and artifact metadata
  - index artifact production
- Python, Node, or TypeScript remain available for exploratory and transitional work:
  - clustering and discovery experiments
  - search-quality evaluation reports
  - review queues
  - parity comparison harnesses
  - migration-only scripts
- the prepared SQLite index and adjacent JSON/JSONL artifacts become explicit versioned contracts owned by the Rust core path once migration parity is reached
- agent integration prioritizes CLI commands plus Codex skills over MCP as the default local workflow
- MCP remains possible as `atlas mcp` only if it is useful as a thin compatibility surface over the same Rust runtime

The expected end state is not a language rewrite for its own sake. The expected end state is a stricter local architecture where invalid runtime states, incomplete search/filter variants, and ad hoc workflow bypasses are harder to express.

The current TypeScript implementation is the parity baseline for behavior, but it is not automatically the best design
for Rust. Migration slices should preserve product semantics while remaining willing to rename, split, type, or reshape
weak inherited contracts. When an implementation choice depends on whether to preserve an existing TypeScript shape or
adapt it to a stronger Rust model, agents should surface the options and ask for input rather than defaulting to either
copying or rewriting by instinct.

## Motivation

### Local agent workflows favor CLI plus skills

For this repo's actual usage, a compact local CLI is often a better agent surface than MCP:

- agents already know how to run shell commands
- skills can teach workflow and judgment instead of only exposing capabilities
- JSON CLI output can be narrower than a full MCP schema catalog
- command exit codes and stderr give familiar failure semantics
- local credentials, files, and index paths are already in the developer environment

MCP is still valuable when the same tool needs to be exposed to many AI clients or many users with governed auth and transport boundaries. That is not the primary local use case for this repo.

### Rust may improve architecture quality

Rust would not automatically prevent poor agent work, but it can raise the floor if the runtime is modeled idiomatically:

- discriminated runtime states become enums instead of loose string conventions
- exhaustive `match` forces new search/filter/tag variants through all owners
- explicit `Result` types make recoverable failures hard to ignore
- ownership discourages incidental shared mutable state
- TUI/editorial workflows can be modeled as explicit state machines
- module visibility and crate boundaries can encode architectural ownership
- Clippy and targeted lint policy can ban common escape hatches such as broad `serde_json::Value`, `unwrap()` in runtime code, or compatibility shims

This is especially relevant because the current TypeScript codebase already has many documentation and lint guardrails, yet still requires substantial review pressure to prevent half-migrations and convention-only architecture.

### Exploratory helpers are low-cost if they stay out of runtime

Using Python, Node, or TypeScript for exploratory analysis does not create the same architecture cost as splitting the live runtime:

- the artifact boundary is durable and testable
- the runtime can validate schema, source, and embedding identity before startup
- fast-changing clustering, discovery, and evaluation reports can iterate outside the Rust binary
- migration parity scripts can compare Rust output to the existing TypeScript runtime while the cutover is underway

The important constraint is that these helpers do not become the owner of canonical runtime artifacts. Deterministic Foundry ingest, normalized record construction, source signatures, SQLite table writes, and artifact metadata should move to Rust. A Python or Node query sidecar is not part of the target architecture; Rust query embeddings have been shown viable for the current MiniLM model.

## Target Shape

### Runtime crates

A plausible Rust workspace shape:

- `atlas-domain`
  Shared record keys, categories, subcategories, metadata fields, search request/filter contracts, rule graph contracts, and derived-tag runtime types.
- `atlas-index`
  SQLite schema validation, typed row loading, vector table access, metadata/source/embedding identity validation, and low-level prepared-query owners.
- `atlas-search`
  Query analysis, lexical search, vector search, hybrid ranking, filter lowering, result shaping, and search evaluation fixtures.
- `atlas-tags`
  Published derived-tag ontology, rules, assignments, exemplars, review registries, matcher runtime, and editorial state machines.
- `atlas-cli`
  Agent-facing command surface with stable JSON output and human-readable optional presentation.
- `atlas-tui`
  Ratatui/Crossterm terminal workbench over the same runtime services.
- `atlas-mcp`
  Optional MCP compatibility mode that translates MCP requests into the same runtime service calls.

Exact crate names can change. The important split is that CLI, TUI, and MCP are surfaces over one runtime core, not competing implementations.

### Artifact and prep ownership

Deterministic prep should move to Rust as the runtime stabilizes:

- parse and normalize the vendored Foundry PF2E checkout
- build the SQLite index
- generate document embeddings
- compute semantic input hashes
- write embedding identity metadata
- emit runtime-required metadata and validation diagnostics

The canonical-ingest spike did not find a Rust-specific blocker for this ownership. Python, Node, and TypeScript may still produce adjacent exploratory artifacts while they are research outputs, but runtime-required artifacts should be versioned, validated, and Rust-owned.

### Agent-facing CLI surface

The CLI should be designed for agents first, with human presentation as an optional mode.

Representative commands map to current MCP tools or existing tag CLI workflows. Final names can change during
Phase 9, but first-pass commands should not add new PF2E capabilities unless the plan records that as an explicit
product change.

```bash
atlas lookup "Treat Wounds" --category rule --json
atlas search --query "low level healing spell" --category spell --limit 10 --json
atlas browse --category creature --filter '<canonical-filter-json>' --json
atlas filters list-values --field traits --category creature --json
atlas rule-context "Grab" --include-backlinks --json
atlas graph get --record-key actions:abc123 --include-backlinks --json
atlas tags evaluate-derived-tags --category spell --json
atlas tags review-derived-tag-migration --session <path> --json
atlas tui
atlas mcp
```

CLI contracts should include:

- stable JSON schemas
- compact default output for agent use
- optional human presentation only where it helps local use; JSON parity is the primary contract
- predictable exit codes
- concise stderr errors
- no hidden network access during runtime commands
- source-signature and embedding-mismatch diagnostics when the artifact can prove the mismatch without reloading the
  source corpus

### TUI shape

The TUI should use explicit state machines rather than implicit controller conventions:

- root navigation state
- list/detail browsing state
- record page/navigation history state
- structured search editor state
- modal/prompt state
- derived-tag review session state
- writeback confirmation state
- long-running task/progress state

The goal is not just terminal polish. The goal is making invalid or incomplete interaction states hard to represent.

### Embedding contract

Query embeddings are the main feasibility gate for a Rust runtime.

The runtime must either:

- compute query embeddings in Rust using the exact same model contract as ingest, or
- disable semantic search with a clear diagnostic when the configured embedding provider cannot be initialized

The index should store enough metadata to reject incompatible runtime providers:

- provider family
- model id
- model revision or artifact checksum
- tokenizer identity
- pooling strategy
- normalization behavior
- vector dimensions
- vector dtype
- distance metric
- document prefix
- query prefix

Using a different engine at ingest and runtime is acceptable only if the produced vectors are compatible. Using a different model, tokenizer, pooling, normalization, or query/document prefix is not acceptable.

## Non-Goals

- Do not start with a full rewrite.
- Do not introduce a Python runtime query sidecar; Rust query embeddings are viable for the first MiniLM migration baseline.
- Do not keep MCP as the architectural center if the product remains local-only.
- Do not create parallel TypeScript and Rust runtime cores with mixed feature ownership.
- Do not use Rust as a thin wrapper over the existing Node runtime; that would preserve the current architecture cost while adding another language.
- Do not migrate full ingest before the runtime artifact contract and validation shell exist.
- Do not treat a compatibility bridge as progress toward the end state unless it removes an old path in the same slice.

## Open Questions

- Which embedding model should become the long-term cross-language contract?
  - Current default is `Xenova/all-MiniLM-L12-v2`.
  - Candidate upgrade path includes BGE-family models such as `BAAI/bge-small-en-v1.5`.
- Should the first Rust document-embedding writer keep MiniLM only, or also prepare a BGE comparison artifact?
- Which Ratatui workbench workflows should land before retiring the Ink workbench?
- Which current MCP tools should have first-class CLI equivalents before any runtime migration begins?
- Which derived-tag editorial workflows are stable enough to model as Rust state machines now?
- Does the optional `atlas mcp` mode need the official Rust MCP SDK, or is MCP no longer worth preserving locally?

## Research Dependencies

This migration is in staged implementation. The first foundation slice records the ADR and creates the Rust workspace, then later slices move capability ownership behind explicit parity gates.

Spike findings:

- [Rust query embedding spike](./spikes/rust-query-embedding.md): keep MiniLM for the first Rust embedding provider; Rust vectors match the TypeScript provider for sampled queries.
- [Search quality bakeoff spike](./spikes/search-quality-bakeoff.md): keep a SQLite-centered hybrid retrieval baseline; defer Tantivy, LanceDB, better models, and heavyweight rerankers.
- [Tantivy lexical search spike](./spikes/tantivy-lexical-search.md): deferred before first migration baseline.
- [LanceDB retrieval store spike](./spikes/lancedb-retrieval-store.md): do not add LanceDB as a parallel retrieval database for the current PF2E runtime.
- [Rust CLI agent surface spike](./spikes/rust-cli-agent-surface.md): continue CLI plus skill as the primary local agent-surface direction, with schema/facet discovery and support shaping still required.
- [Rust TUI state machine spike](./spikes/rust-tui-state-machine.md): Ratatui is viable after the CLI/runtime shell.
- [Artifact contract spike](./spikes/artifact-contract.md): keep a versioned SQLite runtime contract with adjacent JSON/JSONL artifacts for non-runtime reports.
- [Canonical ingest ownership spike](./spikes/canonical-ingest-ownership.md): continue toward Rust ownership of deterministic ingest and artifact/index building.
- [Optional MCP compatibility spike](./spikes/optional-mcp-compatibility.md): still pending; MCP should not block the Rust runtime migration.

Implementation plans should cite ADR 0017, name the relevant capability gate, and identify any rejected alternatives that remain future options.

## Suggested Work Breakdown

### Phase 1: Rust Workspace And Artifact Validation Foundation

1. Add ADR 0017 for the Rust-owned deterministic core, CLI-first surface, Ratatui sequencing, and optional MCP rule.
2. Create the `rust/` workspace with `atlas-domain`, `atlas-index`, and `atlas-cli`.
3. Add standard Rust validation commands: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo build --workspace`.
4. Implement `atlas validate-index --index <path> --json`.
5. Make current TypeScript-built indexes report a clear legacy-contract diagnostic until the Rust artifact contract is written by the index builder.

### Phase 2: Artifact Contract

1. Implement the versioned SQLite runtime contract.
   - schema version
   - source signature
   - embedding identity
   - record table contracts
   - FTS table contracts
   - vector table contracts
   - reference edge contracts
   - derived-tag runtime tables or artifact references

2. Define JSON/JSONL artifact schemas for prep outputs that should not live directly in SQLite.

3. Add cross-language golden fixtures.
   - record normalization fixture
   - search request/filter fixture
   - rule graph fixture
   - embedding identity fixture
   - CLI output fixture

4. Add validation commands that fail fast when prep artifacts do not match the runtime contract.

### Phase 3: Rust Runtime Skeleton

1. Implement domain contracts without importing Node runtime concepts.
2. Implement typed lookup by canonical key and by name.
3. Implement record presentation models separate from CLI/TUI rendering.
4. Add tests around source-signature mismatch, missing embedding provider, and schema mismatch errors.

### Phase 4: Search Runtime

1. Port the canonical search request and filter tree.
2. Implement filter normalization and lowering.
3. Implement lexical search over FTS.
4. Implement vector search with runtime query embeddings.
5. Implement hybrid ranking.
6. Port list filter values and search semantics discovery needed by agents.
7. Add fixture parity tests against the existing TypeScript runtime for representative queries.

### Phase 5: CLI As Primary Agent Surface

1. Implement the core command set:
   - lookup
   - search
   - browse/list
   - filter value discovery
   - search semantics
   - rule graph
   - rule question context
   - pack listing and pack metadata if the CLI is expected to replace the full current MCP lookup/discovery surface
2. Stabilize JSON schemas and output size policy.
3. Write Codex skills that teach the intended command workflows.
4. Add command golden tests.
5. Compare token use and task success against the MCP surface for common local agent tasks.

### Phase 6: TUI Runtime

1. Implement the shared terminal framework.
2. Implement search result list/detail.
3. Implement entity page browsing and reference navigation.
4. Implement structured search editing as explicit state transitions.
5. Implement ontology/search-semantics exploration if still needed.
6. Implement derived-tag review/editorial state machines after the core search path is stable.
7. Add snapshot or terminal-model tests for transition logic and rendering contracts.

### Phase 7: Optional MCP Compatibility

1. Decide whether MCP remains valuable after CLI plus skills exist.
2. If yes, implement `atlas mcp` as a thin surface over the Rust runtime.
3. Preserve current tool names and schemas where useful.
4. Avoid adding MCP-only backend behavior.
5. Keep MCP validation secondary to CLI/runtime validation.

### Phase 8: Migration And Retirement

1. Run Rust and TypeScript runtimes side by side against the same vendored source revision and prepared artifact contract.
2. Compare representative lookup/search/rule/tag workflows.
3. Freeze new runtime feature work in TypeScript once Rust parity is sufficient.
4. Retire or archive the Node MCP/TUI runtime only after the CLI/TUI replacement is complete.
5. Update architecture docs, README, CONTRIBUTING, backlog history, and skills.
6. Remove transitional bridges and compatibility-only paths before declaring the migration complete.

## Validation Requirements

Any implementation plan that comes from this item should include:

- embedding compatibility tests between ingest and runtime
- search result parity or accepted-difference reports
- CLI JSON schema/golden tests
- source-signature and embedding-mismatch tests
- TUI state transition tests
- architecture docs and ADR updates
- explicit checks that the old runtime path is not left as an unowned parallel implementation

## Related

- [Architecture overview](../../architecture/overview.md)
- [Architectural boundaries](../../architecture/boundaries.md)
- [Search architecture](../../architecture/search.md)
- [TUI architecture](../../architecture/tui.md)
- [Editorial architecture](../../architecture/editorial.md)
- [Derived-tag index service layer](../items/derived-tag-index-service-layer.md)
- [Tagging tooling reorganization](../items/tagging-tooling-reorg.md)
