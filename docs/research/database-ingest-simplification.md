# Database and ingest simplification

Findings, 2026-11-01 UTC. Measured production candidate `c1387b92`; final operation
results and the subsequent developer-test correction are reconciled below.

## Question and resulting design

Can Atlas simplify reference and ingest ownership while preserving saved state,
exact native values and atomic publication? The review found incomplete discovery
could be mistaken for removal, repeated whole-corpus payloads, weak saved
relationships and data retained beyond its useful lifetime.

The implementation keeps one SQLite workspace and changes seven areas:

- Require complete checked discovery before publication and account for exclusions.
- Constrain saved enums and encounter membership; strictly decode retained values.
- Keep lifetime turn receipts and one outgoing snapshot; batch six roster collections.
- Share canonical selected sections across search, Core and navigation.
- Stage rich content in disposable streams and reconcile per-component changes.
- Publish exact native handles through frozen values and complete checkpoint targets.
- Lower positive eligibility to indexed relations; preserve shared general semantics.

The [runtime](../architecture/runtime.md) and
[artifact contract](../architecture/artifact-contract.md) own detailed contracts.
This ships as one PR and release slice. Named migrations 25–29 separate integrity,
receipts, sections, component digests and native targets for ownership and focused
review/tests. The existing migrator executes the complete pending set inside one
schema-upgrade transaction, including validation and foreign-key checks. These are
not released intermediate versions. Published 23/24 SQL and their clock-conversion
callback remain unchanged.

## Experimental identity and method

Shipping measurements use `c1387b92dc99a03ab36c460047d9b565a270d89d`, code-input SHA256
`03aa0ad19f36d396b8a6b0be8bd59325e2d9a4ba0881a2cf8a608e9a17ae04a7`.
The matched baseline is main `d7187e7834110e3a524e651147ee866b3ad8cd27`.
Both shipping executables use Rust 1.95.0, normal release optimization level 3,
default/web features and storage without SQL diagnostics, on macOS ARM64.

| Frozen artifact | SHA256 |
| --- | --- |
| Matched baseline shipping CLI | `da34886267e9c2cbfa231954f469dfbe3f9d932b951414b73258119584b988d2` |
| Current c138 shipping CLI | `caa87e957369ec38f2ef892bdd8b53eb5c99eb3859dc20769b6b0b4adfcd91a7` |
| Current c138 diagnostic driver | `5abdb4b2fb6d519e3ae7d56288d402a69ed16a516408a6797553f7d452a2bd74` |
| Current c138 eligibility probe | `6fb0cfbc09460ef99713c726830b348ec550447286e075d014ce16c245c6f424` |

The driver/probe enable developer SQL diagnostics and have separate measurement
scopes. All artifacts are independent frozen copies. Shipping measurements and
SQL diagnostic measurements remain separate. Earlier cohorts `761032be` and `8f1bdd8a` below are explicitly intermediate evidence.

One private workspace/source uses PF2e 8.6.1 commit
`28d9d156f252d084371d994dc359509b49eadf18`, 29,894 roots and two 768-dimensional profiles:
BGE Base revision `a5beb1e3e68b9ab74eb54cfd186867f64f240e1a` and EmbeddingGemma revision
`5090578d9565bb06545b4552f76e6bc2c93e4a66`. Receipts bind complete model, tokenizer,
asset and input identities. Shared seeds/source/models and the installed workspace
remain protected. Small fixtures cover broad edge cases.

Runs are serial observations with uncontrolled OS/model caches. Shipping builds
use the same 500 ms database-family sampler and 2 s stderr inference guard; both can
perturb timing. Driver SQL diagnostics, availability observation and snapshots are
separate. Whole-process peak RSS is not per-phase memory. File lengths/stat blocks
and sampled peaks do not establish exclusive allocation or SSD writes.

## Preservation and measurements

The earlier 761 authentic 24→29 upgrade preserved 30,430 stable identities, all
roots, 1,586 trait-native values, 1,568 centroids and the oracle's 22 saved-table projections.
Clocks, trackers and retained payload bytes remained exact; receipt identities
survive intentional retirement of superseded outgoing snapshots. The workspace key
remained `a9474928b73e8cf89e84c5ed6990a838`. Upgrade revision 10 used context SHA256
`1fa8e1c86b66540e328de16bc2e247db1d44a3b6ff49ea4b59436e7076fb50b8`;
only expected section storage and context-recipe differences were admitted.

