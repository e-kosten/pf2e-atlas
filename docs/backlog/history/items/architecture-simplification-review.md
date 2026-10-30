# Architecture simplification review

Status: done
Reviewed: 2026-10-30

## Scope and conclusion

The review asked whether Atlas's architecture, abstractions and implementation
still fit the product after PR25 and subsequent features, including whether its
crate divisions justify their cost. It combined a first set of scoped repairs
with a later whole-system architectural challenge at
`2d2108be99394ac6e18a0a45988dfcfa607262b1`. The final local repair landed in
[PR161](https://github.com/e-kosten/pf2e-atlas/pull/161), merged as `fe62334e`.

Retain the overall decomposition. Today's product needs a local native application
with offline checked-source reads, shared interpretation, attributable retrieval,
one state-preserving transactional workspace, application-owned rules/projections
and thin CLI/browser transports. The existing responsibilities fit those needs.
All 19 crates already existed at PR25; their age does not establish that each
package is necessary. Responsibility separation is well supported; confidence in
the precise package partition and migration economics is moderate.

This closes a source-based architectural assessment and its selected repairs.
It does not certify every source family, rule, ranking outcome, runtime failure,
visual interaction or platform. Unrelated product backlog remains open.

## Coverage of the original questions

The whole-system challenge examined manifests, exports, actual consumers,
representative implementation/tests and change history for every workspace crate
and the major frontend layers. Depth varied by area; this was not a line-by-line
audit. It derived a requirements/design model before reconciling earlier findings,
and compared concrete simpler designs. Three focused read-only reviewers challenged
source/reference, workspace/runtime and browser/application conclusions.

Coverage included these connected responsibilities:

- Generated source admission/checked codec, shared domain vocabulary and borrowed
  source interpretation: `atlas-foundry-model`, `atlas-domain`, `atlas-record`.
- Source preparation, model inputs/inference/reuse, publication and retrieval:
  `atlas-ingest`, `atlas-embedding`, `atlas-index`, `atlas-search`.
- Session/transaction authority, saved-domain schemas and runtime composition:
  `atlas-storage`, `atlas-local-state`, `atlas-runtime`, `atlas-sqlite-vec`.
- Product semantics/contracts and entry points: `atlas-app-service`,
  `atlas-app-model`, `atlas-web`, `atlas-cli`, `atlas-dev`, `atlas-cli-support`,
  `atlas-dice`, `atlas-tags`.
- Browser transport/conversion, Router identity, Query/reference generations,
  saved mutations/drafts, shared readers and feature composition; generation,
  compatibility, fixtures and delivery where they affect architectural costs.

Six paths tested the reasoning: a physical source field through filters/detail/CLI
([PR135](https://github.com/e-kosten/pf2e-atlas/pull/135)); Core projection changes
and a hypothetical model-policy invalidation; initiative with lost-response receipt
recovery; saved-list note drafts during navigation/reference replacement; table
rolls and linked result reading; and source-free party copies into encounters.
These distinguish necessary multi-owner feature work from avoidable coordination.

## Delivered simplifications

| Area | Delivered outcome |
| --- | --- |
| Encounter reads, [PR150](https://github.com/e-kosten/pf2e-atlas/pull/150) | Aggregate participant counts, narrower fact/presentation preparation and request-local root/roster reuse. Read/decode assertions establish removed work, not latency. |
| Lightweight dependencies, [PR151](https://github.com/e-kosten/pf2e-atlas/pull/151) | Remove app-model's record dependency and CLI-support's runtime dependency; consumers/composition own conversions. No build-time claim. |
| Request intent, [PR153](https://github.com/e-kosten/pf2e-atlas/pull/153) | Require explicit read/action intent for body-bearing requests; read failure no longer implies an uncertain write. Real action uncertainty remains protected. |
| Browser references, [PR155](https://github.com/e-kosten/pf2e-atlas/pull/155) | Query-definition retirement metadata, shared per-surface generation handling and canonical raw-document cache identity. |
| Web boundary, [PR157](https://github.com/e-kosten/pf2e-atlas/pull/157) | Concrete app-service and small real router fixtures replace the mirrored service trait/mock; blocking/read-abandonment and durable receipt behavior remain. |
| Execution lifecycle, [PR159](https://github.com/e-kosten/pf2e-atlas/pull/159) | Equivalent command modes consolidated; artificial per-worker runtime factories/startup reports removed. Shared runtime, job lifetimes and real spawn-failure cleanup remain. |
| Wire conversion, [PR160](https://github.com/e-kosten/pf2e-atlas/pull/160) | Repair selected complete integer-bearing response graphs with typed handwritten converters, compiler controls and real Rust/router JSON fixtures consumed by client adapters. No protocol redesign or production decoder generator. |
| Table-roll results, [PR161](https://github.com/e-kosten/pf2e-atlas/pull/161) | Retain the checked root and retrieval session; project matches through ordinary detail policy. The two-match regression verifies one root decode instead of three, with full-response parity and navigation/guard coverage. No latency claim. |

## Retained decisions and alternatives

Keep index and search separate. Ingest/publication has actual index-only consumers
and tests; retrieval has substantive independently changing lookup/ranking/admission
policy. A combined reference crate is credible: store/retrieval modules could make
some current public forwarding and session APIs private. A broader combined
reference/workspace library could also reduce fixture and composition plumbing.
Neither eliminates the source, transaction, versioning, inference or product
policies, and no better total migration payoff was demonstrated.

The current split gives directed dependencies and selectable compilation/test
units. It is not a proven build-speed advantage or absolute SQL capability
boundary: publicly exposed sessions can yield a connection through an inferred
receiver. No production bypass was established. Both index and search inherit
native embedding dependencies; retaining their split does not establish native
dependency isolation.

Keep runtime separate from app-service because developer/maintenance callers need
real setup/readiness/session composition without application rules/presentation.
Merging domain with record or storage with saved state would broaden actual
lightweight consumers' dependencies or authority. Combining packages could recreate
those separations through features, but would add feature/test-matrix obligations.

Keep checked snapshots and shared source interpretation rather than raw JSON plus
parallel targeted parsers; replacing them must still support offline reads,
invalid/ordered source values, exact navigation and shared semantics. Keep one
workspace: separate reference/state files would require a new consistency/recovery
protocol for mixed reads. Keep Router, remote Query cache and local draft owners
distinct: their identities and replacement lifetimes serve different user intent.

From scratch, a smaller tag validator could be a CLI module, full-workspace fixture
composition could sit above schema owners, pure embedding contracts could have a
clearer native boundary, and browser numeric semantics could be explicit at the
wire. Those preferences do not establish worthwhile migrations now. Small
tag/writer cleanups have no standalone implementation assignment.

## Residual questions and evidence limits

- [Downstream follow-through](../../items/rust-downstream-architecture-follow-through.md#conditional-architecture-questions)
  retains native-isolation and fixture-topology questions with specific evidence
  needed before proposing implementation. Neither is a confirmed product defect.
- [Performance](../../items/reference-build-performance.md) remains deferred until
  a recurring supported release workflow has an attributable avoidable cost.
  Delivered memory, FTS-phase and passage-preparation results have different scopes;
  the old segmentation/maintenance observations do not mandate another optimization.
- [Exact browser numeric transport](../../items/browser-numeric-precision.md)
  remains deferred. Safe bigint conversion cannot recover precision lost during
  JSON parsing. A production converter generator is not required by this review.
- [Encounter polish](../../items/rust-web-encounter-runner-polish.md#reported-narrow-actions-explanation-clipping)
  owns the reported narrow Actions explanation clipping, separately from numeric
  conversion and architecture. It was not reproduced during closeout.

Review conclusions came from static source/history inspection and independent
challenge, not executed benchmarks or browser certification. Implementation PRs
carry their own local checks, independent review and hosted CI provenance. For
example, PR161's [successful final CI](https://github.com/e-kosten/pf2e-atlas/actions/runs/41097116205)
tested `5f9284756b981ebe9128490c06bb1e428f957e06`: selected Rust, frontend/bundle,
Windows publication, standalone developer and both browser shards passed;
unselected jobs were skipped. Its PR reports local Rust and desktop Chromium/phone
WebKit journey validation. Those checks were not rerun for documentation closeout.

Current ownership remains in the [architecture overview](../../../architecture/overview.md),
[runtime](../../../architecture/runtime.md) and their ADRs. This history records
the assessment and dispositions; it assigns no further implementation program.
