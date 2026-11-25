---
name: pf2e-atlas-cli
description: Answer Pathfinder 2e questions and plan for saved parties using the local Atlas CLI, with character builds and equipment, rule-link corrections, source-attributed search, saved lists, encounters and dice rolls.
---

Use `atlas` for PF2e questions. It reads a local artifact built from the Foundry
PF2e source. Product reads work without the original source clone. Run
`atlas --help` and subcommand help when the installed version's surface is unclear.
Use the caller's workspace (`--index PATH`) consistently across related commands;
otherwise Atlas uses its global workspace. In a checkout, use `--path-mode repo`
only when the caller intends its local workspace. Use `atlas characters` and
`atlas parties` for saved PC management and party planning context; `atlas web`
provides interactive encounter play.

## Installing Atlas

If `atlas --version` is unavailable, install the CLI before attempting lookups.
Installing this skill with `npx skills add` installs instructions only, not the
executable or reference data. The skill from the repository's `main` branch tracks
the nightly CLI. For a released CLI, prefer the matching skill bundled with
`atlas agent skills install`; consult `atlas --help` when versions differ.

For the current nightly on macOS (Apple Silicon) or Linux:

```sh
curl -fL https://github.com/e-kosten/pf2e-atlas/releases/download/nightly/latest/atlas-installer.sh -o /tmp/atlas-installer.sh
sh /tmp/atlas-installer.sh --version nightly
```

On Windows (PowerShell):

```powershell
Invoke-WebRequest https://github.com/e-kosten/pf2e-atlas/releases/download/nightly/latest/atlas-installer.ps1 -OutFile "$env:TEMP/atlas-installer.ps1"
& "$env:TEMP/atlas-installer.ps1" -Version nightly
```

