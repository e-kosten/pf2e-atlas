# TUI Architecture

This document describes how the terminal UI is assembled, where its boundaries live, and how it reuses the same backend runtime as the MCP server without pulling terminal concerns into shared services.

## What The TUI Owns

The TUI in `src/tui/` is a separate presentation surface over the same prepared PF2E index and search runtime used elsewhere in the repository.

Its job is to own:

- terminal rendering and input handling
- screen state, workflow state, and interaction routing
- user-facing search editing and ontology navigation flows
- editorial workbench entrypoints for tag-review tasks

It should not own:

- SQLite lifecycle management
- search ranking internals
- ontology domain construction
- normalized catalog access

Those responsibilities stay below the TUI behind app-layer and backend facades.

## Composition Root

`src/tui/app-services.ts` is the TUI composition root. It builds a terminal-oriented service bundle from the shared application runtime:

- `loadPf2eApplicationRuntime()` from `src/app/runtime.ts` loads config and the long-lived `Pf2eDataService`
- `createPf2eApplicationStorageService()` supplies app-scoped storage helpers for workflows that need direct index access, such as editorial workbench flows
- `createPf2eApplicationOntologyService()` prepares cached search-semantics browse models for the ontology area from config, the shared data facade, and the shared app-layer discovery service
- `createPf2eTerminalSearchService()` adapts shared catalog/search capabilities into a TUI-facing query/session API
- the derived-tag workbench services are wired in as a development/editorial area, not mixed into the user search and ontology APIs

That gives the terminal app one explicit dependency object, `Pf2eTerminalAppServices`, with a clear split:

- `user`: TUI-facing services for ontology and search
- `dev`: editorial workbench services

## Runtime Flow

```mermaid
flowchart TD
  Entry["`runPf2eTerminalApp()`\n`src/tui/pf2e-app.tsx`"] --> Framework["Ink runtime + `DerivedTagTerminalProvider`\n`src/tui/framework/provider.tsx`"]
  Framework --> Bootstrap["`Pf2eTerminalBootstrap`"]
  Bootstrap --> Load["`loadPf2eTerminalAppServices(argv)`\n`src/tui/app-services.ts`"]

  Load --> Runtime["`loadPf2eApplicationRuntime()`\n`src/app/runtime.ts`"]
  Runtime --> Config["App config"]
  Runtime --> Data["`Pf2eDataService`"]

  Load --> Storage["`createPf2eApplicationStorageService()`"]
  Load --> Discovery["`createPf2eApplicationSearchDiscoveryService(dataService)`"]
  Load --> Ontology["`createPf2eApplicationOntologyService(config, dataService, discovery)`"]
  Load --> Search["`createPf2eTerminalSearchService(...)`"]
  Load --> Dev["Derived-tag workbench services"]

  Data --> Discovery
  Data --> Search
  Data --> Ontology
  Discovery --> Ontology
  Storage --> Dev

  Load --> App["`Pf2eTerminalApp`"]
  App --> Context["`Pf2eTerminalAppServicesProvider`"]
  Context --> Screens["Area menu, Search screen,\nOntology explorer, Tag workflows"]
```

The important architectural point is that the TUI does not rebuild backend logic. It composes shared services once, then keeps terminal behavior local to the screen tree.

## Shared Backend, Local UI

The TUI shares backend services in two ways:

- search flows are built on top of a TUI-facing search facade backed by the shared `Pf2eDataService`
- ontology browsing is built from the same app-layer ontology service used to assemble readonly domain models

But the TUI keeps UI concerns local:

- `Pf2eTerminalApp` owns top-level navigation orchestration through a dedicated route-transition coordinator rather than ad hoc route mutations in screen callbacks
- screen controllers own transient selection, pane focus, and detail-scroll state
- workflows own prompt flows, modal handoffs, session cleanup, and live-count/result-window behavior
- framework modules own Ink-specific rendering, modal hosting, terminal sizing, and raw input normalization
- shared list/detail owners now own repeated pane measurement, screen-model assembly, shared interaction-context setup, compact default result rows, shared breadcrumb formatting, the reusable rightward behavior contract for screens that follow the common list/detail pattern, and the shared grouped-result presentation path for compatible result readers

This split matters because it lets the TUI add richer interaction behavior without pushing terminal concepts like pane focus, action rails, or staged editors down into `src/app/`, `src/data/`, or `src/search/`.

Shared ontology/detail presenters may also attach optional link metadata to a text line, such as `href` plus a plain-text fallback string. The shared TUI framework owns hyperlink rendering for those lines: supported terminals get a clickable OSC 8 hyperlink, while unsupported terminals and plain-text consumers fall back to a readable `label: url` string.

## Major TUI Layers

