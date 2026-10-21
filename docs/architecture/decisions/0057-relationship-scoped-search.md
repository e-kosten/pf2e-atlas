# ADR 0057: Relationship-scoped search and attributed result evidence

Status: accepted; implementation in progress.
Date: 2026-10-21

## Context

Related provides bounded occurrence previews for an exact selected record or
contained entry. Deriving neighboring keys from a capped preview cannot support
complete record results, ordinary filters or paging beyond that preview. Search
also needs to combine references with ordinary criteria and existing text/Similar
retrieval without introducing a second filter engine.

## Decision

Relationship constraints are typed eligibility inputs alongside the ordinary
query predicate and optional saved-list key scope. Each carries direction, exact
checked record selection and an independent source-contents scope. A collection
of constraints composes conjunctively. They are not new ranking modes, authored
record identities, or general-purpose Boolean expressions in the filter editor.

Index owns parameterized reference eligibility over the existing relationship
occurrences, including exact source/target owner addressing and descendant
selection. The same eligibility feeds browse, lexical and vector retrieval before
candidate selection, verified-edition preference and pagination. A capped list of
neighbor keys is never an eligibility input derived from Related. Search owns
retrieval and distinct-root results; app-service validates checked selections and
projects human labels and evidence; web remains thin transport over app contracts.

**Links to X** keeps X's destination exact. With Include contents off, only links
from the result's main entry qualify; with it on, links from its contained entries
qualify too. **Linked from X** considers links from exactly X with Include contents
off and X plus its descendants with it on. Including descendants of a selected
contained entry never includes its siblings or parent. Root-only selection and
all-source-owner selection remain different operations.

Root results are complete for nonsemantic relationship browsing and count each
root once. Repeated references, multiple owners and multiple matching constraints
do not inflate that count. Existing semantic/hybrid coverage remains bounded and
is reported through its normal coverage contract; relationships do not change the
ranking policy or invent new semantic matches. Edition preference cannot replace
an exact destination with a record that has no matching reference.

Each root carries a bounded preview of its matching occurrences, with separate
occurrence counts and pageable evidence. Evidence retains exact source and target
navigation, field, occurrence ordinal and relationship origin/kind. Root paging is
independent of occurrence-preview limits. Resolved local product roots contribute
to result counts; external/unresolved/blocked occurrences keep their statuses in
the existing bounded previews and do not become fabricated record results.
Compendium equipment provenance remains distinct from a prose reference; neither
automatically establishes that equipment is worn or a condition is inflicted.

Generation checks, cooperative abandonment and explicit recovery follow
[ADR0049](./0049-search-lifetimes-and-pages.md). Exact source selections retain
their snapshot fingerprints. Generation recovery does not silently rebind an
embedded ordinal. Relationship constraints and source scope participate in
request, URL and browser-cache identity. Contextual facet counts must use the
relationship scope or be unavailable; count-free vocabulary remains usable.

The browser displays active relationship constraints at the top level alongside
ordinary filters. Controls use human names, with kind/publication/parent names
where disambiguation is needed, and never display record IDs as a fallback.
Exact keys remain in typed identity and navigation, not visible relationship copy.
The first workflow creates one constraint from a record action or Related's
View all links. Later direct creation reuses the shared list/encounter search
picker and permits multiple independent constraints combined with AND. Their
underlying model and eligibility do not require a separate engine at that point.

Similar remains a separate search criterion. Its active seed occupies the search
control as a named, selectable, read-only criterion with Clear. Clearing it
retains other filters and returns focus to keyword entry. The browser does not
show a competing editable search field while a Similar seed is active.

## Consequences

Full reference browsing and Related previews share reference identity/policy while
retaining different bounded response shapes. Existing encounter-owned Related
readers keep their exact selections and recovery behavior. No source/model pin,
artifact re-preparation, semantic recipe, or saved-state replacement is implied.

Attribute sorting and the direct multi-relationship picker are separate PRs.
General OR/NOT relationship expressions, inferred gameplay relationships,
multi-hop traversal and trait membership reinterpretation remain outside this
decision. Trait identities and associations retain
[ADR0056](./0056-trait-reference-entities.md).
