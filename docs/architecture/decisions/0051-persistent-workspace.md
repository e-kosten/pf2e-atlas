# ADR0051: Persistent workspace and bounded database operations

Status: accepted

## Context

Replacing a generated reference file separately from mutable state makes upgrades,
reference identity and composed reads harder to keep coherent. Atlas needs a stable
upgrade baseline before release, incremental source processing, reusable expensive
inference and room for additional saved domains. Existing pre-release files need
no format bridge. Maintenance downtime is an accepted product tradeoff.

## Decision

Use one persistent SQLite workspace in application-data storage. Source checkouts
and model assets remain external caches. Normal setup and source updates must not
replace the workspace file or recreate user tables.

`atlas-storage` owns connections, process admission, bounded read/write sessions,
migration execution and backups. It does not import domain crates. `atlas-runtime`
composes domain-owned SQL into one ordered, checksum-verified migration ledger.
`atlas-index` owns reference schema, queries and publication validation;
`atlas-local-state` owns saved-domain schema and operations. Both use rusqlite on
the same caller-owned connection for a composed operation. App-service receives
typed capabilities rather than a raw SQL connection.

Each read operation has a bounded transaction. Idle workers retain inference and
configuration, not database snapshots. Source-derived mutations verify reference
and entity revisions inside their write transaction. Durable receipt recovery
precedes stale-preparation checks, so a lost response can be recovered after an
update without rolling again. Receipts have explicit persisted payload versions,
independent of presentation DTOs.

The caller owns final commit. Each multi-statement saved-domain action also has a
savepoint: an error or unwind cannot leave that action's partial changes available
for an enclosing caller to commit. Prior successful actions remain under the
caller's outer transaction. Failure to restore a savepoint abandons that outer
transaction.

Migrations drain cooperating readers/writers, back up an existing workspace and
apply schema, backfills, validation and ledger changes in one transaction. Domain
backfills may provide a versioned executable callback; its immutable identifier
participates in the migration checksum. Published SQL-only checksums are unchanged.
A referenced-table reconstruction explicitly opts in through its immutable,
checksum-bound backfill contract. Under exclusive admission, storage captures the
upgrade backup and suspends foreign-key enforcement on the migration connection
before BEGIN for the **whole pending migration batch**, including ordinary SQL
and callbacks before or after the opted-in backfill. Every migration in that
batch must be safe without foreign-key actions; the final key check detects
dangling references but cannot reproduce a skipped cascade. The published
migration 1–22 suffixes used with the saved-state reconstruction use explicit
changes and have no dependency on such cascades. Domain-owned preflight and
reconstruction, exact row-count checks, ledger updates and complete foreign-key
validation share the upgrade transaction.
Storage restores enforcement after commit or rollback; an interrupted connection
closes without changing subsequent connections' enforcement. This mechanism is
reserved for reconstruction whose parent DROP would otherwise cascade into
retained user rows. Domain policy and reconstruction SQL remain with their owner.
The ordinary admission validator remains read-only and never runs backfills. One
publisher owns source/configuration updates. Preparation can run while ordinary
operations continue: publisher ownership does not hold the exclusive access lock
or a long-lived SQLite write transaction. Final activation takes exclusive access.
Durable maintenance state prevents
ordinary operations from opening incomplete work after interruption. Expensive
inference occurs outside write transactions, with bounded persisted checkpoints.
Final publication validates the exact transaction before advancing reference
revision. User-state edits do not advance that revision.

Startup authenticates an already-current migration ledger under shared admission,
without taking publisher ownership or scanning all foreign-key rows. A new CLI
process can therefore read during preparation. Pending migrations still acquire
exclusive ownership and validate foreign keys before commit.

Keep stable record identities when sources disappear. Remove obsolete current
content/projections and mark identities unavailable; saved references and snapshots
remain. Returning source keys reuse the identity. Requested configuration is
independent of active output. An optional model can prepare while the active
profile remains available, then activate during a short maintenance boundary.
Reuse vectors only with exact verified model/input identity. Schema or display
changes alone do not justify inference.

