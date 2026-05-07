# Rust CLI Runtime Migration Roadmap

Status: in_progress  
Design state: design in progress  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-05-06

## Problem

PF2e Atlas currently centers its local agent-facing product surface on a Node/TypeScript MCP server plus an Ink terminal workbench. That shape made sense while MCP was the primary integration target and while the repo benefited from TypeScript's low-friction schema and tool-surface iteration.

The actual operating model is narrower and more local:

- the tool runs on the developer's machine
- there are no important external AI clients to support
- agents need reliable local lookup/search/rules/tagging workflows more than cross-client protocol interoperability
- many workflows are better expressed as procedure plus compact command output than as broad MCP tool schemas
- the current guardrails still allow agents to land incomplete or transitional TypeScript changes and call them done

That raises a durable architectural question: should the long-term runtime be a Rust local toolchain, with CLI and TUI as the primary surfaces, Python retained for offline ingest and ML experimentation, and MCP demoted to an optional compatibility mode?

## Desired Outcome

Evaluate and, if justified by prototypes, move toward a Rust-centered local runtime architecture:

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
- Python or Node remains available for offline preparation:
  - Foundry PF2E ingest
  - bulk document embedding generation
  - clustering and discovery experiments
  - evaluation reports
  - index artifact production
- the prepared SQLite index and adjacent JSON/JSONL artifacts become explicit versioned contracts between prep tooling and the Rust runtime
- agent integration prioritizes CLI commands plus Codex skills over MCP as the default local workflow
- MCP remains possible as `atlas mcp` only if it is useful as a thin compatibility surface over the same Rust runtime

The expected end state is not a language rewrite for its own sake. The expected end state is a stricter local architecture where invalid runtime states, incomplete search/filter variants, and ad hoc workflow bypasses are harder to express.

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

### Python offline helpers are low-cost if they stay out of runtime

Using Python for offline ingest and ML does not create the same architecture cost as splitting the live runtime:

- the artifact boundary is durable and testable
- the runtime can validate schema, source, and embedding identity before startup
- Python can use the stronger NLP/data ecosystem without putting Python packaging in the runtime path
- bulk embedding generation, clustering, and discovery evaluation remain easier to iterate outside the Rust binary

The important constraint is that Python writes versioned artifacts. It should not become a live query-embedding sidecar unless Rust query embedding proves unworkable.

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

### Prep tooling

Offline prep can remain TypeScript initially or move to Python if that improves ML/data iteration:

- parse and normalize the vendored Foundry PF2E checkout
- build the SQLite index
- generate document embeddings
- compute semantic input hashes
- write embedding identity metadata
- run discovery, clustering, and evaluation workflows
- emit review queues and reports

The prep side must treat the index schema and embedding manifest as public contracts, not incidental implementation details.

### Agent-facing CLI surface

The CLI should be designed for agents first, with human presentation as an optional mode.

Representative commands:

```bash
atlas lookup "Treat Wounds" --category rule --json
atlas search --query "low level healing spell" --category spell --limit 10 --json
atlas browse --category creature --filter '<canonical-filter-json>' --json
atlas filters list-values --field traits --category creature --json
atlas rule-context "Grab" --include-backlinks --json
atlas graph get --record-key actions:abc123 --include-backlinks --json
atlas tags review next --queue spell --json
atlas tags evaluate gaps --category spell --json
atlas tui
atlas mcp
```

CLI contracts should include:

- stable JSON schemas
- compact default output for agent use
- explicit `--pretty` or human mode when needed
- predictable exit codes
- concise stderr errors
- no hidden network access during runtime commands
- clear stale-index and embedding-mismatch diagnostics

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
- Do not introduce a Python runtime query sidecar unless Rust query embedding has been prototyped and rejected.
- Do not keep MCP as the architectural center if the product remains local-only.
- Do not create parallel TypeScript and Rust runtime cores with mixed feature ownership.
- Do not use Rust as a thin wrapper over the existing Node runtime; that would preserve the current architecture cost while adding another language.
- Do not migrate ingest first unless the runtime artifact contract has already been designed.
- Do not treat a compatibility bridge as progress toward the end state unless it removes an old path in the same slice.

