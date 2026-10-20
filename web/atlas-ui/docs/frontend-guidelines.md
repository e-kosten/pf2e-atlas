# Frontend Guidelines

The Atlas web UI is a React frontend over generated Rust app DTOs. It uses Ant Design for general application UI and a small first-party shared UI layer for Atlas-specific interaction patterns.

The application shell owns navigation, system-following appearance, and reference availability. It receives readiness facts, not the search workspace. Healthy availability needs no status control. Connection failures show an actionable error with Retry; search failures and generation recovery belong beside search results.

AtlasApp owns one tab-local search-profile selection from the generated readiness
catalog. An explicit `search_profile` URL parameter wins over the browser's launch
preference, which wins over the server default. Retrieval waits for initial
metadata; while it is pending, the same selection owner supplies the incoming URL
literal to every workspace serializer so reader or criterion updates retain link
authority. This literal does not enable retrieval. Unavailable explicit choices
remain visible. Native Similar See-more links preserve the same pending literal
for copying or opening another tab while their requests still wait for metadata.
A compact Ant Select at the trailing end of the search input row changes the same
selection inherited by search pickers and
Similar previews, including their native See-more links. Browser preferences are
optional and never switch another live tab. Missing query assets block text
inference while stored-vector Similar and reference reading remain available.
An artifact with one stored profile has no chooser or model-choice copy, including
invalid incoming links. Those links retain their explicit error until the user
chooses `Reset search link`, which removes the unsupported selection while keeping
criteria and reading state. Stale unavailable browser preferences are optional
defaults and resolve to the valid server or sole stored default on launch.
Unbuilt catalog entries are never offered as choices in the comparison UI.
`FilterControls` owns the input row and its trailing control slot; the row wraps
when space requires it without covering the existing `Input.Search` control.
The chosen model name stays visible with an accessible Search model label.
Search without committed criteria shows “Search or choose filters.”; committed
text, a Similar seed or a non-null predicate uses the shared executable-search
policy. No-data requests show centered Ant loading feedback, completed empty
requests show “No matching records.”, and local errors remain distinct. Pagination
requires a returned nonempty page, including in search pickers.

A profile switch resets retrieval paging to 1 in the same render and isolates
request/cache/response identity. It preserves submitted criteria, unfinished
controls, exact reference reading, scroll positions and explicit picker candidates
and Add forms. It does not advance the artifact epoch or retire shared detail,
graph, catalog, vocabulary, saved-list or encounter caches. Contextual discovery
carries profile and known generation; option counts load only for active pickers.

## Boundaries

- `web/atlas-ui` owns browser presentation, local browser state, URL state, API calls, and component composition.
- Rust `atlas-app-model` and `atlas-app-service` own product DTOs and workflow semantics.
- Frontend code should not recreate filter catalogs, retrieval semantics, saved-list behavior, encounter mutation rules, or record presentation contracts that already come from the backend.
- Shared family presentation components render the same generated semantic facts
  in detail, previews, comparisons and encounters. Feature modules compose their
  controls around those components; they do not copy the family layout.

## Shared UI Layer

Use `src/shared/ui` for repeated Atlas interaction primitives over Ant Design. Shared primitives should normally compose Ant components instead of recreating generic control behavior with plain elements and local CSS. Current shared primitives include:

- `PrimaryToolbar.tsx`: top-aligned wrapping primary control rows that inherit the root Ant sizes.
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

Search and saved-list browsing compose `shared/records/BrowseReader` with their
own rows, requests and actions. `RecordBrowseSummary` owns shared record identity
and activation; `ReferenceReader` owns the reader controls and reading position,
including the explicit Reader companion pane. Keep match explanations in search
and membership/order/unresolved snapshots in lists. Do not fabricate search DTOs
to reuse a layout or copy the browsing interaction into another feature.

