# Source-backed artifact cutover validation

Status: local validation passed; hosted CI, including Windows publication,
is pending. Owner checks do not replace the independent completion reviews.

## Candidate and inputs

The cutover branch is based on integration/source-modeling at
2cce20f29191731e540022ec7dac7ade9b13546d. The pinned PF2e 6.12.4 source export is
4cbdaa37d6c33e9519561bae2c59a23e0288cbce. It contains 25,641 document roots;
25,560 are product records and 81 Macros are developer-only.

The only embedding model is BAAI/bge-small-en-v1.5 at immutable revision
5c38ec7c405ec4b44b94cc5a9bb96e735b38267a. Actual inference uses the verified cached
assets through FastEmbed. Text-splitter and the model tokenizer govern inputs;
source selection and attribution remain Atlas policy.

## Corpus and query evidence

The direct-DTO query oracle covers 805,663 known authored checks and 30,565 terminal
unavailable-state checks across the complete source, including 79,818 immediate
Actor Items. Fixture truth tables separately cover missing/null/invalid values,
empty collections, unknown negation, same-child witnesses, numeric precision and
counterfactual facets. No generic metric projection is used as an oracle.

NPC default-policy inspection found 5,492 NPC roots: 5,237 omit adjustment, 173
explicitly use null, 56 are elite, 23 weak and three have other values. Every
missing-flag NPC has an integral authored maximum HP. Pinned Foundry code applies
adjustments only to explicit elite/weak flags. Local participant preparation uses
an unadjusted default for Missing inside known attributes, retains the source
Missing state and records default origin separately. Invalid flags remain unknown.

A full lexical artifact retained and checked all 25,641 roots, with 117,157
prepared caches, 112,445 lexical units, 494,825 query projection rows and 165,735
relationship occurrences. Its context contains 568 actually used verified trait
labels. Final foundry-content/v4, structural-source-sections/v2 and relationship
policy v2 build plus a second explicit validation passed in 1,108.15 seconds;
the producer build took 669.153 seconds. All roots were retained, including 203
partial documents with 267 admission diagnostics; no files were quarantined and
no roots became raw-only records. An independent reload of the original files
compared all 25,641 admitted models with their checked artifact reader results,
including all 81 Macros and retained invalid/additional data, with zero differences
(28.51 seconds).

Full semantic-input preparation passed in 1,227.98 seconds: 25,560 root identities
and 105,006 body units make 130,566 inputs. All 45,948,388 selected UTF-8 body bytes
and tails across 92,156 fields and 98,220 sections are covered. Exact tokenizer
counts total 11,392,962, with a maximum input of 286 tokens and no shortened
context. This complete preparation check performed no model inference.

The lexical artifact occupies 291.9 MB including indexes: checked-body allocation
occupies 65.6 MB, prepared content 72.9 MB, relationship occurrences 45.2 MB
and lexical units 22.4 MB. This is a lexical artifact, so the size excludes
semantic metadata and vectors. Compressed snapshot payloads total 56.2 MB and
compressed prepared HTML totals 33.7 MB; table allocations include metadata and
SQLite page overhead.

Debug-profile public reader calls on that complete artifact produced these
median timings over 20 repetitions. Fresh connections include initialization;
the OS page cache was not cleared. They are not release-profile or cold-cache
measurements.

| Operation | Warm connection | Fresh connection |
| --- | ---: | ---: |
| Actor saves and resistance existence | 55.445 ms | 63.870 ms |
| Same embedded spell rank/tradition | 215.601 ms | 225.876 ms |
| Lexical definition | 11.746 ms | 20.739 ms |
| Exact name | 0.027 ms | 8.533 ms |
| Selected detail (two content fields) | 1.212 ms | 10.706 ms |

The filter plans scan root eligibility and use indexes for correlated projection,
IWR and same-child lookups. Selected-content and FTS reads use their respective
indexes. The exact-name helper shares the actual production query, including
its known-name guard; refreshed EXPLAIN confirms the name/alias partial indexes.
Broader
query tuning requires workload evidence rather than speculative restructuring.

## Real-model retrieval sample

The current producer builds an atomic checked sample artifact from 220 real roots.
It contains 2,084 semantic units with 2,039 unique actual-model document inputs.
The v4 bootstrap performed those 2,039 inferences in a 98.12-second proof. The
final v4/v2 rebuild and explicit validation passed in 99.93 seconds, verifying
and reusing all 2,039 existing inputs rather than inferring them again. Query
inference uses the current pinned model. Reuse regression tests independently
reject wrong input-hash/vector associations, token counts and missing reference
sidecars before publication.

All 14 source-grounded expected records are in the top three for both vector and
hybrid retrieval: 12 rank first, Invisibility ranks second and Antidote third.
An earlier synthetic old-name/remaster example is excluded from that count; it
does not establish a verified source relation.

Dragon Form ranks first with its explanatory description opening as the strongest
witness (similarity 0.72264), ahead of its identity unit (0.71157) and tail
(0.71003). This addresses the earlier prototype's less useful tail witness.
The sample is a relevance check, not a claim about every possible corpus query.

## Consumer and platform gates

The frontend verification gate passes 121 tests plus formatting, linting,
typechecking and the real Vite production build. CLI tests pass 29 cases, shared
CLI support passes 13 and standalone developer CLI passes two. Source tooling
passes 100 tests and eight pinned-output freshness checks. The final Rust
verification gate passed formatting, broad and strict Clippy, all workspace tests
and build. Two independent reviews confirmed the complete current corpus/model
evidence and identified frontend event/numeric-boundary defects. The remedies
passed the refreshed full frontend gate and independent focused regressions:
nested controls activate once, native links work, encounter HTTP integers are
checked before bigint arithmetic, and HP formulas use exact intermediate values.

There is no connected browser backend in the session. Actual route validation uses
repository-local Playwright with cached Chromium rather than treating jsdom or an
HTTP readiness response as browser evidence. Five browser workflows passed on
the v4 sample, including saved lists, encounter HP persistence, owned/GM content,
semantic search, RollTables, variants, Similar and navigation errors. The final
expanded run passed all five workflows in 10.6 seconds against a fresh local-state
database and rebuilt CLI. It exercises real HTTP Damage and Heal, preserving HP
through an adjustment, condition update and reload. Nested marker click and
keyboard ownership also have focused regression coverage. Representative actual
screenshots were inspected; this is automated/agent evidence, not human approval.

Windows in-use atomic publication has a platform-specific foundation test and a
dedicated CI job. Local macOS checks do not establish its Windows result.

The two independent reviews have checked the current implementation and local
evidence. Hosted CI, including actual Windows publication behavior, remains the
final acceptance gate and must pass before the full draft PR becomes ready.
