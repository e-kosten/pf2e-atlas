# Backlog

This is the tracked backlog for open durable future work.

## Status Vocabulary

- `proposed`
- `planned`
- `in_progress`
- `blocked`
- `deferred`

Completed and retired items are tracked separately in [history/done-and-superseded.md](history/done-and-superseded.md).

## Now

- [Selective integration recovery](items/rust-integration-recovery.md)
  Recover reviewed behavior in small branches from main; retain integration as
  evidence and a preview. Status: planned.

- [Rust CLI runtime migration research](rust-cli-runtime/README.md)
  Working notes and checklist for the Rust runtime, CLI, artifact, search, graph, skill, and cutover work. Status: in_progress.

- [Rust CLI and skill capability follow-through](items/rust-cli-skill-capability-follow-through.md)
  Preserve useful product-surface ideas as CLI command, JSON contract, setup, discovery, graph, and skill-guidance improvements. Status: proposed.

- [Source-backed search quality evaluation](items/rust-search-quality-tuning.md)
  Evaluate source-backed lexical/semantic selection, explanatory witnesses,
  ranking and bounded candidate windows with judged real-model runs. Status: proposed.

## Soon

- [Rust Ratatui workbench](items/rust-ratatui-workbench.md)
  Build the interactive terminal workbench over the Rust runtime for search, browse, detail reading, filter exploration, graph context, and future editorial workflows. Status: planned.

- [Rust CLI kind preview facts](items/rust-cli-kind-preview-facts.md)
  Add concise kind-specific scan facts to Atlas search preview output so users can reject or select candidates with fewer follow-up lookups. Status: proposed.

- [Rust CLI typo tolerant discovery](items/rust-cli-typo-tolerant-discovery.md)
  Track backend-independent typo suggestions, corpus-token dictionaries, and acronym expansion without weakening strict record resolution. Status: proposed.

- [Record presentation and UI architecture review](items/rust-web-ui-architecture-review.md)
  Review from ingest through borrowed views and app contracts to CLI/UI, using
  the earlier integration branch for feature inspiration. Refine presentation
  boundaries before family-specific UI expansion. Status: proposed.

- [Rust web filter UX expansion](items/rust-web-filter-ux-expansion.md)
  Refine standard and optional web filters, field grouping, labels, counts, and progressive disclosure for the search/browse workflow. Status: proposed.

- [Rust web record detail polish](items/rust-web-record-detail-polish.md)
  Improve record detail readability, navigation, loading states, and shared presentation-contract usage in the web UI. Status: proposed.

- [Rust localization-backed rules terms](items/rust-localization-backed-rules-terms.md)
  Persist selected localization-backed rules vocabulary such as traits and NPC ability glossary entries as canonical hoverable/linkable terms. Status: proposed.

- [Rust web keyboard navigation and focus](items/rust-web-keyboard-navigation-focus.md)
  Define a coherent keyboard-driven web workflow across search, filters, results, detail panes, and editable controls. Status: proposed.

- [Rust web form accessibility audit](items/rust-web-form-accessibility-audit.md)
  Standardize accessible labels and form semantics across Ant Design forms and compact editable controls. Status: proposed.

## Later

- [Source query catalog expansion](items/source-query-catalog-expansion.md)
  Add useful authored fields and collection scopes with shared catalog semantics
  and direct DTO evidence. Status: deferred.

- [Foundry-inspired product actions](items/foundry-inspired-product-actions.md)
  Evaluate selected controls and contextual mechanics without executing source
  Macro scripts or exposing Macro records in ordinary product surfaces. Status: deferred.

- [Rust derived-tag runtime and editorial redesign](items/rust-derived-tag-redesign.md)
  Redesign retained derived-tag concepts against record kinds, explicit source axes, typed filters, and Rust artifact ownership. Status: planned.

- [Rust CLI content output formats](items/rust-cli-content-output-formats.md)
  Decide which optional non-markdown content formats the Rust CLI should expose for record JSON output. Status: deferred.

- [Rust graph context deeper local graph](items/rust-graph-context-deeper-local-graph.md)
  Track secondary links, shared-neighbor scoring, local cluster signals, and degree-aware curation after the V1 one-hop graph context command. Status: proposed.

- [Rust web text-scoped filter counts](items/rust-web-text-scoped-filter-counts.md)
  Refine query-scoped facet presentation and broader contexts beyond the current
  supported shared discovery relation. Status: deferred.

- [Rust web Vite bundle cleanup](items/rust-web-vite-bundle-cleanup.md)
  Evaluate and reduce large frontend chunks once the UI shape stabilizes. Status: proposed.

- [Rust FTS tokenization and stemming exploration](items/rust-fts-tokenization-stemming.md)
  Evaluate SQLite FTS tokenizer/stemming choices for inflection handling. Status: proposed.

- [Rust optional PF2e art ingest](items/rust-optional-pf2e-art-ingest.md)
  Track optional local ingestion of PF2e system icons and module-provided creature portrait/token art for future web presentation. Status: proposed.

- [Rust encounter saved-list import](items/rust-encounter-saved-list-import.md)
  Create runnable encounters from saved lists while preserving saved-list order and explicit unresolved-item handling. Status: proposed.

- [Rust encounter elite and weak projection](items/rust-encounter-elite-weak-projection.md)
  Extend supported adjustment targets with explicit rule evidence beyond the
  current source-backed stat and HP projections. Status: proposed.

- [Rust encounter condition mechanics](items/rust-encounter-condition-mechanics.md)
  Move beyond durable condition annotations into explicit, tested mechanical condition projections where PF2e rules are deterministic. Status: proposed.

- [Rust encounter actor context effects](items/rust-encounter-actor-context-effects.md)
  Model future actor-vs-actor encounter effects such as visibility, targeting, and situational condition reminders without forcing them into participant-local stat mutation. Status: proposed.

- [Rust encounter runtime mechanics surfaces](items/rust-encounter-runtime-mechanics-surfaces.md)
  Extend movement/action explanations and effect targets beyond the initial
  bounded condition projections. Status: proposed.

- [Rust encounter trait runtime effects](items/rust-encounter-trait-runtime-effects.md)
  Let record traits and future participant-applied effects feed the shared encounter runtime-effect machinery, starting with minion action-economy projection. Status: proposed.

- [Rust encounter turn lifecycle mutations](items/rust-encounter-turn-lifecycle-mutations.md)
  Add backend-owned turn event hooks for duration ticks, stunned action-loss timing, and other deterministic encounter lifecycle mutations. Status: proposed.

- [Rust local-state import and export](items/rust-local-state-import-export.md)
  Add scriptable import/export for saved lists and future durable local-state data without making artifact rebuilds responsible for user state. Status: proposed.

See [Backlog Done / Superseded](history/done-and-superseded.md) for completed and retired items.
