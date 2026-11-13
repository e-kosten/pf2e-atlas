# Maintaining dependencies

Keep dependency changes in the owning Cargo or npm lockfile. Use the repository
Rust toolchain and `npm ci` for candidate validation. Follow the
[contribution sequence](../../CONTRIBUTING.md#development-loop) for review and
delivery. Release jobs generate notices from locked Cargo metadata and verify
them in packaged assets. See [release tooling](../../dev-tools/release/README.md).

## JavaScript toolchains

CI and release tooling run on Node 26 LTS. The frontend also supports Node 22.18+
and Node 24; its `engines` field defines the accepted versions for local commands.
Use `npm ci` in each maintained package when switching checkouts or lockfiles.

The frontend uses TypeScript 7's `tsc` command through the `@typescript/native`
alias. Its `typescript` alias supplies Microsoft's TypeScript 6 compatibility
package for ESLint and other compiler API consumers. Keep both aliases when
updating the checker; TypeScript 7.0 does not supply that API. The source generator
retains its separately pinned TypeScript 6 API and `tsc6` command; source-generation
upgrades also require the identity and generated-model checks in
[source research](source-research.md).

## Routine updates and advisories

Dependabot checks Cargo, the three maintained npm packages, GitHub Actions and
the Rust toolchain weekly. Each entry starts with two open routine PRs and a
seven-day cooldown. Related React and Query Builder packages update together.
Review breaking releases separately. An embedding-stack update needs the
compatibility evidence below before landing.

GitHub's advisory alerts and security update PRs are separate repository settings.
PR CI scans the committed Rust and npm lockfiles on dependency changes and
requires the result through `validation complete`. The separate dependency audit
workflow runs the same scanner daily and on manual dispatch, including the locked
dice prototypes. Cargo changes retain Rust/browser/native and embedding checks;
they do not automatically rerun unrelated source generation, frontend unit tests
or the installer matrix. Those checks remain selected by their owning inputs
and by full main verification. Run the same
scanner locally with Python 3.11 or newer and a supported Node/npm installation.

```sh
python3 scripts/validation/dependency-audit.py
```

A scan error needs investigation before interpreting the result. Address a confirmed
advisory with the smallest supported update and re-scan the candidate. Track
unmaintained-package warnings separately as replacement opportunities.

The frontend currently has a scoped exception for
[GHSA-g3hv-75wg-m249](https://github.com/advisories/GHSA-g3hv-75wg-m249)
in minimatch 10.2.6 under TypeScript ESLint's development-only dependency tree.
This exact resolution was re-reviewed on November 5, 2026; the original review
deadline remains unchanged.
The current parent package requires minimatch 10, and lint patterns come from
repository configuration. Revisit when upstream supports a fixed release, if
lint begins to accept untrusted patterns, or by December 4, 2026. The audit fails
when the exception expires or the affected resolution moves outside its scope.

## Review an embedding dependency update

The [embedding compatibility contract](../architecture/decisions/0066-embedding-compatibility.md)
describes when Atlas can compare new queries with stored document vectors. The
[reviewed dependency set](../../scripts/validation/embedding-stack.json) records
the relevant locked packages and manifest selections. CI detects changes to
that set. Review the upstream change and test both profiles before updating
the receipt. Schema-only changes run the offline setup and migration checks
on the release targets without model assets. Retrieval and storage changes run
the compact probe on Linux x86_64. Changes to numerical execution, its pinned
dependencies, or native build settings qualify both profiles across the release
matrix. The compact probe checks stored document samples and queries against the
pinned predecessor, and keeps compatibility transition and maintenance checks.
To run the wider comparison on demand, dispatch the CI workflow with
`full-embedding-probe` enabled.

Build the pre-upgrade and candidate Atlas binaries with the same pinned toolchain
and features. Use disposable checkouts or save a copy of the baseline executable
before rebuilding. Set `ATLAS_TEST_MODEL_CACHE` to the path of a verified model
cache containing BGE Base and EmbeddingGemma. Run from the repository root with
Python 3.11 or newer.

```sh
cargo test --locked -p atlas-embedding \
  pinned_query_and_document_token_ids_match_the_compatibility_baseline -- --ignored
python3 scripts/validation/probe-embedding-upgrade.py \
  --baseline /absolute/path/to/baseline/atlas \
  --candidate /absolute/path/to/candidate/atlas \
  --model-cache "$ATLAS_TEST_MODEL_CACHE" \
  --output /absolute/path/to/new/evidence-directory
```

To reproduce a CI run with the published fixture, authenticate GitHub CLI (`gh`)
and use `--pins --repository e-kosten/pf2e-atlas` in place of `--baseline`. Pass
`--base-ref` with the PR base commit when reviewing a change to the pin. To use a
local archive, pass `--bundle PATH --sha256 DIGEST`. Use Python 3.12 for the archive modes.
Prepare or verify the cache with
`cargo run --locked -p atlas-embedding --example prepare-model-cache -- "$ATLAS_TEST_MODEL_CACHE"`.
If the candidate uses a newer workspace schema, build
`cargo build --locked -p atlas-runtime --example migrate-fixture-workspace`
and pass `--workspace-migrator target/debug/examples/migrate-fixture-workspace`.
The helper runs the candidate's storage migrations on the private copy. For an
unchanged profile, maintenance may refresh content and query projections before
search becomes available. The probe requires zero inference, the same published input
identities and vector checksums, and successful deep vector validation before
comparing queries with the frozen expectations. This exercises the supported
setup migration window without replacing the numerical baseline.

The full manual probe checks ranked records, matching passages and absolute score drift
within 0.00002. It verifies zero document inference on compatible maintenance,
saved list preservation, and retrieval from a mixed index against a fresh
index built from the same edited source. For an intentional passage layout
change, it first requires the candidate to match frozen queries before replacing
the old passages, then reports reuse and compares the updated index with a fresh build.
Review the report for both profiles. For a material inference or tokenizer
upgrade, also compare sampled old and fresh vectors for matching inputs on
affected release targets. Include input shapes changed by the upgrade, such as
trait-only inputs, Unicode near the token budget, or varied batch sizes. Use
packaged predecessor binaries on those targets when native execution changes.

When the evidence supports compatibility, keep the execution ID and update the
reviewed dependency set in the same PR.

```sh
python3 scripts/validation/embedding-stack.py --print > scripts/validation/embedding-stack.json
```

When execution is incompatible, bump the affected profile’s compatibility ID
and set its `transition_to` to the new ID in the existing pin. Keep the old
archive digest. CI checks that the candidate rejects the old query contract and
rebuilds that profile, while the other profile retains its compatibility checks.
Validate `atlas setup` and saved-state preservation for the migration and explain
the affected profiles in the release notes. After publishing and verifying the
[new fixture](releases.md#publish-an-embedding-compatibility-fixture), advance only
the affected profile’s pin and remove its `transition_to`.
