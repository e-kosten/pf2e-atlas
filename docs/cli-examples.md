# PF2e Atlas CLI Examples

Use `--json` for structured results and human output for reading. Examples assume
`atlas setup` has prepared the default workspace. Add `--index /path/to/workspace.sqlite`
to data commands to use another workspace consistently.

## Exact Lookup And Reading

```bash
atlas record resolve 'Treat Wounds' --pack-name actionspf2e --json
atlas record resolve 'Shield Block' --kind feat --alternatives 5 --json
atlas record get actionspf2e:1kGNdIIhuglAjIp9
atlas record get actionspf2e:1kGNdIIhuglAjIp9 equipment-srd:s1vB3HdXjMigYAnY --json
atlas record get equipment-srd:s1vB3HdXjMigYAnY --detail full
atlas record source equipment-srd:s1vB3HdXjMigYAnY --json
```

Resolution accepts strict names and verified aliases. Resolve ambiguous names to
one returned key before continuing. Treat Wounds is a `rule` in `actionspf2e`;
`action` is not a product record kind.

Human `record get` detail is `summary`, `standard` (default), or `full`. Full
expands supported embedded documents such as spells and journal pages.
`--json` returns the complete selected reference and conflicts with explicit
`--detail`. `record source` exports the checked Foundry DTO, not original JSON.
For a spell key, `record get SPELL_KEY --rank 5` previews that casting rank without
spending resources. Embedded spells retain their local casting context.

## Search And Browse

```bash
atlas search 'walk through walls' --kind spell --limit 8
atlas search 'masked spirit' --kind creature --limit 8
atlas search 'healing spell' --kind spell --where 'level <= 2' --page 2 --limit 10
atlas search --kind equipment --rarity uncommon --limit 20
atlas search --kind creature --trait undead --where 'defenses.hp.maximum >= 80'
```

Omitting query text browses eligible records alphabetically. Repeated `--kind`,
`--pack-name`, and `--rarity` values accept alternatives; repeated `--trait` flags
require every listed trait. Those constraints combine with `--where`.

Search returns root records with attributed match evidence. An embedded passage
can explain why a parent matched; it does not make the parent the embedded item.
Semantic/hybrid counts describe bounded candidate windows, not a corpus census.

To inspect a witness, copy its returned navigation JSON into these placeholders:

```bash
atlas record get PACK:ID --embedded-path 'EMBEDDED_PATH_JSON' --field /system/description/value --passage 'PASSAGE_ADDRESS_JSON' --json
```

Include `--source-fingerprint` when the returned navigation supplies one. Do not
invent IDs or passage ranges. A snapshot-local selection may become stale after
an update; rerun the search to get a current address.

## Compare Search Profiles And Ranking

Prepare both profiles once (this may download a model and run inference):

```bash
atlas setup --search-profile bge-base --search-profile embeddinggemma
```

Then compare without rebuilding:

```bash
atlas search 'masked spirit' --kind creature --search-profile bge-base --json
atlas search 'masked spirit' --kind creature --search-profile embeddinggemma --json
atlas search 'harrowkin' --ranking score --json
atlas search 'harrowkin' --ranking rrf --json
atlas search 'harrowkin' --retrieval fts --json
atlas search 'masked spirit' --retrieval vector --json
```

Default search is hybrid with score-based fusion. Ranking switches affect only
hybrid text queries. FTS targets names, verified aliases, typed vocabulary,
embedded names and structurally named definitions; semantic search covers selected
root and embedded prose. Scores are retrieval evidence, not probabilities.

Each request uses one stored embedding profile. BGE Base is the default when both
are stored; a sole stored profile is selected automatically. Semantic text search
needs that model's local assets. FTS needs no model weights, and Similar uses
stored vectors without inference:

```bash
atlas similar 'Dirge of Doom' --kind spell --search-profile embeddinggemma --limit 8
```

Similar uses the seed's root identity vector, not every passage in its document.
Filters constrain candidate results, not the seed lookup. If the seed name is
ambiguous, resolve it separately and pass its canonical key.

## Discover Filters

```bash
atlas filters fields
atlas filters fields --group movement --json
atlas filters fields --text 'maximum hp' --json
atlas filters fields --field defenses.hp.maximum --json
atlas filters fields --all --json
atlas filters fields --field movement.speeds
atlas filters fields --field 'movement.speeds[].type' --json
atlas filters values --field 'movement.speeds[].type' --kind creature --json
atlas filters values --field traits --kind spell --text fire --limit 20 --offset 0 --json
atlas filters values --field defenses.hp.maximum --kind creature --json
```

