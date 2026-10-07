# Authored rule inputs and corpus comparison

Status: accepted
Date: 2026-10-07

## Decision

Keep compiler-extracted schema source shapes separate from authored JSON inputs.
`SourceFromSchema<ReturnType<Rule.defineSchema>>` can describe a collection after
Foundry cleaning even though packs author a scalar. Extraction records the exact
ordinary/strict array field class and declaration location beside each rule root,
without evaluating upstream schema constructors or changing the cleaned graph.

The bounded authored projection in `scripts/source-contracts/rule-inputs.ts`
widens ordinary string-array `selector`/`selectors` fields, and IWR `type` fields
on Immunity, Resistance and Weakness, to string-or-array unions. StrictArrayField
remains an array. Missing provenance or changed selected element shapes stop the
projection. Preserve scalar versus array, missing versus null, numeric values and
additional members; later normalization has a separate owner. This is a source
representation decision, not blanket support for every ArrayField coercion or
Foundry admission validation.

The private TypeScript `compare-rules` command samples every root/embedded Item's
direct rules from the selected packs. It compiles schema and authored portfolios
against the real Rust source primitives in scratch outputs and compares typed
serialization with ordered raw SourceValue in Rust. The independent fidelity
walker follows graph shapes; it does not duplicate enum validation, cleaning,
defaults, rule execution or union admission. Optional trait policies must come
from a manifest identifying the same source bytes and are reported explicitly.

All observed rule occurrences remain counted. Unmodeled keys, parser rejections,
value loss and acceptance regressions make the comparison exit nonzero after
writing its report. Rejections start unresolved; research may classify them using
upstream implementation evidence. A declaration mismatch alone cannot establish
an upstream data error. Executing a pure upstream validator establishes only that
validator's result, not complete Foundry document/rule admission. A missing
Foundry runtime must remain visible as `runtimeAdmission: not-executed`.

## Consequences

This developer comparison provides repeatable evidence without adding a field
owner ledger, receipt gate, product CLI command, stored schema or Node dependency
to Rust runtime consumers. Generated whole-rule portfolios stay in scratch;
production source slices, build ingest, storage, metrics and UI remain unchanged.
Before pipeline adoption, resolve unexplained rejection of demonstrated valid
inputs and unexplained value loss; any confirmed upstream error exception needs
specific evidence and remains included in occurrence totals.

The [authored rule comparison](../../research/authored-rule-source.md) records
the pinned corpus, evidence boundaries and outstanding modeling work.
