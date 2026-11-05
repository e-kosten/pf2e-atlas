# Maintaining dependencies

Keep dependency changes in the owning Cargo or npm lockfile. Use the repository
Rust toolchain and `npm ci` for candidate validation. Follow the
[contribution sequence](../../CONTRIBUTING.md#development-loop) for review and
delivery. Release jobs generate notices from locked Cargo metadata and verify
them in packaged assets. See [release tooling](../../dev-tools/release/README.md).

## Review an embedding dependency update

The [embedding compatibility contract](../architecture/decisions/0066-embedding-compatibility.md)
describes when Atlas can compare new queries with stored document vectors. The
[reviewed dependency set](../../scripts/validation/embedding-stack.json) records
the relevant locked packages and manifest selections. CI detects changes to
that set. Review the upstream change and test both profiles before updating
the receipt.

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

The probe uses five fixture records and a fresh private database. It compares
repeated baseline queries, candidate queries against the original database, and
candidate queries against freshly inferred data. It checks ranked records,
matching passages and absolute score drift within 0.00002 on the same host.
It also checks zero document inference on compatible maintenance, retained saved
lists, and mixed reuse and inference after a source edit. Retain the binary
digests and result report with the PR. For a native runtime change, also run the
release build matrix. The bounded sample provides compatibility evidence for
review, with additional cases needed for changes that touch untested behavior.

When the evidence supports compatibility, keep the execution ID and update the
reviewed dependency set in the same PR.

```sh
python3 scripts/validation/embedding-stack.py --print > scripts/validation/embedding-stack.json
```

When execution is incompatible, bump the affected execution contract and validate
the supported rebuild path and saved-state preservation. Explain the affected
profiles and setup action in the release notes. Changes that only alter passage
layout can reuse verified vectors for identical inputs under the same execution
contract.
