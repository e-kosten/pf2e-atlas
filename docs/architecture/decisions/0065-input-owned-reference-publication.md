# ADR0065: Input-owned vectors and owned reference publication

Status: accepted

## Context

A semantic unit describes an occurrence in a record/profile/location. An embedding
input identifies exact prepared text and model execution. These lifetimes differ:
changing an occurrence need not change the numerical input another occurrence or
trait still uses. Storing reusable bytes in occurrence-addressed vector slots makes
bounded publication depend on copying selected values before those slots change.
Holding every reused vector in RAM or maintaining a full staging copy resolves
that hazard, at memory or temporary-storage cost.

The [ownership study](../../research/embedding-value-ownership.md) found only 0.629%
duplicate document excess. Its shared-input query prototype was slower than the
occurrence index, with approximately 44 ms extra broad warm SQL time at K=1,024.
Input normalization therefore has an explicit retrieval tradeoff. The accepted
reason for it is stable numerical ownership and a simpler publication lifetime.
Those observations do not establish final ingest speed or product search quality.

Early pruning of recovery work also couples cache eviction to complete historical
build membership. Treating recovery data as a disposable cache lets eviction cost
recomputation without threatening a prepared publication.

## Decision

Store one active searchable native vector per full model/input identity, addressed
by input_id in model-owned native_vectors tables. Occurrences and ordered trait
chunks reference that input; retain their separate record/location identities.
Derived trait centroids remain separate. There is one permanent native authority,
without a second full occurrence-vector projection or compulsory value versions.

```mermaid
erDiagram
    EMBEDDING_MODEL ||--o{ EMBEDDING_INPUT : identifies
    RECORD ||--o{ SEMANTIC_UNIT : contains
    SEMANTIC_LOCATION ||--o{ SEMANTIC_UNIT : locates
    EMBEDDING_INPUT ||--o{ SEMANTIC_UNIT : supplies
    TRAIT_DEFINITION ||--o{ TRAIT_SEED_CHUNK : orders
    EMBEDDING_INPUT ||--o{ TRAIT_SEED_CHUNK : supplies
    EMBEDDING_INPUT ||--o| NATIVE_VECTOR : owns
    EMBEDDING_INPUT ||--o| PENDING_VALUE : caches
```

These are logical relationships. A registry input can have a published vector
and a pending replacement; only the published value is searchable. Repeated
occurrences and trait chunks retain separate attribution while sharing that value.

A forced result can differ for the same input. Bind the selected checksum as well
as model/input/token identity. Keep new or forced bytes in owned attempt staging,
then replace the active value only inside atomic publication. Every new-generation
consumer must use that selected value. Preserve ordered/repeated trait weights
and invalidate numerical derivatives even when input text hashes are unchanged.
Reclaim an input only after all document/trait consumers finish and no retained
recovery value needs its metadata.

Use one concrete prepared-publication owner for the captured context/inventory,
expected workspace generation, prepared metadata and build-local value selection.
The constructor and private interfaces maintain their relationship. Consumers
validate bounded frames, vector shape/checksum and actual consumed data; they do
not repeatedly authenticate unrelated complete private files. Stable published
values remain handles until consumed through the publication transaction; only
unpublished selected bytes require numerical staging.

Inference runs outside database transactions in bounded batches. Reuse precedence
is same-attempt selection, matching unpublished recovery value, compatible
published value when reuse is enabled, then inference. Forced execution deduplicates
within the attempt. Cache hits and new inference become owned staged bytes before
cache eviction can invalidate selection.

Retain one recovery value per exact model/input key across failed attempts. Use
simple oldest-first eviction under a coarse logical-payload cap; it is not a
staging quota or physical-space reservation. Clear recovery values only with a
successful complete source-ingest publication. Schema upgrades and metadata-only
projection refreshes preserve the cache because they do not publish pending
numerical results. Do not maintain candidate/active/retired build
membership merely to make early eviction safe.

Group disposable files in one attempt directory under existing update ownership.
Close every child handle before deleting the directory, including Windows error
paths. Normal completion/error removes owned scratch; a subsequent explicit update
may clean recognizable abandoned attempts for that same workspace. Ordinary reads
and startup perform neither inference nor scratch/cache cleanup.

Publish the captured input set. Check external bytes against capture when consumed,
then use the owned data without consulting that file again. Manifests, catalogs,
pack bindings and alias preparation follow the same contract. Later source edits,
additions and removals belong to the next build; remove the final freshness rescan.
Missing or unreadable declared packs still fail capture, while existing empty packs
and deliberate declaration removals remain valid.

Maintain one SQLite workspace and complete final transactional validation. Setup
and upgrades may be maintenance windows; no continuous-availability or short final-
only interval is required. Failed activation preserves the prior publication,
saved state and committed recovery results. Private staging failure cannot authorize
a partial publication. Existing command/force/deep-audit semantics remain.

## Retrieval contract

Admit only native inputs with an eligible occurrence before nearest-neighbor
selection. Expand a hit only into eligible occurrences, retaining exact record,
owner, field and passage attribution. Trait-only and pending values cannot become
document candidates. Preserve Core, exclusions and configured Similar role,
correspondence and identity masks at the appropriate query boundary.

K remains an occurrence budget. Selecting K eligible inputs, expanding occurrences
and capping back to K preserves the nearest occurrence-distance multiset. Native
cutoff ties can choose different occurrence witnesses; the outer sort does not
establish a globally deterministic cutoff. A unique-input coverage policy is a
separate retrieval decision. Native-neighbor cache recipe/generation admission must
follow the new lookup while retaining the bounded occurrence payload contract.

## Migration and alternatives

Published migrations and their transitive executable behavior remain immutable.
The unreleased PR suffix describes the final transformation directly from main:
no schema step creates experimental target history only for a later step to remove
it. Numerical conversion precedes section backfills that can reach current native
validation. Copy and verify every old occurrence and trait-only value through public
vector SQL, preserve pending values independently, then retire old storage inside
the same upgrade transaction. Preserve exact vector bytes, saved identities,
clock/tracker payloads and published historical callbacks; no inference or source
access is required for numerical conversion.

Keeping the occurrence index plus owned transient staging remains a conservative
alternative with less retrieval disruption. A separate stable numerical authority
plus an occurrence index preserves that query shape but duplicates roughly 987 MB of
logical permanent payload at the studied corpus size. The accepted design chooses
stable input ownership and one native authority instead.

## Consequences and validation

The design removes occurrence-slot reuse hazards and the need for a full reused-
vector copy. It introduces eligibility translation/occurrence expansion in native
queries and requires evidence for the actual integrated cost. Common captured-input,
preparation and cache simplifications apply independently; measure their benefits
without crediting every improvement to numerical normalization.

Validate real schema upgrades, rollback/crash recovery, saved/clock/receipt
preservation, forced numerical replacement, trait-only transitions, profile changes,
eviction after selection, consumed-input changes and bounded temporary storage.
Protect all product retrieval consumers and cache lifetimes. In-memory feasibility
fixtures and isolated SQL comparisons do not establish product integration,
migration peaks, relevance quality or final ingest performance.

This amends the numerical ownership, recovery-cache and source-freshness portions
of [ADR0051](0051-persistent-workspace.md) and the numerical execution/publication
portions of [ADR0050](0050-selectable-search-profiles.md). Their workspace/session,
profile identity, source authority and atomicity boundaries remain active. The
[runtime](../runtime.md) and [artifact contract](../artifact-contract.md) own current
composition and stored-family details.