The first 761 build added the intended admission summary: 29,894 discovered/admitted,
237 partial records and zero quarantined/excluded. Its revision 11 context was
`e32e80857e205d49c39a22a3ceac5ca2b971be0b6e7dc9ce63f87094362f24b8`.
Steady 761, instrumented 761 and intermediate 8f advanced once each to revisions 12, 13, 14
with that exact context/profiles. Each build inferred 0 and reused 323,022 inputs
(161,511/profile across 161,735 units); source identity and 323,470 units stayed exact.

Those earlier cohorts passed independent validation and strict comparisons of all
22 saved projections, stable/native identities and complete canonical sections; logical and physical-row
equality passed. Native registry equality is bound to fresh numerical validation
of the same generation; the Python oracle does not read vec0 shadow bytes directly.
Physical-row equality is not file-byte equality. Earlier schemas cannot directly
prove equality of unstored section fields; retained checked content and migration
fixtures supply separate evidence, while 29→29 protects all stored whole extents.

| Operation | Scope | Wall seconds | Peak RSS GiB | Outcome |
| --- | --- | ---: | ---: | --- |
| Baseline unchanged build | Matched schema 24 shipping | 241.939 | 5.572 | Preserved; 0 inferred, 323,022 reused |
| Current unchanged build | c138 shipping | 518.437 | 0.937 | Preserved; 0 inferred, 323,022 reused |
| Current independent validation | c138 shipping | 67.904 | 2.034 | Same-generation deep validation passed |
| One Heal description change | c138 shipping, inference included | 471.358 | 4.370 | Preserved unrelated data; exactly 2 inferred inputs |
| Genuine Gemma-only removal | c138 diagnostic driver | 305.043 | 0.898 | Preserved exact retained subset; 0 inferred |

The current build observed much lower peak RSS than baseline, with longer elapsed
time. Its 518.437 s is above intermediate 8f's 473.419 s and below attested 761's
614.169 s; earlier 761's first build took 651.686 s. Source preparation/prevalidation
also took longer in unchanged code (115.874/67.827 s). Uncontrolled host/cache
conditions prevent a causal or generalized latency-gain claim from these runs;
the structural VM and six-to-four read proofs remain separate evidence.

Current same-generation validation and strict saved/native/whole-section preservation
passed. Initial current validation used 2.034 GiB peak RSS; current retry validation
reached 2.090 GiB. Build RSS establishes no all-operation 1 GiB bound. File-backed
staging exchanges rich memory ownership for disk work.
Scoped boundary observations do not measure global temporary peaks or exclusive
allocation; runners explicitly mark temporary allocation unmeasured.

Current shipping reference reconciliation reported zero root replacements and
zero affected SQLite executions in its scope; selected-checksum metadata still
reported 161,511 affected rows per profile. Operational metadata writes are outside
the reconciliation count. Sampled 500 ms file peaks were 1,746,870,272 B for the
workspace and 498,552 B for its WAL; the existing upgrade backup was 1,747,124,224 B.
These lengths do not measure total temporary allocation or physical writes.

The separate current instrumented run took 535.163 s with peak RSS 956,563,456 B.
SQL tracing and the availability observer perturb it; its wall/RSS is not a
matched shipping comparison. It inferred zero inputs and passed fresh 66.000 s
validation plus strict preservation at reference revision 16. The one-Goblin
source-backed encounter produced 323 coherent successes among 478 samples: 312
on revision 15 and 11 on revision 16, with 155 typed `service_busy` results.
Successful request median/p95/max were 4.308/6.499/76.125 ms; busy median was
253.545 ms. No successful response mixed identities/generations. Goblin HP 6,
AC 16 and level -1 remained exact; optional fingerprints were absent.

Readiness reported both profiles usable before/after; the observer stopped normally
through its stop file and joined. Separately timed readiness/encounter calls do not
establish atomic pairs, exact continuous downtime or roster scaling. Its 1 s sleep
followed each request. SQL events include trigger/rolled-back execution and are
not committed writes. Skipped setup is a separate control. No new CPU snapshots
were taken on c138. The diagnostic build observed 12,992,174 statement-start,
1,940,445 subprogram-start and 14,932,619 completion events across 14,838 workspace
connections. These are scoped execution events, including rolled-back/trigger work;
they are not distinct SQL texts, committed logical writes or a global write count.

The current driver then stopped at the reserved interruption boundary with actual
exit 72. The same frozen driver recovered the checked publication in 64.176 s;
fresh validation took 57.588 s. Reference revision 16, complete context/profiles,
all saved/native/whole-section evidence and allocation rows stayed exact. Maintenance
cleared, and the interrupted temporary filenames/lengths were retained. The ordinary cached retry then completed in 525.882 s, peak RSS 1,012,088,832 B,
with zero inference and 323,022 reused inputs. Fresh 59.386 s validation and strict
preservation passed at revision 17; its validation peak was 2.090 GiB.

