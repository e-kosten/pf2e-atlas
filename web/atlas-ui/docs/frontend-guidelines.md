# Frontend Guidelines

The Atlas web UI is a React frontend over generated Rust app DTOs. It uses Ant Design for general application UI and a small first-party shared UI layer for Atlas-specific interaction patterns.

The application shell owns navigation, system-following appearance, and reference availability. It receives readiness facts, not the search workspace. Healthy availability needs no status control. Connection failures show an actionable error with Retry; search failures and generation recovery belong beside search results.

## Boundaries

- `web/atlas-ui` owns browser presentation, local browser state, URL state, API calls, and component composition.
- Rust `atlas-app-model` and `atlas-app-service` own product DTOs and workflow semantics.
- Frontend code should not recreate filter catalogs, retrieval semantics, saved-list behavior, encounter mutation rules, or record presentation contracts that already come from the backend.
- Shared family presentation components render the same generated semantic facts
  in detail, previews, comparisons and encounters. Feature modules compose their
  controls around those components; they do not copy the family layout.

## Shared UI Layer

Use `src/shared/ui` for repeated Atlas interaction primitives over Ant Design. Shared primitives should normally compose Ant components instead of recreating generic control behavior with plain elements and local CSS. Current shared primitives include:

- `tables/IndexTable.tsx`: clickable, keyboard-activatable index tables.
- `pages/EntityIndexPage.tsx`: common index page structure for title, summary, actions, and table content.
- `forms/EditableCommitField.tsx`: inline draft fields that commit on blur or Enter and revert on Escape.
- `pickers/SearchPickerModal.tsx`: modal record browsing and preview, with an explicit candidate and feature-owned Add form.
- `actions/PaneAction.tsx`: pane-header icon buttons and links.
- `actions/DangerActionButton.tsx`: destructive button plus confirmation modal.
- `actions/useConfirmDangerAction.tsx`: confirmation helper for cases that are not naturally buttons.

Feature modules should pass feature-specific labels, requests, mutations, and DTOs into these primitives. They should not copy the primitive behavior into local components.

Create components for a clear responsibility or actual reuse need. Keep repeated
fact formatting and family layouts in shared presentation owners, and reuse those
owners across features. Avoid both feature-local copies of the same design and
trivial component proliferation that adds indirection without reuse or meaning.
Check visual consistency across surfaces, density modes, themes and viewport sizes.

Shared Ant button and link primitives pass native event props and React refs to
the underlying Ant control, preserving its DOM target for Tooltip and Popover
triggers. Keep the Atlas accessible label and default styling while retaining
library-injected events and classes.

## Ant Design

Ant Design 6 is the selected component library. Generic application UI should use Ant components and component tokens before adding custom controls or local state styling. This keeps hover, focus, selected, disabled, loading, contrast, and light/dark behavior in the same design system.

- `Table` for table mechanics, usually through `IndexTable` for entity indexes.
- `Form`, `Input`, `InputNumber`, `Select`, `Checkbox`, and `Button` for forms and edits.
- `Button`, including `icon`, `type="text"`, `type="link"`, `danger`, and loading states, for generic actions.
- `Typography.Link` or Ant `Button type="link"` for link-colored actions.
- `Menu`, `Tabs`, or `Segmented` for generic navigation or mode selection.
- `Modal`, `Popover`, `Tooltip`, `Alert`, `Empty`, `Tag`, and `Pagination` for standard application states.
- Ant Design loading, disabled, empty, and error affordances where they fit.

Wrap or compose Ant Design when Atlas has a repeated product interaction that should behave consistently across surfaces. Keep custom Atlas components for product-specific surfaces such as record presentation, PF2e stat/rendering layouts, rich result summaries when Ant list/table primitives cannot express the layout cleanly, and encounter-runtime panels. Do not hand-roll generic buttons, icon buttons, links, navigation controls, table selection states, overlays, or form controls when an Ant component exists.

When styling Ant components, prefer `ConfigProvider` component tokens such as `Table.rowSelectedBg` and semantic global tokens such as `colorPrimaryBg`, `colorPrimaryBgHover`, `colorPrimaryBorder`, `colorPrimaryText`, `colorLink`, and `controlItemBgActive`. Avoid using `colorPrimary` directly as a foreground text color or low-opacity highlight fill.