### Framework Layer

`src/tui/framework/` contains the terminal mechanics:

- Ink provider lifecycle in `framework/provider.tsx`
- line rendering and wrapping in `framework/line-rendering.tsx`
- pane and screen geometry in `framework/screen-layout.ts`
- pane and screen React components in `framework/screen-components.tsx`
- input and navigation helpers in `framework/input.ts`
- terminal context in `framework/context.ts`
- modal host and input routing in `framework/modal-host.tsx`
- modal sizing and layout planning in `framework/modal-planning.ts`
- modal prompt bodies in `framework/modal-prompt-bodies.tsx`
- centered-prompt presentation naming and background treatment normalization in `framework/prompt-presentation.ts`
- prompt-session ownership in `framework/provider.tsx`, where multi-step prompt flows replace the current prompt without exposing an intermediate background frame and keep blanked-background prompt layout separate from background workspace sizing

Feature code should build on these helpers instead of importing raw terminal primitives directly.

### App Services Layer

`src/tui/app-services.ts` and `src/tui/app-service-context.tsx` define what the terminal app can use. This keeps most screens from knowing how runtime assembly works.

The service context is intentionally narrow: screens ask for `services.user.search`, `services.user.ontology`, or `services.dev.*`, not for direct storage or low-level query helpers.

### Navigation Layer

`Pf2eTerminalApp` now routes user-visible navigation through a dedicated coordinator hook in `src/tui/pf2e-navigation.ts`.

That layer owns:

- transition intents such as open area, open search, return/back, and open review session
- prepared route handoffs that may need async work before the destination can be committed
- the mounted-screen transition status contract used to show one shared loader treatment during pending transitions
- route-stack commit operations as navigation-internal detail rather than screen-callback API

The architectural rule is that route screens and menu callbacks should request navigation intent, not dispatch raw push/replace/pop route mutations themselves.

Route readiness is part of that same boundary:

- routes that need data for their first meaningful frame must be prepared before commit
- route screens should mount from render-ready payloads, not kick off route-entry bootstrap work after navigation
- the currently mounted screen hosts the shared transition-status affordance while navigation prepares the next route

The current prepared-route set includes:

- prepared ontology inspect routes
- prepared search result routes
- prepared entity-page routes, which now carry a ready `EntityPageDocument` payload instead of reopening page composition on mount

Prepared search result routes are keyed by their prepared search window so pushing a new result route from an existing result-reader host mounts the new reader state instead of reusing the previous route's internal session.

### Shared List/Detail Presentation And Behavior Layer

`src/tui/list-detail-presentation.ts`, `src/tui/list-detail-behavior.ts`, and `src/tui/list-detail-formatting.ts` sit above the lower-level interaction/router primitives and below feature controllers.

It owns the repeated mechanics that several screens were previously rebuilding:

- pane body measurement and detail-window slicing
- shared assembly of `TerminalPaneScreen` / `TerminalTwoPaneScreen` props
- common interaction-context setup for list, detail, optional text-entry, and optional action-target flows
- transient footer-banner notifications for lightweight failed-navigation or informational feedback
- reusable rightward list behavior contracts such as `drill`, `open`, `preview`, or `none`
- shared dead-end handling for qualifying list/detail screens, including notification-vs-noop policy and preview-already-visible behavior
- shared breadcrumb and default result-row formatting for list/detail search and explorer surfaces, using the shared ontology/search vocabulary owner for friendly fallback rendering where no explicit alias exists
- shared grouped-result presentation mechanics such as optional section headers, row badges, and light detail metadata lines when a caller supplies grouping or row-presentation metadata

It does not own feature-domain workflows. Search, filter explorer, and review still decide:

- which actions are available in each context
- how list rows, detail lines, and status text are built
- how successful rightward intents map to local reducer actions or async workflow steps
- which non-contract workflow outcomes still warrant local notification or prompt handling

Structured page/document surfaces sit adjacent to this layer rather than inside it. `src/tui/page-document/` owns compilation of `EntityPageDocument` into a TUI-facing page model with stable section anchors and selectable targets. Targets can address either full rows or inline spans, so one rendered line can contain multiple selectable spans while interaction code still works against one ordered section-local target list. Qualifying screens may then render that page model through the shared list/detail shell without collapsing page semantics back into ad hoc feature-local detail composition. The current qualifying consumers are the search result-reader preview, ontology record detail inside the shared filter explorer host, and the dedicated entity-page route screen.

Page/document navigation must operate against rendered terminal rows when a host adds surrounding chrome or when content wraps. Hosts that prepend metadata, such as search result previews, provide rendered row offsets back to the page-document interaction owner so section focus and target focus stay visible in the actual pane rather than in raw document-node coordinates.

