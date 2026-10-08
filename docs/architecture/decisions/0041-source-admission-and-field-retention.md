# ADR 0041: Source admission and field retention

Status: Accepted

## Context

Strict authored-source parsing rejects useful documents when one field has an
unsupported representation. The pinned corpus includes quoted numeric fields,
old vocabulary identifiers, malformed preparation collections and incomplete
heightening blocks. Dropping a whole embedded spellcasting entry can disconnect
otherwise valid spells from their owner. A parser rejection is not evidence that
the entire record is unusable or that Foundry would reject it after cleaning.

## Decision

`atlas-ingest::source_model` owns two explicit callable boundaries:

- `parse_*` validates the modeled authored representation strictly. Discovery
  comparisons continue to count unsupported values as rejections.
- `admit_*` retains the original ordered `SourceValue`, a typed model when its
  root is supported, and contextual diagnostics. It applies no defaults,
  numeric/boolean coercion, migration, guessed vocabulary mapping or named-record
  exceptions. Production normalization and storage adoption remain separate.

`SourcePresence::Invalid` makes an authored but unparseable field distinct from
missing, null and valid values. It holds the original member values and a
diagnostic. `as_value()` returns no typed value for that state. Duplicate modeled
members retain all their values rather than selecting a winner. Unknown members
continue to retain their authored order and duplicate names.

Admission uses the nearest generated field boundary identified by strict
parsing, then retries the same parser with that field explicitly unavailable.
It never edits JSON, renumbers arrays, deletes embedded documents or weakens
union identity. Object children can retain their own invalid fields. A malformed
scalar collection element makes the containing collection field unavailable as
a whole; valid neighbors remain raw rather than becoming a shortened typed
collection. Every retry must mark a new field, so finite input bounds recovery.

Unsupported or ambiguous document roots retain raw source with no typed model.
Invalid JSON and non-object document envelopes remain errors. Future loaders
must report those errors and choose their own quarantine policy explicitly.

Specific rule admission is atomic: if strict rule parsing fails, the whole rule
stays raw with diagnostics and no typed model. Removing a bad predicate term can
change its meaning. Item's generic rule payload remains separate from specific
rule interpretation, as in ADR 0039. Admitting a parent is not a claim that all
its specific rules can be interpreted.

Upstream constructor-input evidence is resolved with the TypeScript compiler
and kept beside declaration evidence. NPC senses use the input of `Sense`, which
accepts omitted acuity/range and nullable range. The model retains those authored
states without applying Foundry's schema defaults. Constructor signature and
projection drift fail visibly; the real Foundry cleaner is still unexecuted.

## Consequences

Records can remain useful for identity, prose and supported mechanical fields
without asserting knowledge of unavailable data. Normalization must explicitly
handle invalid states rather than substituting zero, false, empty collections or
missing-field defaults. A collection retained raw is not available to typed
consumers even if some elements appear usable.

`compare-portfolio --admission` measures retained, modeled, diagnostic-free,
partial and raw-only occurrences separately, with raw retention and typed/invalid
field fidelity checks. Strict corpus comparison remains a separate mode.
Neither mode proves Foundry admission or gameplay completeness. No database,
metrics, normalized record or UI contract changes follow automatically from
introducing these source APIs.
