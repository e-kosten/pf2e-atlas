# ADR 0013: TUI Canonical Search State And Derived Editor Projections

- Status: Accepted
- Date: 2026-04-23

## Context

ADR 0010 allowed the TUI to keep its own preferred local editing model while adapting to `SearchRequest` at the backend boundary. That was a useful intermediate rule, but the next search-editor refactor makes a stronger ownership decision:

- the TUI should no longer keep a separate long-lived semantic query model beside the shared contract
- the new editor needs tree rendering, insertion slots, summary/document views, and focused draft builders
- those UI needs are real, but they should not become a second canonical search state model

Without a durable rule here, future work could easily drift back into a TUI-owned semantic query shape that competes with the shared request model.

## Decision

In the TUI, canonical long-lived search state is the shared `SearchRequest`.

The TUI may derive additional editor-facing models, but their ownership is constrained:

- filter-tree models with ephemeral node IDs and insertion slots are derived presentation projections of canonical `SearchRequest`
- summary/document models are derived presentation projections of canonical `SearchRequest`
- dedicated search-specific tree-editor surfaces may exist as derived live presentations over canonical `SearchRequest`, but they must not own a second staged query model
- focused editor builders and drafts are transient local state used only while an editor is collecting enough input to emit a valid canonical node
- structured search-editor resume targets are transient host state over canonical `SearchRequest`, limited to root resume, group-local resume by canonical `groupPath`, and exact-node resume for truly node-scoped operations
- projected grouped field buckets may be reselected after projection, but their identity is derived from canonical group/member paths rather than stored as durable editor identity
- unary `not` remains a wrapper over one child node and should not be treated as a peer group-local continuation owner
- incomplete drafts must not become canonical workspace state
- canonical workspace state must not be represented as `Partial<SearchRequest>` or `Partial<SearchFilterNode>`

The TUI therefore owns:

- rendering and navigating derived projections of canonical state
- transient local builder state
- feature workflows around editing, moving, wrapping, and presenting canonical nodes

The TUI does not own:

- a second long-lived semantic query model beside `SearchRequest`
- persistent editor IDs inside canonical search state
- compatibility upgrade shapes that preserve the replaced `query.filters.parts` model as the durable editor contract

## Consequences

- The TUI search editor becomes a consumer and presenter of shared search meaning rather than an alternate owner of that meaning.
- UI-local identity and insertion structure may still exist, but only as derived/transient state.
- A separate tree-editor screen remains acceptable only when it is a live host over canonical state rather than a durable staged query owner.
- Group-local structured-editor continuation should resume from canonical group context first, then derive visible row selection from the current projection.
- Future TUI work should add editor affordances by projecting from canonical `SearchRequest`, not by inventing a second semantic query shape.
- This supersedes the older assumption from ADR 0010 that TUI query parts remain the preferred durable local editing model.