`Referenced By` page sections render grouped backlink rows as section-local search-pivot targets. Activating one of those targets opens a prepared search-results route through the shared app navigation seam using the target's canonical `SearchRequest`, with the rendered result-reader surface as the user-visible outcome.

The `Referenced By` grouped drill request is a `linksTo(targetRecordKey)` browse request for the selected page record. `linkedFrom` remains the canonical inverse-link filter in the shared search contract and may still be exposed by query-editor follow-through, but it is not the entity-page grouped drill shape.

Entity-page metadata pivots are generated from the page model rather than from host-local affordances. Visible traits stay inline on the identity line as `Traits: ...`; each trait value is a selectable `searchPivot` span in the identity section. Classification metadata such as derived tags, families, pack, category, and subcategory appears as section-local search-pivot row targets in the page's classification section. Selectable inline spans use the `accent` tone, and the focused inline span uses the `selected` tone. Row targets use the same selectable/focused tone convention at line level.

Inline record references in prose are compiled from existing outgoing page relation edges. Resolved UUID markup renders as readable selectable record spans, while unresolved UUID markup renders as short readable fallback text instead of exposing raw vendor UUID syntax. Pointer hit-testing is a separate follow-up and should build on the page-document span target model rather than adding another target representation.

Pane-focus changes remain explicit actions. For qualifying list/detail callers, rightward dead ends must not move focus, and `preview` means "keep the selected row visible in the detail pane without moving focus." If that preview is already satisfied, the shared behavior layer treats the input as a dead end.

The current qualifying page-document callers are the search result-reader preview, ontology record detail inside the shared filter explorer inspect/open page contexts, and the dedicated entity-page route screen. The derived-tag review screen still uses the shared presentation mechanics, but it does not fit this rightward behavior contract because its primary rightward interaction is an action-target flow rather than list-row confirm behavior.

Shared pointer routing also lives at this layer boundary. Pane-level list/detail hosts should expose pointer regions for both panes so wheel or trackpad input follows the hovered pane, pane clicks can set focus explicitly, and overlays capture pointer input before background panes. Fine-grained in-pane target activation remains a separate concern from pane routing.

Use this layer when a screen is fundamentally a list/detail surface with shared pane, footer/help, and routing mechanics. Do not push unrelated staged-editor or domain workflow logic into it just to make a screen fit the abstraction.

Lookup stays a consumer of this shared result-view pathway rather than becoming its own durable presentation owner. Lookup supplies `matchType` plus the selected `tiered` or `global` sort policy, and the shared list/detail presentation layer decides whether that becomes grouped sections, row badges, or a light detail metadata line in the result reader.

### Search Service Layer

`src/tui/search/service.ts` is the TUI-facing search facade. It is not the ranking engine itself. Instead, it:

- keeps canonical `SearchRequest` state as the TUI query model
- uses `query.filter` as the shared semantic filter tree instead of a TUI-local parallel query-part model
- exposes category, subcategory, sort, and facet options for the UI
- converts ontology-origin queries into canonical TUI query state
- opens and reads search windows through the shared backend
- owns TUI session concepts such as result buffers, sort changes, session disposal, and lookup match-type metadata on prepared result rows

This keeps query editing and result reading logic in the TUI while leaving shared query semantics in `src/domain/` and search execution in backend/search-owned services.

Within the search screen, the live workspace no longer renders structured rows directly from raw query state. The search-screen workspace derives a summary/document model from the canonical query state first, then renders:

- workspace rows
- live structured-query summaries
- query-status/detail summaries

That summary layer owns stable anchors for major query branches and filter nodes. The durable rule is:

- `SearchRequest` is the canonical TUI query state, including the structured filter tree
- query-state helpers own normalization, projection, and interpretation of canonical `query.filter`
- metadata-node adapters may convert between editor-facing metadata nodes and canonical filter nodes, but they do not own a parallel query semantics model
- `SearchRequest` is the shared semantic contract once query state crosses out of the TUI
- the workspace summary/document model owns editor-facing identity and display structure
- terminal renderers consume that summary model instead of re-deriving structured meaning from raw query state in each pane

The dedicated structured editor presents a visible root boolean group for editing, but that editor framing is a live host surface over canonical query state rather than a separate staged query model. Single-child wrapper groups that exist only to support editor presentation should collapse back to the underlying canonical filter node when they do not carry additional shared semantics.

Structured editing keeps canonical `SearchRequest` state live plus local cursor and move state. Focused add/edit flows such as the query-field builder keep short-lived dialog-local builder state while they collect enough input to emit a valid canonical node. Explorer-backed field editing may derive transient selection state from the current query to drive row badges, grouped buckets, or scalar summaries, but those projections apply changes back into the live query immediately instead of owning a second staged search-edit model.

