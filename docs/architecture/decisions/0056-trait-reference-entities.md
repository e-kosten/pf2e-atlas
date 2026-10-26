# ADR 0056: Source-backed trait reference entities

Status: accepted; implemented in [PR93](https://github.com/e-kosten/pf2e-atlas/pull/93).
Date: 2026-10-20

## Context

Foundry defines reusable trait identifiers, localized labels and explicit
description mappings in system catalogs rather than independent compendium
documents. Atlas needs addressable trait definitions, record membership,
attributed references and Similar navigation without inventing Foundry records.

## Decision

Traits have a typed `TraitKey`, serialized as `trait:<identifier>`, separate from
the pack/source-ID `RecordKey`. Exact identifiers remain distinct even when
Foundry maps several parameterized traits to one description key. The first
delivery publishes the trait namespace; rarity and family-specific otherTags
retain their existing namespaces and filter behavior.

The pinned source-contract generator emits verified trait identifiers, optional
label and description keys, catalog memberships and compact source locations.
It does not infer keys, definitions, aliases or trait hierarchies. Generated
metadata is source evidence; localized selected content is prepared at ingest.
Missing labels or definitions and unknown admitted identifiers remain explicit.

The generated trait catalog is the single authority for label-key lookup as well
as definition metadata. PF2e system pin updates regenerate it with the rest of the
source models; whole-output freshness checks reject stale files, and incomplete
or conflicting extraction stops generation. Upstream identifier changes still
require review rather than inferred aliases. Foundry platform version changes
alone do not define a new trait vocabulary. The private source-contract
[`compare-traits` command](../../../dev-tools/source-contracts/README.md#trait-changes-during-an-upstream-upgrade)
compares retained before/after extraction evidence for source-upgrade review; it
does not replace fresh generation, admission or corpus validation.

Generated metadata retains the upstream system attribution and Apache 2.0
license information in source and distribution notices. Localized rules prose
is loaded at ingest, not compiled into the metadata catalog. Its content licenses
remain separate from the system code license and from Atlas's own license;
Foundry's partnership permissions are not assumed to transfer to Atlas. The
[pinned description provenance review](../../research/trait-description-provenance.md)
records the source chain and the separate unresolved paragraph-level content
permission/attribution work.

Foundry-model owns generated source evidence. Domain owns trait identity.
Record owns pure content interpretation and attribution policies. Ingest resolves
the selected locale with the established English fallback, prepares selected
sanitized HTML, retains source evidence and prepares embedding inputs. Index owns
checked vocabulary publication, persistent schema, validation, bounded readers,
membership/reference projections and stored trait seed vectors. Runtime composes
state-preserving migrations through the existing workspace ledger.

Definitions and their projections must be readable without the source checkout.
No runtime surface loads a localization catalog or constructs a fake Item DTO.
Publication is atomic with the reference generation and preserves saved lists,
encounters, record identities and their retained unresolved references.

Association categories preserve different evidence:

- Membership means the selected source actually carries the exact trait.
  An embedded attack's trait does not become its parent creature's trait.
- References require explicit attributable source evidence. A label match in
  arbitrary prose does not establish a relationship. Missing authored reference
  evidence yields an empty result, not a generated association.
- Similar uses a persisted trait seed vector and the existing profile, eligible
  record candidates, ranking, coverage and paging policies. The initial candidate
  set is normal product records. Missing stored vectors are reported explicitly;
  text search and co-occurrence do not substitute for semantic similarity.

Trait seed preparation has its own versioned recipe. The authored label and
complete selected definition pass through the existing baseline passage splitter
and each selected profile's native token checks. Every selected chunk participates
in exact native-input reuse/inference. The seed is the deterministic L2-normalized
arithmetic mean of the normalized chunk vectors. Persisted input/coverage evidence
must bind the recipe and full selected text; clipping to a prefix or using one
arbitrary passage is not permitted. A definition with no authored prose uses its
label identity. This seed construction does not change normal record vectors,
candidate ranking, or the existing profile contracts.

Search owns retrieval. App-service owns term presentation and workflow composition
through runtime/search; app-model owns transient exported contracts. Web and CLI
are thin consumers. The browser reuses shared reading/navigation behavior and Ant
controls, retaining content, focus and scroll during same-identity refreshes and
checking artifact generation before association navigation.

Primary Search accepts a typed trait seed for Similar and a typed trait-reference
scope. Similar uses the same persisted seed and filtered record ranking as the
trait page. Trait-reference searches resolve the complete distinct root-record
scope from indexed reference evidence before applying normal search predicates,
ordering and paging. They never derive eligibility from a capped preview page.
A reference inside a contained source entry scopes its owning record without
promoting that entry's trait membership to the owner. Trait membership links
continue to use the normal exact root-trait predicate.

Existing record-key APIs, authored trait identifiers and trait filter predicates
keep their meaning. This is an additional source-backed reference identity, not
a second family schema, compatibility record model or generic localization store.

## Consequences

Trait pages require explicit publication and vector support; making a chip a
link alone does not provide record-equivalent retrieval. General search does not
gain trait candidates implicitly. Glossary namespaces, rarity reference pages,
external citations and prose auto-linking remain separate decisions.

Source mappings and locale changes participate in publication identity. Checked
content, membership, references and vector completeness need validation alongside
the normal workspace contracts. Early functional review precedes full UI gates
and independent presentation review.

Native trait chunks reference the same model/input registry as document reuse.
Their vectors are stored as binary f32 only when no published document occurrence
supplies those exact native bytes. Centroids retain their own binary values and
checksums, even for one-chunk seeds: normalization may change floating-point bits.
Selected chunk text and source coverage remain available for model-free validation;
pending rebuild values never replace the published evidence before activation.
