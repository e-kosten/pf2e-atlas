# PF2e Atlas CLI JSON Contract

This contract defines the stable machine-readable CLI surface. JSON is written to stdout; progress and diagnostics remain separate. Fields use `snake_case`.

## Envelopes and exits

Success uses `{ "status": "ok", "data": ... }`. Command failures use `{ "status": "error", "error": { "code": ..., "message": ... } }`. Batch record commands may succeed structurally while reporting `data.partial: true` and per-item errors.

Exit classes are `0` for success, `1` for a domain miss or partial domain result, `2` for invalid input, and `3` for runtime, index, artifact, or environment failure. Invalid readiness/validation findings may use a successful envelope with `data.valid: false` and exit `3` because the check itself ran.

## Record contract

Every record-bearing command serializes the same `atlas_record::RecordJson`. Search, resolve, graph, similar, and list payloads wrap that record; they do not define alternate record DTOs.

The record is a flattened shared base plus a flattened entity-specific body with a mandatory `presentation_type` discriminator:

```json
{
  "key": "pathfinder-bestiary:WQy7HBUcgDLsfVJd",
  "name": "Night Hag",
  "kind": "creature",
  "level": 9,
  "traits": ["fiend", "hag"],
  "source": {
    "publication_title": "Monster Core",
    "pack": { "name": "pathfinder-bestiary", "label": "Bestiary" }
  },
  "presentation_type": "creature",
  "defenses": {
    "ac": { "value": 28 },
    "hp": { "value": 145, "maximum": 145 },
    "saves": {
      "fortitude": { "id": "fortitude", "value": 18 },
      "reflex": { "id": "reflex", "value": 16 },
      "will": { "id": "will", "value": 21 }
    },
    "immunities": [],
    "resistances": [],
    "weaknesses": []
  },
  "perception": { "modifier": 19, "senses": [] },
  "languages": ["Aklo", "Common", "Infernal"],
  "skills": [],
  "movement": { "modes": [] },
  "resources": [],
  "strikes": [],
  "actions": [],
  "spellcasting": { "entries": [], "spells": [] }
}
```

The shared base contains only record identity/classification, source metadata, explicit raw-source opt-in, and `supplementary_sections`. Supplementary sections contain rich prose, tables, structured content, or relationships. Creature mechanics are never represented or duplicated there.

`presentation_type: "creature"` directly exposes:

- `defenses`: AC, HP/thresholds, saves, hardness, immunities, resistances, and weaknesses;
- `perception`, `languages`, ordered `skills`, and ordered movement `modes`;
- ordered `resources`, `strikes`, and `actions`; and
- `spellcasting.entries` and `spellcasting.spells` as separate ordered collections. The CLI does not invent an entry-to-spell parent relationship.

Parent-local `id` and `order` values are stable within a record. Activity rolls, damage, modes, usages, spellcasting preparation, attacks, and DCs remain typed fields rather than label/value fact bags.

Detail hydration is represented by field presence, not placeholder values. A field, object, or array that the requested detail level does not hydrate is absent. An empty object or array may be serialized only when its containing section is included at that detail and the empty value intentionally means that the record has no members in that included section. Callers must test field presence before reading detail-dependent entity fields.

Non-creature families use `presentation_type: "unmigrated"` only while their named H1-H10 family plan is pending. The `migration` object records that registry assignment and H12 acceptance checkpoint. Their generic ordered fact sections are temporary and are replaced, not wrapped by compatibility shims, when the family-specific variant lands.

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