### Structured Editor Continuation

The structured search editor owns continuation semantics for child prompt and explorer flows. The shared explorer remains generic: it reports explorer outcomes and draft/query translations, while the search host decides how those outcomes mutate canonical `SearchRequest` state and where the editor resumes.

The durable owners are:

- `src/tui/search-screen/structured-draft/structured-draft-continuation.ts` for shared-explorer child-flow continuation inside the structured editor
- `src/tui/search-screen/structured-draft/structured-draft-state.ts` for resume targets over canonical search state
- `src/tui/search-screen/structured-draft/structured-draft-support.ts` for deriving visible rows and selection from the current query plus resume target
- `src/tui/search-screen/structured-draft/structured-draft-metadata-actions.ts` for translating child-flow results into bounded structured-editor mutations

Resume targets are host state, not canonical search data. The allowed durable targets are root resume, group-local resume by canonical `groupPath`, and exact-node resume by canonical node path when the operation is genuinely node-scoped. Group-local continuation is the default for live prompt and explorer flows. Projected field buckets are presentation-derived from canonical members, and unary `not` remains a wrapper over one child node rather than a peer group-editing anchor.

Structured-editor child flows should emit or be translated into bounded mutation families such as replace node, append nodes, and replace grouped field. Query-global replacement belongs only to explicitly query-global search flows. Generic helpers in `src/tui/filter-explorer/search-draft-query.ts` and `src/tui/search/service.ts` may keep preparing explorer drafts and generic insertion results, but they do not own structured-editor mutation or resume semantics.

Prompt-local builders may keep incomplete value-entry state while collecting valid scalar, metric, pack, scope, level, price, or action-cost input. Once a valid canonical node or node set exists, the host resumes through the same structured-editor state and focus rules used by explorer-backed flows.

This seam is enforced by named owner tests for the continuation coordinator, resume-target state, grouped-field helpers, and broad search-screen interaction flows. Lint is intentionally deferred for this seam because the risk is callback and workflow ownership rather than an import boundary that can be banned without false positives. Future work should add lint only if a stable syntactic bypass appears, such as direct structured-editor use of a generic whole-query draft writeback helper where a bounded host mutation is required.

### Ontology Explorer Layer

`src/tui/ontology-explorer/` still owns ontology-specific hosting concerns, but the durable browse surface is now the shared filter explorer in inspect mode rather than a separate ontology-only screen.

In the current split:

- `src/tui/filter-explorer/` owns the shared list/detail browser, controller-state and inspect/open helpers, route-handling helpers, query/model draft translation, and shared scalar editing prompts
- `src/tui/ontology-explorer/inspect-screen.tsx` is a thin host for a prepared ontology route payload; entering the ontology area now lands directly in that shared inspect session and routes selected leaves into search
- ontology record mapping, relation-aware page composition, and explorer-cache writeback live under `src/app/ontology/`, not TUI-local wrapper files
- `src/tui/ontology-explorer/` legacy browse-only pieces remain isolated and should not become the primary path for new ontology/search exploration work
- `src/app/ontology/presenter.ts` is the durable plain-line presentation path for non-qualifying ontology/editorial consumers only; qualifying record-page TUI consumers must use `services.user.entityPages` and compile through `src/tui/page-document/*`

The remaining line-detail presenter callers are `src/app/ontology/derived-tags-domain.ts`, `src/app/ontology/node-helpers.ts`, and `src/tags/editorial/ui/review-detail-content.ts`. Those callers produce readonly ontology labels or editorial text-line previews, not structured entity-page routes, inline page targets, or relation-aware page navigation.

The ontology host still adds:

- direct entry from the ontology area into the shared search-semantics explorer
- restoring ontology snapshots when the user returns from search
- launching either a seeded search editor route or a prepared result-reader route from selected ontology nodes
- resolving record nodes into shared entity pages through `services.user.entityPages`, while leaving non-record ontology nodes on the ordinary line-detail path

The durable rule is that the ontology area does not load its first frame after mount. Navigation prepares the ontology model first, keeps the current screen mounted with the shared transition footer, and commits the ontology route only once the route payload is ready.

## Search Semantics Flows

The search screen and the ontology browser share the same explorer shell, but they do not currently load the same model.

Shared between them:

- `OntologyDomainModel` and `OntologyNode` remain the tree contract
- `FilterExplorerScreen` remains the shared list/detail explorer surface
- the shared action rail remains the owner for discovery-mode actions such as `Use Matching Counts` and `Use Catalog Counts`
- the shared explorer may expose generic mode-switch affordances, while each host owns its own mode type and mode semantics
- app-layer ontology and discovery services remain the semantic owners below the TUI