## Styling

- Use Atlas CSS variables and Ant Design theme tokens; avoid raw color values.
- Atlas CSS variables should describe semantic app states, such as active background, active border, active text, muted text, and panel surfaces. They should be derived from Ant semantic tokens rather than raw brand tokens where possible.
- Put shared primitive styles under `src/styles/ui`.
- Keep feature-specific styles in the feature stylesheet.
- Do not create nested cards or duplicate pane/index/table styles in feature modules.
- Prefer stable layout dimensions for repeated controls so dynamic labels, icons, loading states, and validation text do not shift the page.

## Validation

Use the package gate for frontend changes:

```bash
npm --prefix web/atlas-ui run verify
```

For refactors, run focused tests for touched surfaces first, then the full verify.

## Source-backed records and filters

Render generated app DTOs. Prepared content is sanitized HTML with field-local reference and interaction sidecars; use DOMPurify at the browser boundary and make supported markers keyboard accessible. Unknown commands and unavailable targets remain visible inert labels. Never execute Foundry commands, scripts, or rules. Selected owner and passage routes carry backend navigation, including its source fingerprint for snapshot-local addresses.

Use react-querybuilder with its maintained Ant integration for structured predicates.
Backend descriptors supply fields, human labels and groups, operators, units,
choices, discovery policy and the ordered common Kind, Level / rank, Rarity and
Traits fields. Common and advanced controls edit one canonical QueryPredicate.
Common categorical controls cycle each value through neutral, included and
excluded in a compact searchable Ant Popover or mobile Drawer, allowing both
included and excluded values in one field. Menu rows show a name and a small
plus/minus marker; their accessible name states the current value and next action.
Closed controls show only active choices with individually removable selections.
Keep the menu open and its native button nodes stable during edits; changing a
row's Ant variant between text and filled remounts its node, so use a consistent
variant with documented semantic token styles. Search/page changes keep bounded
server discovery, while selection-only requests retain the same suggestion rows.
Initial selections and values edited during the open session remain reachable in
stable edit order outside the current suggestion page, including after clearing
them to neutral. Reset that transient history when the menu closes.
Do not represent exclusion as checkbox indeterminate or a boolean aria-pressed.
They recognize
bounded root conjunctions and an exact numeric range group. Advanced editing
shows residual conditions without duplicating common rows or adding a wrapper for
the common-field conjunction. Negation of a conjunction or disjunction uses the
builder group's native exclusion flag, retaining the group and negation identities;
complex expressions
remain editable through the full builder. Structural true/false conditions have
explicit editor controls and lower to boolean constants, never synthetic backend
catalog fields. Preserve clause/scope identities and unrelated clauses. Invalid
input remains a local draft with local guidance, retaining committed results.
URL restoration resets drafts and retains the exact canonical predicate.

Browser-local remembered Kind is only a default for a fresh bare search visit.
An incoming search URL is authoritative, and clearing Kind clears the preference.
Search and encounter selection share `ReferenceBrowser` and `useReferenceReading`.
A compact common-filter toolbar sits above results and the generous reader. Search
occupies its own row; common pickers and Advanced/Clear actions share a wrapping,
top-aligned row underneath, including when selected chips add height. Use
an icon beside the Advanced label. Clear is an icon with an accessible label and
tooltip, shown when criteria or unfinished drafts can be cleared; clearing returns
keyboard focus to the retained Advanced control. Use
native Ant Splitter on desktop; below the lg breakpoint show results or the reader,
with Results returning focus to the initiating row or match control. Keyword
`SearchEntry` edits stay local until Enter or the Search button; filters commit
independently. Advanced filters always use the native Drawer and full catalog.

The whole result summary previews on ordinary click or Space. Arrow/j/k move
focus without changing selection; Enter opens the full reference in the same tab.
Keep native hrefs and modified clicks. Results contain scan metadata and wrapping
traits; rarity is separate metadata. Never dump matched content into the row.
The N matches text action opens a bounded native Ant click Popover, with checked
links and no chevron. Escape returns trigger focus; choosing a match closes it.