## Queries and saved reads

The current release eligibility probe executed 15 cases, both production and general
plans, with six samples per plan. Full result sets and repeated samples matched.
Representative median VM steps were:

| Predicate cohort | Production plan | General expression |
| --- | ---: | ---: |
| AC | 9,648 | 459,731 |
| Level and AC | 16,738 | 586,147 |
| Price | 95,048 | 577,343 |
| Owned spell, deliberate general path | 2,660,043 | 2,660,043 |

This measures candidate-plan work within the same diagnostic feature graph, not
shipping latency or a universal optimized path. AC used its covering index;
price still scanned its applicable projection index. The shared general compiler
remains intentional for unsupported optimization shapes.

Shipping browse preserved complete ordered memberships of 639/639/419/1,057 roots
for those four cohorts. Four fresh first-page processes per cohort took
16.3–18.8/16.1–18.9/16.8–18.1/101.6–107.2 ms, including process startup.
Full pagination used the first observation plus continuation requests.
Four fresh native-query processes per profile returned identical ordered 100-row
responses within each profile: BGE took 0.611–0.667 s and Gemma 1.840–1.934 s.
Coverage remained bounded, with candidate roots 691/643 and semantic window 1,024.
Stderr was empty, so cache hits, model loading and query inference are unobserved;
fresh processes do not establish cold persisted neighbor caches. There is no
matched native baseline or attribution of native latency to eligibility changes.

Five composed source-backed encounter reads took 3.363–4.463 ms with one exact
response digest and the same Goblin facts. Startup and serialization were excluded;
the first read was context-cold and later reads shared one service. This tiny
encounter does not establish roster scaling. Fixture protection separately covers
all six encounter-scoped child collections and query-count bounds.

The exact Heal description variant changed one root and inferred exactly one input
per profile, reusing 323,020 inputs. Its 471.358 s build included model loading and
inference, with 4.370 GiB peak RSS; cached-build RSS does not describe every build
mode. There is no matched normal-release changed-record baseline. Fresh 57.774 s
validation passed. All 29,893 unrelated roots, whole-section allocation rows, saved
projections, stable identities, trait-native values and centroids stayed exact.
The 784 trait-definition JSON rows changed only their source revision from the
clean Git commit to null, plus their independently checked JSON hash, reflecting
the deliberately dirty source; context changed only the expected source fingerprint
and revision. The oracle retained those actual differences without normalization.

The Heal validator's file-binding receipt initially rejected newly created empty
WAL/SHM sidecars. A narrow binding correction accepted absent versus zero-length
WAL only with unchanged main-file length/SHA, no nonempty WAL/hot journal and no
open handles. Raw failure and before/after evidence remain retained; the product
validator passed once and was not rerun.

A genuine Gemma-only update on the same exact Heal source took 305.043 s and
964,083,712 B peak RSS in the diagnostic driver. It inferred zero inputs, reused
161,511 inputs and published only Gemma with 161,735 units. Fresh shipping validation
took 54.008 s. The strict removed-profile comparison preserved the exact retained
Gemma native values, centroids, Core, whole sections, saved projections and stable
identities. Raw physical rows changed with the intentional BGE removal; this is
retained-subset preservation rather than whole-workspace physical-row equality.

After the profile-removal case, the original private Heal/source bytes and exact
protected schema-22 seed were restored to the same private workspace. A normal
metadata-only 22→29 upgrade took 67.401 s with 428,261,376 B peak RSS; fresh shipping
validation took 35.543 s with 2,111,062,016 B peak RSS. No setup, source acquisition
or inference ran. This reseed is not a profile-addition benchmark.

The final workspace is schema 29, reference revision 9, original workspace key,
context `1fa8e1c86b66540e328de16bc2e247db1d44a3b6ff49ea4b59436e7076fb50b8`
and both profiles. Source Git state is clean at the original commit. Maintenance,
pending numerical values and active targets are absent. Component attestations
remain absent for all 29,894 roots after this metadata-only upgrade; none were
invented. The normal pre-upgrade backup exactly matches the protected original
seed SHA256 `cf54b150105d0842c5a505d215dfb1915d8ae5795155d0307de6e6bb1b3c37ca`.
A fresh backup capture verifies the original 22-entry ledger as a prefix of 29.

Strict original-22 preservation passed for saved meaning, stable identities and
native values. Only known clock-bearing payload tables admit the released clock
conversion's JSON-semantic transformation; their raw reserialized text is not
claimed identical. Direct 24→29 and 29→29 comparisons preserve raw payload text.
Schema 22 lacked stored whole-section extents, and the historical capture omitted
the ledger; no retrospective full-extent or historical-checksum proof is claimed.
The new canonical allocations intentionally differ. Checked retained content and
migration fixtures provide separate section-derivation evidence.

