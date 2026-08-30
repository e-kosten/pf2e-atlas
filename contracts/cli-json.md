# PF2e Atlas CLI JSON Contract

This contract defines the stable machine-readable CLI surface. JSON is written to stdout; progress and diagnostics remain separate. Fields use `snake_case`.

## Envelopes and exits

Success uses `{ "status": "ok", "data": ... }`. Command failures use `{ "status": "error", "error": { "code": ..., "message": ... } }`. Batch record commands may succeed structurally while reporting `data.partial: true` and per-item errors.

Exit classes are `0` for success, `1` for a domain miss or partial domain result, `2` for invalid input, and `3` for runtime, index, artifact, or environment failure. Invalid readiness/validation findings may use a successful envelope with `data.valid: false` and exit `3` because the check itself ran.

## Record contract

Every record-bearing command serializes the same `atlas_record::RecordJson`. Search, resolve, graph, similar, and list payloads wrap that record; they do not define alternate record DTOs.

The record is a flattened shared base plus a flattened entity-specific body with a mandatory `presentation_type` discriminator. This illustrative standard-detail excerpt abbreviates ordered collections to one member and omits some nested member fields:

```json
{
  "key": "creatures:WQy7HBUcgDLsfVJd",
  "name": "Night Hag",
  "kind": "creature",
  "level": 9,
  "traits": ["evil", "fiend", "hag", "humanoid", "unholy"],
  "source": {
    "publication_title": "Pathfinder Bestiary",
    "pack": { "name": "creatures", "label": "Creatures" }
  },
  "presentation_type": "creature",
  "defenses": {
    "ac": { "value": 28 },
    "hp": { "value": 170, "maximum": 170 },
    "saves": {
      "fortitude": { "id": "save:fortitude", "value": 19 },
      "reflex": { "id": "save:reflex", "value": 17 },
      "will": { "id": "save:will", "value": 18 }
    },
    "immunities": [{ "id": "immunity-0", "order": 0, "iwr_type": "sleep" }],
    "resistances": [{ "id": "resistance-0", "order": 0, "iwr_type": "mental", "value": 10 }],
    "weaknesses": [{ "id": "weakness-0", "order": 0, "iwr_type": "cold-iron", "value": 10 }]
  },
  "perception": { "modifier": 18, "senses": [{ "id": "sense-0", "order": 0, "kind": "darkvision" }] },
  "languages": ["aklo", "chthonian", "common"],
  "skills": [{ "id": "skill-occultism", "order": 0, "slug": "occultism", "label": "Occultism", "modifier": 20, "note": "ancient soul lore" }],
  "movement": { "modes": [{ "id": "speed-land", "order": 0, "mode": "land", "value_feet": 25 }] },
  "resources": [{ "id": "resource:focus", "order": 0, "kind": "focus", "label": "Focus", "maximum": 1, "serialized_value": 1, "current_policy": "serialized_value_is_provenance_only" }],
  "strikes": [{ "id": "occurrence:creatures:WQy7HBUcgDLsfVJd:strike:nightHagJaws001", "order": 5, "label": "Jaws", "action_cost": { "kind": "actions", "actions": 1 } }],
  "actions": [{ "id": "occurrence:creatures:WQy7HBUcgDLsfVJd:action:changeShape0001", "order": 7, "label": "Change Shape", "traits": ["concentrate", "occult", "polymorph"], "action_cost": { "kind": "actions", "actions": 1 }, "rolls": [] }],
  "spellcasting": {
    "entries": [{ "id": "occurrence:creatures:WQy7HBUcgDLsfVJd:spellcasting-entry:occultInnate001", "order": 1, "label": "Occult Innate Spells", "preparation": "innate", "tradition": "occult", "attack": 20, "dc": 28 }],
    "spells": [{ "id": "occurrence:creatures:WQy7HBUcgDLsfVJd:spell:magicMissile001", "order": 3, "label": "Magic Missile (At Will)", "parent_entry_id": "occurrence:creatures:WQy7HBUcgDLsfVJd:spellcasting-entry:occultInnate001", "context": { "rank": 3, "location": "occultInnate001", "contextual_label": "Magic Missile (At Will)" }, "base_rank": 1 }]
  }
}
```

The shared base contains only record identity/classification, source metadata, explicit raw-source opt-in, and `supplementary_sections`. Supplementary sections contain rich prose, tables, structured content, or relationships. Creature mechanics are never represented or duplicated there.

`presentation_type: "creature"` directly exposes:

- `defenses`: AC, HP/thresholds, saves, hardness, immunities, resistances, and weaknesses;
- `perception`, `languages`, ordered `skills`, and ordered movement `modes`;
- ordered `resources`, `strikes`, and `actions`; and
- `spellcasting.entries` and `spellcasting.spells` as separate ordered collections. A spell retains typed occurrence context such as rank, location, uses, and `parent_entry_id` when the canonical occurrence models it; the CLI does not infer a relationship from labels or prose.

Parent-local `id` and `order` values are stable within a record. IWR amounts and exceptions, skill notes and variants, resource maximum/serialized provenance, action costs and frequencies, spell slots and occurrence uses, activity rolls, damage, spellcasting preparation, attacks, and DCs remain typed fields rather than label/value fact bags. These values come from the persisted canonical creature body carried with the retrieved record, never from the legacy sparse mechanics projection.

Detail hydration is represented by field presence, not placeholder values. A field, object, or array that the requested detail level does not hydrate is absent. An empty object or array may be serialized only when its containing section is included at that detail and the empty value intentionally means that the record has no members in that included section. Callers must test field presence before reading detail-dependent entity fields.

Non-creature families use `presentation_type: "unmigrated"` only while their named H1-H11 family plan is pending. The `migration` object records that registry assignment and H12 acceptance checkpoint. Their generic ordered fact sections are temporary and are replaced, not wrapped by compatibility shims, when the family-specific variant lands.

## Detail and raw source behavior

`summary`, `preview`, `description`, `standard`, and `full` retain the same tagged record schema and vary only by hydration:

- `summary` is identity-oriented; creature mechanics fields are absent;
- `preview` includes compact typed scan facts and shortened prose; activity `rolls`, `damage`, and `modes` are absent because preview does not hydrate activity detail;
- `description` emphasizes complete descriptive rich content; creature mechanics fields are absent;
- `standard` is the normal typed entity view and includes activity-detail collections, including intentional known-empty collections; and
- `full` adds full source metadata and supplementary content while retaining the standard typed entity body.

`--include-raw` is an independent explicit opt-in at every supported detail level. Without it, `source_json` is omitted. With it, `source_json` may appear even at `preview`; callers must not infer that raw source requires `detail=full`.

## Ambiguity and wrappers

Strict-resolution ambiguity uses `error.code: "record_resolution_ambiguous"` and structured alternatives under `error.data.result.alternatives`. Record-bearing search, resolve, graph, similar, and list responses retain their command-specific match/edge/list metadata around the identical tagged record contract.
