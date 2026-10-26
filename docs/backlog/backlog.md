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

- [Reusable encounter party definitions](items/encounter-party-presets.md)
  Define a party once and add its members to encounters without repeatedly
  entering their details. Name-only implementation in progress; awaiting UX review. Status: in_progress.

- [Reusable development baseline](items/development-baseline.md)
  Publish validated executable/workspace seeds and initialize independent
  development copies without per-workstream ingest. Status: in_progress.

- [Source-modeling capability follow-through](items/rust-integration-recovery.md)
  Track the agreed ten-item sequence for CLI contracts, reference completeness,
  encounter policy and discovery. CF-01 merged in PR #68; CF-02 merged in PR #69. The 6.12.4 reference pin merged in PR #70; the 8.6.1 upgrade merged in PR #83. CF-03 merged in PR #71; consult the capability tracker for later slice approval and progress.
  Status: in_progress.

- [Rust CLI and skill capability follow-through](items/rust-cli-skill-capability-follow-through.md)
  Preserve useful product-surface ideas as CLI command, JSON contract, setup, discovery, graph, and skill-guidance improvements. Status: proposed.

- [Source-backed search quality evaluation](items/rust-search-quality-tuning.md)
  Evaluate lexical evidence, associated-content influence and semantic inputs
  against fresh source-backed tasks; preserve completed study evidence. Status: proposed.

## Soon

- [Apply supported effects to encounter participants](items/encounter-effect-application.md)
  Prefill conditions from supported spells and abilities through the shared workflow.
  Status: proposed.

- [Cursebound encounter mechanics](items/encounter-cursebound.md)
  Deferred from CF-07 pending concrete GM encounter demand. Status: deferred.

- [Remaining localization-backed rules terms](items/rust-localization-backed-rules-terms.md)
  Investigate NPC glossary definitions and verified prose rules destinations;
  trait reference pages shipped in PR93. Status: proposed.

- [Trait description content rights and attribution](items/trait-description-content-rights.md)
  Establish paragraph-level permission and attribution evidence for localized
  descriptions, including mixed setting/rules text. Status: proposed.

- [Foundry inline content fidelity](items/foundry-inline-content-fidelity.md)
  Compare remaining rich-text syntax and context-dependent expressions with the
  pinned upstream handlers before extending inline mechanics. Status: proposed.

- [Ingest and reference build performance](items/reference-build-performance.md)
  PR74 completed the measured optimization; further work targets remaining
  segmentation/tokenization and evidenced inference bottlenecks. Status: proposed.

- [Downstream architecture follow-through](items/rust-downstream-architecture-follow-through.md)
  Resolve remaining bounded consumer ownership and error-contract gaps from the
  downstream review. Status: planned.

- [Rust Ratatui workbench](items/rust-ratatui-workbench.md)
  Build the interactive terminal workbench over the Rust runtime for search, browse, detail reading, filter exploration, graph context, and future editorial workflows. Status: planned.

- [Rust CLI kind preview facts](items/rust-cli-kind-preview-facts.md)
  Add concise kind-specific scan facts to Atlas search preview output so users can reject or select candidates with fewer follow-up lookups. Status: proposed.

- [Query spelling and name suggestions](items/rust-cli-typo-tolerant-discovery.md)
  Evaluate shared CLI/UI spelling suggestions and corpus vocabulary while
  preserving strict name resolution and semantic discovery. Status: proposed.

- [Rust web filter UX expansion](items/rust-web-filter-ux-expansion.md)
  Refine standard and optional web filters, field grouping, labels, counts, and progressive disclosure beyond the completed shared filter/counts pass. Status: proposed.

- [Rust web record detail polish](items/rust-web-record-detail-polish.md)
  Improve record detail readability, navigation, loading states, and shared presentation-contract usage beyond the completed reader/rank passes. Status: proposed.

- [Rust web encounter runner polish](items/rust-web-encounter-runner-polish.md)
  Prioritize combat state and actions, improve roster readability, and reuse
  shared family layouts beyond the completed workflow/dice passes. Status: proposed.

- [Spell form preview](items/rust-spell-rank-preview.md)
  Add interactive alternate-form selection over checked source, composing with
  rank preview and preserving embedded customizations. Status: deferred.

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
  Preserve prior ontology/assessment research; revisit only for demonstrated gaps
  in native filters and semantic search. Status: deferred.

- [Rust CLI content output formats](items/rust-cli-content-output-formats.md)
  Evaluate unmet content-export needs against the current human, reference JSON
  and checked-source surfaces. Status: deferred.

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
  Complete standard condition automation across creature facts, attacks, damage,
  spellcasting and turn effects, with explicit context where required. Status: planned.

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

The [runtime roadmap](rust-cli-runtime/README.md) groups cross-cutting follow-ups.
