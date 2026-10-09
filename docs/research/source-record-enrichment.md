# Source-backed enrichment corpus evidence

The database-independent record library retains only the key and generated Foundry
DTO. Explicit preparation operations derive selected content, references and
relationships; query/text accessors borrow source fields. Its consuming ingest
handoff carries these separate outputs and preserves exact bytes, provenance,
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
| Embedded documents counted through shared traversal, excluding roots | 82,069 |
| Prepared rich-content fields with checked reference/interaction markers | 117,937 |
| Content references / structured relationship occurrences | 85,302 / 79,356 |
| Resolved document references / unverified HTML URLs | 85,300 / 2 |
| Unresolved / blocked recognized content references | 0 each |
| Content preparations returning an error | 0 |

Source availability and rejected values remain in the DTO. Preparation no longer
emits rows for ordinary missing/null/invalid/non-applicable text fields; present
rich fields still produce an explicit prepared/unsupported/failed outcome,
including valid empty and hidden content. No collection is shortened by salvaging
raw neighbors. The developer report counts embedded documents on demand rather
than retaining node or collection inventories on records.

The previous candidate prepared 230,454 fields and retained another 99,082
unavailable outcomes. The revision removes those unavailable rows and 112,517
prepared copies of plain fields. Names, captions and other declared plain text
remain available through borrowed text sources, bypassing macro interpretation.

An independently compiled copy of the previous candidate and the final revision
emit the same 272,892 nonempty text-source rows. Their sorted, length-prefixed
SHA-256 fingerprint is
`a3987daca28082f7b7970aa9966668abf0852978f71d284ad9c2d9f3ee5ea621`.
The comparison includes record key, owner chain, field, text kind, visibility and
text, preserving duplicate occurrences. Total rows change from 305,313 to 304,533
because direct plain-field access omits 780 audience-ineligible empty placeholders;
present empty rich-text results remain. Reference/relationship counts and admission/
content diagnostic counts are unchanged. This checks emitted text fidelity for
the supplied corpus/context, not search ranking or rendering equivalence.

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
build. Sizes and timings below come from the first complete revision corpus run;
a second run verifies final text fingerprints and repeats retention/projection/
marker checks. Largest-root preparation repeats three times with a warm
resolver/locale, borrowing the unchanged record in those intervals.
Times are observations, not performance guarantees or SQLite hydration timings.

| Serialized component | Bytes |
| --- | ---: |
| Original authored source | 239,431,946 |
| Checked typed snapshots | 288,348,086 |
| Separate content and relationship outputs, measured as a JSON tuple | 196,831,541 |
| Prepared HTML within content outputs | 52,196,570 |
| Prepared text within content outputs | 45,746,652 |

Load/admission took 2.52 seconds; identity indexing and preparation 39.56 seconds;
checked snapshot encoding 9.53 seconds and decoding 11.72 seconds. No whole-process
memory measurement was taken for this revision. The proof holds the whole corpus
and evidence in memory; these are not per-record runtime hydration costs.

The earlier complete enrichment JSON measured 322,401,675 bytes. Separate output
now measures 125,570,134 bytes less (39%). Prepared HTML/text shrink because plain
source fields are borrowed, not because their authored content is discarded.

| Largest authored root | Authored bytes | Snapshot bytes | Derived output bytes | Three warm preparations, milliseconds |
| --- | ---: | ---: | ---: | --- |
| Journals: archetypes | 1,305,718 | 1,356,441 | 3,337,696 | 986 / 1,000 / 990 |
| Journals: ancestries | 383,776 | 396,197 | 768,807 | 171 / 158 / 157 |
| Journals: classes | 326,962 | 333,038 | 1,208,286 | 297 / 296 / 298 |
| Rinnarv Bontimar | 297,232 | 323,991 | 223,213 | 40 / 39 / 39 |
| Feiya, level 5 | 261,965 | 289,264 | 190,159 | 34 / 32 / 33 |

The serialized derived outputs are smaller than the typed bodies, but prepared
strings and occurrence metadata still have a real cost. Serialization measures
the library result;
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