Follow the installer's PATH instructions, then run `atlas --version` and
`atlas setup --check --json`. If setup is needed, choose normal semantic search
(`atlas setup`) or a faster lexical-only start (`atlas setup --no-embeddings`)
with the user before starting downloads and indexing. Standard English setup
downloads precomputed corpus embeddings and missing query model weights. If the
precomputed data is unavailable, retry setup or discuss
`atlas setup --build-embeddings-locally`; local generation can take hours.
Installing the skill does not authorize downloads or local generation by itself.
For stable releases, supported platforms and manual archive installation, see
[the installation guide](https://github.com/e-kosten/pf2e-atlas#quick-start).

## Keeping the installed skill current

After updating the executable, run `atlas agent skills doctor --json` to compare
installed copies with that binary's bundled skill and detect workspace/global
shadowing. Refresh the intended target and scope, for example:

```sh
atlas agent skills install --target codex --scope global --force --yes
```

Use the target/scope reported for the copy being updated. `--force` replaces local
skill edits; preserve intentional customizations before using it. The installer
uses the running binary's bundled instructions, not the latest repository file.

## Readiness and search profiles

Check readiness with `atlas setup --check --json`. Use `atlas setup` to prepare
source, the default pinned BGE Base profile and artifact, or `atlas setup
--no-embeddings` for a new lexical-only workspace. On an existing workspace,
`--no-embeddings` preserves configured profiles; it does not remove them.
Setup can download assets and run lengthy inference; inspect readiness before
starting a build and preserve the chosen source, locale and profiles.
`atlas index check --json` checks readiness. Add `--detail full` for storage and profile statistics; `atlas index validate --json` deeply
checks the artifact.
Use `--no-embeddings` with either index command for a lexical workspace. Supported
workspace upgrades preserve saved state through migrations and component backfills.
Pre-workspace artifacts are unsupported; create a new workspace rather than deleting
an existing database on an error. Indexing locale is
English by default; `atlas setup --locale fr` or `atlas index build --locale fr`
prepares that source locale with English fallback. Changing search locale requires
rebuilding. Setup preserves an existing locale unless explicitly overridden.

The built-in profile IDs are `bge-base` and `embeddinggemma`. Use
`atlas setup --search-profile bge-base --search-profile embeddinggemma` to build
both profiles in one workspace. Setup and `index build` add explicitly requested
profiles to the configured set; omission preserves that set.
`atlas web --search-profile ID` chooses the server default;
`atlas search --search-profile ID` and `atlas similar --search-profile ID` select
one request. Omitted launch selection uses the sole stored profile, or BGE Base
when both are stored; fresh setup uses BGE Base. Explicit unavailable selections
fail; never infer a fallback or mix model scores. Similar uses stored vectors and
does not require model weights. Search responses report the resolved profile and
complete artifact generation. These operational choices do not establish a
quality winner between models.

For requested maintenance, `atlas setup --offline` reuses local assets without
downloading missing source, models or precomputed embeddings. `atlas index build`
is an advanced manual source update; its workspace destination flag is `--output`,
whereas setup, reads and `index rebuild` use `--index`. `atlas index rebuild --json`
repairs generated reference data using the current-schema workspace's saved
source, locale and profiles. It needs local source and configured model assets,
validates the replacement before committing it, and retains saved app data.

Inspect cache cleanup with `atlas setup clean --all --check`. To remove those
source/model caches, use `atlas setup clean --all --yes`; choose `--embeddings` or
`--source-checkout` for one cache. Cleanup preserves the workspace and saved work,
but later setup or inference may need downloads again.

## Output and exact lookup

Prefer JSON output. `record get` and `record resolve` return ordered
`data.results`, including for one input. Outer `status` is `ok`, `partial`, or
`error`; each input has `status: ok` with `value`, `status: error` with `error`,
or `status: unprocessed` with `reason: prior_runtime_failure`. Preserve successful
siblings when a batch fails. Whole-command failures use a top-level `error`.
Errors expose `code`, `message`, and optional `details` and recovery hints.
`workspace`, `open`, `completions` and ordinary `web` commands do not offer the
standard `--json` envelope. List and encounter export commands write portable
JSON documents directly, not envelopes.
Record batch exit classes are 0 success, 1 miss/ambiguity, 2 invalid input and
3 runtime failure; mixed failures use the highest class. Other data commands use
the usual
`{ "status": "ok", "data": ... }` envelope. Index check and validate return `data.valid: true` on success. Exit 3 with
`data.valid: false` identifies a completed check that found a problem. Inspect
`data.reason.code`. Exit 3 with a top-level error means the check could not run.
Do not treat a failure, ambiguity or bounded candidate count as a complete answer.

Resolve identities before relying on a guessed key:

```sh
atlas record resolve 'Treat Wounds' --pack-name actionspf2e --json
atlas record get actionspf2e:1kGNdIIhuglAjIp9 --json
```

Keys are Foundry `pack:id` identities. Strict resolution accepts authored names,
normalized names and verified aliases, preserving their evidence. Ambiguous
resolution requires choosing the intended record; use its returned key in later
commands. Name similarity and Suggested variants do not establish identity.
A unique match returns `value.selected` and a generation-bound `value.navigation`.
`--alternatives N` controls how many ambiguous candidates are displayed.
Use the published title, edition and pack to disambiguate. Preserve the total.

Use `record get --detail summary --json` for identity, standard for a readable
reference, and `--detail full --json` for the selected record's detailed
semantic body, prepared fields and relationships. Full human output expands
supported embedded entries. For a specific question, request only the sections
needed (rules, defenses, strikes, abilities or spellcasting):

```sh
atlas record get PACK:ID --section defenses --section strikes --json
atlas record get --navigation 'RETURNED_NAVIGATION_JSON' --json
```

Copy the navigation object exactly. It binds the selected root or child, spell
rank, field, passage and source fingerprint to its artifact generation. Refresh
the original lookup when that selection is stale. In each successful result
`value.identity` identifies the selection and `value.selected` can reopen it.
Keep the containing record distinct when citing an embedded spell or ability.
A field or passage request contains only the selected content, even at full detail.
Use the reported publication for attribution and a desktop record link to open
the root in Atlas.

Read typed availability before using a value. Spell heightening and forms
describe authored changes. `--rank` previews supported changes at that rank.
Sparse overrides retain the distinction between an omitted member and an explicit
null. Embedded spells keep their casting entry's attack, DC and customizations.

To inspect source information outside Atlas's
presentation, use `atlas record source PACK:ID --json`. It exports the root's
complete checked Foundry DTO and embedded documents, with codec identity and
source path/hash. Presence tags and retained invalid/additional members are part
of that checked representation; it is not original Foundry JSON. It works without
the source clone and does not execute rules or macros.

## Trait definitions

Traits use exact `trait:<identifier>` keys, separate from Foundry `pack:id` records:

```sh
atlas record definition trait:agile --json
atlas record definition trait:deadly-d8 --json
atlas search --trait deadly-d8 --json
```

Keep variant suffixes such as `-d8`; do not substitute a base trait. Trait JSON
includes the definition key, label, content and available source information.
Use `filters values --field traits` to discover filter tokens and their definition keys.
A successful response can have `content: null`: report the missing definition
rather than inventing text or substituting another trait. Trait membership uses
the exact identifier; `--trait deadly` does not include `deadly-d8`. `record get`,
`record resolve` and `graph` operate on records.
Trait Members and References views remain available in the web UI.

## Search and evidence

Use strict resolution for a known name, search for a concept or partial name,
and query-free search to browse records satisfying filters. Then inspect the
returned record and matching passage:

```sh
atlas search 'Ghoul Fever' --retrieval fts --json
atlas search fire --search-content core --json
atlas search 'a creature whose disease drains a victim over time' --json
atlas search --kind creature --trait undead --json
atlas search --kind creature --sort level_desc --json
atlas search --kind equipment --sort level_desc --sort unit_price_asc --json
atlas search --kind creature --sort random --seed 42 --json
atlas search --where 'defenses.hp.maximum >= 80 && "undead" in traits' --json
atlas similar spells-srd:1xLVcA8Y1onw7toT --kind spell --json
atlas similar 'Brown Mold' --list-parts --json
atlas similar 'Brown Mold' --list-parts --sections core --json
atlas similar 'Lich' --sections core --search-content core --json
atlas similar 'Brown Mold' --method passages --decay 0.5 --explain --json
```

Similar uses stored vectors and needs no model weights. Records default to
Passage matches. Source
comparison options without `--method` select Passage matches. Candidate scope
alone (`--search-content`) keeps the seed's method default. `--method passages`
compares selected content with geometric rank decay from the strongest logical
part (0) through equal-part contribution (1). `--method content-average` compares
content centroids; it does not accept decay. `--method baseline` preserves root
identity similarity. It does not accept parts, a source preset, candidate-part mode or decay. It can
restrict candidate content with `--search-content core`.

Use `--list-parts` to discover available source content. Copy exact selectors into
repeatable `--part` arguments and supply `--artifact-generation` from discovery.
For records, omission and `--sections all` select the stored root identity once plus all indexed sections;
`--sections core` selects the stored root identity once plus the Core prose chosen
for the seed's kind. The source preset and repeated `--part` conflict. Discover
Core's exact resolved components with `--list-parts --sections core`; ordinary
`--list-parts` also exposes the identity component for a Custom selection. A
record without allowed Core prose retains identity only, with an explicit reason;
missing required vectors are errors. Trait presets retain their existing definition
or clearly labelled label-only representation without adding a second identity.
Never construct selectors from labels or silently replace stale or unavailable
explicit components. `--candidate-parts any`
allows matches anywhere; `corresponding` restricts comparisons to shared content
roles across kinds. Trait definitions support only any-content matching.
`--search-content all|core` independently chooses eligible candidate content in
Similar and ordinary text search. Each candidate uses its own kind's Core recipe,
not the seed's recipe. For example, `atlas similar 'Lich' --sections core --search-content all` compares the Lich's identity/description with all candidate
prose and root identities. Exact Custom selections can omit identity for a prose-only comparison.
Core requests require current Core metadata; refresh setup on a readiness error
rather than silently retrying with All. Candidate filters remain separate from
seed content selection. `--explain`
returns part contributions and paired evidence; average-method excerpts are
representative, not an additive explanation of the centroid score. Scores are
not percentages or calibrated relevance probabilities.

FTS targets names, verified aliases, typed vocabulary, embedded names and structurally
named definitions. Semantic search independently covers selected root and embedded
explanatory prose. Core only restricts existing eligible names, vocabulary,
structural labels and vectors; it never makes description paragraphs FTS input.
`--search-content` requires a text query and is rejected on query-free browsing.
Hybrid is the default; `--retrieval fts` needs no model
weights, and `--retrieval vector` selects semantic search alone. Hybrid
`--ranking score` is the default; `--ranking rrf` compares rank-based fusion on
the same data. Keep the default unless the caller asks to compare approaches.
Ranking selection does not rebuild data and applies only to hybrid text search.
`--limit` sets page size (1-100); `--page` starts at 1. Pass query text when selecting a retrieval mode.
`--sort` accepts `name_asc`, `name_desc`, `level_asc`, `level_desc`,
`unit_price_asc` or `unit_price_desc`. Repeat it for later tie-breaking attributes;
each field appears at most once. Text and Similar also accept sole `relevance`.
Browse defaults to name ascending; text and Similar default to relevance.
Attribute sorting orders the complete matching set before pagination, without
changing semantic candidate coverage or switching to FTS. Missing values appear
last for each rule in either direction; later rules break ties before the final
name/key order. `--sort random` shuffles the same complete matching set before
paging. `--seed` accepts an unsigned32-bit integer only with Random; omission
generates a seed reported in the result. Reuse it to retain the same order for
unchanged matching records across pages. Random does not expand semantic coverage.
With no configured embeddings, ordinary text search automatically uses FTS.
The Name and identity method uses the seed's stored root identity vector; Passage matches and Content average use the selected source components. Similar filters constrain
candidates, not seed resolution; resolve an ambiguous seed separately.
Similar also accepts `--page` and `--limit` for stateless result pages.
Browse and text search return `data.rows`, `page`, `coverage`, ordering, profile
and artifact generation. Use `--detail records` (the default) for record keys and summaries,
`--detail preview` for the best available match, excerpt and relevant relationship
evidence, or `--detail full` for every returned witness and ranking detail. Similar returns up to one representative pair per row by default.
Supply `--explain --json` to read additional pairs and part or role contributions.
Search and Similar return the effective `result_detail` on the page.
Report bounded semantic and hybrid counts with their coverage and count basis.

Use `atlas record get RECORD_KEY --json` with a row’s `record.record_key`
to read the current record. For an exact selection, serialize
`best_match.navigation` (preview) or the chosen `matches[i].navigation`
(full) as one JSON argument to `atlas record get --navigation`. When the match
identifies only the record, use its key. Repeat the search command’s workspace
options on the read. Rerun search if an exact locator’s generation has changed.
Page and relationship `next_args` can be passed directly to a process runner.

## Filter discovery

Discover filters before constructing a more specific expression:

```sh
atlas filters fields --json
atlas filters fields --group movement --json
atlas filters fields --detail full --json
atlas filters fields --all --json
atlas filters fields --text 'maximum hp' --json
atlas filters fields --field defenses.hp.maximum --json
atlas filters values --field traits --kind spell --json
atlas filters values --field defenses.hp.maximum --kind creature --json
atlas search --where 'defenses.saves.fortitude >= 12' --print-filter --json
atlas search --kind spell --where 'level <= 2' --json
```

`filters fields` describes allowed choices; `filters values` retains all supplied
filters and returns contextual counts (including zero-match closed choices).
`--text` searches values, not record content. Relationship conditions use the
same predicate and are retained in counts and statistics. Use `--list ID` to restrict membership and
`--artifact-generation` to require a prior reference generation. Optional
`--clause-id` requests replacements at one clause. Counts describe distinct
matching parent records. Source-availability statistics describe the applicable
vocabulary and can count multiple collection occurrences per parent.
Unavailable counts differ from zero matches. For same-entry constraints,
combine conditions inside one CEL `exists`, for example
`movement.speeds.exists(s, s.type == "fly" && s.feet >= 30)`.
Use `embedded.spells.exists(s, "fire" in s.traits)` for records containing a fire
spell. This does not match a standalone spell itself; use
`kind == "spell" && "fire" in traits` for those. Equipment predicates use
`embedded.equipment.exists(e, e.source.type == "weapon" && "agile" in e.traits)`.
These collections contain immediate embedded copies, not linked records.
`physical.unit_price_gp` is listed unit price in gold pieces.

Scoped value discovery can match a different entry from an existing collection
condition; verify the complete expression with `search --where`.

The catalog defines valid CEL paths, types, operators, family applicability, units,
closed choices and value discovery. Open strings such as traits use admitted
identifiers and observed values. Common flags `--kind`, `--pack-name`, `--rarity`
and `--trait` combine with `--where` through the same typed predicate. Unsupported
CEL syntax or fields fail explicitly. The retired metric/reference/filter-JSON
flags are not available; do not fabricate an alternative expression language.
Unknown, missing, null, invalid and not-applicable values are distinct. Only a true
predicate admits a record; do not assume missing stats have a numeric value.
Use `filters fields --json` for the directory of group IDs and labels. Use
`--group ID` to inspect one editor group or `--detail full` for all editor groups.
`--all`, `--field`, and `--text` return flat query catalog descriptors.
Numeric and collection discovery return statistics/availability with `values: null`;
categorical and Boolean fields return paged values plus availability counts.

## Relationships and saved lists

Use search for complete filtered record-reference results:

```sh
atlas search --kind creature --links-to 'Leather Armor' --json
atlas search --links-to 'Leather Armor' --links-to 'Stunned' --json
atlas search --linked-from 'Skeleton Guard' --json
atlas similar 'Skeleton Guard' --links-to 'Leather Armor' --json
```

Each flag accepts a canonical key, strict name or verified alias. Resolution is
independent of candidate filters, query and Similar seed. Repeated flags combine
with AND. Ambiguity requires choosing a resolved key; never substitute a fuzzy hit.
`--links-to` checks links from each result's whole entry; `--linked-from` checks
links from the chosen whole entry. Their `--links-to-main-entry` and
`--linked-from-main-entry` variants restrict the source to its main entry.
Whole entry includes nested abilities, spells, equipment and other documents.
References alone do not establish possession, worn equipment or condition effects.

CEL `links_to(KEY[, SCOPE])` and `linked_from(KEY[, SCOPE])` combine with ordinary
conditions using `&&`, `||`, `!` and parentheses. SCOPE is `whole_entry` (default)
or `main_entry`. CEL requires canonical keys; resolve names first. An exact
selection-map overload accepts the returned `record_key`, `embedded_path`,
`field`, `passage`, `source_fingerprint` and optional `spell_rank`. Copy exact
returned navigation, including snapshot-local fingerprints; never invent owner IDs.
Link functions test root records and cannot appear inside authored `.exists`.
`search --print-filter` needs no artifact; named link flags fail there with a
resolution hint. Canonical flags and CEL still print the shared typed predicate.

Search pages count distinct matching roots. Positive relevant relationship leaves
carry clause IDs and bounded occurrence previews in each row. Unmatched OR
branches and negative leaves do not invent link evidence. Page occurrences with
`atlas graph occurrences RESULT_KEY --where ORIGINAL_EXPRESSION --clause-id ID
--artifact-generation GENERATION --page N --json`. The clause selector is optional
when exactly one positive link condition exists. Preserve the original expression
and generation; changing it can change generated clause IDs. Stale generations
fail explicitly. Occurrence counts and root counts are different.

Use graph commands for bounded structural context, including unresolved links:

```sh
atlas graph links 'Demoralize' --backlinks 8 --json
atlas graph links 'Frightened' --outgoing 0 --backlinks 25 --json
atlas record get PACK:ID --json
atlas record variants 'Dread Ampoule (Moderate)' --json
```

Links preserve occurrences and source attribution, including unresolved targets.
Remaster relationships require explicit evidence; a legacy-only query match stays
visible. Suggested variants are bounded same-pack/family candidates with naming
and compatibility evidence. They are not canonical aliases or confidence scores.
Macros are developer-only and never appear as product results; RollTables are
ordinary products.

Keep durable collections in saved lists:

```sh
atlas lists create 'Undead Research' --slug undead-research --json
atlas lists add undead-research --record 'Skeleton Guard' --note 'A basic guard option' --json
atlas lists show undead-research --json
atlas lists show undead-research --sort level_asc --json
atlas lists export undead-research --output undead-research.json
# In the destination workspace
atlas lists import undead-research.json --index /path/to/destination/workspace.sqlite --json
```

`lists create` also accepts repeated `--record` inputs to initialize a new list.
`lists show --sort` accepts repeated attribute orders above, sole `saved_order`
(the default), or `random` with optional `--seed`. Attribute and Random order are
temporary and do not change saved positions.
Use already resolved keys when names are ambiguous. Existing ids fail without
changing the list; use `lists add` for later batches. Batch receipts preserve input
order and report added/already-present/failed results. Any failed input exits
nonzero while valid entries are saved; an all-miss create leaves an empty list.
Transaction failures roll back creation and additions together. Mixed item
outcomes give `status: "partial"`. If every item failed, `status` is `error`.
Read `data.created` and `data.list_key` even after a nonzero exit. Correct the
inputs in `data.retry_inputs` and retry them with `lists add` against that list. `lists add` accepts positional refs, repeated `--record` inputs and
`--stdin` (one ref per line); `--note` applies to each added record.
`lists note <ID> <KEY> --text <TEXT>` edits a note; `--clear` removes it, including
for unavailable entries.

`lists export` writes portable JSON to stdout. With `--output PATH`, choose a new file.
List names are unique across active and archived lists, ignoring capitalization
and surrounding whitespace. On a name conflict, use the returned list key for
the existing list or choose a new name. To import a second copy in the same
workspace, change `list.name` in the exported JSON and supply a new `--id`.
Use `--replace` to replace the list at the target id.
Use `lists edit ID` for name, description, tags or id changes. To hide a list
without deleting it, use `lists edit ID --archive`; discover it with
`lists list --include-archived` and restore it with `lists edit ID --restore`.
Use `lists remove ID PACK:ID` to remove one entry. To delete the whole list,
look up its `list_key` with `lists list --json` and pass it to `lists delete LIST_KEY`.
Keep that key for retries, even if the original slug has since been reused.

Lists share the persistent workspace with reference data. A missing record keeps
its saved snapshot and explicit unresolved status after a reference update. Use
the list commands to change saved state; source updates preserve it.

## Character and actual-party context

Discover saved identities, then read the sections relevant to the task:

```sh
atlas characters list --json
atlas parties list --json
atlas parties list --detail full --json
atlas parties show PARTY_KEY --json
atlas characters show CHARACTER_KEY --section equipment --section money --json
atlas characters show CHARACTER_KEY --section spellcasting --section proficiencies --json
atlas parties context PARTY_KEY --section equipment --section money --section proficiencies --json
```

`characters show` is the single character-detail read. Repeat `--section` to
combine identity, statistics, proficiencies, character-options, abilities,
spellcasting, equipment, money, crafting and companions. With no selector it
returns all interpreted sections. Reads accept a stable key or exact unique
name; ambiguous names return candidates. Mutations require stable keys.

Read `data.character` for current canonical metadata/revision and
`data.imported_identity` for the export's name/level. `source_state` distinguishes
`available`, `reference_only` (re-import required for sheet/loadout), and
`no_import`. Selected sections are explicit; omitted sections are not missing
facts. In standard JSON, `sections.SECTION.fields` uses native numbers, booleans,
lists and objects. Omitted field names mean the export supplied no value.
Explicit null, empty lists, zero and false retain their JSON meanings.
Unrecognized values carry a state tag. Equipment groups associate each item
and rule key with its container. Keep repeated stacks and prepared spells.
Use `--detail full` for all imported fields with explicit availability at
each level. Shared
sheet summaries may derive numbers from the imported build; these are separate
from source facts and current encounter state. Metadata edits do not recalculate
an older imported build.

Equipment `record_links` are unique automatic base-item matches. Read their
records before judging an upgrade; custom display names, runes and materials
remain source facts. Look up uncertain names with `record resolve`, `search`
and `record get`. Do not infer an empty inventory or suitability from unavailable
facts. For loot planning, combine equipment with money, proficiencies and casting,
ask for missing budget/current-play context when needed, then save recommendations
and reasons in an existing or requested saved list. Recommendations do not equip,
purchase or award items.

Repair an ability by reading its current revision and stable ability key:

```sh
atlas characters show CHARACTER_KEY --section abilities --needs-review --json
atlas record resolve 'ABILITY NAME' --json
atlas record get PACK:ID --json
atlas characters link CHARACTER_KEY --ability ABILITY_KEY --record PACK:ID --expected-revision REVISION --expected-generation GENERATION --json
```

Use the generation from the inspected record and current character revision.
`sections.abilities.entries` contains the existing link facts and `link_state`:
automatic, manual, unlinked, ambiguous or unavailable. A valid manual link may
retain automatic ambiguity and should not be treated as unresolved. `--needs-review`,
`--ability-kind` and `--query` require only `--section abilities`. To restore an
available automatic match, use `characters link ... --automatic --expected-revision
REVISION`. No equipment-link correction store is implied.

Preview a player's export before saving:

```sh
atlas characters preview character.json --json
atlas characters import character.json --request-id IMPORT_ID --expected-source-sha256 SHA256 --expected-generation GENERATION --json
atlas characters preview character.json --character CHARACTER_KEY --json
atlas characters refresh CHARACTER_KEY character.json --expected-revision REVISION --expected-source-sha256 SHA256 --expected-generation GENERATION --json
```

Standard JSON uses compact section facts in both `proposed` and `current.sections`.
Use `--detail full` for explicit fact availability.
Take SHA256/generation/revision from preview, inspect `proposed` and any `current`
context plus `changes`, then save the same input. `-` accepts bounded UTF-8 stdin.
Refresh replaces canonical name/level, complete source and rule references; retained
manual corrections follow the existing ability identities. A changed file,
generation or revision requires a fresh review for a new action. For refresh
recovery across invocations, supply `--request-id REFRESH_ID` from the first
attempt. Reuse that ID with the original reviewed export, revision, generation
and choices. Character creation/import and encounter PC addition also accept
exact replays of their original request IDs and inputs.

Create manual characters and reusable rosters, or edit reviewed canonical metadata:

```sh
atlas characters create 'Bran' --level 7 --request-id CREATE_ID --json
atlas parties create 'Garden Party' --json
atlas parties add PARTY_KEY --member CHARACTER_KEY=CHARACTER_REVISION --expected-revision PARTY_REVISION --json
atlas characters edit CHARACTER_KEY --level 8 --expected-revision CHARACTER_REVISION --json
atlas parties set-level PARTY_KEY --member CHARACTER_KEY=CHARACTER_REVISION --level 8 --expected-revision PARTY_REVISION --json
```

Copy keys and revisions from each preceding response; the edit and set-level
examples are alternative ways to change a level. `characters edit` also accepts
`--name` or `--clear-level` and preserves the imported build. Manual characters
have no imported sheet. `characters create` and `characters import` can also add
the new PC to `--party PARTY_KEY` with `--expected-party-revision REVISION`.
Import and refresh accept repeated `--rule-link ABILITY_KEY=PACK:ID` for reviewed
ability corrections from the preview.

Party management also provides edit, delete, remove and reorder.
`--member CHARACTER_KEY=REVISION` is repeatable for reviewed membership/order/level
inputs. Existing-party edits require `--expected-revision`. Reorder supplies every
member once; set-level selects only the members to change, with `--clear` instead
of `--level` to clear their levels. Canonical level edits affect every party sharing
that character. Removing a membership or deleting a party preserves characters.
`characters delete CHARACTER_KEY --expected-revision REVISION` removes the
character and memberships, detaching encounter rule references while preserving
encounter combat snapshots. Inspect subcommand help before a saved-state write.

Original source is opt-in with `characters show CHARACTER_KEY --section source
--json`. `--output NEW_FILE` with only this section writes the exact original
export to a new file. It can include biography/custom player fields; ordinary
context does not return it.

## Encounter prep

Use `encounters list` to find an encounter slug and `encounters show` to read
its roster and difficulty. The list includes archived encounters. In JSON,
`archived` is a boolean and `current_turn_participant_name` is present when
a participant holds the turn. The web encounter and its index row offer Archive
and Restore. Create an encounter or add opponents when the caller
requests a saved-state change:

```sh
atlas encounters list --json
atlas encounters create "Garden Sentries" --json
atlas encounters add-party garden-sentries --party PARTY_KEY --expected-party-revision PARTY_REVISION --expected-revision ENCOUNTER_REVISION --member CHARACTER_KEY=CHARACTER_REVISION --request-id ADD_ID --json
atlas encounters add garden-sentries --record PACK:ID --quantity 2 --adjustment weak --json
atlas encounters show garden-sentries --party-level 7 --party-size 4 --json
atlas encounters export garden-sentries --output garden-sentries.json
atlas encounters import garden-sentries.json --slug garden-sentries-copy --json
```

For a what-if question, run a read-only preview first:

```sh
atlas encounters preview garden-sentries --record PACK:ID --quantity 2 --adjustment weak --party-level 7 --party-size 4 --json
```

Compare the current and projected difficulty against the same party. Level and
size each report whether they came from preview flags, a saved reference, the
roster or an unknown input. When asked to save the reviewed addition, execute
`data.add_args` as an argument vector. It binds the canonical record, roster
revision, reference generation and request ID. Reuse those exact arguments to
confirm a retry. For a stale preview, use `error.details.preview_args` to review
again, or use the original `data.repeat_preview_args` to keep supplied party
level and size. Those party options apply only to the comparison.

`encounters show` includes the note and compact
roster. Use `--participant KEY` for one expanded participant or `--detail full`
for the entire encounter. For an addition, choose a request ID before the first
write and reuse the canonical encounter key, request ID and same inputs on a
retry. Fresh replies return participant keys. An already-committed reply has
`participant_keys: null`; the original receipt confirms the addition.

Use a resolved creature or hazard key for `--record`. Each copy gets its own
participant entry. `--adjustment` accepts `normal`, `elite` or `weak` for
creatures. Omission uses the creature's known adjustment, defaulting to Normal
when unavailable. Eligible Elite and Weak additions get that prefix in the
participant name.

For actual-party planning, first read `parties show/context`, then supply every
member's key/revision to `encounters add-party` in the reviewed order. The example
shows one member; repeat `--member` for the whole roster. For one saved PC use
`encounters add-character ENCOUNTER --character CHARACTER_KEY --expected-character-revision
CHARACTER_REVISION --expected-revision ENCOUNTER_REVISION --request-id ADD_ID`.
These additions preserve reviewed name/level snapshots and live character rule
references, not automatic sheet HP/AC/resource imports. Then add opposition and
use `encounters show` without numeric party overrides to inspect actual roster
difficulty, including mixed/unknown levels. Conflicts require review; identical
request retries preserve the original member/order/revision inputs.

Read `data.difficulty.rating`, `total_xp`, `budgets`, `opposition` and
`xp_per_pc`. Report opposition XP separately from the suggested award per PC.
Check `basis`, `reference_note`, `award_note` and opponent availability before
interpreting missing values. The party flags on `show` preview difficulty
without saving a party override.

`encounters edit ENCOUNTER --name NAME --note TEXT` changes metadata; use
`--clear-note` or `--clear-description` to remove optional text. Remove one roster
entry with `encounters remove ENCOUNTER --participant PARTICIPANT_KEY`, taking the
participant key from `encounters show`. To delete the encounter and its participants,
copy `data.encounter.encounter_key` from `encounters show SLUG --json` and use
`encounters delete ENCOUNTER_KEY`. Retry with that same key. Encounter export writes the complete portable
JSON document directly to stdout or a new `--output` file; import creates a new
encounter and fails if its destination slug already exists.

## Saved workspace backup and recovery

For a complete saved-state backup, including characters, parties and running
encounters, use `atlas workspace export campaign-recovery.json`. This creates a
new archive containing saved state and requested reference configuration;
reference records, indexes and embeddings are prepared separately. Imported
characters include their original player exports, including custom/biography data.

Keep the original workspace and archive. Restore into a separate initialized
workspace with empty saved state:

```sh
atlas setup --no-embeddings --index campaign-recovered.sqlite
atlas workspace restore campaign-recovery.json --index campaign-recovered.sqlite
```

Setup may download reference data; restore can run offline once it is initialized.
A reference-configuration conflict requires reviewing the settings in an
interactive terminal with `workspace restore ... --replace-reference-config`.
The flag prompts; it is not an unattended overwrite option. Restore does not
merge into populated saved state. Use list or encounter import/export to move
those individual collections instead.

## Desktop, browser and shell access

If the installed binary includes web support, `atlas web --open` starts the local
browser UI; pass the same `--index` used for related CLI commands. Restart the
server after updating the executable. On macOS, `atlas open` opens the installed
desktop app; `atlas open 'pf2e-atlas://v1/record/actionspf2e%3A1kGNdIIhuglAjIp9'`
opens a resolved record there. Desktop links select entities in the app's chosen
workspace, rather than inheriting a CLI `--index` override.

`atlas completions zsh` prints a completion script; bash, fish, powershell and
elvish are also supported. Generating it does not install shell configuration.

## Dice and source inspection

Dice rolls need no workspace, source or model:

```sh
atlas roll '1d20+12' --json
atlas roll '2d20kh1+12' --json
atlas roll '(2d6+4)*2' --json
```

`kh`/`kl` keep high/low dice. Quote formulas; put `--` before a formula beginning
with a minus sign. JSON retains individual dice and kept/dropped states. Report
the actual returned roll; these commands do not spend resources, deal damage or
change initiative. For a published table, use `atlas record roll-table TABLE_KEY --json` or its strict
name. Report the actual roll and every matching entry with its source. Mention gaps
and invalid ranges when present. Linked entries are references. Running the
encounter and applying its effects remain interactive web actions.

Use `atlas-dev` only for developer diagnostics: `source load`, `source analyze`,
`source audit-paths`, `index inspect` and `index record`. These commands are a
separate unpublished Rust bundle. Original JSON inspection requires an explicit
`index record --original` request and a configured source clone matching the
artifact provenance. `atlas record source` exports the checked DTO; original file
bytes and admission diagnostics belong to the developer tooling.
