# Getting Started With PF2e Atlas

This guide walks through the first commands to run after installing `atlas`.
It focuses on the normal user path: prepare local data, find records, inspect
details, and switch to JSON when a script or agent needs structured output.

## Install And Setup

Install the CLI from the latest release:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/e-kosten/pf2e-atlas/releases/latest/download/atlas-installer.sh | sh
```

On Windows, use PowerShell:

```powershell
irm https://github.com/e-kosten/pf2e-atlas/releases/latest/download/atlas-installer.ps1 | iex
```

Then run setup:

```bash
atlas setup
```

`atlas setup` fetches or updates PF2E source data, prepares semantic search
support, builds local search data, and verifies that the runtime is ready.

To check readiness without changing local data:

```bash
atlas setup --check
```

Use JSON when checking setup from a script or agent:

```bash
atlas setup --check --json
```

## First Lookup

Use `record resolve` when you know a record name and want the matching Atlas
record:

```bash
atlas record resolve "Treat Wounds" --pack-name actionspf2e
```

Atlas records have canonical keys in `pack:id` form. Use `record get` once you
already have one:

```bash
atlas record get actionspf2e:1kGNdIIhuglAjIp9
```

`record get` does not resolve names. If you start from a name, resolve or
search first, then fetch the selected key.

## First Search

Use `search` when you need a ranked result set:

```bash
atlas search "low level healing spell" --kind spell --limit 5
```

When the request includes clear structured constraints, pass them as filters
instead of relying on query text alone:

```bash
atlas search "healing spell" --kind spell --max-level 2 --detail preview --limit 8
```

For lists where the filters fully describe the request, omit the text query:

```bash
atlas search --kind equipment --rarity uncommon --detail preview --limit 20
```

## Details And JSON

Use `--detail preview` to scan result sets, `--detail description` when the
description text matters, and `--detail standard --json` when a caller needs
structured record context.

```bash
atlas search "protect an ally" --kind feat --trait champion --detail preview --limit 8
atlas record get feats-srd:jM72TjJ965jocBV8 --detail standard --json
```

Most commands support a shared JSON envelope. Successful payloads are under
`data`; command, runtime, or input failures are under `error`.

## Ambiguous Names

Some PF2E names identify more than one record. When strict resolution is
ambiguous, inspect the alternatives, choose the best canonical key, then fetch
that key:

```bash
atlas record resolve "Shield Block" --kind feat --alternatives 5 --detail preview --json
atlas record get feats-srd:jM72TjJ965jocBV8 --detail standard --json
```

Use narrower filters when the alternatives are too broad or when you need to
rerun resolution in a smaller record space.

## Similar Records And Graph Context

Use `similar` when you already have a seed record and want records like it:

```bash
atlas similar "Dirge of Doom" --kind spell --limit 8
```

Use `graph links` when you want one-hop references around a known record:

```bash
atlas graph links actionspf2e:1kGNdIIhuglAjIp9 --json
```

By default, graph links include outgoing references and omit backlinks. Use
`graph uses` for records that reference or use a known key:

```bash
atlas graph uses conditionitems:TBSHQspnbcqxsmjL --limit 5 --json
```

## Discover Filters

Use filter discovery when you are not sure which fields or values are
available:

```bash
atlas filters fields --kind spell
atlas filters values --field traits --kind spell
atlas filters values --field metric --kind creature --metric-query save
```

Prefer convenience filters such as `--kind`, `--rarity`, `--max-level`,
`--trait`, `--references`, `--referenced-by`, and `--metric` when they express
the query clearly.

## Agent Skill

Atlas can install a first-party skill that teaches local coding agents how to
use the CLI for PF2E research:

```bash
atlas agent skills install
```

For automation, provide the target and scope:

```bash
atlas agent skills install --target codex --scope global --yes --json
```

Inspect existing skill installs with:

```bash
atlas agent skills doctor
```

