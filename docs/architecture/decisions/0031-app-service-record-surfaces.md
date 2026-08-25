# ADR 0031: App-Service Record Surfaces

Status: accepted  
Date: 2026-06-26

## Context

PF2e Atlas has two record presentation needs that must stay consistent:

- static record browsing in search, detail, and reader views;
- runtime encounter presentation where local participant state, elite/weak variants, conditions, HP, actions, and other adjustments change how source facts should be shown.

The older `RecordPresentationDocument` is a prose-oriented source presentation derived from `atlas-record`. Encounter mechanics later added structured adjusted stats and activities as a separate sidecar. That made encounter views useful, but it also duplicated information such as HP, defenses, movement, and activities when the static presentation was shown below the adjusted sidecar.

The frontend needs a final app-facing presentation contract that is already composed for the current product context. The browser should render and host interactions, not reconcile raw source facts against runtime mutations or parse Foundry prose to rediscover mechanics.

## Decision

`atlas-app-service` owns final app presentation composition through `RecordSurfaceView`-style DTOs exported by `atlas-app-model`.

`atlas-record` continues to own normalized source facts, rich content, presentation-neutral mechanics/activity projections, and reference policy. `atlas-app-service` composes those facts with app context:

- search compact record rows;
- static record detail;
- encounter participant detail with local participant state and adjusted mechanics.

`web/atlas-ui` renders composed surfaces and supplies browser interaction slots such as HP controls, condition editors, note editors, and record-preview popovers. Generic surface rendering must not own mutation behavior directly; feature modules provide mutation handlers through explicit slots.

`RecordPresentationDocument` remains transitional evidence during the creature migration, not a permanent second creature record. After the source-faithful surface passes the independent automated and Checkpoint E human visual gates, G1 removes the creature fallback and duplicate sparse mechanics path. Non-creature kinds remain behind explicit kind boundaries until their separately approved family cutovers.

The final profile vocabulary is `search_compact`, `record_detail`, and `encounter_participant`. Canonical entities own intrinsic facts, occurrences own context/order/overrides, runtime instances own mutable local state, and profiles own only selection, order, density, disclosure, and interaction slots.

Atlas currently has no authentication boundary, but pinned-base app surfaces inherit default-visible/public-only retrieval and are not GM-complete. Checkpoint A's target app surfaces do not suppress useful authored information solely because of typed visibility or absent authorization. Visibility/role/provenance remain typed metadata and make no current privacy/security claim; target exclusions require non-auth product rationale and audit evidence.

## Consequences

Creature presentation can use the same section/value vocabulary across search, record detail, and encounter participant views while each profile controls density, ordering, and collapsed state.

Search rows use compact surfaces without encounter mutation controls or variant selectors. Future table-compatible search views should use stable surface fact keys and filter/discovery fields rather than scraping displayed text.

Encounter participant views attach runtime controls to semantic sections such as vitals and conditions. Adjusted values render final values inline, with modifier explanations available through tooltip or popover details.

Non-creature record kinds can stay on the existing presentation path until their own surfaces are deliberately designed.

## Boundaries

- `atlas-record` owns source projections and must not depend on app-service or frontend layout.
- `atlas-app-model` owns the DTO contract and generated TypeScript bindings.
- `atlas-app-service` owns surface composition and decides which facts appear for each profile.
- `atlas-web` remains HTTP transport glue.
- `web/atlas-ui` owns rendering, layout, local browser state, and interaction components. It must not parse Foundry JSON or prose to compute mechanics.
