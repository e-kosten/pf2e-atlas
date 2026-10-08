# Persisted nullable collection entries

Date: 2026-10-07

## Source and implementation boundary

This slice starts from merged PR38, integration commit
`66fc9d05fd0cd0b2a45ce37730309cbad02f0823`. Strict extraction of PF2e 6.12.4
at the previously authenticated pin
`4cbdaa37d6c33e9519561bae2c59a23e0288cbce` reproduces source digest
`6bf64da835272af22c53db729cb28d18a87e8dee999c11b154fb3e772befdeb8`,
1,509 source files and 3,085 graph nodes with 47 roots. TypeScript is 5.9.3;
the source export has no Git identity of its own.

The generator supports nullable array/fixed-tuple elements and string-indexed
values through inline `Option<T>`. Null remains a slot or keyed value; present
non-null entries still use the declared value parser. Null-only entries use
`()`, a null-only parser and serde's null serialization. Ordinary object fields
retain SourcePresence before defaults. Input declarations, serialization
provenance, the production snapshots and production generated Rust are unchanged.
The new generated outputs committed here are synthetic regression fixtures.

The [ECMAScript array serialization algorithm](https://tc39.es/ecma262/multipage/structured-data.html#sec-serializejsonarray)
writes undefined or sparse array positions as null. Its
[object serialization algorithm](https://tc39.es/ecma262/multipage/structured-data.html#sec-serializejsonobject)
omits undefined properties. Array/fixed-tuple element unions containing undefined
therefore admit persisted null; object index unions containing undefined still
require explicit null to admit a present null value. There is no persisted
undefined variant, reconstructed hole, scalar coercion or per-record repair.
Undefined-only index values still stop generation because no persisted value arm
remains. Optional/rest tuples, nullable value roots and recursive collection
aliases without a nominal anchor remain explicitly unsupported.

Examples of the Rust value shapes are `Vec<Option<Number>>`,
`(Option<String>, Number)` and `SourceMap<Option<Number>>`. A declaration of
`Record<string, number | undefined>` retains `SourceMap<Number>`: keys may be
absent, but a present null is rejected. See
[ADR 0035](../architecture/decisions/0035-source-value-generation-policy.md).

## Shared-shape traversal

After passing the old undefined-array blocker, Actor/Item generation exposed
unbounded structural signature expansion. Repeated descendants embedded their
whole signature strings in each parent, and recursive anchor validation repeated
the expansion. Both roots reached JavaScript's string-size limit rather than a
useful shape diagnostic.

Signatures now intern child shapes and memoize resolved nodes within a generation
attempt. Recursive owners retain declaration identity; each anchor's reachable
shapes are validated without recursively expanding anchor signatures. These
internal shape IDs do not appear in saved inputs or generated Rust. Existing
sharing, names and checked output remain stable. A forty-level repeated-child
fixture verifies bounded generation and deterministic output; unsupported
descendants still fail, including those beneath a nominal recursive anchor.

## Verification and next blocker

- Full Rust fmt, both Clippy gates, workspace tests and build pass.
- TypeScript typecheck and 81 tests pass under Node 22.23.3, including whole-output
  freshness. Compiled Rust fixtures cover null positions, nested collections,
  null-only entries, large integers, ordered maps, duplicate rejection, precise
  diagnostics, cross-module sharing and nullable tuple/array union guards.
  Independent raw-to-typed fidelity traversal verifies selected fixture values.
- All 45 previously emitted roots still emit and compile individually against
  the actual Rust source primitives: all 42 rule roots plus JournalEntry, Macro
  and RollTable. Actor/Item remain blocked; their first blocker is now
  `Record<number, ...>` in `BackgroundSystemSource.boosts`, which Actor reaches
  through embedded Items. Numeric index-key semantics are the next capability.
  First blockers are not an exhaustive list of remaining gaps.
- The combined rule comparison repeats all 31,174 occurrences on packet digest
  `f539f02f13b0802bcc0adbbd5a76d670c1386e1f341c1b3b6eba3ecb12cb4ab3`.
  Authored acceptance remains 31,167, with zero fidelity failures, regressions or
  unmodeled keys. Counts and all seven remaining failure reports exactly match
  PR38; exit 1 is intentional. Their existing dispositions stay unchanged.

This establishes the collection capability and regression evidence, not complete
Actor/Item generation, whole-family corpus admission, Foundry runtime admission
or product adoption. Production ingest, database layout, metrics and UI are
unchanged. No matching Foundry runtime was executed.

## Reproduction

Use the [source tooling instructions](../../scripts/source-contracts/README.md)
for package verification, strict extraction and the documented `compare-rules`
command. Retry the isolated per-root harness described in the
[template report](./template-source-generation.md#reproduce) using this fresh
graph. Run `scripts/verify.sh` for the Rust workspace gate.

Local diagnostic artifacts live under ignored `scratch/source-collections`:
`source-extraction`, `portfolio.json`, `compilation.json`, per-root generated
outputs/compile logs, `rule-comparison` and verification logs. The temporary
portfolio harnesses are research artifacts, not maintained commands or release
dependencies.