List identity, metadata and scoped filters belong above the items and reader,
rather than in a permanent third pane. Use a labelled Saved lists parent link,
not a duplicate title switcher. Keep the description summary and tags in one
readable metadata column; only long descriptions need Read more. Index tags use
one compact line and detail tags use up to two, followed by the actual hidden
count when needed. Full tags remain available in the native popup. The index owns
list-name and local include/exclude tag filters and preserves its position on return. Reuse
`ChoicePicker` for the interaction without manufacturing backend filter metadata.
On narrow screens, show results or the reader and preserve the initiating row's focus and position on return.
Collection indexes reuse `EntityIndexPage` and `IndexTable`, with common insets,
wrapping toolbars, navigation actions and date formatting. Keep table overflow
inside its container and record names readable. Loading, an empty collection,
filtered emptiness and a failed read are distinct states; preserve existing data
on failed refresh and provide local Retry.

### Primary toolbars

Compose primary input, selector and action rows with `PrimaryToolbar`. Controls
inherit the default size from the existing root Ant configuration; adjacent
search inputs, model selectors, filter triggers and primary actions use that same
size. Reserve explicit small controls for dense inline facts, table actions and
secondary metadata. Keep dimensions in Ant tokens; do not compensate for size
mismatches with local pixel heights or additional configuration providers.

The shared owner aligns controls at the top, with consistent gaps and responsive
wrapping. Feature composition supplies meaningful widths: let search shrink
before a primary action wraps, and use a full search row on phone where needed.
Selected filter choices may extend beneath their trigger without centering its
neighbors against the taller group. Verify actual input, selector and button
alignment together at wide, intermediate and phone widths in both themes.

Index pages use the shared page toolbar slot below the heading; feature styles
own their responsive search/action priorities.

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

Give authored prose one left-aligned readable column, including its headings,
lists and horizontal rules. Apply it through shared record presentation so panes
and previews agree. Identity headers and mechanics can use their available width;
wide authored tables retain deliberate local overflow rather than widening the
page. Compact density changes spacing, not the available record information.

## Responsive design and acceptance

Preserve the user's task as available space changes. Width, height, orientation
and text size may change composition; selected references, criteria, unfinished
fields, Add candidates and pending mutations retain their identity. Use the shared
browse/reader, picker and preview owners and Ant breakpoints. Choose layout from
available space rather than device names or feature-local mobile implementations.

- Keep prose, headings, traits and controls readable through wrapping and reflow.
  Wide authored tables may scroll locally. Clipping is acceptable only with a
  working way to reach the complete content; hiding page overflow is not proof.
- Check short height as well as narrow width. Primary actions, validation,
  pagination and dismissal remain reachable through ordinary scrolling and
  activation. A visible heading or a nonzero pane height is insufficient.
- Use the same controls for touch and keyboard. Essential actions cannot require
  hover, dragging or precise pointer placement. Preserve visible focus, native
  links and adequate target size/spacing using the shared Ant controls; follow
  [WCAG target-size guidance](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html).
- Keep scrolling in the intended container. Exact activity landing, nested Back,
  overlay dismissal and Results restore the appropriate position and focus.
  Repositioning, breakpoint changes and reduced motion must not restart landing,
  strand focus or lose a pending/draft state. After reflow, check the same content
  anchor rather than demanding an unchanged absolute pixel position.
