---
name: pf2e-atlas-cli
description: Answer Pathfinder 2e reference questions using the local Atlas CLI, with strict identity lookup, source-attributed search, typed filters, graph context, saved lists and dice rolls.
---

Use `atlas` for PF2e questions. It reads a local artifact built from the Foundry
PF2e source. Product reads work without the original source clone. Run
`atlas --help` and subcommand help when the installed version's surface is unclear.
Use the caller's workspace (`--index PATH`) consistently across related commands;
otherwise Atlas uses its global workspace. Encounters are managed through
`atlas web`; there is no encounter CLI command.

## Installing Atlas

If `atlas --version` is unavailable, install the CLI before attempting lookups.
Installing this skill with `npx skills add` installs instructions only, not the
executable or reference data. The skill from the repository's `main` branch tracks
the nightly CLI. For a released CLI, prefer the matching skill bundled with
`atlas agent skills install`; consult `atlas --help` when versions differ.

For the current nightly on macOS (Apple Silicon) or Linux:

```sh
curl -fL https://github.com/e-kosten/pf2e-atlas/releases/download/nightly/atlas-installer.sh -o /tmp/atlas-installer.sh
sh /tmp/atlas-installer.sh --version nightly
```

On Windows (PowerShell):

```powershell
Invoke-WebRequest https://github.com/e-kosten/pf2e-atlas/releases/download/nightly/atlas-installer.ps1 -OutFile "$env:TEMP/atlas-installer.ps1"
& "$env:TEMP/atlas-installer.ps1" -Version nightly
```

