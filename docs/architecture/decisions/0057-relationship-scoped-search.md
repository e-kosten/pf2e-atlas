# ADR 0057: Boolean reference filtering and attributed result evidence

Status: accepted; expanded implementation in progress.
Date: 2026-10-21
Updated: 2026-10-22

## Context

Related provides bounded occurrence previews for an exact selected record or
contained entry. Capped neighboring keys cannot support complete record results,
ordinary filters or paging beyond that preview. References also need to combine
with ordinary AND/OR/NOT criteria in Advanced filters and CLI queries. A separate
conjunctive relationship collection would make those expressions unavailable and
leave search surfaces with different capabilities.

## Decision

`atlas-domain::QueryPredicate` is the single executable eligibility expression.
Relationship leaves compose with ordinary predicates under AND, OR and NOT.
Each carries a canonical record selection, direction and independent source-contents
scope. Each has a nonempty unique clause ID for presentation, discovery and evidence.
Shared pure construction assigns missing relationship clause IDs deterministically
for CEL and convenience-flag composition; validation retains semantic authority.
The reference-leaf limit is part of the shared query limits.

Owned-source address primitives live in atlas-domain and retain their existing
wire identity. Atlas-record owns source-backed node interpretation and selection
policy. Relationship predicates apply to candidate roots; they are rejected inside
an authored collection Exists. Exact selected embedded addresses are distinct from
correlating conditions on an authored child collection.

Index owns parameterized eligibility over existing resolved relationship occurrences
and the ordinary expression compiler. This complete expression feeds browse,
lexical and vector retrieval before candidate admission, verified-edition preference,
ordering and pagination. Saved-list membership intersects that same eligibility.
There is no parallel relationship request array, capped neighbor-key filter, second
query evaluator or relationship-specific artifact projection.

**Links to X** keeps X's destination root/owner exact. Main entry only admits links
made directly by each result's main entry; Whole entry includes its contained
entries. **Linked from X** considers links from exactly X under Main entry only,
and X plus its descendants under Whole entry. Descendants never include siblings
or parents. Selected fields/passages/rank remain checked reading context; an outgoing
field can restrict the source occurrence, while an incoming reading field does not
invent a field-addressed destination edge.

Shared retrieval admission checks every relationship target and exact source
selection through checked index readers and existing record policy, including
leaves under NOT or a branch that cannot match. Missing/stale targets are errors,
not empty sets that become universal under negation. App-service invokes that
admission before inference, owns workflow generation checks and projects human DTOs.
Preflight sessions close before inference; consuming reads preserve the existing
snapshot/generation contract. Negation means no matching admitted resolved link;
it does not establish the absence of unresolved or external references.

Nonsemantic results are exhaustive distinct product roots. Repeated occurrences,
multiple owners and multiple matching leaves never inflate root counts. Existing
semantic/hybrid coverage remains bounded and labelled through its normal contract.
References do not change ranking or invent semantic matches. Edition preference
cannot replace an exact destination with a record lacking the matching edge.

## Evidence, counts and selection lifetime

Search returns human presentations for every relationship clause, including
negative leaves, unmatched OR branches and zero-result searches. These annotations
are not an alternate eligibility input. They retain clause ID, exact leaf and
artifact generation. Catalog discovery can report an unavailable presentation
without concealing ordinary editor fields or fabricating a new selection.

Per-root occurrence evidence belongs only to positive relationship leaves that
participate in successful Boolean proofs. Shared compiled SQL truth, including
ordinary NULL semantics, determines that attribution. A true link in a failed AND
branch does not explain an OR result. Double negation can yield positive evidence;
negative absence has no fabricated occurrence. Evidence uses clause IDs rather
than positions in a separate constraint list.

