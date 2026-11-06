# ADR0067: Compact legacy/remaster context over current records

Status: Accepted, 2026-11-06. Revised during PR219 after product scope review.

## Context

Users need to recognize older names, navigate actual available counterparts and
read useful remaster explanations. Most predecessors are names in a journal, not
separate imported records. The corpus has a small reviewed set of actual pairs.
A general declaration, citation, endpoint-binding and term-inspection framework
adds substantial machinery without a corresponding product need.

Saved records follow current content at their selected Foundry key. Historical
snapshots, pinning and automatic conversion to another record are outside scope.

## Decision

Retain existing verified aliases and exact counterpart pairs. Add one rebuildable
per-record context projection containing selected owners, relationship type,
relevant names with optional exact record addresses, scope for consolidation and
an optional useful journal field. No separate declaration, citation, binding,
support, term-catalog, captured-file or relationship-digest tables remain.

`atlas-record` interprets direct relationship expressions in audience-selected
prepared journals. Prose mentioning a reference is not a replacement operand;
collective destinations do not establish per-child aliases. The small curated
mapping supplies exact old/new roots and expected source families. It does not
match records by names or rules text. Checked journal context can supplement a
mapping; absent predecessors stay names rather than synthetic records.

Ingest prepares aliases, pairs and context once. Index reconciles these projections
in the normal atomic reference publication. A compiled mapping/recipe identity in
the existing build context requires normal preparation when this metadata changes.
It is separate from content and native-vector reuse dependencies.

App-service projects concise relationship lines, current available target titles
and useful selected journal destinations for the browser and CLI. Consolidation
retains scope and collective meaning. Developer provenance remains internal;
readers do not display upstream commits, script paths or Atlas-provider badges.

## Consumer behavior

- Search, browse, Similar and discovery retain every independently matching record.
  There is no version suppression policy or setting.
- Former-name aliases continue to find current records. Search adds no separate
  former-name annotations or source-statement panel.
- Record reads show compact context above rules. Journal Info actions open the
  supplied page in the reference drawer. Missing counterpart names stay plain.
- Explore omits the reader context in every tab. Versions appears for actual
  alternative records or a recoverable lookup failure. Its relationship appears
  once, with Compare below; comparison keeps both current bodies and identities.
- Owned/customized documents retain their own body. Explicit original-record
  context remains separately labelled and never changes their identity.
- Unavailable saved entries use ordinary broken-link handling. No remaster-specific
  missing-record modal or replacement-on-save behavior exists.
- Existing record-get and record-only graph-remaster CLI reads expose the same
  compact facts. There is no term-inspection command or dedicated relationship API.

## Publication and upgrade

Schema32 retires the experimental schema30/31 tables and restores the alias/pair
projections alongside `record_remaster_context`. Earlier migration checksums stay
immutable for already-created candidate workspaces. Normal source-backed setup
repopulates metadata, preserving saved lists, encounters and compatible vectors.
There is no old-framework serving shim. No-op setup remains stable; unchanged
projection rows avoid writes through normal reconciliation.

See the [artifact contract](../artifact-contract.md), [runtime](../runtime.md) and
[ADR0065](0065-input-owned-reference-publication.md) for publication boundaries.
The [coverage audit](../../research/remaster-counterpart-coverage.md) records the
reviewed mapping scope and exclusions.
