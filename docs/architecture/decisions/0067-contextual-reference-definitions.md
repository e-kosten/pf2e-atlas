# ADR0067: Source-backed contextual reference definitions

Status: accepted; amends the trait-only vocabulary/storage scope of
[ADR0056](0056-trait-reference-entities.md). Its specialized trait filtering and
semantic-seed policies remain active.
Date: 2026-11-05

Amended 2026-11-06: existing glossary records use ordinary record presentation.
The special template-reading selection and per-description approval-hash gate
were removed after product review. Setup also consumes the bundled supplement
directly. Source attribution and input integrity remain.

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

Generated source-contract evidence owns finite catalog coverage and exact mappings.
Ingest verifies mappings against
the actual captured package, declared packs and consumed records. A compatible
same-key custom description remains that selected record's content unless an
explicit required structural signature conflicts. Names and translated labels
never establish identity or activate a mapping for an unrelated package.
The preparation retains the captured root package and only explicit pack-system
declarations. An omitted pack declaration inherits the root; an explicit matching
system permits the new association, while conflicting or unsupported declarations
disable it. Unknown historical scope stays unknown. Record mappings, new non-trait
localization associations use this retained evidence;
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

Glossary records use their ordinary prepared content, including authored
placeholders, values and controls. Golem Antimagic and Wind-Up point to their
existing records; Generic Sneak Attack remains a standalone supplied definition.

## Content acquisition and attribution

The approved supplemental asset contains only selected language speaker context
and published specialization prose, exact mappings, provenance and notices. Runtime
selects the bundled asset directly for normal setup and check mode. Ingest captures
the selected bytes, notice bytes, locale and recipe identities once. Bundled and
explicit directory selections of identical bytes retain the same input identity.
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
PDF alone is not a requirement for new paragraph-specific permission. Publication
metadata is associated with the selected definition identity. Changes in wording
do not independently withdraw that association or suppress its prose. Source-pin
updates retain the normal source, attribution and generated-contract review.

## Consumers

Index owns bounded source-free entry, notice and occurrence reads. Runtime owns
acquisition, freshness and migrations. Search and the existing query owner consume
published evidence; app-service owns semantic labels, help destinations and related
filter actions. CLI and browser render generated product contracts. Contextual
reading uses the shared Drawer, and discoverability follows existing Advanced
Filters. Extending those product surfaces does not change these content owners.


Contextual retrieval uses the existing bounded workspace/search service. Metadata
batches select at most 100 identities and avoid prepared-body deserialization;
they still read the canonical envelope bytes to verify their checksum. Full detail
loads the standalone prepared body and required notices, or returns the existing
record destination. Exact-owner outgoing evidence and complete SQL target
uniqueness support owned-ability help without guessing from names or capped pages.

App-service projects categorical values as raw identity, readable label and optional
typed help. Record destinations retain `RecordNavigationView` and ordinary authored
content, with their existing control and reference addresses.
Standalone definition destinations remain typed definition keys. Related actions
carry the executable predicate, or a numeric field configuration that still needs
an operator/value. Older traits-only coverage preserves ordinary record reads and
reports setup-required help for new namespaces.

Exact reference conditions and evidence retain their typed record/definition
identities even when no body is readable. Their optional reading destinations are
projected from canonical backing metadata, separately from query admission. The
same capability owner decorates fact links, clause summaries, proof endpoints and
outgoing references. Batched metadata reads preserve inert evidence titles without
turning label-only or withheld content into a definition page. Prepared check and
routine badges receive the same metadata-backed help as record presentation badges.

`DefinitionReference` is an incoming query leaf in the existing Boolean eligibility
engine. A record destination unions visible canonical localization occurrences
with existing UUID graph evidence; standalone keys use canonical occurrences.
Definition keys backed by records retain their semantic identity while resolving
to the checked record destination. Missing or inapplicable targets fail admission
even under negation. Existing record-link predicates and symmetric specialized
trait-reference workflows retain their meanings. Generic trait help supplies the
exact membership predicate; the legacy symmetric reference binding stays with the
existing trait References/Similar workflow. Unknown binding metadata remains an
error rather than silently dropping a capability. Artifact-scoped definition-option
discovery includes eligible zero-match and label-only values plus admitted existing
glossary records; it does not change corpus/list value-vocabulary semantics or add
a glossary browser.

### Contextual browser reading

The existing linked-reference session owns one mixed record/definition Drawer and
its bounded local history. An initiating property help value remains separate
from the actual reading destination, so properties sharing a record keep distinct
related-filter actions. Exact owner returns retain their ordinary source address.

The application host supplies a fallback session for links outside an existing
reader. Selected-record readers and contextual encounter readers retain their
source-scoped owner and actions. Both ordinary record links and vocabulary help
delegate through that host's handlers, so nested reads keep one active history
and the correct owner-return affordance. An application fallback must not replace
a selected reader's owner context or action renderer.

Record-backed help retains the actual record URL and an optional semantic
definition key. Full-page reading resolves that key through
the existing definition-detail service and uses its property actions only when
the returned actual destination matches the requested record selection. Standalone
definitions with readable content have direct typed destinations; metadata-only
entries remain filterable without a reading destination. Traits keep their URLs and
specialized membership, reference and Similar workflows. These routes add no
glossary browser or duplicate record page. Native modified links and the Drawer's
Open full page action retain the same checked destination and reading purpose.

Prepared backing alone does not establish readability. Canonical content must
contain readable text, a visible interaction or a safe rendered image. Empty HTML
shells retain their source, identities, associations and stored seed data while
product reading projections withhold an empty destination/body. Single and batched
metadata projections use the same content-presence policy.

Glossary Drawers show the useful definition body and ordinary publication
attribution. Technical extraction provenance stays in its canonical diagnostic/data
owner; it is not reading content. Related-record exploration belongs on the
standalone page using the existing trait association presentation. Existing trait
association and Similar policies are unchanged; other definitions gain no automatic
Similar capability.

Related actions consume the Rust-supplied canonical predicate. Standalone
association sections and manual discovery share that predicate and the existing
query owner. Numeric configuration stays in Advanced Filters and becomes executable
only after an operator and value are supplied. Opening, nesting or closing help
does not change the host's selected record, criteria or drafts.
