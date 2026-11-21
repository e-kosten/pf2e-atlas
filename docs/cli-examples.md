# PF2e Atlas CLI examples

These examples assume you have run `atlas setup`. Run setup after installation
and after upgrading Atlas. Commands normally print text for reading; add `--json`
when passing results to a script or agent. To use a different workspace, add
`--index /path/to/workspace.sqlite` to each command that reads or changes it.

## Exact lookup and reading

Find an entry by name, then read it using the returned `pack:id` key:

```bash
atlas record resolve 'Treat Wounds' --pack-name actionspf2e
atlas record get actionspf2e:1kGNdIIhuglAjIp9
```

Names and verified aliases can resolve to more than one entry. Choose the intended
result and use its key for later commands. Treat Wounds has kind `rule`, so use
`--kind rule` rather than `--kind action` when filtering for it.

Choose how much to read, or request several entries as JSON:

```bash
atlas record resolve 'Shield Block' --kind feat --alternatives 5 --json
atlas record get equipment-srd:s1vB3HdXjMigYAnY --detail full
atlas record get actionspf2e:1kGNdIIhuglAjIp9 equipment-srd:s1vB3HdXjMigYAnY --json
```

Detail levels are `summary`, `standard` (the default), and `full`. Full includes
supported entries contained in the record, such as a creature's spells or journal
pages. It does not open every linked record. For a spell, `record get SPELL_KEY
--rank 5` previews its facts at that rank without spending resources; a creature's
included spells keep their own casting details.

