# Backlog

This is the tracked backlog for open durable future work.

## Status Vocabulary

- `proposed`
- `planned`
- `in_progress`
- `blocked`
- `deferred`

Completed and retired items are tracked separately in [history/done-and-superseded.md](./history/done-and-superseded.md).

## Source-Faithful Contract State

The source-faithful record contract in [ADRs 0033-0036](../architecture/decisions/README.md) is accepted. Its creature implementation now runs through the G1 direct cutover: canonical creature bodies are the only hydrated creature presentation/runtime source, while generic record mechanics remain bounded to non-creature families and one-way query metrics remain available for filtering and discovery. Later record families remain separate owned work and are not implied by the creature cutover.

The contract preserves all 313 exact type-registry assignments, keeps non-creature priority as a later decision, and retains overflow-only child embeddings. The product remains default-visible/public-only rather than an authentication boundary: typed visibility/provenance is data, and target exclusions require non-auth product rationale.

## Now

- [Rust CLI runtime migration research](./rust-cli-runtime/README.md)
  Working notes and checklist for the Rust runtime, CLI, artifact, search, graph, skill, and cutover work. Status: in_progress.

- [Rust CLI and skill capability follow-through](./items/rust-cli-skill-capability-follow-through.md)
  Preserve useful product-surface ideas as CLI command, JSON contract, setup, discovery, graph, and skill-guidance improvements. Status: proposed.

- [Rust search quality and retrieval weight tuning](./items/rust-search-quality-tuning.md)
  Tune FTS weights, RRF windows, rank constants, and weighted-fusion defaults with measured quality runs. Status: proposed.

- [Rust creature level filtering](./items/rust-creature-level-filtering.md)
  Make shared level filters work for creature records through the Rust Atlas CLI and canonical filter path. Status: proposed.

## Soon

- [Rust Ratatui workbench](./items/rust-ratatui-workbench.md)
  Build the interactive terminal workbench over the Rust runtime for search, browse, detail reading, filter exploration, graph context, and future editorial workflows. Status: planned.

- [Rust CLI kind preview facts](./items/rust-cli-kind-preview-facts.md)
  Add concise kind-specific scan facts to Atlas search preview output so users can reject or select candidates with fewer follow-up lookups. Status: proposed.

- [Rust CLI typo tolerant discovery](./items/rust-cli-typo-tolerant-discovery.md)
  Track backend-independent typo suggestions, corpus-token dictionaries, and acronym expansion without weakening strict record resolution. Status: proposed.

- [Rust Foundry JSON field audit](./items/rust-foundry-json-field-audit.md)
  Add an explicit offline audit that inventories Foundry source JSON fields against Rust ingest coverage. Status: active.

- [Rust web UI architecture review](./items/rust-web-ui-architecture-review.md)
  Review the first web UI vertical slice for DTO/API boundaries, state ownership, AntD composition, module layout, and tests before substantial follow-up feature work. Status: proposed.

- [Rust web filter UX expansion](./items/rust-web-filter-ux-expansion.md)
  Refine standard and optional web filters, field grouping, labels, counts, and progressive disclosure for the search/browse workflow. Status: proposed.

- [Rust web filter state policy hardening](./items/rust-web-filter-state-policy-hardening.md)
  Require backend-derived operator policy for option-field value mutations so frontend filter helpers cannot silently fall back to local operator assumptions. Status: proposed.

- [Rust web record detail polish](./items/rust-web-record-detail-polish.md)
  Improve record detail readability, navigation, loading states, and shared presentation-contract usage in the web UI. Status: proposed.

- [Rust localization-backed rules terms](./items/rust-localization-backed-rules-terms.md)
  Persist selected localization-backed rules vocabulary such as traits and NPC ability glossary entries as canonical hoverable/linkable terms. Status: proposed.

- [Rust web keyboard navigation and focus](./items/rust-web-keyboard-navigation-focus.md)
  Define a coherent keyboard-driven web workflow across search, filters, results, detail panes, and editable controls. Status: proposed.

- [Rust web form accessibility audit](./items/rust-web-form-accessibility-audit.md)
  Standardize accessible labels and form semantics across Ant Design forms and compact editable controls. Status: proposed.

## Later

- [Rust derived-tag runtime and editorial redesign](./items/rust-derived-tag-redesign.md)
  Redesign retained derived-tag concepts against record kinds, explicit source axes, typed filters, and Rust artifact ownership. Status: planned.

