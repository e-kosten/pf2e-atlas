# Backlog

This is the tracked backlog for open durable future work.

## Status Vocabulary

- `proposed`
- `planned`
- `in_progress`
- `blocked`
- `deferred`

Completed and retired items are tracked separately in [history/done-and-superseded.md](./history/done-and-superseded.md).

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
  Add an explicit offline audit that inventories Foundry source JSON fields against Rust ingest coverage. Status: proposed.

## Later

- [Rust derived-tag runtime and editorial redesign](./items/rust-derived-tag-redesign.md)
  Redesign retained derived-tag concepts against record kinds, explicit source axes, typed filters, and Rust artifact ownership. Status: planned.

- [Rust content subdocuments for journal pages and table results](./items/rust-content-subdocuments-journal-table-results.md)
  Preserve the deferred design question for journal-page and rollable-table-result rich text as child content. Status: deferred.

- [Rust CLI content output formats](./items/rust-cli-content-output-formats.md)
  Decide which optional non-markdown content formats the Rust CLI should expose for record JSON output. Status: deferred.

- [Rust side data and metric source fact convergence](./items/rust-side-data-metric-source-fact-convergence.md)
  Track cleanup for overlapping Rust side-table and metric projections. Status: proposed.

- [Rust Foundry type mechanics parsers](./items/rust-foundry-type-mechanics-parsers.md)
  Evaluate peer Foundry-type mechanics parsers for source facts currently exposed primarily through metrics. Status: proposed.

- [Rust graph context deeper local graph](./items/rust-graph-context-deeper-local-graph.md)
  Track secondary links, shared-neighbor scoring, local cluster signals, and degree-aware curation after the V1 one-hop graph context command. Status: proposed.

- [Rust FTS tokenization and stemming exploration](./items/rust-fts-tokenization-stemming.md)
  Evaluate SQLite FTS tokenizer/stemming choices for inflection handling. Status: proposed.

- [Rust artifact JSON content model review](./items/rust-artifact-json-content-model-review.md)
  Decide whether rich content JSON, content rows, and reference occurrence provenance should be refactored into a stronger artifact model. Status: proposed.

- [Rust optional PF2e art ingest](./items/rust-optional-pf2e-art-ingest.md)
  Track optional local ingestion of PF2e system icons and module-provided creature portrait/token art for future web presentation. Status: proposed.

- [Rust RichDocument child retrieval policy](./items/rust-rich-document-child-retrieval-policy.md)
  Evaluate whether targeted child embeddings for body or structured rich content improve retrieval beyond the overflow-only RichDocument baseline. Status: proposed.

See [Backlog Done / Superseded](./history/done-and-superseded.md) for completed and retired items.
