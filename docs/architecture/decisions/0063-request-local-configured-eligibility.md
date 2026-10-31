# ADR0063: Request-local configured unit eligibility

Status: accepted for a bounded prototype; performance acceptance is separate.
Date: 2026-10-30

## Decision and ownership

Index may reuse the exact configured semantic-unit eligibility relation only
within one configured candidate operation borrowing one admitted reader/snapshot.
Search retains candidate iteration, checks before each selection, union, work
budgets, scoring and evidence. No reader, result handle or relation survives the
operation. Ordinary TextSearch/Baseline and ADR0062's cross-request store remain
unchanged, including its 64-entry/16 MiB bounds.

One source-owned compiler supplies the original native query, materialization and
complete setup identity. It includes profile, typed predicate, All versus exact
key scope, exclusion, Core/content and comparison-role/identity masks. The immutable
borrowed reader binds artifact/model/context identity. Vector bits and k remain
per-selection concerns. Complete encoded keys, not hashes alone, admit reuse.

Retain flat JSON containing decimal signed-i64 unit IDs. Stream into a bounded
buffer without an intermediate ID vector or floating-point conversion. Preserve
emitted duplicate tokens; SQLite's native IN relation retains set semantics.
The consumer replaces only the IN-set source with builtin `json_each(?)`;
native MATCH, distance arithmetic, per-k cutoff ties, outer metadata joins/decoding,
ordering, root unions and downstream scoring remain unchanged. Empty relations
still run normal selection validation and consumption.

## Finite representation bounds

Internal limits are 4 MiB aggregate accounted retention/materialization state,
64 KiB per complete key, 2 MiB and 262,144 emitted rows per JSON relation, and
16 ready identities plus 16 bypass fingerprints. Accounting includes all retained
key/payload capacities, fixed descriptors, partial buffers and simultaneous old
and new growth allocations, with conservative allocation overhead. A reserved
64 KiB + 512 bytes covers one admitted setup SQL scratch command and fixed plan
bookkeeping. SQL prefix/unit slices borrow the original native statement; setup
SQL is allocated only after encoded-key admission. Its raw bytes cannot exceed
the encoded-key limit.

The preimplementation census on the private research corpus found at most 161,735
emitted IDs and 1,132,146 JSON bytes across both profiles, Core/All and role/identity
masks without filters/exclusion. The limits cover that observed case with margin;
they are not expanded to fit arbitrary corpora.

Capacity refusal uses the original exact query without truncation or a new search
failure. Remember negative admission boundedly; when metadata slots fill, disable
new materializations while allowing existing hits. Never retry an oversized setup
for every vector. Actual SQL/native/integrity errors propagate and are not fallback
signals. Partial/error/cancelled relations never publish.

The 4 MiB counter does not claim total RSS, allocator footprint or all query
working memory. Original SQL/parameter compilation and SQLite statements remain
separately attributable query state. For flat integer JSON of length L <= 2 MiB,
pinned SQLite's JSONB payload is at most L+N+4; conservative parser old-plus-new
allocation requests are at most 3(L+109), and two possible text buffers use at most
2L+3. Together with owned state, 16 MiB + 16 KiB is a conservative incremental
requested representation-buffer envelope, excluding allocator rounding and other
SQL/native working state. Do not turn this estimate into a total heap guarantee.

No explicit request-owned TEMP objects or request-long TEMP files are created.
Existing eligibility SQL and native IN may use implicit statement-local B-trees
and spill; equal membership does not prove equal physical occupancy. Measure
active SQLite/TEMP highwater, native ID arrays, RSS and cooldown separately.
No global heap/TEMP PRAGMA, session architecture or native callback is added.
A hard global SQLite heap/disk policy requires a separate decision.

## Validation, cancellation and cleanup

Every selection keeps the existing ordered guards: limit, unsupported-role empty
return, profile/vector, native readiness/table and predicate rules. Prepare the
original native statement before separate setup execution, including on hits, so
SQL/schema/native-plan errors retain precedence. Validate later vectors and k even
for ready or empty sets. Prove competing runtime failures with fixtures.

Search keeps every existing cancellation boundary. A generic fallible caller check
adds setup opportunities before materialization, every 256 emitted rows and after
completion/before publication, without an index-to-search dependency. These checks
do not interrupt SQLite before its next row or an executing native selection.
No progress callback or cancellation of durable writes is introduced.

RAII releases partial statements/buffers on capacity, error, cancellation and
unwinding. Drop the operation immediately after candidate generation. Independent
owners/readers share no relation or budget; publication/reopen creates new state.

Acceptance requires exact unit sets and native ordered IDs/distance bits including
ties, complete score/witness/result/error/work-limit parity, aggregate bounds and
bypass-once behavior, cancellation/cleanup/isolation tests, and matched optimized
CPU/wall/resource evidence on the actual base. One-selection and ordinary controls
must expose setup regressions. The earlier pre-filter SQL/VM share is not a
promised saving; stop if measured construction/resource costs erase the benefit.
