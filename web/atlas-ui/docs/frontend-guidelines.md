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

Browser titles use `page name · Atlas`, with no query, result count, section
hierarchy or secondary-preview name. Primary route components use `usePageTitle`;
list and encounter readers keep their host's saved name. Standalone records and
traits use their displayed title. Index pages use Search, Lists or Encounters.

Saved-list contents use lexical text filtering and structured filters, preserving
saved order. Main search and Add records retain semantic discovery.

## Design principles

Choose by the user's task and the content's meaning. Each rule below has an
intended use and a concrete failure case; existing UI is not automatically a
positive example. Linked sections define detailed behavior and shared owners.

| Decision                 | Rule                                                                                                                                                                           | Use                                                                                                                                                                                                                               | Avoid                                                                                                                                                                                           |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Component ownership      | Use Ant for generic UI, shared Atlas compositions for repeated behavior, and product-specific layouts for game facts.                                                          | Reuse `RecordReferenceDrawer` for linked records. See [Shared UI Layer](#shared-ui-layer).                                                                                                                                        | A feature-local record overlay or a custom button that recreates Ant behavior.                                                                                                                  |
| Padding and spacing      | Give each boundary one inset owner; children own internal gaps. Align header, body and footer insets.                                                                          | Drawer body padding around flush record content; 8px control, 16px group and 24px section gaps; normal gutters 24px desktop/16px phone. See [Padding and control-group ownership](#padding-and-control-group-ownership).          | A padded Card inside a padded Drawer, missing body inset, or negative margins to repair nesting.                                                                                                |
| Control groups           | Adjacent inputs and buttons share size and aligned height; importance uses variant.                                                                                            | Default-sized toolbar controls; a separate dense row may be uniformly small.                                                                                                                                                      | A small secondary icon button beside a default-sized text action, or local pixel-height repairs.                                                                                                |
| Fact groups and tables   | Group facts by what the user reads together; use tables for a repeated comparison axis.                                                                                        | Hidden Pit separates durability, saves and IWR; Fighter uses Level → Advancement rows. See [Grouping reference facts](#grouping-reference-facts).                                                                                 | One wrapping run of unrelated facts, a card for every number, or a table for a short price/bulk/usage group.                                                                                    |
| Reading width            | Bound the complete single-column document; keep workspace chrome attached to its pane.                                                                                         | Allfood title, traits, facts and prose share a readable column within the full search pane. See [Styling](#styling).                                                                                                              | Capping prose alone under a wider identity block, stretching prose across the screen, or adding filler to occupy space.                                                                         |
| Entity hierarchy         | Lead a single-entity page with its identity; group related metadata below it.                                                                                                  | Allfood's name, classification/rank, aligned traits and attribution.                                                                                                                                                              | A generic Reference bar above the name or detached trait pills floating beside unrelated controls.                                                                                              |
| Reading and editing      | In reading/scanning views, keep values readable with a persistent cue for editing; reveal the editor on activation. Keep fields visible when entry is the primary task.        | Current/temp HP use subtle neutral filled buttons; spell rank has a visible Change rank action. See [Reading and editing values](#reading-and-editing-values).                                                                    | Permanent number fields throughout a read-oriented panel, or editable values indistinguishable from static text until hover.                                                                    |
| Actions and copy         | Keep frequent actions near their context. Use named icons for familiar compact actions and text for unfamiliar choices; each note must add a fact, consequence or next action. | An accessible icon with a concise tooltip; base rank shown inside the picker. See [Choosing an interaction surface](#choosing-an-interaction-surface).                                                                            | Cryptic unfamiliar icons, tooltips repeating clear labels, duplicate Base rank captions or a checkmark repeating selected-row fill.                                                             |
| Disclosures              | Keep essential rules inline; disclose secondary material only for a selective-reading task, with a clear title and enclosing boundary.                                         | Bordered Ant Heightening details around related extra facts, with authored heightening prose still inline. See [Record presentation](#record-presentation).                                                                       | Bare expanding text, nested accordions, hiding essential rules, or adding a disclosure merely because another section uses one.                                                                 |
| Rules and current values | Distinguish selected results from their rules; preserve qualifiers, clears and unavailable facts.                                                                              | Fireball's current damage changes with rank; its per-rank increase stays constant.                                                                                                                                                | Presenting a constant increase as current damage, or inferring that Animal Form's replacement statistics stack like Sound Body's benefits.                                                      |
| Updates and failures     | Preserve controls, focus, drafts, scroll and valid same-identity content; keep loading/failure local and allow deliberate content growth.                                      | Initial filter placeholders occupy control slots; rank changes preserve the trigger; retained content has a local refresh error and Retry.                                                                                        | Replacing search chrome with a spinner, remounting on value changes, treating unavailable data as empty, borrowing another identity's content, or clipping substantial detail to freeze height. |
| Navigation and focus     | Reuse shared navigation and exact native destinations; restore the initiating context.                                                                                         | Linked records open one Drawer with local nested history; full-page/modified links stay native; initial focus goes to the heading and close returns to the opener. See [In-place reference reading](#in-place-reference-reading). | Stacked document Drawers, guessed destinations, a separate reader mode, focus jumping to Add to list, or resetting host scroll.                                                                 |

For affected rules, verify the result in the running UI:

- Inspect computed insets and actual adjacent control heights; matching props or
  token names alone do not establish alignment. Check wrapping at intermediate
  widths as well as desktop and phone, in both appearances.
- Edit a small value and delay or fail its refresh. Confirm the same control,
  draft, focus, reading anchor and surrounding geometry survive where identity
  permits retention; show retained content distinctly from successful updates.
- Open optional details and substantial content. Confirm essential facts remain
  visible and all expanded content and actions remain reachable.
- Exercise keyboard activation, nested Escape, close/return and exact native links.
  An editable value needs an accessible action name and visible focus treatment.

These rules do not prescribe a universal character count or force complex kinds
into a prose layout. Creature and roll-table layouts require individual review.
The usefulness of a particular optional section remains a separate decision from
its styling. Review overlay dismissal in ordinary, dirty, pending and nested
states; do not remove a mutation guard to make every overlay dismiss identically.
Inline-expression evaluation belongs to its separate workstream.

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
for the selected result. Linked documents use RecordReferenceDrawer. Keep match explanations in search
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

### Reading and editing values

On surfaces primarily used for reading or scanning, present editable values as
readable content with a persistent visual cue for editing. Reveal the editor on
activation. Hover and keyboard focus reinforce the cue rather than introduce it;
the interaction must be recognizable without a mouse. Keep fields visible when
entering or adjusting values is the primary task, such as the HP change amount
or encounter setup forms.

For compact single-value edits, make the value itself a quiet button. Current and
temporary encounter HP use Ant's neutral filled Button with the surrounding
value typography; maximum HP remains plain text because it has no inline edit.
Keep the cue subtle but visible in both appearances, with an adequate target and
clear keyboard focus. Do not make read-only values look editable. The two failure
cases are a row of permanent number fields that dominates scanning, and plain
numbers whose editability appears only on hover. Extra pencil icons and helper
copy are unnecessary when the button treatment already supplies the cue.

`EncounterHpValue` composes the shared `EditableCommitField` for this interaction.
Keep keyboard activation, an accessible Edit label, Enter/blur commit and Escape
cancellation. Preserve drafts during pending reads and show validation or mutation
failures locally. Do not put an extra Change HP action or modal in front of this
existing quick edit. A substantial edit involving several related decisions can
use a dedicated form or overlay; presentation does not bypass confirmation,
validation or backend mutation policy.

A named adjacent action is another visible editing cue: reference spell rank
stays readable beside Change rank, which opens its choices. Do not leave a Select
permanently in the identity block. This is the accepted spell composition, not a
requirement to add a Change button beside every editable value.

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

Calculation explanations use the shared `CalculationTable`: cause on the left,
value on the right, with the resulting value emphasized at the bottom. Column
alignment separates cause and value without colons. Keep overridden effects
visible but muted with a dash in the value column; use the backend classification
rather than recomputing stacking rules in the browser. Preserve useful modifier
types and contextual notes alongside their cause.

## Styling

- Use Atlas CSS variables and Ant Design theme tokens; avoid raw color values.
- Atlas CSS variables should describe semantic app states, such as active background, active border, active text, muted text, and panel surfaces. They should be derived from Ant semantic tokens rather than raw brand tokens where possible.
- Put shared primitive styles under `src/styles/ui`.
- Keep feature-specific styles in the feature stylesheet.
- Do not create nested cards or duplicate pane/index/table styles in feature modules.
- Prefer stable layout dimensions for repeated controls so dynamic labels, icons, loading states, and validation text do not shift the page.

Give single-column record pages one document width, including their toolbar,
identity, traits, facts, prose, lists and horizontal rules. Bound the whole sheet
instead of cutting the prose short inside a wider sheet. Search/list browsing
has a different host: its selected-reader pane fills its split region, with a
continuous background and workspace toolbar attached to the results divider.
Center the complete single-column document inside that pane with a readable
maximum width and one content inset. Identity, traits, facts, prose and related
content share this document width. The pane background and workspace toolbar
continue to fill their region; do not center or constrain the pane itself as a
floating sheet. Use the same shared reader policy for search and saved lists.
Most record kinds retain a simple reading flow; wider panes do not justify filler
or a permanent supporting column. Complex kinds such as creatures and roll tables
may use their width for meaningful structures after individual review.
Long journal pages can use a narrow sticky outline to the right when the host can
fit it beside the existing prose measure. The outline supplies a real navigation
task; it does not justify stretching prose. Keep the outline above the content in
narrow hosts, including Drawers, and preserve exact native section links and local
focus/scroll behavior. Decide fit from the actual host: a wide desktop viewport
can still contain a narrow search reader or Drawer. Check the outline after
scrolling, not only at the top; its links must remain keyboard reachable, and a
tall outline must remain usable within the viewport. Keep focus on the target
heading after a jump and preserve the exact section destination for native links.
Do not reserve an empty outline column on short pages.
This follows the [USWDS in-page navigation pattern](https://designsystem.digital.gov/components/in-page-navigation/);
Atlas chooses the responsive fit and keeps its established reading hosts.
In a reference Drawer the document fills its body.
Simple records use one identity flow: title/action glyph and document actions,
then classification/rank on one row, traits on their own row, then source
attribution. Use this identity structure across non-creature kinds and reading
hosts, rather than maintaining spell/equipment exceptions. Put this default in
DocumentIdentityHeader, not a family-name allowlist or a remembered opt-in flag.
Treat this as an identity composition, not a requirement to force result summaries
or encounter roster rows into the same vertical stack. Spell identity is static
text beside a muted Change rank menu.
Do not reserve an invisible reset slot or repeat base-rank copy outside the rank
menu. Let the title and action group share a row when they fit, including on phone; wrap for
actual content rather than forcing every narrow header into separate rows.
Use the same muted `Source: <publication>` caption for every record kind and
reading host; do not invent a separate equipment or Drawer attribution style.

Equipment puts supplied price, bulk and usage before subtype facts and complete
authored rules. Longsword shows base damage; Frost Brand keeps base damage and
runes separate rather than implying a calculated attack. Preserve Steel Shield's
authored durability table; omit duplicate numeric summaries only when the service
confirms matching table values. Kit contents retain nesting, while their prose
keeps qualifications such as rope length and ration duration. These compositions
own internal gaps; the page, pane or Drawer still owns the outer inset.

Spell reading order is identity, supplied casting facts and damage, authored prose,
then structured heightening/forms. For example, Allfood's Cast and Target are
available before its description and authored Heightened paragraphs. Preserve
both authored prose and typed heightening; do not parse or deduplicate one against
the other. Omit the renderer-added Description heading when a simple record has
one principal description field. Battle Medicine then reads Prerequisites,
authored Requirements and prose without a competing generic heading. Keep all
authored headings and other content-field labels, and label multiple description
fields separately. Retain the field label for unavailable prose so the affected
content remains clear. Hazards keep the constrained document flow with essential
rules inline. Creature grouping and width require their own review; a shared
actor data family does not justify reusing a wide creature layout for hazards.
Wide authored tables retain deliberate
local overflow rather than widening the page. Compact density changes spacing,
not the available record information.

### Padding and control-group ownership

Use semantic spacing roles: 8px for neighboring controls, 16px for related groups,
and 24px for major sections. Ant-derived --space-control, --space-group and
--space-section supply these roles. Page gutters and comfortable bodies normally
use 24px, reduced to 16px on narrow screens. Full-bleed workspaces delegate the
inset to their toolbar/pane bodies rather than adding a second page inset.

Every boundary has one owner. DocumentPagePane owns a standalone page's inset
around its composed identity and flush detail. RecordDetailPane owns the body
inset inside a selected-results reader. A reference Drawer body owns its inset
and the contained detail is flush. The
page/pane host owns the surface background, border and shadow: reusable detail
content must not bring another card into an overlay. For example, Battle Medicine
in a Drawer has one reading surface, not a padded surround and nested card. The
shared RecordPresentation arranges facts/prose and adds no outer padding. An
encounter pane owns the inset around its product-specific content. Header/footer
insets align with their body. A section supplies internal gaps, not another body
inset. Do not repair nesting with negative margins or arbitrary local padding.

A single-entity page starts with the entity name and its actions. Do not put a
generic Reference or Details bar above Allfood's title or a trait definition.
Keep the action component mounted while the identity arrives or refreshes, so an
open saved-list dialog survives the read. Use pane labels where they distinguish
actual regions, such as search results and Preview.

Standalone record and trait pages use native document scrolling. DocumentPagePane
and its card grow with the content; do not constrain the card to viewport height
or give it an independent vertical scrollbar. Wheel scrolling over the surrounding
margin and PageDown must move the same page, and the end of a long record must
remain reachable. Search results/readers and reference Drawers retain bounded
scroll regions because they coexist with other reading context; preserve their
landing, close and Back boundaries. A wide authored table may scroll horizontally
inside its own region without moving the surrounding prose.

Apply Ant region insets through its documented semantic styles, using the shared
spacing variables. An Atlas class alone can lose to the library's selector. The
reference Drawer's content-sized header must not flex-shrink: its bounded overflow
keeps a long identity reachable while the body scrolls independently. Verify the
title, close action and computed body/header insets after opening on phone and
desktop; DOM presence and focus assertions do not establish visible geometry.

Paired browse headers use BrowsePaneHeader. Its inner row owns control spacing
and padding; BrowseReader owns the common header height through
useBrowseHeaderAlignment. Measure natural rows so wrapping can grow both headers
and unwrapping can shrink them again. Do not set independent fixed heights or
match headers with feature-local padding. On phone only the visible pane needs
its natural height. The results and Preview separators must align at each split
width, including multi-row pagination and loading/empty results.

One action/control group uses one Ant size. Inputs, Selects, text buttons and
icon buttons in a toolbar align at that size; icon-only and secondary do not imply
small. Express importance with button type. Independent dense row/inline groups
may uniformly use small controls. Do not add pixel-height repairs or per-control
providers. Review rendered control heights, inset and targets together at wide,
intermediate and phone widths in both themes; matching size props is insufficient.

Keep essential facts inline. An optional collapse needs a selective-reading task,
such as a long reference list or nonmechanical creature narrative during combat.
Do not hide mixed source prose merely because its label says description. Prefer
ordinary headings/scrolling when the task requires most sections. Use an aligned
full-width Ant header, explicit expanded state and consistent body inset; preserve
open state on refresh. No nested or automatically expanding collapses. Source
narrative normally remains inline on its full reference page.

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

Condition refinement checkpoints always compare tracked platform/project pixel
references in the normal device suite. Keep references frozen during refactors;
inspect actual/expected/diff evidence before changing them for an approved visual
change. Use the focused Linux runner in CONTRIBUTING.md to reproduce WebKit
failures against the same candidate without repeating full gates. Pixel checks
supplement the interaction, focus, anchor and transition assertions.

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

Search filter URLs use one versioned, self-contained compact tuple encoding of
the existing predicate, compressed with lz-string. Version 3 uses short node tags,
a bare key for root-record selections, and fixed optional slots for exact embedded
addresses and reading context. Trailing empty slots are omitted; direction, scope,
passages, source fingerprints and spell ranks retain their full meaning. Use a
URL-safe alphabet so compression output needs no additional percent escaping. Omit clause/scope editor identities and reconstruct
distinct deterministic identities on restore, preserving exact groups, literals
and collection scopes. Field/operator eligibility and query limits stay backend
owned. Keep query text, similarity seed and nondefault page size as ordinary
parameters; derive retrieval mode rather than serializing it. No CEL translation,
saved-token registry or separate simple-filter URL dialect is introduced.
Malformed or unsupported filter transport is cleared atomically, with one contextual
Ant message explaining that filters were cleared. Replace the current URL entry,
reset paging and preserve independent criteria and exact reading navigation.
Decoded predicates remain subject to backend validation; field/record errors and
retrieval failures never trigger automatic clearing.

Result ordering uses one shared Ant Sort popover beside count and paging,
including in search pickers and the Trait Similar tab. Browse defaults to Name
A–Z, text/Similar to Relevance, and saved lists to Saved order. Keep requested
order optional: a response's effective default must not turn into explicit URL
intent. Selecting the displayed default deliberately does record intent.
Explicit attribute orders survive criterion changes.
Order changes reset paging and belong in request identity and native navigation,
but not filter-count context. A sort choice alone leaves an empty search idle.
The backend orders the entire eligible matching set before pagination; never
sort loaded rows in the browser. Semantic totals retain their Top matches basis.
Keep the word Sort visible alongside the active order. The popover has Sort by /
Then by rules with field, direction, removal and accessible priority controls;
reveal additional rules through Add another sort. Each field appears at most once.
Relevance, Random and list-only Saved order replace the attribute sequence.
Random generates a u32 seed on selection. Keep Shuffle beside its seed input
inside the popover, preserving the toolbar anchor when switching modes. Seed
editing/copying also stays inside the popover. Put the Copy seed icon at the
field’s trailing edge with a tooltip and brief Copied feedback; keep the field
and Shuffle on one row. Commit seed drafts on
Enter/blur. Repeated sort URL tokens preserve rule priority; Random's seed remains
stable through paging, reload and Back. Incoming malformed combinations are errors.
Response identity checks compare complete sort values structurally. Retain loaded
rows and titles with updating feedback during order changes; old placeholders
never cross seed/criterion/scope/profile/generation boundaries in the bounded
picker/Trait consumers. Keep pager controls wrapped within narrow panes, with
Previous/page/Next together.
Saved-list attribute and Random orders are temporary, preserve drafts, and disable
drag reordering. Render supplied unit-price labels verbatim in existing row
metadata whenever any attribute rule orders by price.

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

Condition actions belong beside their condition's edit/remove controls. Keep
recovery rolls and results anchored to those controls, including when success
clears the condition. Do not introduce general action groups or disclosures
between a participant's identity and HP. Pending damage decisions belong beside
HP controls; shared overnight recovery belongs with participant actions.

Apply the root AGENTS.md Product UI Controls principles first. Start with a direct
action or inline control; use an anchored popover when a small action needs context.
A modal requires a substantial editing or focused-decision need.
Choose the surface by the user's task, then adapt it to available space. Reuse
the shared owner for that interaction rather than choosing independently in each
feature.

| Surface                 | Default use                                               | Examples                                               |
| ----------------------- | --------------------------------------------------------- | ------------------------------------------------------ |
| Tooltip                 | Short, noninteractive explanation                         | Icon labels                                            |
| Popover                 | Small anchored interaction or fact explanation            | Roll results; calculation explanations; match evidence |
| Drawer                  | Contextual document reading or substantial secondary task | Linked references; Original record; Advanced filters   |
| Modal                   | Bounded decision or edit requiring focused attention      | HP settings; condition editing                         |
| Existing reader or page | Sustained reading and browsing                            | Search result reader; full record                      |

Essential content must be available through click/tap and keyboard activation,
not only hover. Escape dismisses the innermost surface, and closing restores
focus to its opener. Preserve the underlying selection, scroll position and
unfinished edits. Use native Ant positioning and lifecycle; keep one shared
record renderer across hosts with navigation appropriate to the context.

Idle read-only dialogs and confirmations close with Escape, the close/Cancel
control, or the backdrop. Backdrop dismissal cancels a destructive confirmation;
it never submits it. During a submitted write, disable dismissal, competing edits
and repeat submission until the write and its required canonical refresh finish.
Make the blocked state visible on Cancel/close as well as the submit button.
Restore dismissal after a failure, retain the relevant draft/choice, and put the
error beside the operation with its retry action. For example, Add to List retains
the chosen list on failure; a destructive confirmation retains its original
consequence alongside the local error instead of also showing a page banner.

Draft retention is task-specific: New List preserves its unfinished name/tags
when dismissed; Add to List clears its selection on explicit cancellation.
Do not impose a shared draft-reset policy. A nested Select or source dialog owns
the first Escape; the containing surface stays open. Closing an anchored popup
returns lost focus to its trigger without overriding focus deliberately moved to
another control. Pass the outgoing popup root to
`restoreOverlayFocus(trigger, popupContent)` for that close-time behavior; dice
surfaces retain `useDicePopoverFocus` for entering and leaving their result task.
Test focus and scroll after animations, including touch activation.

Unavailable choices are not an empty catalog. Add to List shows a local Retry
when lists cannot load and keeps supplied choices on a failed refresh with a
retained-content notice. Only a successful empty catalog shows No saved lists yet
and a link to Lists. Optional tag-suggestion failure does not block New List.
Bound selector tracks (`minmax(0, 1fr)`) so a long chosen name cannot stretch its
modal; the Select menu must still make the complete option reachable. List
forms use `transitionName=""` to avoid measuring a child popup during a scale
animation. Check opening a Select immediately as well as after the dialog settles;
a correctly sized field does not prove its menu fits.

Reference previews follow the policy below, including local Back, one Drawer host and delayed loading feedback. Do not change hosts after loading based on
record length. Extended reading remains available through normal page navigation;
a pinned reader would be a separate product feature.

Substantial encounter forms use the encounter-owned modal placement policy: center
within the visible work area bounded by the viewport, with internal scrolling for
tall content. Do not reposition while typing or apply this policy globally to unrelated
dialogs. Small roll and recovery interactions remain anchored.

Attack modifiers roll directly in an anchored result popover. Show backend-derived
condition hints concisely and distinguish applied from conditional effects. Optional
Attack context switches once to a stable expanded view with internal scrolling; target
selection does not resize the surface again. It retains the original dice through target/context edits and checks;
only an explicit new-roll action rolls again. Do not make target adjudication a
prerequisite for an ordinary at-table roll.

### In-place reference reading

There is one record page and one contextual reference Drawer. There is no separate
Reader mode, companion layout or reader route. Search results, saved-list members
and Add candidates keep their primary selected-record detail pane; following a
reference does not replace that selection or commit a mutation.

Use `RecordReferenceDrawer` for linked records and Original record links on every
viewport. Trait badges retain native typed trait-page destinations; never coerce
their identities to record keys. It overlays the host
without reserving an empty column or compressing the underlying layout. It is
full width on narrow screens and has a generous bounded reading width otherwise.
Nested references replace the document in the same Drawer and keep a bounded
local history. Never stack document Drawers or choose a host after seeing content
length. Small fact explanations and short local choices may still use Popover.

Drawer Back restores the prior exact address, selected rank, landing and reading
position. Close and Escape finish the reference session and restore its original
opener without scrolling the host. Browser history remains ordinary page
navigation. Native full-page links, modified clicks and middle clicks preserve
browser behavior and exact destinations. Opening a reference does not alter an
Add candidate, saved-list membership, selected participant or unfinished draft.

Keep action tooltips within the Drawer's stationary viewport root. The shared
Drawer supplies Ant's `getPopupContainer` to all its popup descendants, including
feature-provided actions, through a scoped `ConfigProvider`. A body-level tooltip can
follow a closing button offscreen and widen the mobile viewport, rewrapping the
background document. Do not anchor it inside the scrolling header. Check actual
touch dismissal as well as keyboard dismissal; the owner must keep its width,
scroll and focus throughout the transition. Interactive descendants must receive
pointer events: a rank menu that looks correct but cannot be clicked/tapped fails
the shared overlay contract.

Preserve the complete backend address: record key, embedded path, field/passage,
fingerprint, selected spell rank and separate landing. Rank changes update the
current document through their own callback. The page/reading host passes its
landing explicitly; reusable detail content must not reparse the current URL on
local renders and restart landing when a linked Drawer opens or closes.
Original record follows verified
structured provenance for an embedded copy; an encounter participant's root
Original record opens its indexed baseline. Neither replaces a customized copy
nor guesses a destination by name. Unavailable references remain explicit inert
labels, without invented links.

The Drawer uses the same RecordDetailPane and family renderer as full pages.
Its header shows the document title once and its consistent action group remains
reachable while the body scrolls. Use the shared stacked identity composition:
title/actions, grouped classification and textual action cost, traits, then source attribution.
Keep these facts left aligned; do not leave an empty full-page title slot or
push type to a detached right edge. Action glyphs stay beside the title; textual
costs such as Treat Wounds' supplied "passive" belong with metadata, not in its
heading. Keep rank selection and exact owner/original links in this shared
identity, without duplicating them in the body. Phone action groups wrap by row;
substantial header details can scroll within a bounded header so the document
body remains reachable. Transient references omit Related sections;
full pages retain relationship discovery. The Drawer body owns one content
inset; the embedded detail/family presentation does not add another.

Preserve valid same-identity content through fetching and local failure. Delay
progress feedback by 500 ms and never replace readable content with a loading
skeleton. Retained content must be distinguished from a successful refresh.
Invalidated generations and different sources/owners cannot lend stale content.
Encounter previews keep their separate participant/presentation-bound cache and
Rust-composed facts; they never seed immutable reference caches. Gameplay controls
remain disabled when current context is unavailable.

Keep the Drawer mounted through afterOpenChange(false), then clear the session
and restore focus. Initial focus goes to its document heading, not the first
action button. Updating the displayed record must not move focus from a connected
control. Native source-information dialogs retain innermost Escape and
return focus to their marker before another Escape closes the reference. When
navigation removes the focused link, focus a surviving reading target without
stealing focus from a connected control or nested dialog. Use the real Drawer body
for history restoration and embedded landing; resize/reflow and reduced motion
must not restart landing or discard drafts. Preserve the same content anchor
rather than demanding an identical pixel offset after text reflow.

Prepared HTML links retain the shared semantic link color in both themes,
including visited links. Source effect references retain their supplied meaning;
activity detail links still follow the service has_reference_details decision.
No reader-mode URL adapter or alternate rendering path is retained.

### Record presentation

Before redesigning a record family, audit its generated Foundry DTO fields through
the borrowed source views, app-service presentation and rendered UI. The current
presentation DTO is not a completeness specification. Check nested facts,
qualifiers and root/embedded variants against populated records; a declared DTO
member does not guarantee a supplied value after admission.

Give relevant field groups an explicit disposition: already presented (including
authored prose or tables), useful but omitted, intentionally excluded from the
current reading task, or requiring new modeling/source support. For example,
Longsword's supplied damage dice/type belong in its reference presentation; source
migration metadata and a character's equipped state do not automatically belong
on its reference page. Preserve missing/null/invalid separately from known zero,
false and empty values. Preserve documented source-default policies; do not invent
a fallback just because the generated schema declares a member.

Add needed typed presentation fields under their Rust owner when existing source
facts are in scope. Identify substantial modeling or derived-rule work separately;
do not infer mechanics from prose or render raw rule-element payloads to claim
coverage. Preserve authored structures that already communicate the facts unless
an agreed redesign improves the reading task while retaining their meaning. Steel
Shield's source table remains a table, rather than duplicating it with a custom
stat layout. Treat unavailable or conflicting facts explicitly.

Validate coverage across affected full pages, results/readers, Drawers and lists,
and encounters where applicable. A layout-only preview does not establish field
coverage, and a field census does not establish rendering correctness. Record
findings and decisions in the task's existing audit/plan, with concrete source
examples and any remaining gaps.

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
patch clears. Baseline damage lists show formulas, types and meaningful qualifiers
directly: use “1d6 spirit” instead of “Damage component 1: 1d6 spirit (damage)”.
Do not discard healing, persistent damage, material distinctions or patch clears
with that cleanup. Keep original component IDs for correlation and React keys;
a simpler caption does not authorize merging components or changing calculations.
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

`DocumentIdentityHeader` supplies shared identity structure for records and traits.
Simple records use stacked identity metadata, with classification and its optional
control together and traits on their own row. Keep the name and selected activity
glyph together and wrap to the actual pane. Record classification labels come from
the service; the trait document uses the title-cased Trait classification.
`DocumentPagePane` provides their common page layout without a generic Reference bar. Record-only actions
such as Add to list stay in the record composition. Full pages use
native browser history for Back; they expose Retry or Reload only with a local
error or changed collection. Recovery rereads the current local reference data. Show the title glyph once and ability modifiers before skills.
Do not infer game semantics from prose or show ordinary technical keys.
Compact result classification uses the supplied kind and level/rank labels;
removing the Level/Rank display prefix does not create a typed fact. Product record
paths use literal colons through the shared serializer; keep API encoding and
complete selection identity intact.

Rituals use the service-provided Ritual display label in summaries and detailed
identity, including selected ranks. Their canonical spell kind, record key and
filter membership stay unchanged. Group casting time/cost and ritual checks
before the authored prose, without nested cards or extra disclosures. Preserve
compound check qualifications and caster counts: Atone's supplied zero remains
0; Borrow Time's null count reads Not specified; an authored empty secondary-check
string reads None listed. Do not infer that no secondary checks are required from
a zero caster count, or replace an unspecified count with zero.

For authored HTML tables, PreparedContent preserves native rows, captions,
headers, source links and controls. Its wrapper owns local horizontal overflow;
cells own their inset: space-control vertically and space-group horizontally,
with space-control on both axes in compact panes. Use semantic row separators.
Only overflowing wrappers add a keyboard stop and labelled region. Do not pan the
whole prose block with the table, force every table into permanent overflow, or
rebuild authored rules as a calculated application table. An unchanged-content
refresh must retain the table nodes, horizontal position and focused source link.

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

Spell headers show the supplied classification/rank as static text, with a muted
Change rank button and an anchored Ant menu. The menu identifies the base rank;
do not repeat a Base rank caption beside the trigger. For an owned spell it exposes
return to its opening casting rank. Preserve the trigger while fetching, return
focus after selection or Escape without scrolling, and close the menu before a
containing Drawer on Escape. Traits occupy a separate aligned row.

Rank selection can update some facts while others retain base values. Preserve
the backend's explicit unavailable note and original-rank identity when a preview
fails; do not add a generic warning to every higher-rank selection. Fly 7 is a concrete partial
case: its duration change is in prose rather than the structured duration fact.
Keep all authored heightening prose inline, including dependencies and choices.
Sound Body inherits earlier benefits, Animal Form replaces statistics, and
Enlarge permits alternative versions; ordinal headings cannot establish their
effective combination. Never parse prose or infer those mechanics in the browser.

Structured rank changes supplement the prose and supplied selected-rank facts.
Keep them under one optional Heightening details disclosure rather than another
expanded account of the same spell. Allfood's current Target and Fireball's current
damage remain inline; their full authored heightening text stays visible. Do not
infer which source facts duplicate prose: less common unique structured facts
must remain reachable. Inside the disclosure, use compact rank/change rows and
mark only an exact selected row, without marking all lower rows active or claiming
a complete effective spell. The selected fill and aria-current identify that row;
do not add a redundant checkmark. Omit an empty sparse patch such as Dancing Shield's
rank-4 patch, while retaining its prose, explicit clears and unavailable facts.
Keep unique structured facts even when no Heightened prose heading exists.
Put supplied selected-rank damage with the casting facts: Fireball's 6d6 at rank 3
becomes 10d6 at rank 5 through the existing backend projection. Its per-rank 2d6
increase is a rule and stays constant; it is not the selected-rank result. Omit a
known zero area increase in this baseline rule summary, retaining nonzero and
unavailable values and explicit patch clears. Do not infer which structured
changes duplicate prose or add instructions merely to explain both displays.

Spell descriptions stay authored when the selected rank changes. Inline-expression
evaluation is a separate workstream; do not substitute formulas or infer prose
heightening during presentation changes.

Optional reference accordions use a clear title and a visible boundary around
their related content. The current Heightening details panel is a positive visual
example: the header, enclosing border and body form one recognizable section;
its usefulness as a section is a separate product decision. Prefer Ant Collapse's
bordered composition. Avoid a bare chevron or heading that expands unbounded text
into the surrounding document. Do not wrap the Collapse in another Card: the
reading host owns its page/pane inset, Collapse owns header/body insets, and child
content adds only its own internal row spacing. A single bounded section may hold
several related facts; do not create nested accordions or parallel disclosures
for pieces of the same task. Essential rules and frequently used controls stay
inline, and the expanded content must remain keyboard-accessible and scrollable.

Initial filter discovery uses quiet placeholders in the common-filter control
slots. Do not insert a separate loading block above the search row: search input,
model/ranking choices, Links and Advanced filters stay mounted. Placeholder shapes
describe layout only, not available filter definitions or choices. The server's
catalog remains authoritative. During an ordinary refresh, retain the existing
identity-checked catalog through useFilterDiscovery; never retain invalidated
generation or different-catalog definitions to stabilize a layout. Verify a delayed
real response at desktop and phone widths, with an unfinished focused search draft;
the search row, toolbar height and results origin should remain stable.

Reference composition prioritizes the current reading task and groups related
facts/mechanics without separating essential qualifiers. Spell casting facts and
current damage precede prose as described above; creature reference order needs
its own review. Encounter composition puts current HP, conditions and actions first;
participant edits and maximum-HP configuration live in secondary disclosures.
Hazard detection, disable instructions, defenses, triggers/effects, routine and
reset are essential reference material and stay inline when supplied. Hidden Pit's
Pitfall reaction must not require opening an Abilities accordion. Secondary
provenance may be disclosed; that does not make all owned records or a long spell
catalog essential inline content. Keep bounded continuation for substantial
content instead of clipping it or requiring an unrelated disclosure.

For inline record inclusions, prioritize reading over occasional source navigation.
Reuse a meaningful existing rule heading or authored prose reference as the link;
keep description-only inclusions in the document flow. For example, Censer
Implement's introductory Censer text needs no invented heading or standalone
“Source record” row; its underlying record remains available in Related records.
Paragon Benefit and Spiritual Scar can link from their existing named rule titles,
while an in-sentence reference stays in its sentence. Do not add a floating icon,
toolbar or repeated card solely to expose an inclusion's provenance. Explicitly
authored inclusion headers remain meaningful content; unavailable or bounded
content still needs local status and a useful recovery/destination link. This rule
does not suppress essential actions or change existing owned-ability permalinks.
Verify the body-only, named-heading, prose-link and unavailable cases when changing
shared inclusion presentation.

Immediate Ability-class activities use the shared bounded embedded-reading
workflow; the optional whole-section Collapse in creature reading is not the
hazard default. Creature composition receives its own review. The existing
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

Encounter quick lookup uses the same complete family reader in RecordReferenceDrawer. Its separate contextual cache holds the requested document view,
keyed by participant presentation and exact selection. Rust composes current
participant facts and explanations on each request; presentation changes trigger
a fresh request without seeding immutable reference caches.
The retained visible document uses exact participant, source generation and reading
address identity, separately from the presentation-dependent request key. A resource
refresh must not discard its content, move its anchor or reset reading position.
Replace fresh facts in place; failed refresh keeps readable context with an explicit
error and disabled gameplay actions. Genuine participant/generation/address changes
do not reuse stale context. Disappearance and unresolved sources remain explicit
without baseline-detail fallback. Keep the native full reference link,
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
Similarity URLs preserve the root seed and canonical filters. The active Similar
criterion occupies the main search control with its selectable, read-only human
name; retain the same input shell, clear/search affordances and toolbar dimensions
as keyword entry. Clearing it
preserves other filters and returns focus to keyword entry. Contextual text counts are disabled while
catalog choices and vocabulary remain available. Label bounded pages Top matches.
Comparison panes reuse family presentation without recursively mounting related disclosures.
Incoming references use checked `source_navigation`, outgoing references use the
checked target, and separate occurrence ordinals remain attribution metadata;
they do not imply automatic scrolling to an individual marker. Counterparts and variants apply only to root selections; they never appear as
relationships of an embedded selection. A separate Similar action can name the
parent root explicitly. Exact root references use an empty owner path, not an
all-owners graph query.

Record-link conditions are structural leaves in the same predicate edited by
common controls and Advanced. Use the existing group AND/OR/NOT owner and a Record
links condition choice, not a fabricated catalog field or separate relationship
array. Links with a chevron is a toolbar shortcut. It belongs to FilterControls/
useFilterEditor so adding a root conjunction retains unfinished sibling drafts.
Positive root conjunctions can expose individual editable chips. Mixed or negated
expressions use one bounded human summary retaining operators and grouping; clicking
opens Advanced. Never extract flat link chips from an OR/NOT expression. Clear filters
clears the complete predicate while retaining text/Similar and result order.

Reuse SearchPickerModal for record selection. Supply its concrete link-condition
composer through a stable app-level component context with a lightweight shared-filter
interface; FilterControls must not import its concrete picker and create a static
cycle back through SearchPickerModal. Each owner retains local open/draft state;
nested lookups use Ant overlays with innermost Escape and focus restoration.
Selection callbacks carry one record summary and artifact generation, or null on
reset. Existing list/encounter forms keep their selection and draft semantics.

Lookup criteria are independent of parent search and membership. Defaults are
Links to / Whole entry. Editing direction/scope preserves an existing exact embedded
selection until an explicit root-row choice, including a row with the same parent
key. Suppress duplicates only in a safe direct conjunction; never rewrite grouped
expressions. Record/Related actions conjoin with the entire active Search expression
or create a leaf-only search outside Search.

Human clause presentations are keyed by clause ID, echoed exact leaf, generation
and recovery epoch, including negative/zero-result conditions. Retained catalog
fields must not lend stale source labels to a new identity. New uncommitted root
choices get human draft labels from the generation-aware picker event. Never display
IDs in labels, errors, tooltips or unavailable fallbacks. Root relationship conditions
are rejected inside authored collection Exists scopes.

Whole entry changes sources: matching result entries for Links to, or the selected
source and its descendants for Linked from. Targets stay exact and descendants
exclude siblings/parents. Related supplies a typed predicate, never a list of keys
reconstructed from previews. Positive proof evidence and occurrence pages use clause
IDs and the full predicate/generation. Negative absence and failed Boolean branches
have no invented occurrence evidence; preview limits do not truncate result roots.
Existing unsupported count states remain honest, with count-free vocabulary retained.

Keep relationship discovery actions inside the Related experience with its
reference previews and full-result escapes. Result relationship evidence reuses
the shared Matches button/popover and its focus/dismissal behavior; do not expand
a separate disclosure into each row. Source-to-target labels supply the useful
evidence without a repeated Text reference label. Scope options use the existing
source-side semantics, with a concrete example in their explanation popover.

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

### Reference table states

Keep a reference table's formula/roll controls, column headings and valid existing
rows in place while reading results. Initial loading is not an empty table:
“Loading results…” belongs in the results region; “No results” requires a
successful empty response. A failed read shows a local unavailable state and
Retry. Preserve meaningful zero ranges and unavailable values separately.

If a reference-table reroll fails, retain the previous confirmed result for the
same table/selection/generation and explicitly identify it as the previous result.
Wellspring Surges is the example: the failure sits beside Reroll and says “Previous
result shown.” The error does not become a new outcome or a successful reroll.
Keep recovery with the existing action; do not add a second competing submission
or automatically roll again. A different table, invalidated generation or changed
selection must not borrow that result. This reading-retention pattern does not
replace an encounter mutation's request/revision/retry contract.

### Grouping reference facts

Use one compact inline group for facts read together. Break at a change of
meaning, not wherever a flex row happens to wrap. Keep each label with its value
and essential qualifiers; let a long value wrap inside its group. For example,
Hidden Pit uses a durability row (AC, HP, Hardness), a separate saves row, and
individual Immunities/Weaknesses/Resistances rows when supplied. Do not put all
seven facts into one flowing sentence, and do not create a card for each number.
Apply the same distinction in vehicle defenses and future creature/encounter work;
the exact groups depend on the task, not a universal number of facts per row.

Use an Ant Table when users compare a repeated dimension across rows, rather than
forcing tabular information into label/value runs. Class feat and skill advancement
uses Level → Advancement rows so the user can find everything supplied for one
level. Preserve the supplied schedule and unavailable categories; do not infer a
complete class progression from these arrays. A short price/bulk/usage group does
not need a table. Compact table rows suit short progression values; prose-heavy
roll tables need more cell spacing. Keep source-authored tables as tables.

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

The roster separates current-turn progression from participant management. Keep
the current participant and primary Start/Next turn control together above a
single Add menu and secondary Roll initiative action. The Add menu offers
creatures/hazards, individual PCs and whole saved parties.
The roster presents initiative, participant and HP in compact labelled columns,
with distinct selected, current-turn and defeated states. PC initiative uses a
borderless, centered inline field with a visible focus outline; creature totals
use the same numeric alignment and retain their roll/set popover. Accessible movement
actions disclose the target initiative. Phone layouts drill into a selected
participant and restore roster focus/scroll on Back. Advancing the turn follows
the canonical server-selected participant in the reader. Turn progression skips participants without initiative scores, as well as defeated,
disabled and dead participants. Missing initiative must be explained where it prevents
progression, with a direct way to assign or roll initiative; ordinary selection does
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
Encounter reference Drawers explicitly carry encounter context and omit
owner-return captions and Related records; actual
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

Next turn commits eligible progression once, then presents one non-blocking
notification for the just-completed turn. Closing, Escape and backdrop clicks finish
that interaction without changing HP, conditions or initiative. There is no reminder
queue, acknowledgment, selector or reopen action. The browser never ticks durations,
chooses timing anchors or invents outcomes. A failed advance shows its actual failure;
reminder loading cannot masquerade as a progression failure. Empty boundaries do not
open a notification. Position dialogs within the work area using wrapper padding;
keep Ant's wrapper over the full viewport so all backdrop clicks still dismiss it.

Encounter rosters and saved-list members share `shared/ui/sortable/SortableList`
for pointer and keyboard pickup/movement/drop and temporary visual order while a
reorder is saved and refreshed. Keep reorder writes serialized and refresh feedback
outside the row layout. Saved-list handles disappear under active filters; moving
members writes their persisted positions, including unavailable records, without
changing their snapshots or notes. Drag previews retain the complete member layout.
Current turn uses the strong filled row treatment with `aria-current` and
an accessible label; preview selection uses a separate outline. Each row's
actions menu offers Set turn and confirmed removal. Cancelling a secondary dialog
returns focus to its visible menu trigger. Fade defeated and disabled-hazard names and facts without dimming
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

Participant panels do not expose standalone Strike history, Relationships or Action
context disclosures. Optional attack context belongs to the roll interaction; general
relationship editing remains deferred. Condition forms accept incomplete persistent
damage as reminders, without eager required-field errors. Group Add duration and
Add details in the footer opposite Cancel/Apply, wrapping on narrow screens.

Attack assistance shows target-specific reminders and direct optional reference
flat checks. Exclude the attacker from ordinary target choices. Do not require a
detection/relationship questionnaire or a preparatory resolve step to roll these
reference checks. Keep the original attack dice and expanded view geometry stable.
Persistent damage and recovery render the backend's ordered sources and active
source; source comparisons are edited only in Add/Edit precedence. Do not add
another source-selection form to damage or recovery.
Recovery uses an anchored DC reminder and Roll; render its outcome in that popover,
including after successful condition removal. Table recovery uses ordinary clear.
Creature dying settings belong at the bottom, revealing compact fields directly
when enabled; avoid nested disclosures and additional save forms.

Turn reminders are one notification for the just-finished turn. Every close path
ends the interaction; no acknowledgment button, past-turn selector, reminder inbox
or reopen action. Conditions and HP remain independently editable on the creature.
Target & checks is present from initial attack loading, disabled until ready, so
revealing that action never shifts the roller. Recovery uses one inline DC input
and shows the DC actually resolved with its result. Dying settings contain the
threshold and an Alive/Dead segmented control, not a general level override.

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

Recovery popovers provide short rule hints and optional checks with already known
DCs/modifiers. They never ask for missing DCs, prerequisites, recovery reasons or
elapsed time. Use existing clear/decrement controls unless an action performs a
meaningful coordinated update. Table outcomes use direct buttons; an unknown DC
uses guidance and the existing statistic roller instead of another custom form.

HP values use consistent typography for current, maximum and temporary HP;
Add temp HP belongs beside those values. The status split button keeps the main
defeat/mortality correction direct and opens knockout or dying controls from its
arrow. Pending knockout and initiative decisions share that entry point. Closing
the popover keeps a current decision available; it has no separate Skip action.
Center inline HP number inputs. Preserve
the mounted consequence preview while values are recalculated; stale previews
may remain visible during loading, but must not authorize Apply.

Persistent turn reminders show all sources directly, with the active source first
and superseded entries muted. Recovery and per-type clear icons share one aligned
action row. Clear records the table outcome without a DC form or dice roll; it
ends that turn's applicable sources and leaves later sources intact. Table damage
is labeled Damage taken with an After IWR tooltip. Incomplete persistent entries
show only supplied details, without an internal Reminder prefix or empty metadata.

Keep persistent turn review organized by damage type: sources, damage entry or
result, then a labelled Recovery row with adjacent roll/clear icons. Do not
repeat the types in separate damage and recovery lists. Keep receipts and dice
outcomes with their section/popover rather than adding duplicate modal banners
or footer rolls. Add/Edit presents a compact Damage precedence list using the shared sortable
controls, with the backend default order and the first entry marked Active. Show
it for ambiguity anywhere in the group, including lower fallbacks. Reordering is
a local form edit with pointer/touch and keyboard support. Existing Apply saves
the complete order and condition together; the condition list has
no selection controls or separate confirmation flow.

Damage review shows why a defense changes the amount, including known immunity
before applying damage and recorded defense facts alongside the result. When an
unresolved protection requires table damage, explain it before submission and
select the table-result input. Deterministic turn actions such as reducing
Frightened are direct labelled buttons; a dice icon must initiate a dice/check
interaction rather than hide a plain counter action.
