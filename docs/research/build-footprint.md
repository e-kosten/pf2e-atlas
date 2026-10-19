# Development build footprint

Measured on 2026-10-12 at source revision
`70ca64bb09b0232abf4855dd4c483d78fdf89e9e` (PR52 integration merge).

## Selected defaults

Development and test profiles use `debug = "limited"` and enable incremental
compilation except for the generated `atlas-foundry-model` crate. This measured
7.94 GiB for a clean workspace test/build target, versus 10.48 GiB with full
debug/incremental defaults, about 24% less. Clean CI jobs disable incremental
compilation globally; the equivalent nonincremental local comparison measured
6.46 GiB. Features, test selection, assertions, overflow checks, optimization
and release settings are unchanged. Tests inherit development settings.

Limited debug preserves file/line backtraces and qualified function names. Full
local-variable/type debugging is available through `--profile debug-full`.
Incremental application builds use the same defaults in hooks and manual checks.
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
| Limited | Off | 6.46 | 81.2 | 6.8 |
| Line tables only | Off | 6.44 | 85.4 | 7.3 |
| Limited | On | 8.71 | 91.3 | 8.6 |
| Line tables only | On | 8.69 | 87.7 | 8.4 |
| Limited | On except generated Foundry model | **7.94** | 85.8 | 8.7 |

These are single samples, not a speed guarantee. In particular the variation
between full builds is not evidence that incremental compilation slows every
clean build. A representative application edit rebuilt in 4.1 seconds without
incremental state and 2.1 seconds with it at line-table debug settings. With
limited debug, the same edit took 2.14 seconds with incremental enabled everywhere
and 2.15 seconds with it disabled only for the generated model. The selected
partial configuration preserves the sampled edit benefit and uses about 0.77 GiB
less than enabling it everywhere. Tests after the edit took 4.43 and 4.34 seconds
respectively; this does not establish a speedup for every command or edit.

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

An isolated sccache 0.18.0 experiment at revision `6696d535`, using nonincremental
profiles, found real cache reuse but no sampled test/build speedup: 91.5 seconds
without the wrapper, 100.2 seconds in a second checkout and 94.6 seconds rebuilding
the first checkout with all eligible compilations hitting the cache. Two targets
plus cache occupied 12.82–13.15 GiB, versus an estimated 12.93 GiB without it.
Checkout-specific fixture and cached-library frontend HTML checks passed. Single
samples do not establish performance on other workloads. The private cache,
daemon, download and temporary checkouts were removed; no compiler cache was
adopted, and Cargo targets remain checkout-local.

Retain active targets and remove retired worktrees rather than repeatedly cleaning
and rebuilding active output. Incremental state adds writes, but may avoid other
recompilation work. Neither experiment measured physical device writes, APFS write
amplification or SSD endurance; no write-volume benefit is claimed.

## Implementation validation

The earlier fully nonincremental Rust gate passed formatting, both Clippy gates,
507 tests with 20 existing ignores, and the workspace build. With Clippy output
included, its target occupied 6.81 GiB; this is separate from the clean
test/build comparison above. The full-debug comparison passed 37 local-state
tests, plus the temporary backtrace and static type-inspection probe.

The selected partial-incremental configuration passed the full Rust gate with
the same 507 passing tests and 20 ignores. Compiler invocations confirmed the
exception in both library and test builds. Including both Clippy gates, this
target occupied 9.04 GiB, distinct from the 7.94 GiB test/build comparison.
Actionlint passed the workflow containing the CI override. Active task output
is retained for PR review rather than repeatedly cleaned and rebuilt.

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