Typed parent-visible ability, strike and casting-entry matches land in that parent.
Other owners and precise fields/passages retain the exact addressed reader.
Briefly mark an actual within-parent landing, never the owned document's primary
heading. Search keeps its criteria, page and in-memory result/reader positions
when navigating out and Back. Keep page in browser history state, not a new URL
query dialect. The search owner remains mounted and ignores non-search routes;
scroll state is independent of query caches and mutable encounter drafts. Fixed
viewport search composition and clipped non-scroll hosts keep native landing
scrolling inside the reader. Lists and encounter runtime retain their layouts.

The encounter chooser shares browsing/reading without copying search state or
mutation rules. Following references changes the reader, not the Add candidate.
Provide Return to selection, preserve quantity/initiative through retrieval and
Add failures, and disable competing actions during Add. Only the encounter owner
commits the explicit candidate. Reset the chooser after successful close.

The web product uses hybrid text retrieval and filter-only browsing. Retrieval
options are not browser criteria or URL overrides; CLI modes remain independent.
Page size belongs with results. The advanced editor uses searchable grouped Ant
fields and readable operator labels; value controls stack at narrow editor widths.
Its narrow Ant Drawer preserves native keyboard behavior and focus return.
Do not offer another collection scope inside an existing Exists condition.

Optional contextual counts use the backend's available/unavailable outcomes.
Unavailable counts receive condition-local text; actual request failures remain
errors. Never label retained counts from another context as current. Open values
use explicit count-free vocabulary requests with artifact or saved-list scope,
server search and bounded paging; selected literals remain visible separately
from suggestion pages. Common pickers show concise totals as values, never result
counts, with an optional plain saved-list scope label. Advanced value controls
retain their scoped vocabulary explanation, including legacy entries.
Closed choices use catalog labels without values requests. The adapter does not
parse CEL, infer fields from records, or decide contextual capability in React.
Reject unsafe integer input before creating numeric literals.

Reference previews use native Ant Popover positioning on desktop and the shared
Ant Drawer below the Ant `xl` breakpoint (1200px), including tablet layouts.
Both hosts use RecordDetailPane and its shared family renderer. Encounter preview
detail is composed by Rust from current participant state, with separate cache
identity including the participant presentation and exact selected navigation.
Do not substitute baseline detail or implement a second activity-only reader.
Compact changes density, not information. The document title appears once; generic
Reference labels remain accessible without becoming visible overlay headings.
An Original record link uses only a resolved structured CompendiumProvenance
relationship; full-page links still open the exact local embedded copy.
Source content in an encounter loads only on an
explicit request. HP mutations carry explicit edit intent; unknown current HP
disables arithmetic and remains unknown until explicitly set.

Use matched React 19 runtime and type packages with Ant Design 6. Preserve the
root StrictMode and native overlay keyboard handling. Shared preview tests run
under StrictMode and verify that Escape closes a nested source dialog first,
returns focus to its marker, then closes the preview. Drawer dismissal keeps the
drawer mounted through its native close lifecycle and returns focus to the
original trigger after `afterOpenChange(false)`; the shared preview hook owns
that trigger, including across responsive overlay changes. The parent unmounts
the drawer after that callback; do not also destroy it during the native opening
or closing animation, which can skip the close callback.

Following a reference within an open preview can replace its focused content.
If that element disappears and focus falls to the page body, the shared preview
must move focus to a surviving in-preview reading target. Preserve connected
focus, user-selected focus elsewhere and nested-dialog focus; do not refocus the
header on every content update. Native initial focus and dismissal still apply.

Use the semantic family union rather than a generic section/value/profile layout
interpreter. Availability belongs to the backend facts; known zero, false and
empty differ from unavailable parents. Omit cosmetic empty or not-applicable
sections while retaining meaningful unknown facts. Authored spell forms are
changes to the base, including explicit clears, not calculated effective forms.
Complete owner/field/passage/fingerprint identity drives shared detail queries
and all reader/preview transitions.

Baseline spell requirements, cost and casting-entry facts arrive as absent when
their known typed system contains missing/null or empty/whitespace-only prose.
The source remains unchanged. Render supplied nonempty values and unavailable
facts; spell patches independently preserve explicit clears and empty changes.