Different between them:

- the ontology browser loads a broad browse model
- the search explorer loads a prepared search-scoped model
- picker-style prompt flows use the same discovery vocabulary, but they are not the same surface as the explorer

### Broad Browse Model vs Prepared Search-Scoped Model

The ontology browser uses the broad search-semantics browse tree from `loadSearchSemanticsDomain(...)`. That model exists to browse the ontology of fields, values, families, and tags even before a user has entered a live scoped search query.

The search explorer uses `loadSearchFilterExplorerDomain({ request, discoveryMode })`. That model is prepared from a concrete canonical `SearchRequest`, so it is scoped to the current query and can change meaningfully when discovery mode switches between `matching` and `catalog`.

The practical distinction is:

- broad browse model: "show the ontology/catalog of what exists in general", while broad derived-tag families still honor the current discovery mode
- prepared search-scoped model: "show what exists or applies for this specific search query right now"

### Shared Explorer Shell

```mermaid
flowchart TD
  subgraph SharedApp["Shared App / Domain / Data"]
    SR["SearchRequest"]
    ODM["OntologyDomainModel"]
    AOS["ontology service"]
    ASDS["search discovery service"]
    SSD["search semantics domain builder"]
  end

  subgraph SharedTUI["Shared TUI Explorer Layer"]
    FES["filter explorer screen"]
    FEC["filter explorer controller"]
    FEM["filter explorer screen models"]
    AR["shared action rail"]
  end

  AOS --> SSD
  ASDS --> SSD
  SR --> SSD
  SSD --> ODM
  ODM --> FES
  FEC --> FES
  FEM --> FES
  AR --> FES
```

The explorer shell is shared. The important difference is which model gets passed into it.

### Shared Explorer Session State Machine

Both surfaces enter the same shared explorer behavior after their host-specific setup is done. What differs is how they prepare the first model and what they do when the user exits that shared session.

```mermaid
stateDiagram-v2
  [*] --> OntologyPrepared : ontology route prepared
  [*] --> SearchPrepared : search edit session opened

  OntologyPrepared --> InspectSession : mount inspect-and-open host
  SearchPrepared --> SearchLiveSession : mount inspect-and-open host

  state SharedExplorer {
    [*] --> Browsing
    Browsing --> ActionTarget : enter action rail
    ActionTarget --> Browsing : leave action rail
    Browsing --> SearchInput : enter local text filter
    SearchInput --> Browsing : cancel or commit
    Browsing --> DiscoveryRefresh : switch matching/catalog mode
    DiscoveryRefresh --> Browsing : model replaced
    Browsing --> ScalarPrompt : edit numeric scalar target
    ScalarPrompt --> Browsing : save or cancel
  }

  InspectSession --> SharedExplorer
  SearchLiveSession --> SharedExplorer

  InspectSession --> OpenSearch : open editor or results
  OpenSearch --> [*]

  InspectSession --> [*] : exit at root
  SearchLiveSession --> ReturnToTree : back outcome
  ReturnToTree --> [*]
  SearchLiveSession --> [*] : exitRoot or cancel
```

The durable split is:

- ontology browse enters the shared explorer in inspect mode and exits by opening a search route or returning to the ontology area
- search field editing enters the shared explorer in inspect-and-open mode with host-provided activation and selection presentation, and the search host applies live query updates plus close/back outcomes around that shared session
- matching/catalog refresh, action rail behavior, list/detail routing, and scalar edit prompts are shared explorer behaviors even when the surrounding host lifecycle differs

For the sequence views below:

- green participants and phases are shared between both flows
- blue participants and phases are unique to the ontology-browser host path
- amber participants and phases are unique to the search filter-builder host path

### Ontology Browser Flow

