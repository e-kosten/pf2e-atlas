# Getting Started With PF2e Atlas

[Install Atlas](../README.md#quick-start), then prepare a local workspace:

```bash
atlas setup
```

Setup downloads the supported Foundry PF2e source and BGE Base model, builds the
reference data, and checks readiness. The first build includes embedding inference
and can take a while. Later updates reuse compatible embeddings and preserve saved
lists and encounters. For a new workspace without semantic search, use
`atlas setup --no-embeddings`.

Check readiness without downloads or writes:

```bash
atlas setup --check --json
```

## Browse In The Web App

```bash
atlas web
```

Open the address printed by the command. Search and preview records, maintain
saved lists, and run encounters with initiative, HP, conditions and dice controls.
Encounter automation covers supported mechanics; it does not execute every Foundry
rule or replace GM decisions.

## Find A Record

Use strict resolution for a name, then fetch a returned `pack:id` key:

```bash
atlas record resolve 'Treat Wounds' --pack-name actionspf2e
atlas record get actionspf2e:1kGNdIIhuglAjIp9
```

`record get` accepts keys, not names. If resolution is ambiguous, inspect the
alternatives and choose the intended record; a similar name is not proof of identity.

For concepts or partial names, use search:

```bash
atlas search 'walk through walls' --kind spell --limit 5
atlas search 'healing spell' --kind spell --where 'level <= 2' --limit 8
```

Use filters for explicit constraints. Omit query text for alphabetical browsing:

```bash
atlas search --kind creature --trait undead --where 'level <= 4'
```

For text search in a lexical-only workspace, add `--retrieval fts`.

## Read More Or Use JSON

Human record output has three levels:

```bash
atlas record get actionspf2e:1kGNdIIhuglAjIp9 --detail summary
atlas record get actionspf2e:1kGNdIIhuglAjIp9 --detail standard
atlas record get actionspf2e:1kGNdIIhuglAjIp9 --detail full
```

Standard is the default. Full also expands supported embedded records; it does
not follow every graph link. Search returns compact matches; use `record get`
to read a result.

For structured output, use `--json` without `--detail`:

```bash
atlas record get actionspf2e:1kGNdIIhuglAjIp9 --json
```

This returns the complete selected Atlas reference response. To inspect the
checked source DTO, including data outside the reference presentation:

```bash
atlas record source actionspf2e:1kGNdIIhuglAjIp9 --json
```

Both work without the original source clone. The DTO export includes typed
presence/union tags and is not the original Foundry JSON.

## Discover Filters And References

```bash
atlas filters fields
atlas filters fields --group movement
atlas filters fields --field movement.speeds
atlas filters values --field traits --kind spell
atlas search --kind creature --where 'movement.speeds.exists(s, s.type == "fly" && s.feet >= 30)'
atlas similar 'Dirge of Doom' --kind spell --limit 8
atlas graph uses 'Frightened' --limit 5
```

Field discovery supplies supported CEL paths, types and examples. Similar finds
semantic neighbors using stored vectors. Graph commands follow explicit reference
relationships instead of inferring similarity.

## Install The Agent Skill

```bash
atlas agent skills install
```

Choose the agent target and scope, or supply them explicitly:

```bash
atlas agent skills install --target codex --scope global --yes --json
atlas agent skills doctor --target codex --scope global --json
```

After updating Atlas, rerun the installer to update its bundled skill. A differing
installed copy requires `--force`; check for local edits before replacing it.

See [CLI examples](cli-examples.md) for batch JSON, embedded selections, profiles,
filters, lists and dice. See the [README](../README.md) for upgrades and data paths.
