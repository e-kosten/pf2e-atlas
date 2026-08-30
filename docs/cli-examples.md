# PF2e Atlas CLI Examples

These examples are organized by research task. Use `--json` when a script or
agent needs structured output; use human-readable output for quick manual
scanning.

## Find One Known Record By Name

Use strict resolution when you know the record name:

```bash
atlas record resolve "Treat Wounds" --pack-name actionspf2e --detail description
```

Actions such as Treat Wounds are Atlas `rule` records in the `actionspf2e`
pack. There is no separate `action` record kind.

## Fetch A Known Canonical Key

Use `record get` when you already have a `pack:id` key from a previous result:

```bash
atlas record get actionspf2e:1kGNdIIhuglAjIp9 --detail standard --json
```

Batch exact keys in one process:

```bash
atlas record get actionspf2e:1kGNdIIhuglAjIp9 equipment-srd:s1vB3HdXjMigYAnY --detail standard --json
```

## Handle An Ambiguous Name

Ask for alternatives, inspect their keys and context, then fetch the chosen
record:

```bash
atlas record resolve "Shield Block" --kind feat --alternatives 5 --detail preview --json
atlas record get feats-srd:jM72TjJ965jocBV8 --detail standard --json
```

The same pattern applies when graph or similar commands report ambiguity: pick
one returned canonical key, then rerun with that key.

## Search With Structured Constraints

Translate obvious constraints into filters:

```bash
atlas search "healing spell" --kind spell --max-level 2 --detail preview --limit 8
atlas search "protect an ally" --kind feat --trait champion --detail preview --limit 8
atlas search --kind equipment --rarity uncommon --detail preview --limit 20
```

Use pagination when you need more than the first page:

```bash
atlas search "healing spell" --kind spell --max-level 2 --page 2 --limit 10
```

## Compare Retrieval Behavior

Use retrieval controls only when diagnosing ranking behavior:

```bash
atlas search "low level healing spell" --kind spell --retrieval fts --explain --json
atlas search "low level healing spell" --kind spell --retrieval vector --explain --json
atlas search "low level healing spell" --kind spell --retrieval hybrid --explain --json
```

## Find Similar Records

Use a name when it resolves strictly, or use a canonical key when the seed may
be ambiguous:

```bash
atlas similar "Dirge of Doom" --kind spell --limit 8
atlas similar feats-srd:jM72TjJ965jocBV8 --limit 12 --explain --json
```

Structured filters apply to the seed resolution and to the candidate result
set:

```bash
atlas similar "Dirge of Doom" --kind spell --max-level 5 --limit 8 --json
```

## Inspect Graph Context

Use `graph links` for outgoing references, with optional backlinks:

```bash
atlas graph links actionspf2e:1kGNdIIhuglAjIp9 --json
atlas graph links actionspf2e:1kGNdIIhuglAjIp9 --backlinks 4 --json
atlas graph links actionspf2e:1kGNdIIhuglAjIp9 --outgoing 0 --backlinks 8 --json
```

Use `graph uses` for backlinks-only context:

```bash
atlas graph uses conditionitems:TBSHQspnbcqxsmjL --limit 5 --json
```

Common conditions, actions, and traits can have many backlinks, so keep limits
small while exploring.

## Use Relationship Filters

Relationship filters require canonical keys:

```bash
atlas record resolve "Battle Medicine" --kind feat --detail standard --json
atlas search --referenced-by feats-srd:wYerMk6F1RZb0Fwt --detail preview --limit 10
atlas search --references actionspf2e:1kGNdIIhuglAjIp9 --detail preview --limit 10
```

Read the relationship from the candidate record's perspective:

- `--referenced-by KEY` finds records linked from `KEY`
- `--references KEY` finds records that link to `KEY`

## Discover Filter Fields And Values

List available fields:

```bash
atlas filters fields
atlas filters fields --kind spell --json
```

Discover values for normal fields:

```bash
atlas filters values --field traits --kind spell
atlas filters values --field rarity --kind equipment --json
```

Discover metric keys before building metric predicates:

```bash
atlas filters values --field metric --kind creature --metric-query save --json
atlas filters values --field metric --kind creature --metric-label "Armor Class" --json
atlas filters values --field metric --kind creature --metric-prefix speed. --json
```

Then use metric predicates in search:

```bash
atlas search --kind creature --metric 'ac.value>=25' --detail preview --limit 8
atlas search --kind creature --metric 'speed.fly.value>=100' --metric 'ac.value>=30' --detail preview --limit 8
```

## Diagnose Setup Or Index Readiness

Check setup readiness:

```bash
atlas setup --check --json
```

Check the current artifact:

```bash
atlas index check --json
atlas index check --no-embeddings --json
```

Run deep validation only when diagnosing an artifact problem or when explicitly
requested:

```bash
atlas index validate --json
```

## Validate Authored Tag Data

Validate the tag catalog, assignments, and ontology suggestions before an
index build consumes them:

```bash
atlas tags validate --path data/tags
atlas tags validate --path data/tags --json
```

## Use JSON Safely

Most JSON commands return a shared envelope:

- successful command payloads are under `data`
- top-level command, runtime, or input failures are under `error`
- batch commands can have per-record failures inside `data.results[].error`
- artifact validation can return `status: "ok"` with `data.valid: false`

Every record-bearing result uses one tagged record contract. Check
`presentation_type` before reading entity fields. Creature records expose
`defenses`, `perception`, `languages`, `skills`, `movement`, `resources`,
`strikes`, `actions`, and separate `spellcasting.entries` and
`spellcasting.spells` directly. Read IWR amounts and exceptions from
`defenses`, skill notes and variants from `skills`, resource maxima and
serialized provenance from `resources`, action costs and frequencies from
`actions`, and rank, use, slot, and parent-entry context from spellcasting.
Rich prose and relationships live under
`supplementary_sections`; do not search generic sections for creature
mechanics. A non-creature `presentation_type: "unmigrated"` payload names its
temporary H-family registry assignment in `migration` and retains generic fact
`sections` until that family contract lands.

Detail-dependent fields are absent when that detail level does not hydrate
them. In particular, `summary` and `description` omit the creature mechanics
fields, while `preview` omits activity `rolls`, `damage`, and `modes`. Empty
objects or arrays are meaningful only inside a section that is included and
known to have no members, so check field presence before reading entity data.
Source-missing concepts are omitted even at a detail level that otherwise
includes creature scan facts.

Raw source is always an independent explicit opt-in. It is omitted without
`--include-raw`, and it may be requested with any supported detail level:

```bash
atlas record get pathfinder-bestiary:WQy7HBUcgDLsfVJd --detail preview --include-raw --json
```

Do not require `--detail full` merely to use `--include-raw`; choose `full`
only when full source metadata and hydration are useful.

For agent workflows, check the top-level `status`, inspect `error` when
present, and inspect per-result errors before trusting batch output.