- [Rust content subdocuments for journal pages and table results](./items/rust-content-subdocuments-journal-table-results.md)
  Preserve the deferred design question for journal-page and rollable-table-result rich text as child content. Status: deferred.

- [Rust CLI content output formats](./items/rust-cli-content-output-formats.md)
  Decide which optional non-markdown content formats the Rust CLI should expose for record JSON output. Status: deferred.

- [Rust entity-aware query facts and filter modeling](./items/rust-entity-aware-query-facts-and-filter-modeling.md)
  Evaluate family-owned typed query-facts projections and entity-aware product filters over a common query/filter IR without creating a second semantic model. Status: proposed.

- [Rust side data and metric source fact convergence](./items/rust-side-data-metric-source-fact-convergence.md)
  Track cleanup for overlapping Rust side-table and metric projections. Status: proposed.

- [Rust Foundry type mechanics parsers](./items/rust-foundry-type-mechanics-parsers.md)
  Evaluate peer Foundry-type mechanics parsers for source facts currently exposed primarily through metrics. Status: proposed.

- [Rust graph context deeper local graph](./items/rust-graph-context-deeper-local-graph.md)
  Track secondary links, shared-neighbor scoring, local cluster signals, and degree-aware curation after the V1 one-hop graph context command. Status: proposed.

- [Rust web text-scoped filter counts](./items/rust-web-text-scoped-filter-counts.md)
  Design query-aware dynamic filter counts for the web search experience without duplicating text retrieval semantics in the app layer. Status: deferred.

- [Rust web Vite bundle cleanup](./items/rust-web-vite-bundle-cleanup.md)
  Evaluate and reduce large frontend chunks once the UI shape stabilizes. Status: proposed.

- [Rust FTS tokenization and stemming exploration](./items/rust-fts-tokenization-stemming.md)
  Evaluate SQLite FTS tokenizer/stemming choices for inflection handling. Status: proposed.

- [Rust artifact JSON content model review](./items/rust-artifact-json-content-model-review.md)
  Decide whether rich content JSON, content rows, and reference occurrence provenance should be refactored into a stronger artifact model. Status: proposed.

- [Rust optional PF2e art ingest](./items/rust-optional-pf2e-art-ingest.md)
  Track optional local ingestion of PF2e system icons and module-provided creature portrait/token art for future web presentation. Status: proposed.

- [Rust encounter saved-list import](./items/rust-encounter-saved-list-import.md)
  Create runnable encounters from saved lists while preserving saved-list order and explicit unresolved-item handling. Status: proposed.

- [Rust encounter elite and weak projection](./items/rust-encounter-elite-weak-projection.md)
  Add creature-only effective stat projections for elite and weak encounter adjustments without persisting copied stat blocks. Status: proposed.

- [Rust encounter condition mechanics](./items/rust-encounter-condition-mechanics.md)
  Move beyond durable condition annotations into explicit, tested mechanical condition projections where PF2e rules are deterministic. Status: proposed.

- [Rust encounter actor context effects](./items/rust-encounter-actor-context-effects.md)
  Model future actor-vs-actor encounter effects such as visibility, targeting, and situational condition reminders without forcing them into participant-local stat mutation. Status: proposed.

- [Rust encounter runtime mechanics surfaces](./items/rust-encounter-runtime-mechanics-surfaces.md)
  Add speed and action-budget projections so deterministic encounter effects can target runtime surfaces beyond ordinary stat rows. Status: proposed.

- [Rust encounter trait runtime effects](./items/rust-encounter-trait-runtime-effects.md)
  Let record traits and future participant-applied effects feed the shared encounter runtime-effect machinery, starting with minion action-economy projection. Status: proposed.

- [Rust encounter turn lifecycle mutations](./items/rust-encounter-turn-lifecycle-mutations.md)
  Add backend-owned turn event hooks for duration ticks, stunned action-loss timing, and other deterministic encounter lifecycle mutations. Status: proposed.

- [Rust local-state import and export](./items/rust-local-state-import-export.md)
  Add scriptable import/export for saved lists and future durable local-state data without making artifact rebuilds responsible for user state. Status: proposed.

- [Rust RichDocument child retrieval policy](./items/rust-rich-document-child-retrieval-policy.md)
  Evaluate whether targeted child embeddings for body or structured rich content improve retrieval beyond the overflow-only RichDocument baseline. Status: proposed.

See [Backlog Done / Superseded](./history/done-and-superseded.md) for completed and retired items.
