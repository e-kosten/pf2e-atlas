# ADR0067: Source-backed contextual reference definitions

Status: accepted; amends the trait-only vocabulary/storage scope of
[ADR0056](0056-trait-reference-entities.md). Its specialized trait filtering and
semantic-seed policies remain active.
Date: 2026-11-05

## Context

Reusable Foundry definitions appear both as real compendium documents and as
localized catalog values. Senses, languages, domains and equipment groups need
the same contextual reading and related-record workflows as traits. Duplicating
existing action or ability documents into a glossary would create competing
content, identity and attribution owners.

## Decision

Keep existing records under their actual `pack:id` identity and prepared-content
owner. A vocabulary property can have a checked pointer to such a record, but
that pointer stores neither another body nor another vector. The record remains
the reading destination. No glossary browser or additional ordinary Search
candidate family is implied by this model.

Generalize the existing trait definition owner in place to `reference_entries`.
`DefinitionKey` admits exact typed namespace/token identities and
`rule:<source-package>:<exact-localization-key>` identities. The complete rule-key
tail is retained. Existing `TraitKey` strings, specialized trait APIs and filter
meanings remain unchanged. They are views and inputs over the canonical owner;
there is no parallel trait definition store or legacy runtime reader.

An entry retains its selected label, finite availability reason, source evidence,
locale/fallback, publications, permissions and filter bindings. Its backing is
prepared standalone content, an existing record pointer, or no supplied body.
Prepared bodies use one record-owned `PreparedReferenceDefinition` representation
and the existing content interpreter. A missing or conflicting record mapping
retains its intended target as evidence, without offering a pointer or inventing
a standalone body. Availability is independent of identity and filter metadata.

Generated source-contract evidence owns finite catalog coverage, exact mappings
and the reviewed generic-template signatures. Ingest verifies mappings against
the actual captured package, declared packs and consumed records. A compatible
same-key custom description remains that selected record's content unless an
explicit required structural signature conflicts. Names and translated labels
never establish identity or activate a mapping for an unrelated package.
The preparation retains the captured root package and only explicit pack-system
declarations. An omitted pack declaration inherits the root; an explicit matching
system permits the new association, while conflicting or unsupported declarations
disable it. Unknown historical scope stays unknown. Record mappings, new non-trait
localization associations and template selections use this retained evidence;
annotation metadata cannot assert its own applicability. Ordinary records and
legacy trait behavior remain available for conflicting packs.

## Exact reference evidence

`definition_reference_occurrences` replaces the trait-reference storage owner.
Both endpoints are checked unions of real record addresses and definition keys.
This represents record-to-definition, record-to-record localization mappings,
definition-to-record and definition-to-definition evidence without fake records.
Owners, fields, authored occurrence paths and audience visibility remain explicit.

The record-owned authored parser collects exact localization references. Expanded
prose is not new authored evidence. A reference normalizes to a real record only
through a verified mapping. Missing, wrong-family, conflicting and inapplicable
record mappings do not become synthetic definition targets. Distinct properties
can share a record destination without merging their property identities;
ambiguous standalone targets remain ambiguous.

One canonical publication path reconciles these occurrences. Trait APIs project
their existing visible record/trait subset. Record graph edges retain their
existing owner and are not copied into this table. Reference membership, an
explicit mention, and a saved encounter condition remain different facts.

## Publication and upgrade

Definitions, notices and their evidence publish atomically with the source
generation through [ADR0065](0065-input-owned-reference-publication.md)'s owned
publication. Compact selected-key count/digest evidence binds complete glossary
coverage. Ordinary publication checks that inventory while writing the selected
entries; it does not add a whole-database audit or rich-frame replay. Explicit
deep validation checks stored inventory, body/notice hashes, mappings and exact
occurrences against retained checked source and the shared authored parser.

Runtime's ordered migration moves retained trait definitions into the canonical
owner offline. Trait memberships, prepared content, exact identities, native
values, centroids, ordered seed evidence and saved state survive. Historical
migration SQL and callbacks retain their released meanings. Old names remain
only in those historical owners and their authentic fixtures.

Physical schema version does not establish new glossary coverage. Missing
`reference_preparation` means retained traits-only coverage, preserving existing
record and trait reads. Explicit setup prepares the additional definitions and
records their separate recipe, combined input identity and selected-key inventory.
Startup does not fetch sources or run setup. Retained traits-only evidence
requires the legacy visible incoming and outgoing associations. Hidden authored
occurrences remain intact in prepared content but do not newly require association
rows or destinations. Any stored hidden association must still match checked
source and a valid destination. Complete preparation attests all visible and
hidden evidence; its audit remains exact. New namespaces gain no automatic
vectors; existing record and trait numerical policies retain their owners.

Named SQL projections retain authored sense acuity, range and source states, and
armor group. Offline backfill uses compatible retained snapshots and preserves
missing, null and invalid values. A retired preparation remains deferred to its
existing explicit setup path rather than being decoded by the migration.

Ten verified templates (eight general actions and two family abilities) retain finite role-selection metadata
beside their existing canonical prepared field. Whole description/rules signatures,
source package, authored spans, unique parser paths and checked HTML/control
bindings establish the selection. Owned or customized abilities retain ordinary
content and meaningful zeroes. This metadata introduces no second body, markup
interpreter, generic UI layout language or automatic rules execution. Golem
Antimagic and Wind-Up reuse their existing record bodies through this mechanism;
Generic Sneak Attack remains a standalone supplied definition.

## Content acquisition and attribution

The approved supplemental asset contains only selected language speaker context
and published specialization prose, exact mappings, provenance and notices. Runtime
places the bundled normalized asset in its verified content-addressed cache;
normal setup uses the approved selection and never polls upstream HEAD. Ingest
captures the selected bytes, notice bytes, locale and recipe identities once.
Notice-only changes and explicit absence/disable participate in setup freshness.
Changed or missing selected inputs fail the attempted update without publishing
partial content or silently retaining an obsolete provider.

Software licenses and game-content grants remain separate. Permission metadata
distinguishes body grants from identifying-label grants. Each grant states whether
it covers functional content or limited descriptive references. ORC/OGL functional
material and CUP descriptive names can require accumulated bases; those are not
alternative licenses over the entire field. Required notices are deduplicated
content assets available without the source checkout. Publication components
remain independently attributable within the same database.

Existing trait and record policy does not acquire invented license details through
this migration. Unknown publication facts remain unknown; the generalization
does not retroactively clear all existing prose or translations. New selected
material carries its approved source-specific notices and provenance. A published
upstream content notice is affirmative source evidence; a missing original-book
PDF alone is not a requirement for new paragraph-specific permission.

## Consumers

Index owns bounded source-free entry, notice and occurrence reads. Runtime owns
acquisition, freshness and migrations. Search and the existing query owner consume
published evidence; app-service owns semantic labels, help destinations and related
filter actions. CLI and browser render generated product contracts. Contextual
reading uses the shared Drawer, and discoverability follows existing Advanced
Filters. Extending those product surfaces does not change these content owners.
