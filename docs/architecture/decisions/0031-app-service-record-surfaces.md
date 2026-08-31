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

`atlas-app-service` owns final app presentation composition through generated DTOs exported by `atlas-app-model`. `RecordSurfaceView` is the one app-facing record contract: common metadata, a profile tag, a tagged entity-specific typed presentation body, and an optional exact `EncounterRuntimeView` bag. The first entity body is creature; non-creature families return an explicit typed unavailable presentation until their separate migration. Encounter composition uses participant metadata beside that surface and places all current/final mechanics in the optional encounter bag. The former `StatBlockView.values[]` and generic record-section bridge are removed directly.

Public encounter-runtime incompleteness is limited to concise automation limitations with stable typed codes and typed participant, condition, activity, or spellcasting placement targets. Their human-readable messages are display-only and must never be parsed for behavior. Projection details about malformed, duplicate, unsupported, unmapped, raw-path, publication, null, or source-noise facts remain internal to `atlas-app-service`; raw source and provenance stay available through their owning contracts instead of being copied into the runtime DTO. Critical missing or unsafe data fails closed or makes the affected typed value unavailable.

`atlas-record` continues to own normalized source facts, rich content, presentation-neutral mechanics/activity projections, and reference policy. `atlas-app-service` composes those facts with app context:

- search compact record rows;
- static record detail;
- encounter participant detail with local participant state and adjusted mechanics.

`web/atlas-ui` renders composed surfaces and supplies browser interaction slots such as HP controls, condition editors, note editors, and record-preview popovers. Generic surface rendering must not own mutation behavior directly; feature modules provide mutation handlers through explicit slots.

`RecordPresentationDocument` is not an app DTO or an app-service fallback. CLI and storage-neutral presentation contracts remain owned by `atlas-record`; app-service projects `RetrievedRecord` directly into `RecordSurfaceView`. Non-creature kinds remain behind explicit typed unavailable boundaries until their separately approved family cutovers.

The final profile vocabulary remains `search_compact`, `record_detail`, and `encounter_participant`, but profile shape is not a generic section/value registry. Canonical entities own intrinsic facts, occurrences own context/order/overrides, runtime instances own mutable local state, and profiles own only selection, order, density, disclosure, and interaction slots. Zero-or-one semantic areas use named optional typed fields; arrays are reserved for genuinely repeated entities. Serialized record surfaces omit genuinely known-empty collections and generated TypeScript marks those collections optional. Populated collections retain authored ordering. For a selected creature domain, missing, null, unsupported, ambiguous, failed, or unsafe canonical roots and required nested members are recorded in an optional, named `unavailable_domains` sidecar rather than being serialized as empty or silently omitted. Its causes expose typed state, typed affected field, optional component identity, and canonical-field provenance; display messages have no behavioral meaning. An unsafe value is omitted from the ordinary domain payload while its typed cause remains public, so consumers can distinguish partial failure from known-empty without raw source paths, generic keys, or prose parsing.

Creature activities carry their own optional nonempty `content` collection. `atlas-app-service` resolves each typed activity occurrence target and attaches matching occurrence-owned content plus content owned by the exact `ActorOwned` entity target. Attached content is removed from the general overview/lore collection, so it appears once. Each document retains its key, role, authored order, optional label, visibility, hash, provenance, and ordered typed rich-content blocks. Missing targets, duplicate occurrence/entity/content identities, ambiguous targets, or one document matching multiple activities fail the affected activity closed; app and frontend consumers never join content by labels, parse IDs, or recover associations from prose.

Atlas currently has no authentication boundary, but pinned-base app surfaces inherit default-visible/public-only retrieval and are not GM-complete. Checkpoint A's target app surfaces do not suppress useful authored information solely because of typed visibility or absent authorization. Visibility/role/provenance remain typed metadata and make no current privacy/security claim; target exclusions require non-auth product rationale and audit evidence.

## Consequences

Creature presentation can share the same canonical facts across search, record detail, and encounter participant views while each profile controls density, ordering, and disclosure through its typed contract.

Search rows use compact typed surfaces without encounter mutation controls or variant selectors. Future table-compatible search views should use named contract fields and filter/discovery fields rather than scraping displayed text.

Encounter participant views attach runtime controls to named semantic fields such as vitals and conditions. Adjusted values render final values inline, with modifier explanations available through tooltip or popover details. Canonical mechanics that have final/current runtime counterparts are omitted from the encounter-profile creature body rather than serialized twice.

Activity content is backend-composed for record-detail and encounter-participant profiles. Encounter spellcasting is likewise composed as ordered entries with final attack/DC/slots, nested occurrence spells, and an explicit standalone-spell collection; spell rows do not remain in generic runtime activities. The UI renders the ordered typed blocks, reference targets, and runtime mechanics it receives; it does not parse Foundry HTML, infer associations, join public owner maps, or duplicate content into overview sections.

Non-creature record kinds return the explicit unavailable presentation until their own typed bodies are deliberately designed.

## Boundaries

- `atlas-record` owns source projections and must not depend on app-service or frontend layout.
- `atlas-app-model` owns the DTO contract and generated TypeScript bindings.
- `atlas-app-service` owns surface composition and decides which facts appear for each profile.
- `atlas-web` remains HTTP transport glue.
- `web/atlas-ui` owns rendering, layout, local browser state, and interaction components. It must not parse Foundry JSON or prose to compute mechanics.
