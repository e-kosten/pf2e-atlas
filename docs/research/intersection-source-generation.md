# Field names and indexed intersections

Date: 2026-10-07

## Implementation and input identity

This slice starts from merged PR34, integration commit
d015e5287fb3c179dd04dd53879674e3b05578bd. Rust fields now support `_id`
and map keys such as `greater-darkvision` to `greater_darkvision`, preserving
the source key in parsers and serialization. Reserved path words and digit-leading
keys receive deterministic mappings. Colliding names fail generation.

TypeScript extraction now saves the compiler-resolved index signatures on
intersections. This includes effective value intersections and narrowed unions;
the emitter uses these results directly alongside resolved named fields. Named
indexed intersections share the existing struct representation; pure maps remain
ordered SourceMap values. No manually reconstructed intersection algebra or
arbitrary-value fallback is introduced. See
[ADR 0037](../architecture/decisions/0037-open-and-indexed-source-values.md) and
[ADR 0035](../architecture/decisions/0035-source-value-generation-policy.md).

A fresh strict extraction of PF2e 6.12.4 at
4cbdaa37d6c33e9519561bae2c59a23e0288cbce succeeds using TypeScript 5.9.3.
The exported inputs have source digest
6bf64da835272af22c53db729cb28d18a87e8dee999c11b154fb3e772befdeb8 and
1,509 source files. Both extraction products remain complete; the declaration
graph has 3,085 nodes and 47 roots. The archive has no Git identity itself;
the digest matches the previously authenticated pinned export.

Committed selection remains 791 nodes across six snapshots. Four saved
intersection nodes gain index metadata; production generated Rust remains
unchanged at 1,754 lines across nine files. Runtime ingest, storage, metrics
and product adoption remain deferred.

## Verification and remaining gaps

Package verification passes 65 tests on Node 22.23.3. The compiled generic Rust
fixture passes 22 tests, covering source-name serialization and diagnostics,
collision rejection, constrained named intersections, pure maps and recursive
indexed layouts. Compiler fixtures verify narrowed and merged index values.
The standard Rust workspace fmt, clippy, test and build gates pass.
Fresh extraction reproduces the committed selected snapshots and Rust output.
The selected Item fidelity probe accepts and compares all 98,651 occurrences
with zero value differences.

Full-root emission improves from 41 to **45 of 47**. All 45 outputs compile
individually against the actual source primitives. These include all 42 rule
roots plus JournalEntry, Macro and RollTable. They remain diagnostic outputs;
this does not establish combined ownership or corpus fidelity for every root.

ActorSourcePF2e and ItemSourcePF2e pass the indexed flags and reach the same
first blocker: an array element union containing undefined in a DeepPartial
materials field. TypeScript undefined array entries can become null during JSON
serialization, so ordinary normalization must not silently erase that fact.
The existing generator rejects this collection shape. These are first blockers,
not proof that no other shapes remain behind them.

The newly compilable BattleForm parser attempts all 115 pinned occurrences:
47 parse and serialize; 68 reject. First rejection categories are 55 scalar
values where arrays are declared, eight closed-vocabulary values, one scalar
where an object is declared and four union-identity failures. The probe exits 1
and retains every diagnostic. This is acceptance/serialization evidence, not
a complete independent value-fidelity comparison or a judgment that Foundry
rejects all these sources. The authored/cleaned-source discrepancies described
in the [template report](./template-source-generation.md) still need investigation.

## Follow-up and reproduction

Next define persisted null/undefined collection semantics, then repeat all-root
generation and compilation. Compare authored rule source interfaces and Foundry
cleaning with SourceFromSchema before adopting full-family parsers. Keep source
fidelity, Foundry admission and product interpretation as separate evidence.

Use the [source tooling instructions](../../dev-tools/source-contracts/README.md)
for strict extraction and selected snapshot generation. Repeat the isolated
per-root Cargo harness described in the [template report](./template-source-generation.md#reproduce),
using this refreshed graph. Durable local artifacts are under ignored
scratch/intersection-source-generation: source-extraction, portfolio.json,
compilation.json, item-corpus.json, per-root compile.log and BattleForm/corpus.json.
The temporary harness scripts remain there; they are not a CLI or release dependency.