`--json` returns the complete selected entry and cannot be combined with explicit
`--detail`. See [source data](#inspect-source-data) for information beyond Atlas's
usual presentation.

## Search and browse

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

A creature can match because of text in one of its abilities or spells. Results
identify the matching passage while keeping the creature as the result. Search by
meaning returns a limited set of Top matches; its counts do not describe every
entry that might fit the idea. See [matching passages](#inspect-a-matching-passage)
to open the exact text from JSON results.

## Compare search profiles and ranking

The [search guide](search.md) explains the browser settings. To compare models,
prepare both once (this may download files and take time to process the content):

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

The default combines keyword and meaning-based matches (`hybrid`) using
score-based ranking. `--ranking rrf` combines those matches by their rank instead.
Ranking options affect hybrid text search; `--retrieval fts` uses keyword search
and `--retrieval vector` searches by meaning alone.

Keyword search covers names, verified aliases, indexed terms, names of contained
entries and named definitions. Meaning-based search covers selected descriptions
and other prose, including text in contained entries. Scores help order matches;
they are not probabilities.

Record Similar defaults to Passage matches. Try `--method baseline` for Name and
identity, or `--method content-average` to compare selected content as a whole.
Starting from a trait uses Trait definition by default. If the starting name is
ambiguous, resolve it separately and use the returned key. Filters apply to the
results, independently of resolving the starting entry.

```bash
atlas similar 'Dirge of Doom' --kind spell --search-profile embeddinggemma --limit 8
atlas search 'forest guardian' --search-content core
atlas similar 'Lich' --sections core --search-content all
atlas similar 'Brown Mold' --list-parts --sections core --json
atlas search 'healing' --kind spell --sort level_asc
atlas search --kind equipment --sort level_desc --sort unit_price_asc
atlas search --kind creature --sort random --seed 42
```

Use `--list-parts` to inspect selectable sections. All and Core include the name
and basic details; Core selects the main sections for each type of entry. A
retained random seed keeps ordering consistent between pages; omit `--seed` to
generate one. See [model selection details](#search-model-selection) for defaults,
missing model files and keyword-only setup.

## Discover filters

List available fields, inspect a field, or find values to filter by:

```bash
atlas filters fields
atlas filters fields --group movement --json
atlas filters fields --text 'maximum hp' --json
atlas filters fields --field defenses.hp.maximum --json
atlas filters values --field traits --kind spell --text fire --limit 20 --offset 0 --json
```

Discovery shows supported field names, types, units and examples. Use those names
in `--where` expressions instead of guessing paths. Use one `exists` expression
when several requirements must match the same ability, spell or other contained
entry. See [collection expressions](#collection-expressions) for examples and
[discovery details](#filter-discovery-details) for counts and scripting options.

## Read definitions

Discover trait tokens for filtering with `atlas filters values --field traits`.
When a value has a definition key, copy it into `atlas record definition`.

```bash
atlas filters values --field traits --text deadly
atlas record definition trait:deadly-d8
```

## Follow references

Find entries that reference a rule, inspect links, or look for other editions and
suggested variants:

```bash
atlas graph links actionspf2e:1kGNdIIhuglAjIp9 --backlinks 4 --json
atlas graph links actionspf2e:1kGNdIIhuglAjIp9 --outgoing 0 --backlinks 8 --json
atlas graph links 'Frightened' --outgoing 0 --backlinks 5 --json
atlas record variants 'Dread Ampoule (Moderate)' --json
```

`links` shows references made by an entry and, with `--backlinks`, references to
it. Set `--outgoing 0` for a backlinks-only preview. Read remaster information in the record reader:

```bash
atlas record get spells-srd:gKKqvLohtrSJj3BM
atlas record get bestiary-ability-glossary-srd:hFtNbo1LKYCoDy2O --json
```

Record reads include former names or replacement links and useful changes-journal
destinations. Search, Similar and filter discovery retain all independently matching
legacy and remaster entries. Suggested variants are
possible alternatives, rather than verified names for the same record. Some
source links have unresolved targets; graph previews retain those links.

Graph commands return limited previews. Use Search to find all entries matching
reference conditions, with ordinary filters and pagination:

```bash
atlas search --kind creature --links-to 'Leather Armor' --json
atlas search --links-to 'Leather Armor' --links-to 'Stunned' --json
atlas search --linked-from 'Skeleton Guard' --json
atlas search --links-to-main-entry 'Leather Armor' --json
atlas similar 'Skeleton Guard' --links-to 'Leather Armor' --json
atlas filters values --field traits --kind creature --links-to 'Leather Armor' --json
```

Repeated link flags require all their conditions. Each accepts a name, verified
alias or canonical key. Resolve ambiguous names before using them.
`--links-to` includes references in a matching entry's contents;
`--links-to-main-entry` checks only the main entry. `--linked-from` follows links
from the chosen entry and its contents; `--linked-from-main-entry` checks only
that entry. A reference to equipment or a condition does not by itself mean the
equipment is worn or the condition is applied.

See [reference expressions](#reference-expressions) for AND/OR/NOT combinations
and [occurrence paging](#search-json-and-occurrence-paging) for individual references.

## Saved lists and dice

These list commands create or change saved state:

```bash
atlas lists create 'Undead Research' --slug undead-research --json
atlas lists add undead-research --record 'Skeleton Guard' --json
atlas lists show undead-research --json
atlas lists export undead-research --output undead-research.json
atlas lists import undead-research.json --id undead-copy --json
```

The v2 export preserves saved item order, keys, notes, and snapshots. It can be
written without loading reference records. Import accepts v2 files. A new list
imported from one of these files starts active. Use a full workspace backup to
carry archive status between workspaces.

Archive a list when you want to keep its contents and hide it from the default
list view. You can still open it with `lists show`.

```bash
atlas lists edit undead-research --archive
atlas lists list --include-archived
atlas lists edit undead-research --restore
```

Creation generates a unique slug from the name. Use `--slug` to choose a slug explicitly. An explicit collision fails. Use `lists add` to extend an existing list. Batches report
added, already present and failed inputs. Valid entries are saved together; a failed
input gives a nonzero exit. If every input fails resolution, a new list is empty.
A storage failure rolls back the complete action.

Notes belong to one entry in one list and survive updates and export/import.
Copy the canonical key from `lists show`, then set or clear its note:

```bash
atlas lists note undead-research PACK:ID --text 'A potential garden sentry'
atlas lists note undead-research PACK:ID --clear
```

Reference updates keep lists and encounters. If an entry disappears from the
reference content, the list keeps its saved name and details and marks it
unavailable.

### Characters and party loadouts

Find your saved party and inspect its actual equipment and build context:

```bash
atlas parties list --json
atlas parties show "Example party" --json
atlas characters list --json
atlas characters show "Mira" --section equipment --section money --json
atlas parties context "Example party" --section equipment --section spellcasting --section proficiencies --json
```

Replace the example names with your party and character names. Read commands
accept an exact unique name or the returned stable key; ambiguous names return
candidates. Use stable keys for edits.

`characters show` returns all interpreted details by default. Select identity,
statistics, proficiencies, character-options, abilities, spellcasting, equipment,
money, crafting or companions with repeated `--section` options. Focused text
output shows the selected fields; JSON retains their values and availability.
`parties context` uses the same selectors and reads the ordered roster and each
member's details together.

`data.character` contains the current name, optional level and revision;
`imported_identity` identifies the last imported build. Changing a canonical level
does not update the statistics in an older export. `source_state` distinguishes
an available complete export, an older rules-only import (`reference_only`), and
a manual PC (`no_import`). Re-import rules-only PCs to obtain their loadouts.

The machine fields under `sections.SECTION.fields` use `state` values `value`,
`missing`, `null` and `unrecognized`. Read `value` only when its state is `value`.
Zero, false and an empty list are supplied values. Lists retain their original
entries, including repeated gear or prepared spells. Equipment keeps quantities,
container IDs, customization and exported investment strings; `record_links`
provides unique automatic base-item matches where available.

For loot planning, read equipment with money, proficiencies and casting. Resolve
uncertain names, inspect candidate item rules, and save recommendations with
their reasons in a saved list. Use the player's current budget and confirm any
facts absent from the export before treating a recommendation as an upgrade.

#### Review and correct ability links

Find abilities needing attention, then read the intended rule before linking it:

```bash
atlas characters show CHARACTER_KEY --section abilities --needs-review --json
atlas record resolve 'ABILITY NAME' --json
atlas record get PACK:ID --json
```

Copy the stable ability key and character revision from the character read, and
the artifact generation from the inspected record. Replace the uppercase
placeholders below with those returned values:

```bash
atlas characters link CHARACTER_KEY --ability ABILITY_KEY --record PACK:ID --expected-revision REVISION --expected-generation GENERATION --json
atlas characters show CHARACTER_KEY --section abilities --json
```

Ability entries expose `link_state`: automatic, manual, unlinked, ambiguous or
unavailable. A missing linked rule differs from having no link. A manual correction
can resolve an ambiguous automatic match. `--ability-kind` and `--query` also
filter the sole abilities section. Restore an available automatic match with
`characters link CHARACTER_KEY --ability ABILITY_KEY --automatic --expected-revision REVISION`.

#### Review an import before saving it

Export JSON from Pathbuilder, save it to a file, then preview it:

```bash
atlas characters preview character.json --json
```

Inspect `proposed`, then copy `source_sha256` and `artifact_generation` into the
save command. Choose a unique action ID so an identical retry returns the
original creation outcome:

```bash
atlas characters import character.json --request-id IMPORT_ID --expected-source-sha256 SHA256 --expected-generation GENERATION --json
```

To review a replacement for an existing character, use its key:

```bash
atlas characters preview character.json --character CHARACTER_KEY --json
```

This preview includes the current revision/context and import `changes`, including
retained and removed ability corrections. Refresh replaces the canonical name and
level, complete source and rule reference. After review, use its returned guards:

```bash
atlas characters refresh CHARACTER_KEY character.json --expected-revision REVISION --expected-source-sha256 SHA256 --expected-generation GENERATION --json
```

A changed file or stale revision/generation requires another review. `-` reads
UTF-8 export text from stdin, up to 512 KiB. `--rule-link ABILITY_KEY=PACK:ID` on
import/refresh applies reviewed ability choices. Refresh is revision guarded;
creation replay receipts do not make refresh automatically retryable.

Read or export the original text explicitly, including custom fields:

```bash
atlas characters show CHARACTER_KEY --section source --json
atlas characters show CHARACTER_KEY --section source --output original-character.json
```

The output file must be new and preserves the original accepted bytes. Original
exports can contain player biography and custom data; default context includes
interpreted fields rather than that original text.

#### Edit a reusable roster

Use the reviewed party revision and each selected character revision. `--member`
takes `CHARACTER_KEY=REVISION` and can be repeated:

```bash
atlas parties create "Example party" --json
atlas characters create "Bran" --level 7 --request-id CREATE_ID --json
atlas parties add PARTY_KEY --member CHARACTER_KEY=CHARACTER_REVISION --expected-revision PARTY_REVISION --json
atlas parties set-level PARTY_KEY --member CHARACTER_KEY=CHARACTER_REVISION --level 8 --expected-revision PARTY_REVISION --json
```

Reread the affected roster after each edit to obtain its new revisions. `reorder`
supplies every member once, in the desired order. `remove --character KEY`
removes a membership. `set-level --clear` clears selected canonical levels.
`characters edit` changes a saved name/level; `parties edit` renames a party.
Shared character edits appear in all their rosters. Removing a membership or
deleting a party preserves characters; character deletion detaches encounter
rule references while preserving participant names, levels and combat state.

### Encounters

Create an encounter, add opponents, and inspect the roster and suggested award:

```bash
atlas encounters create "Garden Sentries"
atlas encounters add garden-sentries --record PACK:ID --quantity 2 --adjustment weak
atlas encounters list
atlas encounters show garden-sentries --party-level 7 --party-size 4
```

Use `encounters edit garden-sentries --note 'Starts behind cover'` to change
encounter metadata. Copy a participant key from `encounters show` to remove
that instance with `encounters remove garden-sentries --participant KEY`.
Use `encounters delete garden-sentries` to delete the saved encounter.

The party options preview difficulty and suggested XP per PC for the supplied
party without saving those options to the encounter. Omit them to use the
encounter's saved difficulty reference. Pass `--json` for structured output.

To plan against your saved party, copy the party revision and every member's
key/revision from `parties show`, and the encounter revision from `encounters show`:

```bash
atlas encounters add-party garden-sentries --party PARTY_KEY --expected-party-revision PARTY_REVISION --expected-revision ENCOUNTER_REVISION --member CHARACTER_KEY=CHARACTER_REVISION --request-id ADD_ID --json
atlas encounters show garden-sentries --json
```

Repeat `--member` for the entire reviewed roster in order. Add one saved PC with
`encounters add-character ENCOUNTER --character CHARACTER_KEY --expected-character-revision
CHARACTER_REVISION --expected-revision ENCOUNTER_REVISION --request-id ADD_ID`.
PC additions copy reviewed names/levels and associate live rules; imported sheet
HP, AC and resources are not automatically applied to encounter combat state.
Difficulty then uses the actual encounter roster, including mixed or unknown
levels. For an identical retry, keep the same request ID and all original input
values, even if revisions have since changed. A new intent needs a new ID.

#### Move a saved encounter

Export a saved encounter, including a fight in progress, and import a copy into
an initialized workspace:

```bash
atlas encounters export garden-sentries --output garden-sentries.json
atlas encounters import garden-sentries.json --slug garden-sentries-copy --json
atlas encounters show garden-sentries-copy
```

Export writes a versioned JSON document to stdout by default. A named output
file must be new. Import uses the exported slug unless you supply `--slug`
and fails if that slug is already in use. The result reports missing or
changed reference records. Saved participants and state survive missing
references, though source-dependent actions may need compatible reference data.

Reopen the imported encounter before continuing play. Pending decisions and
committed effects come with the encounter. Existing action contexts and
uncommitted roll drafts belong to the original encounter. Each file is limited
to 64 MiB and 100,000 entries including historical identity slots.

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

## Back up and restore saved workspace data

```bash
atlas workspace export campaign-recovery.json
```

Export creates a new archive file with saved lists, characters, parties,
encounters, and requested reference configuration. When a character has a
complete Pathbuilder import, the archive includes the original JSON, including
any biography and custom fields supplied by the player. Share these archives
with people who should have access to that character data. It includes archived lists, archived
encounters and the state needed to resume a running encounter. Backups
containing archived lists require a version of Atlas that supports list
archiving. The archive contains saved display
snapshots and record keys. Atlas prepares reference records, indexes, and
embeddings separately.

Keep the original workspace and archive. Choose a new database path and prepare
reference records there. Atlas may download reference data. The
`--no-embeddings` option skips preparing embedding models for the new database.

```bash
atlas setup --no-embeddings --index "campaign-recovered.sqlite"
atlas workspace restore "campaign-recovery.json" --index "campaign-recovered.sqlite"
```

For an upgrade recovery, substitute the path to the
`workspace.sqlite.upgrade-archive.json` file beside the original database.
Restore requires a workspace with empty saved state. It can run offline after
setup and retains saved record keys even when those records are unavailable.

If Atlas reports a difference in requested reference settings, run this command
in an interactive terminal:

```bash
atlas workspace restore "campaign-recovery.json" --index "campaign-recovered.sqlite" --replace-reference-config
```

Atlas displays the settings before prompting. Check that any source and
manifest paths are usable on this computer, especially if the archive came
from a different machine. If the settings are unsuitable, decline the prompt.
Accepting an archive without requested settings clears the destination's
requested settings.

If you need to prepare reference data using the restored settings, run
`atlas setup --index "campaign-recovered.sqlite"`. If your build includes the
web UI, open the recovered workspace with
`atlas web --index "campaign-recovered.sqlite" --open` and check your saved
lists, parties, and encounters. You can also check the saved data by exporting
the recovered workspace with
`atlas workspace export "campaign-recovered-check.json" --index "campaign-recovered.sqlite"`.

## Readiness and JSON

Check whether setup is needed:

```bash
atlas setup --check --json
```

Most JSON commands return `status: "ok"` with `data`, or a top-level `error`.
`record get` and `record resolve` instead return ordered `data.results` entries,
including duplicates, with per-input success/error status. Runtime failures mark
remaining inputs `unprocessed`; keep successful siblings. Their outer status is
`ok`, `partial`, or `error`. Exit codes are 0 success, 1 miss/ambiguity, 2 invalid
input and 3 runtime failure; mixed record failures use the highest code.
Index validation can return `status: "ok"` with `data.valid: false` and exit 3.
Check both exit status and payload before treating output as a successful answer.

See [database diagnostics](#database-diagnostics) for deeper checks.

## Shell completion

Generate completion with `atlas completions zsh` (or `bash`, `fish`, `powershell`,
`elvish`). For the current zsh session:

```sh
source <(atlas completions zsh)
```

Shells that support live completion suggest saved-list IDs and record keys from
the workspace selected by `--index` and `--path-mode`. If the workspace is
unavailable, those suggestions are empty. Completion does not upgrade your data
or load a search model. Regenerate scripts after upgrading Atlas.

## Advanced reference

The following sections cover exact JSON navigation, filter counts and detailed
search options for scripts and tools. JSON field names and CLI flags are shown
literally so you can use them in commands.

### Inspect source data

```bash
atlas record source equipment-srd:s1vB3HdXjMigYAnY --json
```

This exports Atlas's checked, structured copy of the underlying Foundry record,
including contained entries and information outside the usual presentation. It
includes typed presence/union tags; it is not the original source-file JSON.
It works without the original source checkout.

### Inspect a matching passage

To open the exact passage that matched, copy the navigation values from a search
result into these placeholders:

```bash
atlas record get PACK:ID --embedded-path 'EMBEDDED_PATH_JSON' --field /system/description/value --passage 'PASSAGE_ADDRESS_JSON' --json
```

Include `--source-fingerprint` when the returned navigation supplies one. Do not
invent IDs or passage ranges. Navigation values can become stale after a reference
update; rerun the search to get a current address.

### Filter discovery details

The default field list and `--group` use the same groups as the browser filter
editor. `--all`, `--text` and `--field` JSON return field descriptions. Text output
for collections also lists entry fields. `--text movement.speeds --json` discovers
the collection and its members together; `movement.speeds[]` alone is not a field ID.

`filters values` returns counts for categorical/Boolean fields, ranges for numeric
fields, or availability for collections. Fields with a fixed choice list include
choices with no matches; other fields list values found in the reference data.
`--text` searches values, not record prose.

Counts respect supplied filters and count each containing record once per value.
Source-availability statistics describe the applicable artifact or saved-list
vocabulary. A collection may have several occurrences in the same record.
Unavailable counts do not mean zero matches.

`--list ID` restricts results to a saved list. `--artifact-generation` requires the
same reference-data version as an earlier response. Optional `--clause-id` asks
for replacement suggestions for that filter condition. `--query`, `--retrieval`
and `--search-profile` are not discovery options; use ordinary search for ranked
matches.

Additional field and value queries:

```bash
atlas filters fields --all --json
atlas filters fields --field movement.speeds
atlas filters fields --field 'movement.speeds[].type' --json
atlas filters values --field 'movement.speeds[].type' --kind creature --json
atlas filters values --field defenses.hp.maximum --kind creature --json
```

### Collection expressions

Use one `exists` for constraints that must match the same embedded entry:

```bash
atlas search --kind creature --where '"common" in languages'
atlas search --kind creature --where 'movement.speeds.exists(s, s.type == "fly" && s.feet >= 30)'
atlas search --kind creature --where 'defenses.resistances.exists(r, r.type == "fire" && r.value >= 10)'
atlas search --where 'embedded.spells.exists(s, "fire" in s.traits && s.spell.rank >= 3)'
atlas search --where 'kind == "spell" && "fire" in traits'
atlas search --where 'defenses.saves.fortitude >= 12' --print-filter --json
```

`embedded.spells` finds records that directly contain spell copies, rather than
standalone spells. It checks only immediate contents. Separate `exists` expressions can match different entries. Value discovery
within a collection can also match a different entry from an existing condition;
use `search --where` to check requirements on the same entry.
Unknown source facts do not become zero. Discover fields and their units rather
than guessing paths; for example, `physical.unit_price_gp` is unit price in gold.

### Reference expressions

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
For a related entry that meets structured conditions, use
`links_to_matching` or `linked_from_matching`. The expression applies to one
standalone entry on the other end of the reference. For example, find entries
that link to a rare spell, or entries linked from a creature of level 5 or higher.

```bash
atlas search --where 'links_to_matching(kind == "spell" && rarity == "rare")'
atlas search --where 'linked_from_matching(kind == "creature" && level_or_rank >= 5, "main_entry")'
```

Link functions operate on result roots and cannot appear inside authored
collection `exists` expressions. `--print-filter` works without opening a workspace
and requires canonical keys for exact reference targets, including when using
convenience flags.

### Search JSON and occurrence paging

Browse, text search and Similar JSON return a search page in `data`: `rows`,
`page`, `relationship_clauses`, `coverage`, `mode`, ordering and generation.
Each row contains its record, text witnesses and `relationship_evidence`. Counts
describe distinct containing records. Meaning-based results retain their limited
Top matches coverage.

Positive link conditions have clause IDs, limited occurrence previews and separate
occurrence totals. Negative conditions and OR branches that did not match do not
contribute evidence.

Page occurrences independently using the complete filter and returned generation:

```bash
atlas graph occurrences RESULT_KEY --links-to 'Leather Armor' --artifact-generation GENERATION --page 2 --limit 20 --json
```

Record relationships and prepared references use a tagged `destination` object
in CLI JSON. For example, `destination.kind` is `resolved_record` when
`destination.target` contains record navigation, or `unverified_url` when
`destination.url` contains an authored external address. Unresolved and blocked
destinations carry only their kind. This shape is shared by record details,
graph output, and relationship evidence in Search and Similar.

When several positive link conditions exist, pass `--clause-id` from the search
relationship summary. Changing the expression can change generated clause IDs;
repeat the original expression. If reference data has changed, the command fails;
rerun the search to obtain current navigation values. Encounter and Related
previews remain limited previews.

### Search model selection

Each command uses one prepared search profile. BGE Base is selected when both
profiles are available; a single prepared profile is selected automatically.
Text searches by meaning need that model's downloaded files. Keyword search
needs no model files. Similar uses comparisons prepared during setup, so it can
run even after model files have been removed.

Setup without profile flags keeps the configured set; explicit `--search-profile`
flags add profiles to it. Setup repairs missing or changed files for selected
models. Selecting a prepared model for a query does not download or rebuild it.

For a new workspace without models:

```bash
atlas setup --no-embeddings
atlas search 'healing' --retrieval fts
```

On an existing workspace, `--no-embeddings` keeps already configured models.

### Database diagnostics

```bash
atlas index check --json
atlas index check --no-embeddings --json
atlas index validate --json
```

Deep validation scans the reference database; ordinary source ingest does not run
this complete audit. Use it to investigate problems, rather than before every query.
For a workspace without models, also add
`--no-embeddings` to validation.

### Rebuild corrupted reference data

If validation reports damaged reference data, rebuild the Foundry-derived records,
text indexes, and configured embeddings:

```bash
atlas index rebuild
```

The rebuild uses the saved Foundry source, locale, and search profiles. Have the
configured source checkout and model files available locally. A lexical-only
workspace stays lexical-only. Use `--index` to select a specific workspace.
Use `--source PATH` or `--manifest PATH` if a saved path is invalid. Atlas saves
the corrected paths when repair succeeds.

Atlas keeps saved lists, parties, encounters, and stable record IDs. It validates
the replacement before committing and clears any interrupted maintenance marker
in that same commit. This command requires a current-schema SQLite workspace with
an authenticated migration ledger and intact saved data. Physical SQLite damage
and saved-data recovery require a separate recovery process.