Follow the installer's PATH instructions, then run `atlas --version` and
`atlas setup --check --json`. If setup is needed, choose normal semantic search
(`atlas setup`) or a faster lexical-only start (`atlas setup --no-embeddings`)
with the user before starting downloads and indexing. A first semantic build can
take hours; installing the skill does not authorize that work by itself.
For stable releases, supported platforms and manual archive installation, see
[the installation guide](https://github.com/e-kosten/pf2e-atlas#quick-start).

## Readiness and search profiles

Check readiness with `atlas setup --check --json`. Use `atlas setup` to prepare
source, the default pinned BGE Base profile and artifact, or `atlas setup
--no-embeddings` for a new lexical-only workspace. On an existing workspace,
`--no-embeddings` preserves configured profiles; it does not remove them.
Setup can download assets and run lengthy inference; inspect readiness before
starting a build and preserve the chosen source, locale and profiles.
`atlas index check --json` checks readiness; `atlas index validate --json` deeply
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

## Output and exact lookup

Prefer JSON output. `record get` and `record resolve` return ordered
`data.results`, including for one input. Outer `status` is `ok`, `partial`, or
`error`; each input has `status: ok` with `value`, `status: error` with `error`,
or `status: unprocessed` with `reason: prior_runtime_failure`. Preserve successful
siblings when a batch fails. Whole-command failures use a top-level `error`.
Errors expose `code`, `message`, and optional `details` and recovery hints.
Exit classes are 0 success, 1 miss/ambiguity, 2 invalid input, 3 runtime failure;
mixed failures use the highest class. Other data commands use the usual
`{ "status": "ok", "data": ... }` envelope. Deep index validation can return
`status: "ok"` with `data.valid: false` and exit 3; check the payload as well as status.
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

`content.body` contains semantic creature, hazard, spell, activity, physical,
table, or content facts. Read each fact's availability before using its value.
Spell heightening and forms describe authored changes; `record get --rank` previews
supported rank changes but does not execute a form or spend resources. Embedded
spells retain their local casting context and customizations.
Form and fixed-heightening changes are sparse authored overrides of the base;
an omitted member is not an unavailable full form. Explicit null clears remain
distinct from omitted members. Read casting-entry attack/DC and Lore modifiers
from their typed activity facts rather than reconstructing them from prose.

In a successful `record get` result, read `value.identity` for the selected
content's identity, `value.selected` for its exact navigation, and `value.content`
for text `fields`, structured `body`, classification and embedded navigation.
`value.parent` appears only for embedded selections and identifies the containing
record. Do not mistake the parent for the selected spell or ability. Rank previews
use effective identity/facts. There are no `record` or `presentation` aliases.

For a compact root heading, use `atlas record get PACK:ID --detail summary`.
This terminal path reads body-free summaries. Selected-child summaries identify
the child from its parent snapshot. `--json` returns the complete selected Atlas
reference response and conflicts with explicit `--detail`. The default `--detail standard`
reads the main reference with inline actor abilities and compact spell rosters.
Use `--detail full` to also read supported embedded documents, such as spells,
journal pages and roll-table results. It expands embedded content, not graph links.
An explicit `--field` or `--passage` keeps human output focused on that selection
at every human detail level. To inspect source information outside Atlas's
presentation, use `atlas record source PACK:ID --json`. It exports the root's
complete checked Foundry DTO and embedded documents, with codec identity and
source path/hash. Presence tags and retained invalid/additional members are part
of that checked representation; it is not original Foundry JSON. It works without
the source clone and does not execute rules or macros.

## Search and evidence

Use strict resolution for a known name, search for a concept or partial name,
and query-free search to browse records satisfying filters. Then inspect the
returned record and matching passage:

```sh
atlas search 'Ghoul Fever' --retrieval fts --json
atlas search 'a creature whose disease drains a victim over time' --json
atlas search --kind creature --trait undead --json
atlas search --where 'defenses.hp.maximum >= 80 && "undead" in traits' --json
atlas similar 'Dirge of Doom' --kind spell --json
```

FTS targets names, verified aliases, typed vocabulary, embedded names and structurally
named definitions. Semantic search independently covers selected root and embedded
explanatory prose. Hybrid is the default; `--retrieval fts` needs no model
weights, and `--retrieval vector` selects semantic search alone. Hybrid
`--ranking score` is the default; `--ranking rrf` compares rank-based fusion on
the same data. Keep the default unless the caller asks to compare approaches.
Ranking selection does not rebuild data and applies only to hybrid text search.
A lexical-only workspace needs `--retrieval fts` for text queries.
Similar uses the seed's stored root identity vector. Its filters constrain
candidates, not seed resolution; resolve an ambiguous seed separately.
Semantic/hybrid candidate windows are bounded: report the
returned coverage and count basis honestly, never describe them as an exhaustive
corpus census. Result scores are retrieval evidence, not probabilities. Inspect
up to three supplied root, embedded-item or passage witnesses before explaining why
a result matched.

A named affliction may live within a creature or effect. Atlas preserves its
parent identity and provides an embedded_path/field/passage address. Use the exact
returned address to focus detail:

```sh
atlas record get PACK:ID --embedded-path 'EMBEDDED_PATH_JSON' --field /system/description/value --passage 'PASSAGE_ADDRESS_JSON' --json
```

Copy embedded-path and passage JSON from a returned witness. If its navigation contains
`source_fingerprint`, also pass `--source-fingerprint` with that exact value;
snapshot-local embedded documents become stale when the artifact changes. Do not invent embedded IDs,
byte ranges or synthetic affliction record keys. GM prose and check DCs are included
by the artifact policy; unavailable source fields stay unknown instead of zero.

## Filter discovery

Discover filters before constructing a more specific expression:

```sh
atlas filters fields --json
atlas filters fields --group movement --json
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
`--text` searches values, not record content. Value counts refer to distinct
parent records after remaster preference; source-availability statistics are
before that preference and may count multiple collection occurrences per parent.
Unavailable discovery is an error, never a zero count. For same-entry constraints,
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
The default catalog uses shared UI product groups; `--group ID` browses one.
`--all` lists the flat catalog; `--field` expands one descriptor. Default and
`--group` JSON return the grouped editor catalog. `--all`, `--field`, and `--text`
return the query catalog with complete descriptor metadata.
Numeric and collection discovery return statistics/availability with `values: null`;
categorical and Boolean fields return paged values plus availability counts.

## Relationships and saved lists

Use graph commands for actual structural context:

```sh
atlas graph links 'Demoralize' --backlinks 8 --json
atlas graph uses 'Frightened' --limit 25 --json
atlas graph remaster PACK:ID --json
atlas graph variants 'Dread Ampoule' --json
```

Links preserve occurrences and source attribution, including unresolved targets.
Remaster relationships require explicit evidence; a legacy-only query match stays
visible. Suggested variants are bounded same-pack/family candidates with naming
and compatibility evidence. They are not canonical aliases or confidence scores.
Macros are developer-only and never appear as product results; RollTables are
ordinary products.

Keep durable collections in saved lists:

```sh
atlas lists create undead-research --name 'Undead Research' --json
atlas lists add undead-research 'Skeleton Guard' --json
atlas lists show undead-research --json
atlas lists export undead-research --output undead-research.json
atlas lists import undead-research.json --json
```

Lists share the persistent workspace with reference data. A missing record keeps
its saved snapshot and explicit unresolved status after a reference update. Use
the list commands to change saved state; source updates preserve it.

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
change initiative. Roll-table and encounter actions are available in the web UI.

Use `atlas-dev` only for developer diagnostics: `source load`, `source analyze`,
`source audit-paths`, `index inspect` and `index record`. These commands are a
separate unpublished Rust bundle. Original JSON inspection requires an explicit
`index record --original` request and a configured source clone matching the
artifact provenance. `atlas record source` exports the checked DTO; original file
bytes and admission diagnostics belong to the developer tooling.
