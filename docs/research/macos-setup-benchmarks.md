# Historical macOS setup benchmarks

These two rehearsals measured different operations on pre-release Atlas builds.
They preserve a baseline for first-run setup and a separate measurement of a
workspace-format upgrade.

## Clean first run on November 9, 2026

One run used an Apple M4 Max with 64 GiB of memory and macOS 27.2. The source
was `15f8ae34fad552ea93af0510331e07d312aea957`. Both the native host and its
bundled CLI were optimized arm64 builds. The test app had a distinct bundle
identifier and ran with an empty Atlas home and a restricted PATH.

Setup started in the native app and generated BGE Base embeddings locally.
The observer sampled roughly once per second. Elapsed times start when it
first detected the setup process.

| Milestone | Elapsed |
| --- | ---: |
| Pinned Foundry archive available | 12.8 s |
| BGE Base model file available | 29.7 s |
| Setup process exited | 3 h 38 min 29.2 s |
| Backend served the first readable record and reported BGE Base usable | 3 h 38 min 29.5 s |
| First vector search returned three results | 3 h 38 min 30.5 s |

The workspace held 29,894 records and 160,717 embeddings. The first vector
query took 0.992 seconds including CLI startup and reused the running backend.

The native product window stayed blank after setup. Standalone WebKit rendered
Search against the same backend. Quitting and reopening the native app with the
completed workspace restored Search and returned results. The elapsed time to a
readable backend response therefore leaves the time to a usable native window
unmeasured. This run predates staged setup and the prebuilt embedding path, so
it provides a historical baseline for local generation.

## Schema 42 to 43 upgrade on November 12, 2026

A disposable full-corpus workspace with BGE Base already available took
**9.833 seconds** to upgrade in the
[PR #322 rehearsal](https://github.com/e-kosten/pf2e-atlas/pull/322).
The upgrade produced a 706-byte saved-work recovery archive. The record count
(29,894), trait membership count (244,543), and BGE Base profile were the same
before and after. A separate comparison found identical vector bytes across
157 chunks.

The recorded CLI SHA-256 was
`334b506eeb54636bb5eae52408756afaedb2a4140fe8626e4cc3644bddaeea42`.
The rehearsal measured the upgrade of an existing workspace with reference data
and embeddings in place.
