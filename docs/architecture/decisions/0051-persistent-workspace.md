# ADR0051: Persistent workspace and bounded database operations

Status: accepted; numerical ownership, recovery, captured-input publication and
publication assurance amended by [ADR0065](0065-input-owned-reference-publication.md).

## Context

Replacing a generated reference file separately from mutable state makes upgrades,
reference identity and composed reads harder to keep coherent. Atlas needs a stable
upgrade baseline before release, incremental source processing, reusable expensive
inference and room for additional saved domains. The release upgrade floor is
workspace schema 41. Maintenance downtime is an accepted product tradeoff.

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

### Release schema and migration admission

Fresh workspaces apply the frozen schema 42 baseline and the schema 43
migration. Schema 42 is the minimum direct upgrade. The schema 42 checksum
still binds the original domain SQL and the retired cutover inputs, including
the schema 41 ledger fingerprint and bridge identity. Earlier builds can
upgrade an older workspace to schema 42.

Under exclusive update and access admission, storage authenticates the applied
ledger and lends runtime the read-only pre-upgrade snapshot. Runtime uses the
versioned workspace archive to capture saved work. It validates and stages the
archive before atomically replacing the previous sibling archive. Only after
capture succeeds does storage apply the SQL, backfills, validation and ledger
updates in one transaction. Failed migration leaves the completed archive
available for explicit restore. The upgrade path operates on the database in
place. Developer snapshots retain their separate full SQLite capture.

Schema 43 owns a typed preferred search model in a singleton workspace settings
table. Runtime reads it on the same admitted snapshot as the published profiles.
A default change updates settings alone. A model membership change publishes the
exact requested configuration, available vectors and preference in one reference
transaction. Ordinary publication clears a removed preference at commit.

A referenced-table reconstruction can opt in through its checksum-bound
backfill contract. Storage suspends foreign keys for the whole pending migration
batch and runs a final foreign-key check before commit. All migrations in that
batch must account for skipped foreign-key actions. Enforcement is restored
after the transaction. Schema 43 keeps enforcement enabled throughout.

### Publication and recovery

One publisher owns source/configuration updates. Preparation can run while ordinary
operations continue: publisher ownership does not hold the exclusive access lock
or a long-lived SQLite write transaction. Normal source publication uses one
immediate transaction. Data-only publication admits readers on committed snapshots.
Main-schema changes require exclusive access. SQLite rejects structural SQL during
shared admission, and an attempted structural update rolls back before retrying
under exclusive admission. Recovery and repair use exclusive admission. Publication advances the
reference revision with its writes. It adds no durable maintenance marker; failure
restores the prior committed state and permits readers to reopen it. Pre-existing
markers still block admission. Schema/metadata upgrades and physical reclamation
retain durable maintenance and validated recovery. Expensive inference occurs
outside write transactions, with bounded recovery results. User-state edits do
not advance the reference revision. [ADR0065](0065-input-owned-reference-publication.md)
records why ordinary publication trusts unchanged data and keeps deep audit explicit.

Explicit reference repair can authenticate the workspace and read its requested
configuration while maintenance blocks product sessions. It binds preparation to
that workspace and revision without trusting old generated data. The final
exclusive transaction checks the ledger again, validates the replacement and saved
references, and commits the new revision, marker removal, and corrected source
configuration together.

Index owns one concrete prepared publication binding its context/inventory,
expected generation, prepared metadata and selected numerical data. Published
native values are input-owned; occurrence edits cannot overwrite another input's
bytes. Only new/forced selected values require owned numerical staging. The
publication transaction resolves reused handles through its own connection and
retains values until all document/trait consumers finish.

Unpublished inference results form one recovery cache with simple bounded eviction.
Selection is independent of eviction once owned. Successful complete publication
clears the cache atomically; staging/publication failure retains useful committed
recovery results and the previous publication. Ordinary reads and startup do no
cleanup or inference. [ADR0065](0065-input-owned-reference-publication.md) owns the
numerical lifetime and occurrence-budget rationale.

Startup authenticates an already-current migration ledger under shared admission,
without taking publisher ownership or scanning all foreign-key rows. A new CLI
process can therefore read during preparation. Pending migrations still acquire
exclusive ownership and validate foreign keys before commit.

Keep stable record identities when sources disappear. Remove obsolete current
content/projections and mark identities unavailable; saved references and snapshots
remain. Returning source keys reuse the identity. Requested configuration is
independent of active output. An optional model can prepare while the active
profile remains available, then activate under exclusive maintenance. Maintenance downtime is accepted; a
short final-only interval and continuous availability are not requirements.
Reuse vectors only with exact verified model/input identity. Schema or display
changes alone do not justify inference.

