# ADR 0056: Source-backed trait reference entities

Status: accepted; implementation in progress.
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
in checkpoint reuse/inference. The seed is the deterministic L2-normalized
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