```mermaid
sequenceDiagram
    actor User as User
    box rgba(219, 234, 254, 0.65) Ontology-only host and navigation
        participant Nav as pf2e-navigation.ts
        participant Host as ontology-explorer/inspect-screen.tsx
    end
    box rgba(217, 247, 208, 0.65) Shared explorer and app services
        participant Explorer as FilterExplorerScreen
        participant Controller as filter-explorer/controller.ts
        participant Ontology as app/ontology-service.ts
        participant Search as search/service.ts
        participant Data as search discovery + Pf2eDataService
    end

    rect rgba(219, 234, 254, 0.28)
        User->>Nav: Open ontology area
        Nav->>Ontology: loadSearchSemanticsDomain({ discoveryMode: matching })
        Ontology->>Data: build broad browse model
        Data-->>Ontology: OntologyDomainModel
        Ontology-->>Nav: prepared route model
        Nav->>Host: commit ontology route payload
    end

    rect rgba(217, 247, 208, 0.28)
        Host->>Explorer: mount inspect-and-open session
        Explorer->>Controller: create shared list/detail state
        Controller-->>Explorer: rows, detail, action rail
        User->>Explorer: browse, drill, inspect
    end

    rect rgba(217, 247, 208, 0.28)
        User->>Explorer: switch matching/catalog counts
        Explorer->>Host: request discovery-mode change
        Host->>Ontology: loadSearchSemanticsDomain({ discoveryMode })
        Ontology->>Data: rebuild broad browse model
        Data-->>Ontology: refreshed domain
        Ontology-->>Host: refreshed model
        Host->>Explorer: replace shared model
    end

    rect rgba(219, 234, 254, 0.28)
        User->>Explorer: open selected ontology query
        Explorer->>Host: onOpenQueryIntent(intent, snapshot)
        Host->>Nav: hand off search intent
        opt launch results directly
            Nav->>Search: executeQuery(createQueryFromOntologyQuery(...))
            Search->>Data: run prepared search session
            Data-->>Search: initial result window
            Search-->>Nav: prepared results route payload
        end
        Nav-->>User: open search editor or results route
    end
```

The ontology browser owns:

- route preparation
- snapshot restore when returning from search
- launching seeded search flows from ontology leaves

The ontology browser still uses the broad browse builder rather than the prepared search-scoped builder, but the shared discovery-mode switch now has a real semantic effect for broad derived-tag families:

- `matching` keeps only nonzero derived-tag children visible inside each family and uses the nonzero child count as the family count
- `catalog` keeps authored derived-tag children visible even when their live count is `0`, and family counts reflect the full authored family size in scope
- zero-count catalog derived-tag leaves stay actionable through the standard shared explorer open/search path

### Search Explorer Flow

```mermaid
sequenceDiagram
    actor User as User
    box rgba(255, 243, 205, 0.75) Search-builder-only host and workflow
        participant SearchHost as search-screen/controller.ts + query-field-builder session
        participant Workflow as search-screen/filter-explorer-workflow.ts
        participant SearchExplorerHost as search-screen/filter-explorer-screen.tsx
    end
    box rgba(217, 247, 208, 0.65) Shared explorer and app services
        participant Explorer as FilterExplorerScreen
        participant Controller as filter-explorer/controller.ts
        participant Search as search/service.ts
        participant Ontology as app/ontology-service.ts
        participant Data as search discovery + Pf2eDataService
    end

    rect rgba(255, 243, 205, 0.35)
        User->>SearchHost: Open dedicated filter builder
        SearchHost->>Workflow: openFilterExplorer(...)
        Workflow->>Search: normalizeQuery()
        Workflow->>Ontology: loadSearchFilterExplorerDomain(request, matching)
        Ontology->>Data: build prepared search-scoped model
        Data-->>Ontology: prepared OntologyDomainModel
        Ontology-->>Workflow: prepared explorer model
        Workflow->>SearchExplorerHost: create live explorer session
    end

    rect rgba(217, 247, 208, 0.28)
        SearchExplorerHost->>Explorer: mount inspect-and-open session with host selection state
        SearchExplorerHost->>Search: derive transient selection state from the live query
        Explorer->>Controller: create shared list/detail state
        Controller-->>Explorer: rows, detail, action rail
        User->>Explorer: browse, drill, edit clauses
        Explorer-->>SearchExplorerHost: target activation and explorer outcomes
    end

    rect rgba(255, 243, 205, 0.35)
        User->>Explorer: switch matching/catalog counts
        Explorer->>SearchExplorerHost: request discovery-mode change
        SearchExplorerHost->>Ontology: loadSearchFilterExplorerDomain(request, nextMode)
        Ontology->>Data: rebuild prepared scoped model
        Data-->>Ontology: refreshed domain
        Ontology-->>SearchExplorerHost: refreshed model
        SearchExplorerHost->>Explorer: replace shared model
    end

    rect rgba(255, 243, 205, 0.35)
        User->>Explorer: toggle or edit a clause
        Explorer->>SearchExplorerHost: onQueryChange(nextQuery)
        SearchExplorerHost-->>SearchHost: updated live canonical query
        SearchHost-->>User: continue in the live query tree
        User->>Explorer: back, exitRoot, or cancel
        Explorer->>SearchExplorerHost: onOutcome(...)
        SearchExplorerHost-->>SearchHost: close or return with current live query state
    end
```

The search explorer owns:

- taking the live canonical query as scope
- loading a prepared model for the current `SearchRequest`
- caching and refreshing discovery-mode transitions within the search workflow
- deriving transient selection state from the live query and applying query updates immediately through host callbacks