Source preparation binds every consumed file to a captured SHA-256 inventory.
Pack bindings and document/migration discovery use that inventory; each read
validates its bytes before parsing. Publication rechecks the complete inventory,
so transient edits or temporarily omitted files cannot produce mismatched
provenance.

Public filter names express domain concepts, with explicit `embedded.*`
collections. Embedded predicates match immediate children only; a root-kind filter
never changes their meaning. Standalone-or-embedded matching requires explicit OR.
Preserve same-entry semantics and unavailable values. Retired pre-release paths
fail explicitly rather than silently translating or dropping clauses.

## Consequences

One file enables ordinary joins and coherent cross-domain transactions, but SQLite
still serializes writes. Long computation must not hold a write transaction.
Maintenance can take longer than a few minutes for expensive source changes;
progress and interruption recovery are required, not an assumed timing guarantee.
Application-managed admission must cover CLI and web callers. External SQLite
tools are outside this coordination protocol.

This supersedes ADR0030's separate-file decision and the immutable-file lifetime
parts of ADR0046/0049. Snapshot authority, typed projections, revision checks,
bounded retrieval, receipt idempotency and profile-scoped ranking remain.

Changes to this boundary require production lifecycle integration,
delta/backfill/checkpoint tests, consumer replacement and matching architecture
guidance. Isolated storage tests do not establish those obligations.

## Saved-domain integrity

Encounter status and participant kind, variant and side are closed SQL enums.
Readers reject unknown saved values, including provenance enums, rather than
interpreting them as defaults. A participant's encounter/key pair is unique;
pending consequences, boundary claims and consumed application markers reference
that pair. Consumed markers may retain a null or historical condition ID after
its live source is removed.

The encounter's current participant references the same pair with deferred
checking. Participant removal clears that pointer before deleting the row;
encounter deletion still cascades through its saved children. Reconstruction
preflight rejects invalid enums and missing or cross-encounter references with
the affected row's identity. It preserves all data on failure and does not infer
a repair. The retired adjustment table is removed only when empty; historical
rows require an explicit preservation decision.

## Development snapshots

Whole-workspace developer seeds are storage-owned snapshots, not a second live
reference store. Capture holds shared update/access admission and uses SQLite
backup to include committed WAL contents. Installation verifies a staged clone or
copy before publishing only to an absent path under exclusive admission. It never
migrates, replaces existing saved state, or shares writable pages with the seed.
Migration backups and developer snapshots share one storage-owned capture and
durable publication implementation. Callers retain their admission policy and
explicitly choose replacement (latest migration backup) or missing-only publication
(developer snapshot). Both outputs are closed standalone SQLite files.
Developer manifests and tool retention belong to atlas-dev orchestration.

## Implementation evidence

The migration/session APIs, shared schema, state-preserving reference writer,
versioned receipts, durable embedding checkpoints, component classification and
incremental source/preparation reuse are implemented. Requested configuration is
checked under update ownership before preparation; setup permits source
reprocessing when a component needs more than a projection-only backfill.

Storage tests cover cross-process admission, abrupt process exit, interrupted
maintenance and migration while recovery is pending. Runtime regression coverage
exercises automatic query backfill while retaining saved lists, encounter state,
vector bytes and reusable checkpoints. CLI tests protect workspace retention
during cleanup and fail-closed handling of inconsistent schemas. Rust, frontend
and real-browser gates passed for the implementation; native-model inference and
full-corpus performance were not rerun. These checks establish local implementation
evidence, not release or hosted-CI acceptance.

Physical free-page reclamation runs only during explicit maintenance, outside a
SQL transaction under exclusive admission and a durable recovery marker. It keeps
workspace identity and logical reference revision; interrupted reclamation uses
the existing validated maintenance recovery. Before starting, storage checks free
space on the workspace and SQLite temporary volumes, conservatively budgeting
twice the logical database size on each. Insufficient or uncheckable capacity
skips optional reclamation without setting maintenance or failing the completed
reference update. The preflight cannot reserve capacity against other processes.
Startup never vacuums implicitly.
