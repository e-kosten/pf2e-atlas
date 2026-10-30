# ADR0062: Bounded exact TextSearch neighbor reuse

Status: accepted.
Date: 2026-10-30

## Decision

Ordinary unscoped TextSearch may reuse each exact native KNN window across
requests. Search owns one runtime-shared transient store; index produces its
opaque unit-ID/distance-bit batches only after the unchanged native query and
complete location decoding succeed. Index rehydrates every retained neighbor
in the consuming snapshot through profile, eligibility and Core joins. Missing
or unexpected members fail rather than silently reduce coverage.

The key binds the admitted workspace key/reference revision, complete canonical
context digest, complete selected profile contract, exact query text and prepared
f32 vector bits, validated typed predicate including clause/collection identities,
explicit All scope/no exclusion, content scope and exact k. One process-local
store and the explicit candidate-query recipe bind executable/native behavior.
Hashes accelerate lookup; complete encoded bytes determine equality.

Each k is independent: a larger native result cannot supply a smaller window by
slicing, because cutoff ties belong to native selection. Page size/number, result
ordering and fusion choices are downstream. Preferred-root window stopping and
final remaster suppression still run for each request. Query inference, token/model
checks, lexical membership, ranking, hydration, snippets and relationship evidence
retain their existing owners and validation.

Only the ordinary TextSearch app branch explicitly opts in. The general text
method, key/saved scopes, trait-reference search, discovery and Similar stay
uncached. Reuse adds no public handle, session or wire field. Every request
continues to supply complete criteria and expected-generation intent.

## Bounds and concurrency

Internal bounds are 16 MiB accounted live reuse allocations, 64 ready windows,
256 KiB per entry and 64 KiB encoded key. Keys retain their allocated capacity;
charges conservatively reserve the maximum 4096-neighbor payload before miss
production, plus typed descriptors and allocation/Arc overhead. Fixed lookup/LRU
storage and store bookkeeping are charged. Witnesses, snippets, document vectors,
record bodies, database sessions and transactions are never retained.

The least recently used ready entry is evicted at the entry limit. Borrowers keep
their charges until released, including after eviction/retirement. Oversized keys,
entries and unavailable budget bypass retention and run the ordinary exact query.
Successful empty windows can be retained. Runtime shutdown releases ownership.
Accounted bytes are a policy bound, not a process-RSS or allocator-footprint claim.

Concurrent misses compute independently and publish at most one equivalent entry.
The store mutex protects only lookup, accounting ownership and ready-entry order;
no SQL, hydration, model work or waiting runs under it. There is no single-flight
queue or subscriber lifetime. Existing request abandonment and blocking permits
are unchanged; durable mutations remain uncancellable.

## Publication and validation

Public generation is workspace_key:reference_revision. Complete context SHA-256
is an additional preparation/reuse identity, not the wire generation.
Source/vector/profile publication and component upgrades increment reference
revision atomically. Logical native repacking runs inside that publication.
Schema migrations require the exact executable schema and exclusive admission,
so a cache cannot bridge an incompatible schema. Failed publication rolls back
data/revision together and maintenance recovery validates committed data before
reopening admission.

SQLite VACUUM relocates physical pages without logical repacking. The bundled
SQLite preserves the native vector-chunk hidden rowids because those tables have
primary-key indexes; sparse-chunk exact-neighbor regression coverage protects this
dependency behavior. No new public generation or schema is introduced.

A store is bound to one durable workspace. Observing a newer numeric reference
revision retires ready entries; older operations cannot roll its epoch backward
or publish into a retired epoch. A different workspace requires a new runtime.
Both hits and misses retain normal reader/catalog/profile admission and native
readiness, vector guards, filter validation and metadata decoding. Remaster
evidence/key failures still run on nonempty requests, including unrelated pairs.
Failures/unavailable outcomes are not retained. Failed hit hydration evicts the
entry and reports its error; later requests can recover normally.

The guarantee uses cooperating publication/admission. Arbitrary external SQLite
edits without reference advancement and live-file replacement are outside that
protocol. Reuse is not a fresh full-vector integrity scan on every read.

## Consequences and evidence

This targets repeated KNN work while retaining modest bounded numeric data.
Remaining inference, lexical/remaster work and page presentation still cost time.
It introduces no approximate membership, smaller window, new ranking or quality
claim. Deterministic fixtures must prove per-k cutoff/tie and cold/hit equality,
publication/retirement, validation, duplicate misses, accounting and excluded
consumers. Matched release measurements use real reopened operations, both
profiles, hit/miss/eviction/concurrent workloads and CPU/resource plateaus.
Storage mapping remains independently owned; its benefit cannot be inferred from
retained-snapshot component timings or subtracted as a promised reuse speedup.
