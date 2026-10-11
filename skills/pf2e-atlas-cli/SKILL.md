---
name: pf2e-atlas-cli
description: Answer Pathfinder 2e reference questions using the local Atlas CLI, with strict identity lookup, source-attributed search, typed filters, graph context and saved lists.
---

Use `atlas` for PF2e questions. It reads a local artifact built from the Foundry
PF2e source. Product reads work without the original source clone. Run
`atlas --help` and subcommand help when the installed version's surface is unclear.

Check readiness with `atlas setup --check --json`. Use `atlas setup` to prepare
source, the pinned BGE embedding model and artifact, or `atlas setup
--no-embeddings` for lexical search and reference lookup. `atlas index check
--json` checks readiness; `atlas index validate --json` deeply checks the artifact.
Use `--no-embeddings` with either index command for a lexical artifact. Older
artifacts require rebuilding, with no compatibility reader. Indexing locale is
English by default; `atlas setup --locale fr` or `atlas index build --locale fr`
prepares that source locale with English fallback. Changing search locale requires
rebuilding. Setup preserves an existing locale unless explicitly overridden.

Prefer JSON output: commands emit `{ "status": "ok", "data": ... }` or an error
envelope. A nonzero exit means the requested action did not complete successfully.
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

Record JSON keeps the root summary in `record` and the exact selected address in
`selected`. `presentation.identity` describes the selected child when an owner
chain is supplied; `presentation.body` contains semantic creature, hazard, spell,
activity, physical, table, or content facts. Read each fact's availability before using its
value. Spell heightening and forms describe authored changes; Atlas does not
calculate a selected rank or execute a form. Embedded spells retain their local
casting context and customizations.
Form and fixed-heightening changes are sparse authored overrides of the base;
an omitted member is not an unavailable full form. Explicit null clears remain
distinct from omitted members. Read casting-entry attack/DC and Lore modifiers
from their typed activity facts rather than reconstructing them from prose.

For a compact root heading, use `atlas record get PACK:ID --detail summary`.
This terminal path reads body-free summaries. Selected-child summaries identify
the child from its parent snapshot. `--json` always requests full selected detail,
including when `--detail summary` is supplied.

Search broadly, then inspect the returned record and matching passage:

```sh
atlas search 'Ghoul Fever' --retrieval fts --json
atlas search 'a creature whose disease drains a victim over time' --json
atlas search --kind creature --trait undead --json
atlas search --where 'actor.hp.maximum >= 80 && "undead" in traits' --json
atlas similar 'Dirge of Doom' --kind spell --json
```

FTS targets names, verified aliases, typed vocabulary, owned names and structurally
named definitions. Semantic search independently covers selected root and owned
explanatory prose. Hybrid combines both lanes. Similar uses the seed's stored
root identity vector. Semantic/hybrid candidate windows are bounded: report the
returned coverage and count basis honestly, never describe them as an exhaustive
corpus census. Result scores are retrieval evidence, not probabilities. Inspect
up to three supplied root, owned-item or passage witnesses before explaining why
a result matched.

A named affliction may live within a creature or effect. Atlas preserves its
parent identity and provides an owned chain/field/passage address. Use the exact
returned address to focus detail:

```sh
atlas record get PACK:ID --owners 'OWNER_CHAIN_JSON' --field /system/description/value --passage 'PASSAGE_ADDRESS_JSON' --json
```

Copy owner and passage JSON from a returned witness. If its navigation contains
`source_fingerprint`, also pass `--source-fingerprint` with that exact value;
snapshot-local owners become stale when the artifact changes. Do not invent embedded IDs,
byte ranges or synthetic affliction record keys. GM prose and check DCs are included
by the artifact policy; unavailable source fields stay unknown instead of zero.

Discover filters before constructing a more specific expression:

```sh
atlas filters fields --json
atlas filters values --field traits --kind spell --json
atlas filters values --field actor.hp.maximum --kind creature --json
atlas search --where 'actor.saves.fortitude >= 12' --print-filter --json
```

The catalog defines valid CEL paths, types, operators, family applicability, units,
closed choices and value discovery. Open strings such as traits use admitted
identifiers and sampled values. Common flags `--kind`, `--pack-name`, `--rarity`
and `--trait` combine with `--where` through the same typed predicate. Unsupported
CEL syntax or fields fail explicitly. The retired metric/reference/filter-JSON
flags are not available; do not fabricate an alternative expression language.
Unknown, missing, null, invalid and not-applicable values are distinct. Only a true
predicate admits a record; do not assume missing stats have a numeric value.

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

Lists live in a separate local-state database. A missing record keeps its saved
snapshot and explicit unresolved status after an artifact rebuild. Do not mutate
the reference artifact to implement saved state.

Use `atlas-dev` only for developer diagnostics: `source load`, `source analyze`,
`source audit-paths`, `index inspect` and `index record`. These commands are a
separate unpublished Rust bundle. Original JSON inspection requires an explicit
`index record --original` request and a configured source clone matching the
artifact provenance. Product commands do not emit raw DTOs or admission reports.