Baseline spell area and defense arrive absent only for explicit no-area/no-defense
under a known system; supplied unavailable facts remain visible. Do not show the
raw baseline spell casting-entry identifier as a web fact; checked casting context
supplies parent membership, while JSON and CLI retain the source fact. In the existing family readers,
omit missing price-per and IWR exception/double qualifiers and baseline missing/null
damage categories. Preserve required magnitudes, null/invalid IWR qualifiers and
patch clears. Damage captions describe their presentation order rather than raw
source IDs; keep those IDs for correlation and keys.
Omit headings for known-empty optional content and heightening damage-increment text;
unavailable content/formulas and explicit patch clears remain visible. Spell
forms use their authored names or an ordered “Spell form” caption, retaining
invalid/null name facts separately. Baseline heightening/forms arrive
not-applicable only for a missing optional structure under a known typed system;
unknown system parents retain explicit availability. Do not suppress their
missing state in the browser or apply this exception to patches.

Authored action costs use the shared `ActionCost` renderer over typed Rust facts.
Fixed one/two/three-action, reaction and free-action glyphs are decorative;
their readable label remains accessible. Variable, timed and passive costs keep
their supplied text, and unavailable required costs remain explicit. Spell
patches preserve clear/empty meanings. Physical usage is separate from action
cost. Prepared inline glyph attributes use the same font styling with an empty
CSS content alternative, so font characters do not add spoken text or enter
the canonical/terminal words. Never classify cost strings or authored markup in
the browser.

Use the shared fact reading policy for optional baseline whitespace-only text
and empty lists; omit their row and resulting empty heading. Mark required text
explicitly so empty damage formulas/types and table formula/range/type remain
visible. Preserve meaningful unavailable facts, numeric zero and typed false.
Authored changes omit missing members and show explicit null clears and empty
values, including nested damage qualifiers. Keep existing narrow source defaults
separate from this baseline/change policy; never guess emptiness from prose/HTML.
Physical prices format the supplied currency number facts without inferring free
from empty prices; table results use their one fact-owned link collection.

The shared identity header places the name and selected activity glyph on the
left and the service-supplied compact classification on the right, wrapping to
the actual pane. Show the title glyph once and ability modifiers before skills.
Do not infer game semantics from prose or show ordinary technical keys.
Compact result classification uses the supplied kind and level/rank labels;
removing the Level/Rank display prefix does not create a typed fact. Product record
paths use literal colons through the shared serializer; keep API encoding and
complete selection identity intact.

`RecordDetailPane` owns shared detail loading, unavailable-record, refresh-error,
retry and back states. Keep query errors with their affected pane and feature
mutation errors with their action; do not add detail errors to the global banner.
React Query may retain a valid result during refresh of the same complete address;
never borrow previous detail across a root, owner, field, passage or fingerprint
change. `RelatedRecords` is the shared secondary disclosure inside that pane.

Reference composition puts prose first and groups facts/mechanics according to
pane width. Encounter composition puts current HP, conditions and actions first;
participant edits and maximum-HP configuration live in secondary disclosures.
Immediate Ability-class activities read inline by default through bounded
owned-reading batches, with an optional whole-section Collapse. The existing
activity facts supply the title and action cost once; shared prepared content
supplies authored controls and exact relationships. Keep partial content in its
own full-generation/parent/target cache, never in selected-detail cache. Scroll
progression must use the actual container (including native overlays and comparison),
not viewport visibility alone. Idle visible continuation cannot drain a collection;
collapse halts future requests and late errors retain text with local retry.
Direct late-owner landing loads only its bounded range and exposes earlier/next
continuation honestly. Complete owner anchors and landing state stay distinct
from selected addresses. Native small permalinks and parent breadcrumbs retain
modified clicks and known search/list/reader history context. Compact reference
presentation changes density without suppressing content, facts or availability.