- Preserve browser zoom. Review 200% text enlargement and reflow at 320 CSS pixels
  without losing content or function, allowing genuinely two-dimensional tables.
  See [resize text](https://www.w3.org/WAI/WCAG22/Understanding/resize-text.html)
  and [reflow](https://www.w3.org/WAI/WCAG22/Understanding/reflow.html).

The maintained `playwright.devices.config.ts` suite samples these contexts:

| Context                             | Appearance/input             | Coverage                                                                                                        |
| ----------------------------------- | ---------------------------- | --------------------------------------------------------------------------------------------------------------- |
| Desktop Chromium                    | Light, mouse/keyboard        | Core journeys, native tab order, 320px reflow, open-reader breakpoint transitions and unavailable query assets. |
| Phone Chromium, Pixel 7 descriptor  | Dark, touch                  | Filter drawer, reader/Results, list editing/Add and encounter reference/recovery.                               |
| Phone WebKit, iPhone 13 descriptor  | Light, touch, reduced motion | The same core journeys with compact height and native drawer lifecycle.                                         |
| Tablet WebKit, iPad Mini descriptor | Dark, touch                  | Core journeys plus fresh result selection and retained Add drafts after resizing an open picker to 844×390.     |

These are representative test contexts, not product breakpoints or certification
of physical devices. Device descriptors come from the pinned Playwright version.
Resize tests prove layout behavior; touch projects must actually tap controls.
Viewport reflow and CSS enlargement proxies do not prove actual browser text
zoom, software-keyboard avoidance, safe-area insets or mobile browser chrome.

For presentation changes, Astra reviews the affected journeys in actual browser
screenshots and interactions across wide/intermediate/phone layouts, both themes
and the relevant input methods. Review hierarchy, density, readability, action
reachability and recovery, alongside automated geometry assertions. Report the
exact candidate, browsers, viewports and interactions exercised. Actual-device
checks, when available, cover typing with the software keyboard, rotation with
an open draft/overlay, browser chrome changes and enlargement; record device/OS/
browser and mark unavailable checks NOT RUN. Browser emulation, visual review,
hosted CI and human acceptance are distinct evidence.

## Validation

Use the package gate for frontend changes:

```bash
npm --prefix web/atlas-ui run verify
```

For refactors, run focused tests for touched surfaces first, then the full verify.

Run `npm --prefix web/atlas-ui run test:devices` for shared layout, navigation,
overlay, form or responsive changes. This starts the real Rust app over a small
checked-source fixture with isolated mutable state; it needs no source clone or
model download. Stored fixture vectors are structural data, not search-quality
evidence. Filter browsing is real; text-inference unavailability stays explicit.
Do not fake readiness/DTOs or introduce a production fallback to pass tests.
Selected network aborts may test recovery while retaining real durable writes.
The existing corpus-backed `test:e2e` suite remains separate. See CONTRIBUTING
for browser installation, CI execution, evidence and resource ownership.

## Source-backed records and filters

Render generated app DTOs. Prepared content is sanitized HTML with field-local reference and interaction sidecars; use DOMPurify at the browser boundary and make supported markers keyboard accessible. Unknown commands and unavailable targets remain visible inert labels. Never execute Foundry commands, scripts, or rules. Selected owner and passage routes carry backend navigation, including its source fingerprint for snapshot-local addresses.

Use react-querybuilder with its maintained Ant integration for structured predicates.
Backend descriptors supply fields, human labels and groups, operators, units,
choices, discovery policy and the ordered common Kind, Level / rank, Rarity and
Traits fields. Common and advanced controls edit one shared builder tree and
project executable QueryPredicates from it.
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
They recognize simple conditions through positive conjunctions and exact numeric
range groups. Advanced displays the complete expression. Individual conditions
combine polarity with the operator, stored in the builder's rule metadata. Group
match controls combine combinator and exclusion while retaining group identities;
complex expressions
remain editable through the full builder. Structural true/false conditions have
explicit editor controls and lower to boolean constants, never synthetic backend
catalog fields. Preserve clause/scope identities and unrelated clauses. Invalid
input remains a local draft with local guidance, retaining committed results.
Successful edits retain the builder tree rather than reconstructing it from the
executable predicate. External changed criteria reset drafts; preview navigation
and local URL updates preserve them. Operator input shapes come from app-service
metadata; apply value transitions atomically before rendering a different input.
Editor metadata remains available for invalid criteria. Valid committed filters
can narrow addable fields using backend applicability for positive conjunctions;
OR and negated draft groups retain the complete catalog so users can broaden the
query. Existing conditions remain editable. Keep displayed options/counts during refresh within the same artifact,
profile and membership scope, replacing them in place when responses arrive.
Do not retain counts after an actual unavailable/error response. The drawer is the
root expression surface; reserve group framing for nested expressions and put
Add condition/group after the group's entries.

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
independently. Advanced filters always use the native Drawer, full catalog and
complete in-progress expression. Common controls edit recognized conditions in that
same query; never hide their conditions in a separate residual editor. Recognize
simple conditions through positive AND wrappers without rewriting unrelated
groups. Conditions whose branching cannot be represented by a quick control use
its `Kind: Advanced`-style opener at the same native Ant size as peer controls. Preserve unrelated unfinished rules
when a common control changes. Incomplete drafts stay local and do not execute
or enter URLs.

Search filter URLs use one versioned, self-contained lz-string encoding of the
existing predicate JSON. Omit clause/scope editor identities and reconstruct
distinct deterministic identities on restore, preserving exact groups, literals
and collection scopes. Field/operator eligibility and query limits stay backend
owned. Keep query text, similarity seed and nondefault page size as ordinary
parameters; derive retrieval mode rather than serializing it. No CEL translation,
saved-token registry or separate simple-filter URL dialect is introduced.

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
scrolling inside the reader. Saved lists share this browsing interaction;
encounter runtime retains its combat-focused composition.

Saved-list Add records uses the same `SearchPickerModal` as encounters. Keep
search, preview and the explicit candidate open through additions; the list owner
commits membership and reports the API receipt, including Already present. Scope
pending writes and invalidation to the submitted list identity.

The encounter chooser shares browsing/reading without copying search state or
mutation rules. Following references changes the reader, not the Add candidate.
Provide Return to selection, preserve quantity/initiative through retrieval and
Add failures, and disable competing actions during Add. Only the encounter owner
commits the explicit candidate. Successful encounter additions keep the chooser
open with Done, retain the candidate for another addition and reset quantity and
initiative only after canonical refresh. Report the completed addition without
adding selection captions or empty-selection prompt rows. Reset the chooser after
close; a committed-write refresh failure retains the submitted draft until Retry
loads current state. Consume that recovery once rather than resetting later edits.

The web product uses hybrid text retrieval and filter-only browsing. Retrieval
options are not browser criteria or URL overrides; CLI modes remain independent.
Page size belongs with results. The advanced editor uses searchable grouped Ant
fields and readable operator labels; value controls stack at narrow editor widths.
Its narrow Ant Drawer preserves native keyboard behavior and focus return.
Within each rule, top-align operation, value and Remove controls independently
of helper text, validation and counts. Keep messages below their value control.
Load query-builder defaults in their CSS layer before Atlas styles; lazy feature
loading must not override the filter row layout.
At narrow group widths, keep operation and Remove together with a full-width
value beneath; use the group's available width so nested rules also fit.
Do not offer another collection scope inside an existing Exists condition.

Contextual option counts are optional annotations for the active categorical
picker's displayed page and retained selections, bounded by catalog limits.
They count distinct eligible parent records for each replacement value, preserving
other conditions, same-child witnesses and edition preference. A multi-selection's
count is for one replacement literal, rather than the union with current selections.
Count requests retain profile/generation identity and cancellation; an incomplete
draft, text search or Similar mode keeps count-free choices. Unsupported target
contexts and count errors leave annotations absent without warnings or disabled
choices. Missing counts never become zero. Catalog loading does not request counts.
Long choice sets use paged contextual vocabulary when supported, with a Show
unavailable option for the full vocabulary. Unsupported contexts use count-free
vocabulary; transport and artifact-identity failures remain errors. Short choice
sets retain declared choices and dim known-zero positive options. Exclusion must
still allow zero-positive-match values. Selected values remain available across
paging and refresh. Small closed selectors use catalog labels locally; long
closed selectors use the same paging path as open vocabulary.
The adapter does not
parse CEL, infer fields from records, or decide contextual capability in React.
Reject unsafe integer input before creating numeric literals.

### Choosing an interaction surface

Choose the surface by the user's task, then adapt it to available space. Reuse
the shared owner for that interaction rather than choosing independently in each
feature.

| Surface                 | Default use                                                                       | Examples                                       |
| ----------------------- | --------------------------------------------------------------------------------- | ---------------------------------------------- |
| Tooltip                 | Short, noninteractive explanation                                                 | Icon labels                                    |
| Popover                 | Quick anchored lookup or small interaction that keeps surrounding content visible | Encounter spell references; match explanations |
| Drawer                  | Substantial secondary task, or reference reading on narrow screens                | Advanced filters; phone reference previews     |
| Modal                   | Bounded decision or edit requiring focused attention                              | HP settings; condition editing                 |
| Existing reader or page | Sustained reading and browsing                                                    | Search result reader; full record              |

Essential content must be available through click/tap and keyboard activation,
not only hover. Escape dismisses the innermost surface, and closing restores
focus to its opener. Preserve the underlying selection, scroll position and
unfinished edits. Use native Ant positioning and lifecycle; keep one shared
record renderer across hosts with navigation appropriate to the context.

Reference previews follow the policy below, including local Back, responsive
hosts and delayed loading feedback. Do not change hosts after loading based on
record length. Extended reading remains available through normal page navigation;
a pinned reader would be a separate product feature.

### In-place reference reading

Use the current reader for selected search results, saved-list members and Add
candidates. Inline document references open the shared reference preview while
preserving that reader and its scroll position: native Ant Popover at `md` and
above, native Ant Drawer below `md` (768px). Use the same quick-lookup interaction
in reference pages and encounters. Explicit Reader view retains its dedicated
secondary reading pane. Following a reference does not change the selected result,
Add candidate, list membership or encounter participant.
Both hosts use `RecordDetailPane` and the same family renderer. Long records
scroll inside their host; do not switch hosts after loading based on record
length or family. Inline owned abilities follow their existing reading policy;
arbitrary linked documents do not become inline expansions. The title appears
once; generic Reference labels remain accessible without visible overlay headings.

Top-level entity links, permalinks and Open full page actions navigate normally
in the same tab. Resolved in-place document links carry the exact native href and
intercept only unmodified activation; preserve modified and middle clicks.
Within a reading context, document references follow one policy. Tooltips are
for short noninteractive hints, not documents; small fact explanations can use
accessible Popovers without becoming document-reading sessions.

Preserve complete backend navigation, including embedded owners, field/passage,
fingerprint, selected spell rank and landing. Rank changes update the current
document through a separate rank-change callback; they are not reference-link
activations. Full-page links open the exact selected copy. Original
record is a separate action offered only for a resolved structured
`CompendiumProvenance` relationship and opens its page at the selected rank.
Encounter preview detail remains composed
by Rust from participant state, with separate presentation/address cache identity;
never substitute baseline detail or add a second activity-only renderer.
Unavailable references remain visible inert labels, without guessed destinations.

Follow nested references in the same reader or preview rather than stacking
document overlays. Back restores the prior exact address and scroll position;
final Close restores the original opener. Reading does not commit a mutation,
change an Add candidate or discard drafts. Keep feature forms and their
pending/error lifecycle in their feature owner.

Transient record previews omit related-record sections in every context,
including their narrow-screen drawer presentation. Full record readers retain
relationship discovery. Prepared HTML links use the same semantic link color as
application links in both themes, including visited links. Shared result rows
emphasize identity and kind/level; match evidence is a secondary control. Rarity
uses the same labeled badge in result rows and record headings, separate from
traits. Bounded search totals read as top matches, not exhaustive record counts
or internal passage-window sizes.

Activity summaries link to a deeper view only when the service's
`has_reference_details` flag identifies additional reference information. Use
that same contract in regular readers and encounters. Fully expanded inline
abilities retain their explicit permalinks for direct addressing. References in
source prose, including effect-application references, are a separate policy and
remain intact pending the effect-reference research.

Use matched React 19 runtime and type packages with Ant Design 6. Preserve root
StrictMode and native Ant focus/Escape handling. Escape closes the innermost
source-information dialog first, returning focus to its marker; the next Escape
closes the preview. Keep a closing Drawer mounted through
`afterOpenChange(false)`, then restore the original trigger. Do not destroy it
during native animation or skip that callback.

When reference navigation removes focused content, move focus to a surviving
reading target without stealing connected, intentional or nested-dialog focus.
Initial desktop reference reads delay the trigger's loading indicator by 500 ms
using Ant's built-in delay. Open the content-sized native popover as soon as the
record or error arrives; never delay ready content to display an indicator. Do not
open a short loading panel that jumps as content loads. Pending reads remain dismissible
with Escape or an outside click; visible popovers and phone drawers retain native
dismissal and positioning. Moving focus into a preview must not scroll its owner.
When viewport resizing reflows an open anchored preview, preserve its visible
opener using native scrolling; keep tablet and desktop encounter pane scrolling
consistent. Phone drawers retain their underlying reading position.
Use the actual reader/preview scroll container, contain overlay scrolling and
retain reading/results positions. Narrow Results returns focus to its initiating
row. Reuse the shared reader/preview owners and native link activation guard;
do not create feature-local overlay or navigation policies.

### Record presentation

Use “Original” in product labels for source values before encounter adjustments.
Keep technical terms such as “authored” in implementation and architecture
discussion rather than user-facing copy.

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
Same-document view updates retain the last successful content through loading and
failure, preserving layout, scroll and focus. The shared `useRetainedRecord` owner
retains display data only, without seeding another query's cache. Rank changes do
not create reading-history entries. Show loading feedback after 500 ms without
inserting a row that shifts the content; errors explicitly label retained content
and offer Retry. Never borrow previous detail across a root, owner, field, passage,
fingerprint, artifact refresh epoch or encounter-context change. An explicit
generation mismatch invalidates retained content. `RelatedRecords` is the shared
secondary disclosure inside that pane.

Spell headers show classification once, a compact rank selector and a
Base rank caption below it when the selected rank differs from base. Reset rank
sits beside that caption, leaving classification adjacent to the selector. Reset rank restores the opening casting rank for an
owned spell or the base rank for a standalone spell. Its reserved space prevents
header movement. Selecting a rank is permitted for prose-only heightening too;
do not claim that unstructured rules have been calculated. Annotating prose as
applicable requires reliable source semantics, not just an ordinal heading.

Reference composition puts prose first and groups facts/mechanics according to
pane width. Encounter composition puts current HP, conditions and actions first;
participant edits and maximum-HP configuration live in secondary disclosures.
Immediate Ability-class activities read inline by default through bounded
embedded-reading batches, with an optional whole-section Collapse. The existing
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
focus recovery. Later encounter facts use visible Languages and compact Ability
modifiers before Skills, with the shared family availability/empty-value policy.
Keep these useful reference facts after combat actions without an opaque
catch-all disclosure. Selected casting entries show their checked parent and
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

### Dice presentation

`shared/dice` renders backend outcomes. `DieFace` owns static Simple/Shaded SVG
drawing and `motion.ts` owns Spin/Tumble/Rock/None lifecycle. Shaded plus Spin is
the product default; drawing and motion are independent options. Reduced motion
bypasses animation. Motion never determines faces, changes totals or submits
requests; strategies settle to their resting transform and cancel on unmount.
The evaluator's arithmetic tree preserves individual faces and discarded dice.
No physics or Three.js runtime is needed for the static projected assets.
`appearance.ts` owns the shared defaults. Nonstandard dice use the d10 drawing
with a visible actual-sides label (for example, d78), preserving the backend's
formula, range and result. Feature consumers do not choose different defaults.
Clicking an encounter attack bonus or damage expression opens its result popover
and rolls immediately. Reroll is explicit; opening via an effect or rerender must
never submit a roll. Fortune/misfortune changes apply to the next deliberate roll.

Initiative uses the encounter mutation owner and backend-resolved choices.
Preserve a request ID and unchanged intent for retry after an uncertain response.
A new deliberate roll gets a new ID. A batch shows participant-attributed results;
there is no combined batch total. Ordinary manual initiative entry remains valid.

Source content in an encounter loads only on explicit request. HP mutations
carry an explicit action, raw integer/addition/subtraction input and the read
participant revision. Rust interprets formulas, damage/healing and correction
intent. Damage and Heal accept signed raw amounts; Rust reverses the operation
for a negative result. Enter in participant HP change submits Heal intent once
(positive heals, negative damages). Enter remains inert in the Damage-only turn
review. Unknown current HP disables amount operations
and remains unknown until explicitly set. Display-only HP meter ratios stay local.

The roster presents initiative, participant and HP in compact labelled columns,
with distinct selected, current-turn and defeated states. Accessible movement
actions disclose the target initiative. Phone layouts drill into a selected
participant and restore roster focus/scroll on Back. Advancing the turn follows
the canonical server-selected participant in the reader; ordinary selection does
not change the turn. Use distinct Encounter settings, Participant settings and HP
settings. The participant pencil opens name, initiative, side, adjustment and
notes, with subdued Original record access inside that editor. Defeated belongs
beside HP; only applied Elite/Weak appears by classification. Mounted Ant forms need distinct
names so their visible labels focus the correct inputs.

Whole-list deletion and saved-member removal use the shared contextual danger
confirmation and await their action. Keep membership/order semantics in the list
owner; do not reconstruct undo from a partial display snapshot.

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

### Encounter reading context

Encounter composition permits collapsing the roster only; its participant reader
stays available and fills the remaining pane. Shared `WorkspaceLayout` accepts
explicit collapsible pane identities without changing other browsing defaults.
Encounter preview hosts explicitly carry encounter context. Both wide popovers
and phone drawers omit owner-return captions and Related records; actual
reading-history Back and Original record provenance remain independent. Direct
record pages retain owner context and Related records. Drawers use their native
header and close control, with the document title shown once.
Creature Lore rows use the authored title and signed modifier once. Actor spell
lists display known authored casting rank, preserving an unavailable casting
rank explicitly; base rank and authored heightening remain in the opened detail.
The browser does not calculate selected-rank spell effects.

Participant settings uses one labeled Save/Cancel form and the complete guarded
participant update. Edit HP shows Current, Maximum and Temporary fields; Save
submits touched raw inputs in one atomic Rust HP action. Omitted fields retain
their values and provenance. HP change submits raw formula text through explicit Damage/Heal
buttons or the participant Enter shortcut described above. Original record opens
immutable authored data through the shared record renderer.

Condition Add defaults to explicit No duration and never requires timing unless
the user enables a round duration. Add and Edit show the whole form, with adjacent
duration amount/unit and Count down at before Whose turn. Numeric controls
validate whole, nonnegative values before conversion; selecting No duration
removes the duration. Full condition drafts retain their opening encounter revision. After a
conflict, explicit Reload condition replaces the draft with canonical remaining
duration and a fresh revision. Do not silently refresh a retained draft's guard.

Next turn reads the Rust turn review and opens a review only for its reminders.
The browser never ticks durations, chooses timing anchors, infers persistent
condition identity, rolls damage/recovery, or removes an expired condition.
Immediate review actions target the reminder's participant and exact condition,
show a saved receipt, and refresh both encounter and review before Continue.
Cancel leaves already saved actions intact. Removing the last reminder does not
advance; Continue uses the current review revision in the atomic advance command.
Read-only Retry never repeats an action or advances a turn.

Encounter roster handles use the sortable-list library for whole-row movement,
touch and keyboard pickup/movement/drop, then submit one guarded reorder command.
Current turn uses the strong filled row treatment; preview selection uses a
separate outline. Play and confirmed
delete remain direct row actions. Fade defeated and disabled-hazard names and facts without dimming
controls. Hazard Active/Disabled controls live beside the same participant HP/status
controls; no HP display is required to disable or reactivate a hazard. Encounter-owned section spacing belongs under encounter styles; shared
record CSS must not override HP borders, padding or encounter control spacing.
HP and Conditions share a row when the reader has room and stack on narrow panes;
HP identity/value and action groups align left and right. Creature-level facts,
including ability modifiers/skills, precede embedded activities. In encounters,
the Rust-owned has_reference_details flag controls activity-title links: known
complete summaries stay plain, while additional content/richer views open the
shared preview. Condition-specific manual notices attach by exact condition ID;
do not repeat those notes as generic Runtime diagnostics.

### Saved-list entry notes

The list feature supplies a footer to the shared browse reader. It stays below the
scrolling reference content and limits its own height for long notes. Note state is
keyed by canonical list identity and entry key. Inspecting another record hides the
footer while retaining its draft; returning to the selection restores it. Save and
Cancel restore focus to the entry's Edit/Add note control. Note mutations update
their cache from the committed receipt before invalidating list reads.

Lightweight entry removal uses the shared danger button's controlled popover.
Pending writes block dismissal; failures retain the confirmation and local error
for explicit retry. Whole-list deletion retains its modal confirmation.
