# Source-backed enrichment corpus evidence

The database-independent record library retains the generated Foundry DTO and
derives owned identity, selected content, references, relationships and borrowed
query/text views. Its consuming ingest handoff preserves exact bytes, provenance,
pack metadata, admission diagnostics and unavailable/quarantined outcomes. See
[ADR 0045](../architecture/decisions/0045-source-backed-record-enrichment.md) and
the [initial projection inventory](./source-record-query-projections.md).

## Corpus checks

Measured 2026-10-09 against PF2e 6.12.4, commit
`4cbdaa37d6c33e9519561bae2c59a23e0288cbce`, using its `static/lang/en.json`.
Source contract: `6bf64da835272af22c53db729cb28d18a87e8dee999c11b154fb3e772befdeb8`.
The example uses an explicit public audience, with GM-only implicit check DCs;
this is a source-inspection choice, not a product default.

| Check | Result |
| --- | ---: |
| Discovered / retained / typed / addressed documents | 25,641 each |
| Partial documents / admission diagnostics | 203 / 267 |
| Raw-only / identity-unavailable / quarantined / unavailable packs | 0 each |
| Exact checked snapshots unchanged before/after enrichment | 25,641 |
| Exact decoded Rust model equality | 25,641 |
| Selected root projection values independently compared to authored JSON | 146,620 |
| Owned nodes, excluding roots | 82,069 |
| Prepared content fields with checked reference/interaction markers | 230,454 |
| Content references / structured relationship occurrences | 85,302 / 79,356 |
| Resolved document references / unverified HTML URLs | 85,300 / 2 |
| Unresolved / blocked recognized content references | 0 each |
| Content preparations returning an error | 0 |

Another 99,082 selected content-field outcomes are unavailable. These are field
states such as missing, null, invalid or non-applicable, not discarded documents.
The retained source remains the authority for their distinct states and rejected
values. No collection is shortened by salvaging raw neighbors.

External HTML URLs pass the sanitizer but are not checked for reachability;
their unverified status is separate from document identity resolution.

Fixtures cover corpus-absent families and invalid states: all eight Actor and
24 Item families, nested physical subitems and Consumable spells, duplicate and
missing IDs, unsupported formats, visibility, ambiguous names, exact-ID precedence,
unavailable-root name exclusion, same-child predicates, grants, provenance,
casting target families and resolved Embed cycles without expansion.

Content-field selection uses typed declarations plus actual upstream HTMLField,
sheet enrichHTML and editor-template evidence. Names and captions remain plain
strings. Item public/GM descriptions, Actor family prose, Journal HTML pages and
RollTable descriptions/results enter the shared interpretation library. Macro
commands, rule/patch payloads and arbitrary additional strings remain source data.

## Remaining content diagnostics

These count occurrences, not document rejection. Every source body is retained.

| Diagnostic | Count | Interpretation |
| --- | ---: | --- |
| EmbedNotExpanded | 2,171 | Destination identity/options retained; prose is not copied or recursively expanded |
| RuntimeContextRequired | 1,338 | Authored expressions retained without gameplay evaluation |
| UnknownActionGlyph | 1,489 | Existing preparation lacks some legacy glyph mappings; first samples use `A` |
| UnresolvedLocalization | 88 | Supplied locale lacks the requested key; authored evidence/diagnostic retained |
| MalformedSyntax | 1 | Broken enrichment syntax retained literally; first sample is a hazard disable field |

No source-specific repair or runtime formula engine is introduced. Before product
rendering adoption, review legacy glyph coverage and localization inputs, decide
Embed expansion and asset handling, and test leaf navigation/controls. Resolving
a journal page with a heading fragment establishes the page identity, not heading
existence. This evidence does not establish Foundry runtime or browser equivalence.

## Measured cost and storage implications

Environment: Darwin 25.5.0, arm64, Rust 1.98.1 (Homebrew), unoptimized developer
build. One complete corpus run; largest-root preparation repeated three times
with a warm resolver/locale, excluding source cloning from those intervals.
Times are observations, not performance guarantees or SQLite hydration timings.

| Serialized component | Bytes |
| --- | ---: |
| Original authored source | 239,431,946 |
| Checked typed snapshots | 288,348,086 |
| Complete enrichment JSON, including prepared content | 322,401,675 |
| Prepared HTML within enrichment | 53,731,012 |
| Prepared text within enrichment | 47,280,970 |

Load/admission took 2.24 seconds; identity indexing and enrichment 37.18 seconds;
checked snapshot encoding 9.01 seconds and decoding 10.97 seconds. The whole
proof process took 114.99 seconds, including pre-enrichment hashing, independent
comparisons, marker checks, serialization and repeated preparations. macOS
`time -l` measured maximum resident size 2,078,883,840 bytes and peak memory
footprint 1,775,159,240 bytes. The proof holds the whole corpus and evidence in
memory; these are not per-record runtime hydration costs.

| Largest authored root | Authored bytes | Snapshot bytes | Enrichment bytes | Three warm preparations, milliseconds |
| --- | ---: | ---: | ---: | --- |
| Journals: archetypes | 1,305,718 | 1,356,441 | 3,591,722 | 943 / 947 / 964 |
| Journals: ancestries | 383,776 | 396,197 | 827,012 | 154 / 151 / 149 |
| Journals: classes | 326,962 | 333,038 | 1,236,620 | 287 / 288 / 288 |
| Rinnarv Bontimar | 297,232 | 323,991 | 325,567 | 38 / 37 / 37 |
| Feiya, level 5 | 261,965 | 289,264 | 295,711 | 31 / 31 / 31 |

Enrichment JSON exceeds the typed body size: repeated locators/availability and
prepared strings have a real cost. Its serialization measures the library result;
it is not the proposed artifact encoding. Physical storage must evaluate compact
owner/field references, compression and selective prepared-content caching against
actual hydration and query needs. Retaining source data does not require loading
every corpus buffer into an application response or persisting every measured
representation. FTS weighting, semantic units/pooling, filter SQL, graph eligibility
and product response contracts remain adoption work.

## Reproduction and limits

Use the [contributor command](../../CONTRIBUTING.md#source-declaration-research):

```sh
cargo run -p atlas-ingest --example source_enrichment_probe -- /path/to/pinned/pf2e /path/to/pinned/pf2e/static/lang/en.json
```

For process memory/timing on macOS, build the example first and run the produced
binary under `/usr/bin/time -l`. Resource counters require permission outside
some sandboxes. The report stays on stdout; full dumps remain ignored developer
evidence rather than a tracked corpus receipt or published command surface.

The full Rust fmt/clippy/test/build gate passed. The existing product artifact,
metrics and RichDocument consumers remain active until their coherent cutover.
This proof establishes source retention, extraction and marker consistency; it
does not establish search relevance, embedding quality, Foundry preparation,
browser routes or database performance. Verified alias policy and product audience
defaults are still decisions rather than inferred fields.