This is still the path where `matching` versus `catalog` has the broader query-scoped semantic effect, even though the broad ontology browser now also uses the same labels for derived-tag family visibility and counts. The shared explorer layer does not own one repo-wide `matching` / `catalog` mode type; search and ontology each own the host type and semantics for their own flow.

### Centered Prompt Flows

Centered prompts remain the smaller selection surface for short non-discovery decisions such as choosing:

- a metadata field family
- a clause kind
- a comparison operator

These prompts do not own live discovery mode state or query-scoped value exploration. Scoped discovery steps such as pack selection and metric-key selection mount the shared explorer screen instead and use the shared action rail for `matching` / `catalog` transitions.

### What The Similarity Means

Search explorer, ontology browser, and centered prompt flows should not invent different core discovery semantics. They should share:

- the same discovery vocabulary
- the same app/domain owners for applicability and counts
- the same explorer/action-rail interaction family where the shared explorer surface applies

But they are not required to share the exact same host screen or model-builder entrypoint. The architecture intentionally allows:

- broad ontology browsing without a live query
- search-scoped exploration from a concrete `SearchRequest`
- narrow modal selection flows for focused prompt work

When these surfaces feel inconsistent, the first question is whether the inconsistency comes from:

- shared explorer/action-rail plumbing
- the broad browse model
- the prepared search-scoped model
- or a picker-local prompt flow

That distinction determines which layer should own the fix.

## Screen, Workflow, And Controller Split

The TUI generally separates visible screens from stateful interaction logic and service calls.

```mermaid
flowchart TD
  subgraph Framework["Framework / Terminal Runtime"]
    Provider["`framework/provider.tsx`"]
    Rendering["`framework/{line-rendering,screen-layout,screen-components}.ts*`"]
    Input["interaction adapters + input routers"]
  end

  subgraph Screens["Screens"]
    App["`pf2e-app.tsx`"]
    SearchScreen["`search-screen/screen.tsx`"]
    OntologyScreen["`ontology-explorer/inspect-screen.tsx`"]
  end

  subgraph Controllers["Controllers / Workflows"]
    SearchController["`search-screen/controller.ts`"]
    SearchWorkflow["`search-screen/session-workflow.ts`"]
    FilterExplorerWorkflow["`search-screen/filter-explorer-workflow.ts`"]
    FilterExplorer["`filter-explorer/{controller,screen-models}.ts`"]
    OntologyHost["`ontology-explorer/inspect-screen.tsx`"]
  end

  subgraph Services["TUI Services"]
    AppServices["`app-services.ts`"]
    SearchService["`search/service.ts`"]
    OntologyService["`src/app/ontology-service.ts`"]
  end

  subgraph Backend["Shared Backend"]
    Runtime["`src/app/runtime.ts`"]
    Data["`Pf2eDataService`"]
  end

  Provider --> App
  Rendering --> SearchScreen
  Rendering --> OntologyScreen
  Input --> SearchController
  Input --> OntologyController

  App --> SearchScreen
  App --> OntologyScreen
  App --> AppServices

  SearchScreen --> SearchController
  SearchController --> SearchWorkflow
  SearchController --> FilterExplorerWorkflow
  FilterExplorerWorkflow --> FilterExplorer
  SearchController --> SearchService

  OntologyScreen --> OntologyHost
  OntologyHost --> FilterExplorer
  OntologyHost --> OntologyService

  AppServices --> SearchService
  AppServices --> OntologyService
  AppServices --> Runtime
  SearchService --> Data
  OntologyService --> Data
  Runtime --> Data
```

In practice, the responsibilities break down like this:

- screens choose which visual shell to render and pass callbacks around
- controllers derive screen props from state, terminal size, and selected services
- workflows handle async tasks, prompts, cleanup, and multi-step editing flows
- services translate TUI intent into app/backend operations

That split keeps most feature files from mixing rendering code, terminal event interpretation, and backend access in the same module.

## Search Screen As The Best Example

The search flow shows the intended layering most clearly.

`src/tui/search-screen/screen.tsx` is thin. It decides whether to render:

- the normal search screen
- a shared filter-explorer session
- a structured editor session

`src/tui/search-screen/controller.ts` is the orchestration layer. It:

- pulls `user.search` from the app-service context
- creates the reducer-backed screen state
- derives pane sizes, result-reader detail output, and page-document presentation state from terminal dimensions
- coordinates result sessions, structured editor sessions, and shared filter-explorer sessions
- installs the search interaction router

The async work is pushed further down:

- `search-screen/session-workflow.ts` manages live counts, result-window execution, prefetch, sort changes, and session disposal
- `search-screen/filter-explorer-workflow.ts` opens the shared filter explorer as a search-hosted inspect-and-open session for ontology-backed field editing and live query mutation
- `filter-explorer/controller.ts` owns reducer-backed browser state and screen orchestration, while `filter-explorer/workflow-actions.ts` owns command/help/open/compose workflow behavior
- `search-screen/interactions.ts` maps state into terminal actions and help/command models
- `search-screen/query-field-builder/query-field-builder-session.ts` owns the structured-editor menu bindings, footer copy, and help sections so live query-tree screens do not hand-maintain separate action tables
- `list-detail-presentation.ts` now owns the shared measurement and route-setup seam used by search result-reader, filter explorer, and review screens

The search screen is intentionally render-only at route entry:

- editor/browse routes mount from seeded query state
- result-reader routes mount from a prepared session carried on the route
- navigation-origin execution should happen in the navigation layer before commit, not as a search-screen bootstrap effect after mount

This is the pattern to preserve when the TUI grows: keep rendering, workflow state, and backend calls separate enough that each layer can change without forcing a full rewrite of the others.

## Shared Interaction Conventions

The TUI does not just share low-level input normalization. It also shares user-facing interaction contracts so screens can opt into common behavior instead of redefining it.

### Navigation And Binding Model

The expected interaction stack is:

1. shared key normalization
2. shared interaction verbs and actions
3. shared list/detail navigation helpers
4. shared action-target behavior
5. shared help and footer generation
6. shared list/detail presentation assembly where the screen shape matches that contract
7. screen-specific workflow logic on top

That means:

- vim keys and arrow keys should resolve to the same navigation behavior
- feature screens should prefer shared interaction routers and helpers over branching on raw terminal events
- footer and help text should be derived from the same action tables that actually execute on the screen
- list/detail screens should prefer the shared presentation layer over open-coded pane measurement and route-setup glue once their workflow fits that shape

### Action Rail And Picker Commands

The TUI uses the shared action-target rail for screen-level and selection-level actions. Broad page actions should not fall back to a hidden command palette when the same action set can be surfaced through the rail.

- focused action rails are the default fit for constrained, high-frequency action sets on a selected record, result, or editor surface
- discovery-mode changes and other auxiliary selection actions should route through the same shared explorer and action-rail model instead of picker-local command paths
- page-specific one-letter commands should remain rare; new screen-specific actions should usually be surfaced through the shared action rail

### Action-Target Contract

When a screen adopts the shared action-target model, preserve the same focus and exit semantics across surfaces:

- `:` is the explicit entry point for command-oriented interactions
- on action-target pages, `:` enters the action target, and `:` or `Esc` leaves it
- `Enter` applies the selected action
- arrows and vim keys should act inside the focused target only
- persistent action rails should still require explicit entry; users should not move into them accidentally
- `Left` should not be repurposed as a generic exit inside action UIs when it already means horizontal movement there
- `Backspace` should not be treated as a generic exit because prompts and palettes need it for text editing

### Search-Semantics Explorer UX

The shared exploration model also carries a few durable presentation and return-path expectations:

- field and semantic labels should emphasize the entity name; long inline operator lists should not dominate metadata-field views
- derived-tag organization should preserve axis and family structure consistently across search-semantics and scoped query-entry surfaces
- scoped query-field entry should preserve shared explorer state instead of rebuilding a separate picker-only path
- scoped structured-editor discovery steps such as pack selection and metric-key selection should use the same shared explorer surface and action rail as other live field discovery flows
- concrete semantic entities that advertise live record counts should open the normal result behavior instead of a special-case sample view

### Modal Presentation Defaults

Prompt and modal sizing should continue to flow through the shared layout planner rather than feature-local sizing heuristics.

- dialog, text, and command prompts default inline
- select-like prompts default to screen-modal unless explicitly overridden
- narrow forced-inline choice prompts should collapse to a readable single-column body instead of forcing an unusable split layout

## Design Rules To Preserve

- Treat `src/tui/app-services.ts` as the TUI composition root. Do not let screens assemble runtime dependencies ad hoc.
- Keep Ink and terminal-framework details in `src/tui/framework/` and shared interaction helpers.
- Route search behavior through `src/tui/search/service.ts` instead of letting screens call backend search APIs directly.
- Treat ontology models from `src/app/ontology-service.ts` as shared readonly inputs; navigation state belongs in the TUI.
- Prefer controllers and workflows for stateful interaction logic; keep screen components mostly declarative.
- Prefer `src/tui/list-detail-presentation.ts` for reusable list/detail pane mechanics instead of rebuilding sizing, visible-detail slicing, and context wiring in each feature controller.
- When a new TUI abstraction becomes the mandatory path, add or extend lint rules so the boundary is enforced rather than implied.
