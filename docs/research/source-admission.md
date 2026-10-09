# Retained source admission

The maintained `atlas_foundry_model::admit_*` APIs retain original ordered
source values and typed usable fields with diagnostics. Strict `parse_*` APIs
remain available and keep their rejection boundary. See
[ADR 0041](../architecture/decisions/0041-source-admission-and-field-retention.md).

## Measured corpus

PF2e 6.12.4 at `4cbdaa37d6c33e9519561bae2c59a23e0288cbce`, TypeScript 5.9.3.
Source digest: `6bf64da835272af22c53db729cb28d18a87e8dee999c11b154fb3e772befdeb8`.
Combined packet digest:
`b11d1bae6151272929ffc99380c7c6cd743c4a2ed440b2005b69dafa485520b7`.

| Scope | Occurrences | Retained | Typed models | Partial models | Raw-only | Rejected | Fidelity failures |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Root documents | 25,641 | 25,641 | 25,641 | 203 | 0 | 0 | 0 |
| All document and rule occurrences | 136,832 | 136,832 | 136,825 | 282 | 7 | 0 | 0 |

The overlapping document/rule counts include embedded and nested Items sampled
independently from their parents. They are not distinct lost records. Root
documents have 267 diagnostics; all occurrences have 400. A retained invalid
collection gets a diagnostic for its first failing member and remains entirely
unavailable as a typed collection; these counts do not enumerate every bad leaf.
The seven raw-only occurrences are specific rules, not dropped documents.
Generic rule payload preservation does not certify specific rule interpretation.

The independent fidelity oracle checks both usable typed values and invalid-field
raw values, including numeric value preservation, collection positions, duplicate
members and unknown-field ordering. Complete raw roots are also compared with the
original ordered source representation. These checks do not establish Foundry
cleaning, runtime admission or mechanical completeness.

## Representation and handling

For an ancestry feature with `level: "1"`, its UUID remains typed while its level
is `SourcePresence::Invalid`, retaining the string and diagnostic. Desna's old
alternate domain identifier similarly leaves the alternate-domain array raw.
Neither representation is guessed into the current model.

Somnalu's preparation strings make its prepared-slot collection unavailable, but
the spellcasting entry stays present with its identity and usable fields. All
prepared entries stay in the raw collection; none are removed or renumbered.
Spells keep their original location links. Unsupported heightening and boolean
strings likewise remain invalid fields instead of being assigned new semantics.

Specific rule admission uses the strict parser atomically. For example, the
predicate combining `nor` and `not` stays raw and has no specific typed rule
model. Admission does not delete one operator to manufacture a usable condition.

## NPC sense input

The source constructor takes
[`SenseConstructorParams`](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/actor/creature/sense.ts#L111),
not the fully populated `SenseData` interface. The generator obtains its shape
through the TypeScript compiler's construct signature. NPC perception's senses
use that shape because
[`PerceptionStatistic`](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/system/statistic/perception.ts#L30)
passes those entries to `new Sense(data, ...)`.

Protosoul's bare lifesense therefore keeps its authored missing acuity/range
states. Nullable range is retained, and unknown sense identifiers still fail
strict parsing. No defaults are supplied. Compiler-derived constructor provenance
and drift checks remain distinct from executing Foundry's DataModel cleaner.

## Reproduction and adoption

Use `compare-portfolio --admission` with the small
`portfolio-admission-baseline.json` fixture to reproduce retention counts and
diagnostic outcomes. Run strict `compare-portfolio` separately with
`portfolio-corpus-baseline.json`; it continues to report unsupported input as
rejections. Both commands check fresh source-to-Rust generation first. See the
[tooling README](../../dev-tools/source-contracts/README.md#maintained-rust-corpus-comparison).

Generated types and admission APIs remain a source boundary. Production source
normalization, SQLite storage, metrics and product rendering are separate adoption
work. That work must distinguish invalid fields from authored missing values and
avoid emitting invented numeric/boolean facts or shortened collections.