Source preparation binds every consumed file to a captured SHA-256 inventory.
Pack bindings and document/migration discovery use that inventory. A consumed
external file is checked against capture, then its data is owned and the original
is no longer consulted. Manifests/catalogs follow the same rule. Publication uses
that captured set; subsequent edits, additions and removals belong to the next
build. Missing/unreadable declared packs fail discovery before publication.

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

## Disposable build staging

Product preparation stages checked record metadata and occurrence attribution in
one owned attempt directory. The concrete publication binds complete source/context
and expected workspace revision. Private child streams retain observed allocation
bounds, counts and fallible typed decoding without per-frame checksums or repeated
whole-file authentication. Update ownership permits safely cleaning recognizable
abandoned attempts for that workspace; live child handles close before cleanup.
This keeps rich source/HTML state bounded without introducing another persistent
store. The writer owns SQL and internal identity allocation, and all input sources
use the same fallible streaming contract. Record append seals compact identity
facts; one per-root bundle supplies component digests and derived write inputs.
Missing digests mark components changed. Ordinary ingest trusts unchanged components;
explicit deep audit reconstructs expected semantics independently.

One repeatable stream pairs native text and occurrence metadata. Bounded windows
prepare once, select reusable values and infer missing inputs. Every occurrence
and ordered trait repetition survives numerical deduplication. Only misses load
an inference session, whose lifetime ends with its profile. New/forced values are
staged, while published values remain stable input/checksum handles. Compact
capture/resolver identities, live sets and component digests grow with corpus size;
model memory and observed process RSS remain separate measurements.

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

### Saved-state ownership

Product access to saved lists, parties and encounters uses an owner bound to the
consuming transaction. The local CLI and web runtime resolve the seeded `local`
principal by key when they request saved-state capabilities. Missing local
ownership fails that access. The caller's document, slug, stable key or child ID
does not establish ownership.

Bind the owner explicitly on root inserts and root lookups and mutations.
Access children through an owned parent in the same transaction, including
list-scoped retrieval and filtering, encounter turn snapshots and receipt
recovery. Verify the parent before inspecting a stored receipt, then preserve
receipt recovery before the existing revision and reference checks. Import uses
the destination scope.

Public reference-only reads do not acquire a saved owner. Workspace-wide
maintenance, migrations and backups operate on the complete workspace. When
adding saved access paths, cover the scope using a foreign owner's key or child
ID, or a slug shared by two owners as appropriate to the operation.

## Workspace recovery archives

The runtime captures all saved owners, lists, parties, encounters, referenced
identities and requested reference configuration on one authenticated read
snapshot. Local state owns the saved data, including receipt identity and the
condition ID allocation floor. Index owns requested reference configuration.
The CLI stages and syncs a new archive file before publishing it. Restore
validates the archive and applies it to an empty saved state in one transaction
on a current-schema workspace. Generated reference data and the SQLite
migration backup have their own storage lifecycles.

The archive version starts at `1.0` and advances independently of the database
schema. Bump the minor version for compatible additions, including new encounter
statuses. Older readers can ignore optional data when doing so preserves saved
behavior. Use a detectable behavioral value for new behavior that an older
reader must reject before writing. Bump the major version for incompatible
changes that cannot be represented safely under that contract. Translate
historical values explicitly when their representation changes. New readers
support archives from prior supported releases and published nightlies, across
major versions. Keep frozen format fixtures and exercise them in offline tests
as the format evolves.

## Development snapshots

Whole-workspace developer seeds are storage-owned snapshots, not a second live
reference store. Capture holds shared update/access admission and uses SQLite
backup to include committed WAL contents. Installation verifies a staged clone or
copy before publishing only to an absent path under exclusive admission. It never
migrates, replaces existing saved state, or shares writable pages with the seed.
Developer snapshots use missing-only publication and produce closed standalone
SQLite files.
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
during cleanup and fail-closed handling of inconsistent schemas. Recorded Rust,
frontend and real-browser gates establish local implementation evidence for their
tested inputs. Native-model inference, full-corpus performance and hosted-CI
acceptance require separate evidence for the candidate under review. The
[database ingest benchmark](../../research/database-ingest-revision.md)
records the dated candidates, corpus observations and their limits.

Physical free-page reclamation runs only during explicit maintenance, outside a
SQL transaction under exclusive admission and a durable recovery marker. It keeps
workspace identity and logical reference revision; interrupted reclamation uses
the existing validated maintenance recovery. Before starting, storage checks free
space on the workspace and SQLite temporary volumes, conservatively budgeting
twice the logical database size on each. Insufficient or uncheckable capacity
skips optional reclamation without setting maintenance or failing the completed
reference update. The preflight cannot reserve capacity against other processes.
Startup never vacuums implicitly.
