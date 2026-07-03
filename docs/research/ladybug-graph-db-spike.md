# LadybugDB Graph Database Spike

## Question

The spike evaluated whether PF2e Atlas should replace or supplement its SQLite artifact with a graph-native backend. The main candidate was LadybugDB, the maintained Kuzu fork exposed through the Rust `lbug` crate. GraphQLite was also tested as a SQLite-adjacent graph projection.

The preserved reference branch is:

```text
origin/spike/ladybug-db
```

## Decision

Do not adopt LadybugDB or GraphQLite as a production runtime dependency now.

SQLite remains the preferred artifact and query backend. The graph databases were useful research tools, but the spike did not find enough unique product value to justify the additional build, packaging, query-performance, concurrency, and architecture cost.

## What Worked

LadybugDB made graph-shaped questions easy to express:

- direct and inverse reference traversal;
- multi-hop path probes;
- same-evidence co-reference;
- relationship-aware expansion around mechanics such as conditions, actions, traits, and spells;
- graph-supported similarity experiments;
- variant, remaster, source, and impact-map exploration.

The graph model helped identify useful product ideas:

- exact evidence for "why did this record match?";
- same-content-unit queries such as "which specific ability mentions both X and Y?";
- backlinks and uses grouped by relation direction;
- variant and remaster navigation surfaces;
- semantic-first "similar" search with modest graph/trait evidence as explanation;
- precision-oriented FTS lanes for hybrid search.

## What Did Not Work Well Enough

LadybugDB did not prove compelling as the main runtime backend:

- ordinary read surfaces were usually slower than SQLite;
- persistent graph/vector/search artifacts added operational complexity;
- Rust/native packaging required source-build and extension-loading workarounds during the spike;
- embedded read concurrency and file-lock behavior need more product design than SQLite;
- robust FTS remained less proven than SQLite FTS5 and the existing SQLite-oriented search path;
- many graph queries were SQL-expressible with normal join tables;
- long-chain path explanations often explained database connectivity more than gameplay meaning;
- raw graph similarity was sparse or noisy unless heavily weighted and constrained.

GraphQLite proved a graph projection can live beside SQLite, but did not add enough value over ordinary SQLite relationship tables to justify keeping it as a dependency.

## Concrete Findings

Packaging and runtime operations were a major adoption cost:

- `lbug = "0.16.1"` worked in Rust, but the default prebuilt static archive path failed on macOS arm64 during the spike with missing native symbols such as `simsimd` and `yyjson`.
- `LBUG_RUST_BUILD_FROM_SOURCE=1` worked, but requires a native C++ toolchain and CMake for development, CI, and release builders.
- Loading Ladybug `FTS`, `VECTOR`, and `ALGO` extensions required an extension-capable binary. The spike binary needed `cargo:rustc-link-arg=-rdynamic` on non-Windows platforms.
- The Homebrew dynamic-link path worked only with an internal source include workaround, which is too brittle for a product install story.
- Concurrent embedded CLI reads against the same `.lbug` artifact could hit file-lock errors, unlike the normal SQLite read path.

Bulk loading was necessary and effective:

- The first row-by-row Cypher writer was not viable. A full build took about 86 minutes for Ladybug output, with relationship writes alone taking about 57 minutes.
- Parquet staging plus Ladybug `COPY FROM` reduced the Ladybug output phase to roughly 83 seconds on the same full corpus.
- The full-corpus graph writer proved the bulk shape against about 29,674 records, 82,382 content units, 73,239 reference edges, and copied embedding units.
- The lesson is general: if a future graph backend is evaluated, bulk ingestion must be part of the first real prototype rather than a later optimization.

Search and filtering findings were mixed:

- Direct key lookup, scalar filters, relationship-backed filters, metric range filters, and facet counts were all expressible in Ladybug.
- Ladybug vector search supported a projected-graph shape that could apply a parent-record filter before vector top-k. This was the strongest technical parity result for graph/vector integration.
- Ladybug FTS did not support the same projected-graph prefilter shape. It targets concrete indexed node tables, so arbitrary UI filters require overfetch/post-filtering, per-scope materialized indexes, or upstream extension work.
- Product-facing search comparison exposed that much of Ladybug's apparent slowness was initially reader-path overhead, especially identity resolution hydrating too much data before ranked search. Direct identity lookup fixed the worst case, but SQLite remained simpler and generally faster for ordinary product reads.
- SQLite's catalog path remained very strong for UI discovery. Ladybug dynamic graph discovery was competitive in some relationship-heavy cases, but not enough to displace the SQLite catalog model.

Graph product findings:

- One-hop links/backlinks, variant navigation, remaster links, source maps, and impact counts are useful product surfaces, but they are not graph-database-specific.
- Same-evidence and same-content co-reference was the most compelling graph-style product result.
- Relationship-aware "more like this" was useful only when semantic similarity stayed primary and graph/trait overlap was used as modest, explainable support.
- Long multi-hop explanations were easy to generate but often not useful enough; many paths were technically true without being a good user answer.

## Durable SQLite Learnings

The strongest reusable outcome is not a graph database. It is a narrower SQLite provenance model:

```text
reference_occurrences(record_key, content_key, occurrence_ordinal, target_record_key, source_kind, visibility, display_text, reference_text)
```

This table records which logical content unit contained a resolved reference without duplicating content text. Logical content keys identify:

- `description` for primary description content;
- `blurb` for generated/source blurb content;
- `content:N` for supplemental `record_content` rows by ordinal.

This preserves the useful part of evidence-level graph retrieval while keeping SQLite as the source of truth. It enables:

- same-content co-reference queries;
- future snippets/highlighting;
- better "why did this match?" explanations;
- UI drill-down from record-level links to source sections;
- impact and backlink views with concrete evidence.

Other useful lessons to carry forward:

- FTS should be treated as a precision lane, not broad conceptual recall.
- Weak OR-style FTS hits should be demoted or gated before hybrid fusion.
- Semantic search should own broad natural-language recall.
- Graph/trait overlap can supplement "similar" results but should not dominate semantic similarity.
- Product graph surfaces are still useful over SQLite when they expose direct links, uses/backlinks, variants, remaster links, and focused expansion.
- Research spikes should preserve conclusions under `docs/research/` instead of keeping large experimental crates in the production workspace.

## Recommended Integration Shape

Keep:

- SQLite artifact contract;
- `reference_occurrences`;
- precision FTS and hybrid-ranking lessons;
- SQLite-backed graph product surfaces that remain useful;
- embedding token-budget and document-unit improvements if they are backend-agnostic;
- architecture refactors that clarify crate ownership without preserving retired backends.

Remove or avoid:

- LadybugDB production read/write paths;
- GraphQLite projection writes;
- `atlas-ladybug-spike` from the production workspace;
- user-facing backend flags for retired graph databases;
- generic multi-backend traits whose only purpose is retaining retired code.

## Follow-Up Checks

Before completing cleanup:

```bash
rg -i "ladybug|lbug|graphqlite|kuzu"
cargo metadata --format-version 1
```

Remaining hits should be limited to research/history docs or intentionally preserved references. Production crates should not depend on LadybugDB or GraphQLite.
