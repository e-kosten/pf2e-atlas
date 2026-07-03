# Phase 4 Semantic Search Validation

Date: 2026-05-13

## Scope

Validated Rust keyset-prefiltered semantic search against the current Phase 4 MiniLM artifact:

- Rust index: `.cache/pf2e-rust-index.batch32.sqlite`
- Node index: `.cache/pf2e-index.node-minilm.sqlite`
- Embedding cache: `.cache/hf-models`
- Rust command surface: `rust/target/release/atlas search semantic`
- Node comparison surface: `Pf2eDataService.search` with `profile: "concept"`

The Node comparison is a product-level sanity check, not an exact rank parity check. Rust returns raw vector KNN hits; Node returns hybrid search results with lexical/semantic fusion and a different document input policy.

## Method

The runner in `scratch/validation/phase4-semantic-search-validation.mjs` executes:

1. Rust semantic search with `limit=10`.
2. Rust semantic search with `limit=4096` for filtered cases.
3. A top-10 comparison between the two Rust result sets.
4. Node concept-profile search for comparable filters.
5. A top-10 overlap count between Rust raw-vector hits and Node hybrid hits.

`4096` is the sqlite-vec KNN ceiling observed during validation. If sqlite-vec were applying filters only after a small global KNN candidate set, selective filters would be expected to diverge between `k=10` and `k=4096`.

## Results

| Case | Filter Types | Rust `k=10` vs `k=4096` | Rust `k=10` time | Rust `k=4096` time | Filtered hits at `k=4096` | Node time | Rust/Node top-10 overlap |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| unfiltered-healing | none | n/a | 156.0 ms | n/a | n/a | 511.5 ms | 4/10 |
| spell-primal-healing | family, set, all_of | exact | 111.7 ms | 110.8 ms | 26 | 89.2 ms | 10/10 |
| creature-dragon-rarity | family, set, rarity/in, all_of | exact | 115.1 ms | 113.4 ms | 241 | 72.3 ms | 4/10 |
| spell-sustained-burst | family, boolean, enum, all_of | exact | 110.9 ms | 122.6 ms | 44 | 64.8 ms | 5/10 |
| creature-undead-high-ac | family, set, metric, all_of | exact | 114.3 ms | 119.3 ms | 226 | 99.8 ms | 3/10 |
| equipment-consumable-mid-price-not-unique | family, set, price/between, not, all_of | exact | 121.5 ms | 114.2 ms | 717 | 106.1 ms | 3/10 |
| spell-mental-or-fear | family, set, any_of, all_of | exact | 111.2 ms | 117.3 ms | 280 | 65.3 ms | 6/10 |
| links-to-frightened | links_to | exact | 124.0 ms | 132.6 ms | 1577 | 101.6 ms | 8/10 |

## Interpretation

Rust filtered vector retrieval did not show lossy global-top-k-then-filter behavior in these cases. For all selective filters tested, the top 10 from `k=10` matched the top 10 from the maximum supported `k=4096`.

The current Rust CLI timings are process-level timings for a release binary invocation, including tokenizer/model load and query execution. Node timings are measured inside an already loaded application runtime, and the first Node query includes lazy embedding-provider warmup. These numbers are useful for rough smoke validation, but not a final apples-to-apples microbenchmark.

Node overlap varied by case because the surfaces are not rank-equivalent:

- Rust uses raw vector distance only.
- Node uses hybrid lexical/semantic ranking.
- Rust and Node use different semantic document inputs.
- Rust includes more generated/default-visible records in the artifact.

## Follow-Up

No vector-side metadata or partition columns are needed for the Phase 4 baseline based on this validation. If query latency becomes a production issue, add a dedicated warm-process benchmark route before changing the vector schema.
