# ADR 0050: Selectable search profiles

Status: accepted.
Date: 2026-10-15

## Decision

The product supports two pinned baseline profiles: `bge-base` (BGE Base English
v1.5) and `embeddinggemma` (EmbeddingGemma 300M). Fresh setup selects BGE Base,
without declaring a research quality winner. Omitted launch selection resolves
from the stored manifest: a sole profile becomes the default; both select BGE Base.
Explicit launch intent remains optional until runtime reads that manifest.
A request selects one profile; omission uses the resolved server/command default. An explicitly missing
profile fails without fallback. Profiles do not fuse scores or change lexical,
eligibility, exact-name, RRF60, window or root-collapse policies.

Domain owns the opaque `SearchProfileId`. Embedding owns the two built-in model
and baseline input contracts. Index owns persisted profile identity, schema,
profile-scoped vectors and SQL. Search selects the profile within retrieval.
Runtime composes shared inference sessions; application services and transport
forward typed requests and expose generated metadata.

Artifact format 3 replaces the single-model context with a canonical, duplicate-
free sorted manifest. One SQLite artifact retains shared checked source bodies,
prepared HTML, lexical units, filters and relationships once. Profile metadata
pins every required model/configuration/tokenizer asset digest, external weights,
output/pooling, normalization, dimensions, token limits, prefixes, inference policy
and baseline recipe. Each profile explicitly binds the full shared BGE segmentation
tokenizer digest alongside its selected native tokenizer digest; Gemma-only
manifests and document reuse keys retain that shared dependency independently.
Shared preparation hashes exclude that manifest. Complete
artifact generation hashes include it, while source fingerprints keep their
snapshot-local navigation meaning. Old artifact formats require rebuilding.

Within format 3, readers validate structurally complete persisted contracts for
the finite known IDs and 768-dimensional storage. They do not require every stored
profile to equal today's executable revision. New writers require current exact
contracts; query inference and reuse require exact selected-profile identity.
Thus a Gemma recipe/model revision change invalidates Gemma query/reuse without
preventing unchanged BGE vector reuse, model-free reads or internally coherent
stored-vector Similar. This does not admit a previous-format reader or arbitrary
profile/dimension extensions.

Both profiles emit 768-dimensional normalized vectors, held in a model-owned vec0 table
with globally unique unit IDs. Semantic units identify their profile explicitly.
Every KNN scan restricts unit IDs by selected profile and canonical eligible roots
inside the vector scan. Similar identity seeds, witnesses and vector reuse use the
same profile. Equal dimensions never establish model compatibility. Adding another
dimension requires a separate physical-index decision.

Ingest admits, enriches and prepares shared source content once. Both profiles
retain baseline BGE tokenizer segmentation: body 256, context 64, overlap 32,
identity 480 and baseline assembled input 512 including special tokens. Owned
headers omit the pack label; root/identity inputs retain their baseline recipe.
There are no parent excerpts. Selected native tokenization checks the final
officially prefixed input: BGE limit 512, Gemma 2048. Gemma's capacity does not
expand passages. Raw selected input hashes and final model execution/reuse
identities remain distinct. A Gemma-only build requires the shared BGE tokenizer,
but does not require BGE inference weights or stored BGE vectors.

Setup without profile flags preserves a readable current-format artifact's stored
set before checking/downloading assets. Fresh setup uses BGE Base; explicit flags
define the desired complete set. Model packages live at cache-root/model-ID/revision.
Setup repairs selected missing or mismatched files using verified temporary-file
downloads and rename, preserving old revision directories and existing bytes until
each replacement verifies. Incomplete packages remain unusable and retryable; no
old-layout bridge or package-wide staging is introduced. Cache location is outside
vector execution identity.

Requested profiles build sequentially. Exact reuse verifies selected identity,
reconstructed raw input, source address, ordinal and native token count. All
profiles finish and source/catalog freshness is rechecked before one validated
atomic publication. Any failure preserves the previous complete artifact.

Runtime retains one query-inference session for each loaded profile, shared across
independent SQL workers. The default warms at startup; another profile loads on
first use and remains retained. One global native mutex serializes loads/inference
with four inference threads, using the existing bounded retrieval admission.
The mutex releases before SQL ranking. Reader pools retain their single verified
physical artifact instance and full generation; adopting publication requires
restart. Short-lived commands retain their first successful service.

Lookup, lexical search, filter vocabulary, lists, encounters and stored-vector
Similar do not load or require unused model weights. Metadata distinguishes stored
vectors from verified query assets and query usability, reporting only stored profiles.
Readiness and query workers retain the same runtime owner. That runtime retains
per-profile checksum results behind cheap required-file metadata witnesses; changed
or repaired unloaded packages are rechecked. These are witnessed availability
snapshots, not a fresh checksum claim on every poll. Native loading always verifies
actual bytes. Loaded sessions remain usable after disk changes. Metadata checks
require no native mutex merely to observe loaded membership: the inference owner
publishes a finite retained-profile snapshot only after successful session insertion.
A text-vector request
requires the exact selected executable profile and its assets. Missing assets do
not prevent safe stored-vector Similar within the artifact.

Search pages, direct search/Similar and contextual discovery carry request-local
profile selection. Responses identify the resolved profile and artifact generation.
Expected generations are checked before query inference. Web handlers remain thin;
the generated readiness contract exposes the default and built/usable catalog.
Browser selection belongs to each browser/tab, never a global mutable server model.

## Consequences

Single-profile artifacts remain supported. Two-profile storage and retained native
sessions cost more disk/RAM; measured warm RSS and latency determine local resource
expectations. The bounded frozen-vector parity check establishes sampled inference
agreement, not full-corpus quality, artifact import compatibility or latency.
FastEmbed owns inference, generic pooling, normalization and batching; no custom
ONNX runner or general model/plugin framework is introduced.

Build progress uses the existing tracing events and CLI renderer, labeling profiles
and separating shared occurrences, unique execution inputs, reuse, inference and
publication. Successful completed work drives throttled counts and measured rates;
ETA stays unknown without a supported denominator/rate. TTY updates remain in place,
nonTTY output stays bounded on stderr, and JSON stdout/disabled progress stay clean.

This supersedes ADR0018's default/catalog and ADR0046's initial single-model
restriction. ADR0049's artifact lifetime, generation and abandonment guarantees
remain, with query sessions shared independently of SQL readers.

Native reuse normalizes complete model contracts and raw input metadata. Published
vec0 entries provide document reuse; no second full numerical checkpoint corpus is
retained. Unfinished computations have separate pending bytes/checksums and survive
failed builds. Trait-only native values share input identity; derived centroids
remain independent. Exact-distance cutoff selection is arbitrary; compaction may
change tied witnesses without changing eligibility or ranking policy.