Default/group field output uses the product editor groups. `--all`, `--text`
and `--field` JSON expose query descriptors. Human collection output also lists
entry fields; `--text movement.speeds --json` discovers the collection and members
together. `movement.speeds[]` by itself is not a field ID.

Values returns contextual counts for categorical/Boolean fields, numeric ranges
for numeric fields, or collection availability. Closed fields retain zero-match
choices; open fields enumerate observed values. `--text` searches values, not
record prose. Counts retain all supplied filters and count distinct parents per
value after remaster preference. Source-availability statistics are before that
preference; collection occurrences may exceed the number of parents. Unavailable
counts are not zero counts.

Use one `exists` for constraints that must match the same embedded entry:

```bash
atlas search --kind creature --where '"common" in languages'
atlas search --kind creature --where 'movement.speeds.exists(s, s.type == "fly" && s.feet >= 30)'
atlas search --kind creature --where 'defenses.resistances.exists(r, r.type == "fire" && r.value >= 10)'
atlas search --where 'embedded.spells.exists(s, "fire" in s.traits && s.spell.rank >= 3)'
atlas search --where 'kind == "spell" && "fire" in traits'
atlas search --where 'defenses.saves.fortitude >= 12' --print-filter --json
```

`embedded.spells` matches parents containing immediate spell copies; it does not
match a standalone spell itself. Separate `exists` expressions can match different
entries. Scoped value discovery can also match a different entry from an existing
collection condition; use the full `search --where` to check same-entry constraints.
Unknown source facts do not become zero. Discover fields and their units rather
than guessing paths; for example, `physical.unit_price_gp` is unit price in gold.

## Follow References

```bash
atlas graph links actionspf2e:1kGNdIIhuglAjIp9 --backlinks 4 --json
atlas graph links actionspf2e:1kGNdIIhuglAjIp9 --outgoing 0 --backlinks 8 --json
atlas graph uses 'Frightened' --limit 5 --json
atlas graph variants 'Dread Ampoule' --json
```

`links` exposes outgoing relationships and optional backlinks. `uses` finds
incoming references. Use `graph remaster PACK:ID` for verified counterparts.
Relationships retain unresolved targets and source occurrences. Suggested variants
are bounded candidates, not aliases. Search currently has no `--references` or
`--referenced-by` filter; use graph commands for those questions.

## Saved Lists And Dice

These list commands create or change saved state:

```bash
atlas lists create undead-research --name 'Undead Research' --json
atlas lists add undead-research 'Skeleton Guard' --json
atlas lists show undead-research --json
atlas lists export undead-research --output undead-research.json
atlas lists import undead-research.json --json
```

Reference updates preserve lists and encounters in the shared workspace. Missing
records retain saved snapshots and unresolved status. Encounters are managed in
the web app, not through an encounter CLI command.

General dice rolls need no setup and do not change encounter state:

```bash
atlas roll '1d20+12'
atlas roll '2d20kh1+12' --json
atlas roll '2d20kl1+12' --json
atlas roll '(2d6+4)*2' --json
atlas roll -- '-1d6+10'
```

`kh`/`kl` keep high/low dice. JSON retains individual results, kept/dropped states
and intermediate totals. Quote expressions containing shell operators.

## Readiness And JSON

```bash
atlas setup --check --json
atlas index check --json
atlas index check --no-embeddings --json
atlas index validate --json
```

Deep validation scans the artifact; use it for diagnostics, not before every query.
For a lexical-only workspace add `--no-embeddings` to validation too.

Most JSON commands return `status: "ok"` with `data`, or a top-level `error`.
`record get` and `record resolve` instead return ordered `data.results` entries,
including duplicates, with per-input success/error status. Runtime failures mark
remaining inputs `unprocessed`; keep successful siblings. Their outer status is
`ok`, `partial`, or `error`. Exit codes are 0 success, 1 miss/ambiguity, 2 invalid
input and 3 runtime failure; mixed record failures use the highest code.
Index validation can return `status: "ok"` with `data.valid: false` and exit 3.
Check both exit status and payload before treating output as a successful answer.
