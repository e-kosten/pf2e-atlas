# Maintained Rust source portfolio

## Scope and identity

The checked-in authored source portfolio covers all 47 extracted roots from
PF2e 6.12.4: Actor and Item, JournalEntry, Macro, RollTable, and 42 built-in rule
schemas. Actor/Item unions cover eight/24 registered families. The source pin is
`4cbdaa37d6c33e9519561bae2c59a23e0288cbce`, with source digest
`6bf64da835272af22c53db729cb28d18a87e8dee999c11b154fb3e772befdeb8`.
TypeScript compiler version is 5.9.3.

Fresh extraction retains original schema closures, serialization evidence and authored
projections in ignored caches. The small source pin records upstream identity;
the selected input contains 3,333 graph nodes. Generation emits 43,403 Rust lines in
96 files, including module indexes. These are generated outputs, not manually
maintained parser implementations. Regeneration and whole-file-set freshness
checks use the same full selection recipe and freshly extracted pinned source.

## Structure and use

Common Item/Actor/physical components precede family refinements. Explicit family
source/system roots reserve their module even when first reached through another
family's embedded optional source. Global structural owners reuse equivalent
payloads; extraction evidence retains distinct declaration constraints. Other document
kinds and specific rules have separate modules. Existing field-level slices
remain useful callable projections of the same source space.

`atlas_ingest::source_model` exposes byte parsers for the five document kinds and
a keyed `RuleSource`. `source_model::generated` exposes typed values and selected
root parsers taking an ordered SourceValue and SourceContext. These parsers retain
missing/null/value states before defaults, ordered additional fields and ordinary
numeric values. Small closed vocabularies remain checked; explicit broad trait
identifier policies remain strings. Large object union payloads are boxed through
a bounded layout heuristic; RuleSource boxes each specific rule payload.

Item.rules deliberately retains upstream's generic RuleElementSource. Successful
document parsing does not establish specific-rule acceptance; parse those rules
separately. An invalid supplied modeled field rejects the containing parsed
document. No fields or documents are silently discarded or repaired.

## Corpus evidence

The contributor `compare-portfolio` command compiles the actual maintained
atlas-ingest example. It does not substitute newly emitted scratch parsers.
Sampling includes root Actors, root/embedded/nested Items, the other three pack
document kinds, and specific rules from all sampled Items. Raw JSON payloads reach
Rust without a JavaScript numeric round trip. The shared fidelity oracle checks
typed values, presence and ordered additional members against freshly extracted declarations.

| Space | Occurrences | Accepted | Rejected |
| --- | ---: | ---: | ---: |
| Actors and Items | 105,395 | 105,113 | 282 |
| JournalEntry | 113 | 113 | 0 |
| Macro | 81 | 81 | 0 |
| RollTable | 69 | 69 | 0 |
| Specific rules | 31,174 | 31,167 | 7 |
| Total | 136,832 | 136,543 | 289 |

There are zero measured fidelity failures. Compared with the preceding authored
document probe, acceptance, first-error paths and actual values are unchanged.
Four expected-type diagnostic labels change with shared generated owner names.
The specific-rule outcomes match the preceding authored rule comparison.

The saved `fixtures/portfolio-corpus-baseline.json` records source/corpus identity,
counts and a digest of every failed context/result. It detects changed corpus
bytes and changed first-error outcomes; it is not an ingest allowlist. Known
rejections still make the diagnostic command return 1 when the baseline matches.
Full per-root counts and failures remain in the generated comparison report.
Changing the upstream pin requires remeasuring and reviewing the baseline.

The [authored document report](./authored-document-source.md) and
[authored rule report](./authored-rule-source.md) retain the constrained failure
dispositions. Occurrences can overlap: an Actor and an embedded Item can both
reject for one issue. First-error reporting can conceal additional problems.
Not every rejected value is established to be invalid in Foundry.

## Validation and limits

The Rust workspace fmt, broad/strict Clippy, tests and build gate passed. The
private TypeScript package passes typechecking and fixture/freshness tests.
Focused Rust tests exercise every Actor/Item family and every specific rule key,
unknown-key rejection, presence/additional data, authored patches/sentinels,
the other document kinds, nested failures and bounded top-level enum layout.
The developer baseline tests distinguish acceptance, context, input and fidelity
changes. Corpus files are local evidence; CI runs deterministic fixtures and
generation freshness without fetching the upstream corpus.

The generated models cover the pinned extracted declarations and explicit
authored projections. Optional/rest tuples and unanchored recursive aliases
remain unsupported general shapes; they do not occur in this portfolio. Explicit
upstream open domains retain raw data. No claim is made to model arbitrary module
extensions, undocumented source, or all Foundry runtime admission constraints.

The production ingest pipeline, normalized records, database, metrics and UI are
unchanged. Foundry core cleaning/migration evidence remains unavailable. Later
ingest work must decide how rejected raw documents/rules are retained and reported,
how contextual patches are applied and validated, and which source values become
canonical product facts. No rejection/drop policy is selected by this PR.