## Open Questions

- Which embedding model should become the long-term cross-language contract?
  - Current default is `Xenova/all-MiniLM-L12-v2`.
  - Candidate upgrade path includes BGE-family models such as `BAAI/bge-small-en-v1.5`.
- Can Rust query embedding match ingest embeddings closely enough for current semantic search quality?
- Should document embedding generation use Python `fastembed`, Python `sentence-transformers`, TypeScript `@huggingface/transformers`, or the same Rust embedding stack as runtime?
- Is a Rust TUI meant to fully replace the Ink workbench, or should the first milestone be CLI-only?
- Which current MCP tools should have first-class CLI equivalents before any runtime migration begins?
- Which derived-tag editorial workflows are stable enough to model as Rust state machines now?
- Does the optional `atlas mcp` mode need the official Rust MCP SDK, or is MCP no longer worth preserving locally?

## Research Dependencies

This migration is design-in-progress and should not move into implementation until the research spikes in this folder produce enough evidence to choose the runtime and retrieval architecture.

Required decision inputs:

- [Rust query embedding spike](./spikes/rust-query-embedding.md)
- [Search quality bakeoff spike](./spikes/search-quality-bakeoff.md)
- [Tantivy lexical search spike](./spikes/tantivy-lexical-search.md)
- [LanceDB retrieval store spike](./spikes/lancedb-retrieval-store.md)
- [Rust CLI agent surface spike](./spikes/rust-cli-agent-surface.md)
- [Rust TUI state machine spike](./spikes/rust-tui-state-machine.md)
- [Artifact contract spike](./spikes/artifact-contract.md)
- [Canonical ingest ownership spike](./spikes/canonical-ingest-ownership.md)
- [Optional MCP compatibility spike](./spikes/optional-mcp-compatibility.md)

The first implementation plan should explicitly summarize the outcomes of these spikes, name the chosen retrieval stack, and identify any rejected alternatives that remain future options.

## Suggested Work Breakdown

### Phase 1: Decision Prototypes

1. Build a standalone Rust query-embedding prototype.
   - Use the same current model contract if feasible.
   - Also test a likely upgrade candidate such as a BGE small/base model.
   - Compare vector dimensions, normalization, runtime latency, startup latency, and packaging footprint.
   - Add fixture comparisons against the current TypeScript embedding provider.

2. Build a standalone Rust SQLite/vector-search prototype.
   - Open a small prepared SQLite fixture.
   - Run FTS search, vector search, and hybrid ranking.
   - Validate `sqlite-vec` or the chosen vector extension on the target development machines.
   - Measure top-k query latency and index-open latency.

3. Build a minimal Rust CLI prototype.
   - Implement `lookup`, `search`, and `rule-context` against a fixture or existing index.
   - Return stable compact JSON.
   - Exercise commands from Codex with a small local skill draft.

4. Build a minimal Rust TUI state-machine spike.
   - Implement one list/detail search result screen and one modal/prompt flow.
   - Use explicit enum state and transition functions.
   - Compare development friction against the current Ink workflow.

5. Record a decision.
   - If prototypes pass, add an ADR for Rust runtime plus Python/offline prep.
   - If prototypes fail, record which constraints blocked the migration and which parts remain useful for the Node architecture.

### Phase 2: Artifact Contract

1. Define the versioned SQLite runtime contract.
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

1. Create a Rust workspace in a branch or experimental directory.
2. Implement domain contracts without importing Node runtime concepts.
3. Implement index opening and metadata validation.
4. Implement typed lookup by canonical key and by name.
5. Implement record presentation models separate from CLI/TUI rendering.
6. Add tests around stale index, missing embedding provider, and schema mismatch errors.

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

1. Run Rust and TypeScript runtimes side by side against the same prepared index.
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
- stale-index and embedding-mismatch tests
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