## Cost findings and regression protection

Earlier cached attempts were aborted after 1,992.025 s (`8a4552a7`) and 941.151 s
(`1de6bb19`); neither is a completed-build timing or speed ratio. They exposed
avoidable candidate/payload lookups and repeated workspace-session costs. Exact-slot
probes, finite verified statement/metadata caching and phase-correct checked reuse
batches now replace that work; recovery and preservation passed after both aborts.

Two declared 761 one-second snapshots at 10 ms sampling localized membership paging and
outer JSON numeric-byte-array parsing; they are not whole-phase percentages.
Intermediate 8f pages by membership `n.input_id`, left-joins metadata with exact guards,
and encodes transient snapshot bytes as strict UTF-8 strings for the same checked
decoder. Persistent codecs, dependencies and schema remain unchanged. Fixtures using production SQL/schema measured old paging at 11,263/73,390 VM steps for 1,024/8,192 inputs;
new paging used 3,415 across tested sizes/sparse/unrelated inputs, with zero observed
sorts/full scans. Regression checks bound work without pinning full planner text.
Unicode, control, deep/full-spool and rejection fixtures protect exact bytes.

Current c138 reuses compact key/optional-name/product facts returned by the unchanged
full pre-mutation input check for two identity consumers. Focused publication counts
drop from six to four full-record reads, including the unchanged trait pass. Full
snapshot checks, final SQL validation and saved-state safeguards remain. The compact
operation-local vector still grows with corpus size and contains no rich records.
No API/format/schema/hash/recipe/geometry/history change is introduced. The current
run measures elapsed/RSS, while an isolated causal effect of this cleanup remains
unestablished; remaining elapsed cost stays an explicit tradeoff.

Current c138 passed the maintained 11-phase CoreRust gate: 1,694 workspace tests
with 34 default skips, plus the separate CLI-only suite at 107 passed/1 skipped.
Current packaged browser checks passed 28 desktop-Chromium/phone-WebKit cases.
Two tokenizer fixtures and one native lifecycle fixture each explicitly passed
1 test with 0 ignored. The release native fixture exercised four scenarios including
true profile addition, second-profile failure/checkpoint retry and removal. Other
default skips are not claimed executed.
Unchanged 8f frontend evidence (1,762 tests/169 files), source generation and release
tooling checks are explicitly reused through matched input provenance. This combines
fresh affected checks with valid reuse; it is not 17 freshly run current phases.
These establish local evidence for the recorded code inputs; hosted CI is a
separate result tied to the published PR head.

Hosted CI subsequently exposed a concurrent tracing-callsite race in the developer
experiment fixture: an unobserved fixture build could disable a shared callsite
for an observed build. A test-only correction gives every fixture build its own
observer/file lifetime while retaining the exact stage/component assertions. The
eight-test example passed twice under normal concurrency; the maintained five-phase
standalone developer gate passed (55 tests, 4 default skips, format, both Clippy
policies and build). Independent review passed. This correction is entirely under
`cfg(test)`; production example bytes and frozen shipping artifacts are unchanged.
Its code-input SHA256 is
`1811e0b019c44f509adb06e77706b82f36ff110b8ff786fa0698adc7b49ae876`;
local receipts live in `.atlas/validation/dev-ci-observer-fix/run.08URnr/`.
The shipping/corpus evidence above remains bound to c138, rather than a newly
measured descendant. Final PR-head CI remains a separate delivery gate.

Batch fixtures observe 1/1/5 reuse batches for 1/32/129 records, bounded by 32 raw plus
32 retained records or 32 prepared plus 32 layout roots. API requests cap at 128 and
owned results outlive their closed snapshot. Compact manifests/resolvers/identity
sets, component evidence, trait offsets and publication facts remain corpus-sized.

All eight current operation closeouts passed. The final composite indexes 67
runner measurements, including the deliberate exit-72 interruption. All controlled
processes and workspace handles closed; source/data/CPU lanes were released before
final documentation. The task workspace and local evidence remain for PR review.
The installed workspace and shared source/model assets were untouched.

Historical performance reports remain unchanged. The retained local evidence lives
under `scratch/database-ingest-candidate-evidence/main29/finalc138/`; frozen earlier
cohorts retain their own directories. Local validation identities are recorded in
`.atlas/validation/final-checked-identities/closeout.json` and
`.atlas/validation/release-checked-identities/closeout.json`, with explicit reuse
provenance. These task artifacts are evidence for this review, not product runtime
dependencies or portable benchmark guarantees.
