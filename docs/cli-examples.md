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

See the [search guide](search.md) for browser settings, Core content and sorting.

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

Record Similar defaults to Passage matches over selected content. Use
`--method baseline` for Name and identity, or `--method content-average` to compare
content averages. Trait Similar defaults to Trait definition. Use `--list-parts`
to inspect selectable sections; see the [search guide](search.md) for Core scopes.
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
Counts use structured filters and saved-list membership. `--list ID` intersects
that scope; `--artifact-generation` can bind discovery to an admitted reference
generation. Optional `--clause-id` asks for replacement suggestions at that clause.
`--query`, `--retrieval` and `--search-profile` are not discovery options. Semantic
candidate summaries are deferred; use ordinary search for ranked retrieval.


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
are bounded candidates, not aliases. These graph previews can include unresolved
links and have explicit limits. Use search for complete filtered root results:

```bash
atlas search --kind creature --links-to 'Leather Armor' --json
atlas search --links-to 'Leather Armor' --links-to 'Stunned' --json
atlas search --linked-from 'Skeleton Guard' --json
atlas search --links-to-main-entry 'Leather Armor' --json
atlas similar 'Skeleton Guard' --links-to 'Leather Armor' --json
atlas filters values --field traits --kind creature --links-to 'Leather Armor' --json
```

Repeated link flags require every condition. Each flag accepts a strict name,
verified alias or canonical key independently of the candidate filters. Ambiguous
names require resolving and choosing a key. `--links-to` checks links made by
matching roots and their contents; `--links-to-main-entry` checks only the roots.
`--linked-from` checks the chosen entry and its contents;
`--linked-from-main-entry` checks only that entry. References do not establish
that equipment is worn, a condition is inflicted, or any other gameplay effect.

Use CEL for AND/OR/NOT combinations. CEL link functions require canonical keys;
resolve names with `atlas record resolve NAME --json` first:

```bash
atlas search --kind creature --where 'links_to("equipment-srd:ARMOR_KEY") || links_to("equipment-srd:OTHER_ARMOR_KEY")' --json
atlas search --where '!links_to("conditionitems:CONDITION_KEY", "main_entry")' --json
atlas search --where 'linked_from({"record_key":"bestiary:CREATURE_KEY","embedded_path":[{"collection":"/items","identity":{"Stable":"OWNED_ID"}}]}, "main_entry")' --json
```

Replace example keys with returned identities. The optional scope is `whole_entry`
(default) or `main_entry`. Exact selection maps accept `record_key`, `embedded_path`,
`field`, `passage`, `source_fingerprint` and optional `spell_rank`. Copy addresses
and fingerprints from returned navigation; do not invent embedded identities.
Link functions operate on result roots and cannot appear inside authored
collection `exists` expressions. `--print-filter` remains artifact-free and
requires canonical keys, including when using convenience flags.

Browse, text search and Similar JSON return the shared search page directly in
`data`: `rows`, `page`, `relationship_clauses`, `coverage`, `mode`, ordering and generation.
Each row contains its record, text witnesses and `relationship_evidence`. This
replaces the earlier CLI `results`/`pagination` shapes; there are no aliases.
Root counts are distinct records. Semantic results keep their bounded coverage.
Positive link conditions have clause IDs, bounded occurrence previews and separate
occurrence totals. Negative or unmatched OR branches have no invented evidence.

Page occurrences independently using the complete filter and returned generation:

```bash
atlas graph occurrences RESULT_KEY --links-to 'Leather Armor' --artifact-generation GENERATION --page 2 --limit 20 --json
```

When several positive link conditions exist, pass `--clause-id` from the search
relationship summary. Changing the expression can change generated clause IDs;
repeat the original expression. Stale generations fail rather than rebinding a
selection. Existing encounter and Related previews retain their bounded behavior.

## Saved Lists And Dice

These list commands create or change saved state:

```bash
atlas lists create undead-research --name 'Undead Research' --json
atlas lists add undead-research 'Skeleton Guard' --json
atlas lists show undead-research --json
atlas lists export undead-research --output undead-research.json
atlas lists import undead-research.json --json
```

Creation rejects an existing list ID; use `lists add` to extend it. Batches report
added, already present and failed inputs. Valid entries are saved together; a failed
input gives a nonzero exit. If every input fails resolution, a new list is empty.
A storage failure rolls back the complete action.

Notes belong to one entry in one list and survive updates and export/import.
Copy the canonical key from `lists show`, then set or clear its note:

```bash
atlas lists note undead-research PACK:ID --text 'A potential garden sentry'
atlas lists note undead-research PACK:ID --clear
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

## Shell completion

Generate completion with `atlas completions zsh` (or `bash`, `fish`, `powershell`,
`elvish`). For the current zsh session:

```sh
source <(atlas completions zsh)
```

Supported live completion shells suggest saved-list IDs and canonical record keys
from the selected workspace, respecting `--index` and `--path-mode` overrides.
Completion does not upgrade the workspace or load models; an unavailable workspace
returns no saved-value suggestions. Regenerate scripts after upgrading Atlas.