Occurrence pages accept the full predicate, selected clause, result identity and
expected generation. They validate the complete expression and the selected
positive proof before paging. Evidence retains source/target owner navigation,
field and occurrence ordinal; root paging is independent of occurrence limits.
Structured source navigation opens the referring entry without selecting unreadable
metadata, while the occurrence locator retains its exact field. External,
unresolved and blocked occurrences retain their existing preview statuses and
never become fabricated resolved-result evidence. Equipment provenance is distinct
from a prose reference and does not imply worn equipment or inflicted conditions.

Contextual counts, value discovery and field statistics retain the complete
expression and optional saved-list scope. Existing self-exclusion rules and typed
unavailability for unsupported OR/compound-negative suggestion contexts remain;
unsupported suggestions do not weaken the executable query. Option counts retain
distinct-root and per-option edition-preference semantics. Field statistics retain
their existing basis. Count-free vocabulary keeps its independent artifact/list
scope. Existing text/Similar count availability policies remain unchanged.

Generation checks, cooperative abandonment and recovery follow
[ADR0049](./0049-search-lifetimes-and-pages.md). Exact snapshot-local selections
retain their source fingerprints and never silently rebind embedded ordinals.
The full expression participates in request, URL and browser-cache identity.

## Web and CLI composition

The toolbar's Links picker and Advanced's structural Record links condition edit
one tree, retaining unfinished siblings in the existing editor owner. Common
positive conjunctions support direct chips. Mixed/negated groups use a bounded
full-expression summary retaining Boolean operators and grouping, with Advanced
providing complete editing. Global Clear filters clears ordinary and link conditions
while retaining text/Similar and result order.

The record finder reuses SearchPickerModal and searches independently of parent
criteria. A stable app-composed picker component is supplied through a lightweight
shared-filter composition context, avoiding a filter-to-picker import cycle.
Each editing owner retains its local selection/draft state; nested lookups use Ant's
overlay stack. Explicit root-row selection replaces an embedded target; direction
or scope edits alone preserve that exact target. Human names and useful kind/source/
parent disambiguation are displayed; IDs never substitute for relationship labels.

Record and Related actions create the same predicate leaves. On active Search,
new actions conjoin with the entire existing expression; they do not extract or
rewrite arbitrary OR/NOT branches. Outside Search they create a leaf-only search.
Duplicate suppression applies only where direct conjunction identity makes it safe.

CLI CEL and structured web requests target the same predicate. CEL links_to and
linked_from accept canonical keys or literal typed selections with independent
source scope. Strict-name convenience flags add conjunctive root conditions through
the existing resolver. Names never inherit candidate filters. Artifact-free
print-filter accepts canonical conditions; name resolution requires the explicit
runtime operation. No unresolved-name AST or alternate relationship parser is added.

CLI browse, text and Similar use the common search-page projection. JSON returns
complete rows, clause presentations, evidence, coverage and page facts within the
existing envelope, replacing summary-only result payloads. Contextual CLI discovery
uses the shared context, and graph occurrences pages the same clause evidence.
Existing graph links/uses remain bounded previews and never become eligibility.

URLs serialize the canonical predicate through a versioned compact browser
transport. Unsupported or malformed filter transport, including retired
relationships parameters, clears the complete expression automatically, replaces
the URL entry and resets paging. One nonblocking message reports the cleared
filters; independent criteria and exact reading navigation remain intact. A
decoded predicate with invalid fields or selections remains an editable validation
error. Retrieval failures never clear filters. No converter or old request/execution
path is retained.

Similar remains a separate search criterion. Its named, selectable read-only seed
occupies the keyword search shell with identical clear/search affordances. Clearing
it retains filters and returns focus to keyword entry.

## Consequences

The shared query model, CLI grammar, web editor, discovery and evidence readers
must change together. Architecture and validation must cover Boolean truth,
source identity and surface parity, not just picker appearance. Existing bounded
Related/encounter readers retain their policies and exact selections.

No source/model pin, reference schema, artifact re-preparation, semantic recipe or
saved-state replacement is implied. Inferred gameplay relationships and multi-hop
traversal remain outside this decision. Trait identities and associations retain
[ADR0056](./0056-trait-reference-entities.md).
