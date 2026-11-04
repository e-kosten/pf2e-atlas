# Database ingest benchmark

The [publication decision](../architecture/decisions/0065-input-owned-reference-publication.md)
defines the design. These observations record what it cost to run on the pinned
PF2e 8.6.1 corpus (`28d9d156`), with 29,894 roots, 323,470 document occurrences and
both embedding profiles. The measured final executable was the normal release
build of `ffe1b50f`, with SHA-256
`44a81e4c231e91e24ff2e1ece227be01373e2026f29b6d60296fe878745ae708`.

| Candidate and workload | Wall time | Peak process RSS | Inferred / reused |
| --- | ---: | ---: | ---: |
| Earlier main control `d7187e78`, cached | 241.94 s | 5.57 GiB | 0 / 323,022 |
| Later main control `d7187e78`, cached | 319.59 s | 4.53 GiB | 0 / 323,022 |
| Initial PR `c1387b92`, cached | 518.44 s | 0.94 GiB | 0 / 323,022 |
| Pre-simplification `8bc73b73`, cached | 243.98 s | 0.845 GiB | 0 / 323,022 |
| Mechanical cleanup `ac1261b5`, cached | 247.04 s | 0.889 GiB | 0 / 323,022 |
| Final implementation `ffe1b50f`, cached | 141.99 s | 0.894 GiB | 0 / 323,022 |
| `8bc73b73`, Heal edit | 249.78 s | 4.432 GiB | 2 / 323,020 |
| `ffe1b50f`, same Heal edit | 144.68 s | 4.223 GiB | 2 / 323,020 |

The last five rows used the same restored private checkpoint, source and model
assets for each workload. The edit added one fixed paragraph to Heal. The main
controls are earlier observations of the same corpus, with the ingest inputs
of the integrated main base `f76232c8`. Main was measured separately from the
final comparison. The mechanical cleanup showed no measured runtime improvement.

Both final publications passed separate deep audits covering all 29,894
snapshots. These took 56.98 s cached and 57.63 s edited. Ingest plus audit took
198.97 s and 202.31 s respectively. The audit now runs on committed data when
requested. Its placement changes the assurance contract, so the timings compare
the ordinary workflows with their respective validation policies. Single runs,
uncontrolled caches and the SQLite change from 3.50.2 to 3.51.3 limit attribution.

Cached reference metadata matched the checkpoint logically and physically, and
publication advanced its revision once. The edit changed only Heal and its two
expected native inputs. All 22 saved tables, their allocation
counters, 30,430 stable identities and 323,020 retained input/token/checksum
bindings survived the edit. A fresh payload audit verified the stored bytes.
The edited workspace main file was 1.558 GiB.

## Retrieval tradeoff

The ownership study observed 0.629% duplicate document excess. Across 192 warm SQL
cases, the input-owned prototype preserved eligible capped counts and distances,
with three same-record cutoff witnesses changing. Broad K=1,024 queries took
about 260 ms with occurrence-owned vectors and 304 ms with input-owned vectors.
This measured isolated SQL, without inference or product latency. The design
chooses stable numerical ownership to avoid reused-vector copies during ingest.

A separate root eligibility probe found exact accepted-key parity with the
existing compiler across 15 corpus cases, with six repetitions per lowering.
The original report and SQL work counters are in the archive.

## Repeat the ingest comparison

Set `atlas` to the pinned normal release binary and use disposable copies of the
same compatible workspace checkpoint. Keep source bytes, both requested profiles and model assets
fixed. For the edit cohort, apply the same edit to each copy of the source before
building. On macOS, `time -l` from `/usr/bin/time` reports process peak RSS.

```sh
/usr/bin/time -l "$atlas" index build --output "$workspace" --source "$source" \
  --embedding-cache-path "$model_cache" --json --progress always
/usr/bin/time -l "$atlas" index validate --index "$workspace" --json --progress always
```

Capture the binary hash, input identities, wall time, peak RSS, inference/reuse
counts and audit result. Time upgrades separately. The full reports and historical
harness are preserved outside PR184 with the source snapshot `6999a84e` and the
original receipts. The final receipt index is `final-closeout.json` under the
archived `.atlas/validation/r8-simplification/` directory.
