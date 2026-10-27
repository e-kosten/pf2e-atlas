# Running the frozen experiment

These commands describe the maintained runner added after input freeze. They are
execution guidance, not an amendment to frozen selections or judging rules.
Use the existing task artifact and release builds. No setup or embedding inference
is required. Do not publish the raw source packets or replace old research data.

```sh
cargo build -p atlas-dev --release --offline --locked
python3 docs/research/evaluations/2026-10-similarity/verify.py
```

Run a development smoke first. Supply the actual full build commit and record any
uncommitted compiler inputs in the build receipt. Reports bind the executable's
SHA256, caller-declared commit, observed Git state and all frozen input digests;
observing Git at invocation does not itself prove which tree built the executable.
The developer executable enables `atlas-search/test-support` for read counters.
Timings cover the complete production algorithm in that instrumented executable;
cross-check representative shipped CLI or web requests separately.

```sh
target/release/atlas-dev similarity evaluate \
  --manifest docs/research/evaluations/2026-10-similarity/manifest.json \
  --corpus docs/research/evaluations/2026-10-similarity/corpus.json \
  --scenario dev-h01 --phase fixed-pool \
  --build-revision '<full-commit-hash>' \
  --index .atlas/workspace.sqlite --path-mode repo \
  --output-dir scratch/similarity-evaluation/dev-fixed-pool --json
```

Repeat with `--phase oracle` and a fresh output directory. Omit `--profile` to
capture both stored profiles and construct a shared cross-profile fixed pool.
`--profile ID` and `--variant ID` deliberately narrow a smoke or timing run; the
report states profile coverage rather than claiming a complete comparison.

`--all-scenarios` explicitly opens the frozen holdout. Only run it after input
integrity and independent protocol review pass. `--phase all` includes production
capture, fixed-pool scoring, oracle comparison and the declared performance subset;
use explicit phases for staged work. Each scenario/profile report is create-new,
so reruns use fresh directories. Standard JSON stdout indexes those reports;
progress goes to stderr. Classified method failures remain in the reports and do
not become favorable latency or relevance values.

## Fixed pools and oracle scopes

Production captures use the full corpus and current remaster preference. Candidate
preparation is reused only across mathematically equivalent passage-decay lanes.
Derived decay rankings identify that reuse and have no fabricated fresh product
latency. Their final top-20 union across selected profiles defines the common
fixed pool; every eligible key in that pool is then directly scored per variant.

The oracle phase constrains both bounded retrieval and direct scoring to the
same explicit corpus, with remaster preference disabled. Admission recall compares
the oracle top ten with the complete pre-rerank candidate union; returned-top-ten
agreement is separate. Baseline admission is unavailable because the legacy
pipeline does not expose its complete union. Source evidence is bounded; a promoted
remaster result can require a checked-source fetch before judging.

## Performance lifecycle

For primary process-cold measurements, select exactly one declared scenario,
profile and variant in each separate runner invocation:

```sh
target/release/atlas-dev similarity evaluate \
  --manifest docs/research/evaluations/2026-10-similarity/manifest.json \
  --corpus docs/research/evaluations/2026-10-similarity/corpus.json \
  --scenario dev-c02 --profile bge-base \
  --variant passages-corresponding-0p25 --phase performance \
  --build-revision '<full-commit-hash>' \
  --index .atlas/workspace.sqlite --path-mode repo \
  --output-dir scratch/similarity-evaluation/performance-lich-bge-corresponding --json
```

The first complete product request is process-cold; its fresh service/context is
retained for 20 subsequent complete warm requests. OS caches are uncontrolled.
Do not label a new process per warm sample as warm. Multi-case/all runs can only
claim context-cold samples, and the report flags that they do not satisfy the
manifest's fresh-process cold definition. `--repetitions 1` is a smoke override;
actual count and whether the declared 20-sample count was met remain explicit.
A classified cold failure skips identical warm repeats and has no warm percentile.

Capture external process peak RSS using the platform's process-resource wrapper
and bind its receipt to the command, output index and executable hash. The runner
reports its PID and leaves peak RSS null; sampled RSS is not a substitute for
process peak. Cancellation and concurrent-admission probes are separate required
measurements, not silently inferred from single-request timings. Instrumented
read-statement counters are partial diagnostics, not a complete SQLite trace.

## Replay the completed recorded run

```sh
node --experimental-strip-types docs/research/evaluations/2026-10-similarity/verify.ts
```

The research-gate entrypoint invokes the unchanged frozen-input verifier, Python analysis regression tests and `verify_results.py`. It needs Node and Python 3, but no artifact, source corpus, model weights, server or network. The compact result manifest binds original prompt/opinion/adjudication bytes, pre-grading pool/packet digests, cited source addresses/hashes, generated score tables and raw timing samples. Dense generated tables use standard lossless gzip.

Recorded replay checks the canonical analysis metrics, score/rank joins, controlled admission and returned-top-ten comparisons, and cold/warm timing summaries. It does not recompute vectors, retrieve candidates, reapply product remaster policy or re-verify omitted source bodies. Full fresh execution still requires the pinned source/artifact and compiled provenance described above. The completed historical run and its limits are documented in [RESULTS.md](RESULTS.md).