Encounter quick lookup uses the same complete family reader in native
Popover/Drawer. Its separate contextual cache holds the requested document view,
keyed by participant presentation and exact selection. Rust composes current
participant facts and explanations on each request; presentation changes trigger
a fresh request without seeding immutable reference caches.
Participant/generation changes, disappearance and unresolved sources remain
explicit without baseline-detail fallback. Keep the native full reference link,
ready-lifecycle initial focus, nested source-modal Escape and original trigger
focus recovery. Keep secondary
encounter facts below usable actions in a local disclosure rather than placing
all reference facts before the combat actions. Selected casting entries show their checked parent and
associated customized spells; base and authored cast ranks do not imply executed
rank or form changes. A hazard's backend defense-block fact controls its primary
AC/HP/hardness display; keep independent zero saves and unavailable facts intact.

`RelatedRecords` starts closed and requests one bounded availability response on
opening, cached by complete selection and artifact generation. Render only content
categories; keep loading, unavailable checks and local retryable errors distinct
from known empty categories. Similar is a separate root discovery preview using
stateless search pages, with a native link into the ordinary search workspace.
Similarity URLs preserve the root seed and canonical filters. Keyword entry replaces
the seed; clearing it preserves filters. Contextual text counts are disabled while
catalog choices and vocabulary remain available. Label bounded pages Top matches.
Comparison panes reuse family presentation without recursively mounting related disclosures.
Incoming references use checked `source_navigation`, outgoing references use the
checked target, and separate occurrence ordinals remain attribution metadata;
they do not imply automatic scrolling to an individual marker. Counterparts and variants apply only to root selections; they never appear as
relationships of an embedded selection. A separate Similar action can name the
parent root explicitly. Exact root references use an empty owner path, not an
all-owners graph query.

Search page keys include committed criteria, the local page number, the current
artifact generation and the search owner's recovery epoch. An artifact generation
conflict pauses pagination until the user selects Refresh results. Recovery cancels
affected immutable reference queries before removing them, resets to page 1 and
preserves filter input and mutable encounter/list drafts. Never reuse previous-page
data across that transition or seed caches from an aborted or superseded response.
An initial live response seeds its generation-keyed page before adopting that key,
so adoption does not repeat inference. Related and Similar recovery reload the
owning record; contextual encounter previews reload their encounter and require the
user to reopen the activity using its current checked selection. Exact source
fingerprints and snapshot-local ordinals are never silently rebound.

Live browser checks use the maintained Playwright runner against a separately started real Atlas server and isolated local state:

```sh
npm --prefix web/atlas-ui run test:e2e
```

The default server URL is http://127.0.0.1:8765 (optionally overridden with ATLAS_BROWSER_BASE_URL). Build the real frontend and CLI first, then start `atlas web --port 8765 --index <current-sample-artifact> --embedding-cache-path <verified-model-cache>` with an isolated artifact directory so its local state is temporary.

The browser suite does not manufacture readiness, start a mock API, or substitute screenshot success for interactions. Its artifact must contain the current source-grounded sample records; screenshots and traces stay under ignored scratch/browser-validation.

Contextual feedback uses the root Ant `App` inside `ConfigProvider`. Destructive
button and callback flows use `useConfirmDangerAction`; prepared source dialogs
use contextual `Modal.useModal`. Do not use static Ant feedback methods. Style
Ant internals through documented `classNames`, `styles` and component tokens;
Atlas-owned classes describe product layout rather than internal Ant DOM.

## Mutable workflows

Encounter detail and edit routes use `useEncounterMutations` as their route-local
command owner. Keep controls pending through the write and canonical-key refresh;
do not maintain optimistic HP, condition, or participant copies. A successful
receipt followed by a failed read must show Saved with a read-only Retry and
disable edits until current state loads. Typed participant conflicts refresh
without resubmitting and retain the failed draft. Other errors stay with their
action. Initial encounter and condition reads expose local Retry.

Await Save before navigation and deletion before dismissing confirmation. Return
the action promise to `useConfirmDangerAction`; observe its confirmation promise
so a rejected action stays open with contextual feedback. Index deletion owns its
own asynchronous lifecycle; do not add a detail-route owner to each index row.
Disable competing controls while their action is pending. Failed inline edits
retain their draft, and a successful retry releases the field to canonical props.

Preserve touched same-entity metadata during background refresh, and reset drafts
on entity or editing-session change. Key route owners by identity and prevent old
Save completions from navigating a newer editor. Keep source-backed facts and
gameplay derivation in Rust; the browser submits explicit edit intent and revision.
