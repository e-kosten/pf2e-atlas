# Development build footprint

Measured on 2026-10-12 at source revision
`70ca64bb09b0232abf4855dd4c483d78fdf89e9e` (PR52 integration merge).

## Selected defaults

Development and test profiles use `debug = "limited"` and `incremental = false`.
This reduced a clean workspace test/build target from 10.48 GiB to 6.46 GiB,
about 38%, without changing features, test selection, assertions, overflow
checks, optimization or release settings. Tests inherit development settings.

Limited debug preserves file/line backtraces and qualified function names. Full
local-variable/type debugging is available through `--profile debug-full`.
Incremental compilation remains a session opt-in for sustained iteration.
See [contributor workflow](../../CONTRIBUTING.md#build-profiles-and-disk-budget).
[Cargo documents these profile settings](https://doc.rust-lang.org/cargo/reference/profiles.html).

## Comparable clean builds

All seven configurations passed the same 507 tests with 20 existing ignores,
followed by a workspace build. Each started with an empty checkout-local target.
The registry and native dependency downloads were already cached.

| Debug information | Incremental | Allocated target GiB | Test command seconds | Subsequent build seconds |
| --- | --- | ---: | ---: | ---: |
| Full (normal Cargo defaults) | On | 10.48 | 93.3 | 9.4 |
| Full | Off | 7.60 | 106.1 | 8.6 |
| Limited | Off | **6.46** | 81.2 | 6.8 |
| Line tables only | Off | 6.44 | 85.4 | 7.3 |
| Limited | On | 8.71 | 91.3 | 8.6 |
| Line tables only | On | 8.69 | 87.7 | 8.4 |
| Limited | On except generated Foundry model | 7.94 | 85.8 | 8.7 |

These are single samples, not a speed guarantee. In particular the variation
between full builds is not evidence that incremental compilation slows every
clean build. A representative application edit rebuilt in 4.1 seconds without
incremental state and 2.1 seconds with it at line-table debug settings. That
tradeoff supports an explicit opt-in rather than an unconditional disk cost.

Limited costs only about 24 MiB more than line tables in the clean comparison
and retains qualified function names for debugger lookup. Static LLDB inspection
and a deliberate panic verified source backtraces and symbol/type differences:
full retained a local struct's type information; limited and line tables did not.
The configured full-debug profile also passed the local-state tests. This was
static debugger inspection, not a live variable-inspection session.

## What made the retained cache large

The merged mutable-workflows checkout held 25.60 GiB of allocated compiler
output, despite an empty incremental directory. It contained multiple library
and test hashes and roughly 54,000 loose object files. Approximate grouped sizes:

| File role | Allocated GiB |
| --- | ---: |
| Object files, including unpacked debug information | 10.53 |
| Executables | 6.93 |
| Rust library archives | 6.51 |
| Compiler metadata | 1.28 |
| Native/build-script output | 0.16 |
| Dynamic libraries | 0.16 |

Retained hashes prove that multiple outputs accumulated. The inventory cannot
assign every hash to a particular feature, profile, compiler flag or source edit.
The 25.60 GiB cache is not the clean-build baseline used above.

Generated Foundry types are about 2 MB of source across 97 files; the JSON corpus
is not embedded in that crate. Serde implementations, linked code, debug data
and compiler metadata are separate contributions. One matching full-debug
Foundry archive was about 398 MiB, versus 321 MiB with limited debug. Its full
object DWARF was about 170 MiB, compiler metadata 47 MiB and text sections 38 MiB.
The object inspection tool labels the metadata pseudo-section as DWARF too;
adding it again would double-count it. Archive/container overhead remains
unattributed.

## Alternatives evaluated

The smallest CLI integration harnesses spawn the common application executable
and are only about 1–2 MiB each. Combining them would save MiB rather than GiB.
Large index/search/runtime tests live under distinct owners and require their
actual dependencies. This investigation found no justification for reducing
coverage or crossing crate boundaries to combine them.

Native inference is compiled into many consumers, including index's catalog-only
use of atlas-embedding. A feature boundary might improve focused storage builds,
but the complete workspace still enables inference through ingest/search.
Its full-gate benefit is unmeasured; no dependency split was introduced.

A bounded compiler cache could improve reconstruction after retiring checkouts,
but adds storage and does not cache linked Rust executables. This repository's
frontend asset embedding also needs correctness checks before cache reuse.
No compiler cache was installed, and Cargo targets remain checkout-local.

## Implementation validation

The full configured Rust gate passed formatting, both Clippy gates, the same
507 tests with 20 existing ignores, and the workspace build. With Clippy output
included, its target occupied 6.81 GiB; this is separate from the clean
test/build comparison above. The full-debug profile passed 37 local-state tests,
plus the temporary backtrace and static type-inspection probe.

All 46 real-Git hook fixture cases passed with fake Cargo/npm commands, and shell
syntax and ShellCheck passed. The complete frontend verification also passed
407 tests and its format/lint/type/style/build checks. No UI source changed.
Hosted CI is a separate validation surface.

## Reproduction and limits

- Hardware: Apple M4 Max, 16 CPUs, 64 GiB RAM; macOS 26.5.2 arm64.
- Actual compiler: Homebrew Cargo/rustc 1.98.1, LLVM 22.1.8. The repository pins
  1.95.0, but Homebrew's compiler was selected on PATH rather than a rustup
  proxy. These are not 1.95 measurements.
- Cargo.lock SHA256:
  `f7f2edbfb3bc23385df1120bc5d1bafbb328c4657475a4842050a3c434500383`.
- UI lock SHA256:
  `e6ae671853f66ae321e21589a4270312f04e33c4d974385a9dbb9dd33ec1b191`.
- The frontend bundle was copied byte-for-byte from the reviewed checkout with
  the same source tree. No frontend package or source changed.
- Commands: `cargo test --workspace --locked --offline`, then
  `cargo build --workspace --locked --offline`. Compare explicit matching dev/test
  `debug` and `incremental` settings through Cargo's `--config` option.
- Clean between configurations; run compilation serially in one local target.
  Restore any sampled source edit. Retire build outputs after recording evidence.
- Disk measurements sum `st_blocks * 512`, once per inode, and report GiB.
  APFS sharing and other machine activity can make changes in device free space
  differ from this sum. Logical sizes alone overstate some retained outputs.

The small raw logs, inventories and temporary measurement scripts are retained
locally under `scratch/build-footprint` for review. They are research evidence,
not an additional maintained toolchain. Current operational guidance belongs in
CONTRIBUTING.md and AGENTS.md.
