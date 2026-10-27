# Core section policy census

Date: 2026-10-27. This is implementation coverage and an operational metadata probe, not a relevance or decay study. The initial controls retain the existing defaults.

The retained generation-7 / schema-17 artifact uses `foundry-content/v10` at source `28d9d156f252d084371d994dc359509b49eadf18`. Its checked source/section contracts support the typed Core policy. The separate latest-main v22 preparation refresh and actual Core publication are not measured by this rollback-only probe.

## Coverage

| Policy family | Records | Identity and prose | Identity only | Core sections |
| --- | ---: | ---: | ---: | ---: |
| army | 22 | 22 | 0 | 112 |
| campaign feature | 115 | 115 | 0 | 115 |
| character | 160 | 157 | 3 | 644 |
| character option | 954 | 954 | 0 | 1,048 |
| deity | 483 | 483 | 0 | 483 |
| equipment | 5,914 | 5,715 | 199 | 5,790 |
| familiar | 5 | 5 | 0 | 19 |
| hazard | 1,231 | 1,231 | 0 | 3,498 |
| npc | 6,442 | 5,008 | 1,434 | 6,490 |
| rule | 12,186 | 12,176 | 10 | 12,976 |
| spell | 2,017 | 2,014 | 3 | 2,041 |
| unmodelled | 182 | 0 | 182 | 0 |
| vehicle | 102 | 93 | 9 | 259 |

All 29,813 product records are accounted for; 81 Macro records are excluded from product search. Core membership contains 33,475 prose sections, 67,234 shared semantic locations (including root identity), and 34,806 existing lexical units. Both stored profiles have one identity per product root and no missing Core-section vector presence. This census does not repeat complete numerical vector validation.

Traits are separate reference entities: 668 substantive definitions and 116 label-only definitions, with stored representation in both profiles and no added record-style identity. Audience counts include declared HTML and Plain fields, availability, admitted fields and Core fields. Exact-owner/empty-description name rules and privacy/category exclusions also have typed regression fixtures.

## Operational evidence and limits

The optimized debug probe used about 132 seconds and 60.6 MiB peak RSS. It ran with warm caches and concurrent background preparation, so these are observations rather than service guarantees or a controlled speedup. Net metadata/schema allocation was 12,795,904 bytes. The transaction rolled back; all 87 existing table fingerprints, reference context and generation matched before/after, including FTS, native-vector shadow blobs and saved state.

Do not infer relevance, candidate recall, a preferred decay or a new default from coverage or metadata cost. The original Similar study remains unchanged; broader Core evaluation is deferred.

## Subsequent v22 implementation checks

The separate real runtime upgrade was verified against implementation commit
`74ef423e31eda87cbfce6a0f15e0844d4ccc2939`, on the same source pin with
`foundry-content/v22`. Schema 17 became 18 and reference generation 8 became 9
exactly once. All existing lexical/FTS, prepared content, native vector and saved
state tables retained identical complete values. Only the expected Core context,
reference revision and appended migration-ledger row changed. A second startup
and deep validation left all 91 table fingerprints unchanged.

The resulting 29,813 product roots have 33,475 Core prose sections and 67,233 Core
semantic locations per profile. Both profiles passed complete native validation.
The largest Core seed has 23 native units, below the existing 128-unit ceiling;
no Core seed exceeds that ceiling. The one-location difference from the earlier
67,234-location census follows the separate main content-format refresh.

Whole-runtime startup, upgrade, complete validation and the first request took
155.1 seconds and peaked at **1,562.4 MiB RSS**. Phase memory attribution was not
measured. This is a different operation from the projection-only pilot above.
Twenty subsequent serial fresh CLI requests across both profiles passed the
provisional query budgets and peaked at **54.7 MiB RSS**. Those requests required
no model weights; OS caches were not flushed. These bounded observations establish
operational availability, not a p95 estimate or a general memory guarantee.

Twelve bounded legacy All cases matched the pre-Core build exactly in result order,
raw scores, witnesses, coverage and paging after expected wire/generation
normalization. No score tolerance or tie expansion was used. This is regression
evidence, not a comparative quality study. Original research bytes remain intact.

Raw receipts are retained under `scratch/core-sections/v22-upgrade/`,
`legacy-all-parity/` and `operational/`; preservation requirements are recorded in
the [research handoff](search-evaluation-handoff.md).

## Receipt identity

[Curated machine-readable census](core-section-census/2026-10-27.json.gz) includes source/policy hashes, all family/visibility counters, profile readiness, trait coverage and bounded resource/preservation metadata. It contains no source bodies or vectors. Raw consistent artifacts, executables, before/after manifests and logs remain in the task evidence directory and must survive worktree cleanup.

- Compressed SHA-256: `035102d08047bab060fe24ebcdda3bca46ecc8ad833df25c74aa041ef97af16d`.
- Decoded SHA-256: `aa46c21bf95ad2bfa6c7bc2f54ccdac5f123fe855bbeebc0372f65f6412e78c6`.

Policy and ownership: [ADR0060](../architecture/decisions/0060-core-sections-and-search-scope.md).
