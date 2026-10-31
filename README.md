# PF2e Atlas

Search Pathfinder Second Edition content, prepare saved lists, and run encounters
in a local browser workspace. Atlas also provides a command-line interface and a
bundled skill for local coding agents.

Reference content comes from the [Foundry PF2e project](https://github.com/foundryvtt/pf2e).
The supported reference release is **8.6.1**; setup downloads the pinned source for you.

## Quick Start

**1. Install Atlas.** On macOS or Linux:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/e-kosten/pf2e-atlas/releases/latest/download/atlas-installer.sh | sh
```

On Windows, use PowerShell:

```powershell
irm https://github.com/e-kosten/pf2e-atlas/releases/latest/download/atlas-installer.ps1 | iex
```

Follow any PATH instructions printed by the installer so your terminal can find
`atlas`.

**2. Prepare your local data.**

```bash
atlas setup
```

Setup downloads the reference content and default search model, then builds your
local search data. The first run can take a while. You do not need a Foundry
installation, a source checkout, or model settings to get started.

**3. Open Atlas.**

```bash
atlas web --open
```

Keep the command running while you use Atlas. If the browser does not open,
visit the local address printed in the terminal. Later sessions only need
`atlas web --open`.

Search for a name or describe what you need, open a result, then use **Add to list**
to collect options. Creatures and hazards also offer **Add to list or encounter**.
Your lists and encounters are saved locally and survive normal setup updates.

See [Getting started](docs/getting-started.md) for your first list and encounter.
For alternate versions, supported platforms or a manual download, see
[Installation](docs/installation.md).

## Use the CLI or an agent

```bash
atlas search "low level healing spell" --kind spell --limit 5
atlas roll '1d20+12'
atlas agent skills install
```

The skill installer lets you choose your agent and installation scope.
See [CLI examples](docs/cli-examples.md) for lookup, filters, lists, dice and JSON,
and [agent skill installation](docs/installation.md#agent-skill) for updates and automation.

## Documentation

| I want to… | Read |
| --- | --- |
| Browse, save records and run an encounter | [Getting started](docs/getting-started.md) |
| Change search settings or explore related records | [Search guide](docs/search.md) |
| Install, update, try a nightly or build from source | [Installation](docs/installation.md) |
| Check setup, clean caches or choose data locations | [Setup and maintenance](docs/setup-and-maintenance.md) |
| Use commands or structured output | [CLI examples](docs/cli-examples.md) |
| Contribute code or documentation | [Contributing](CONTRIBUTING.md) |
| Understand internal boundaries and decisions | [Architecture](docs/architecture/overview.md) |

Questions or problems? Use the [issue tracker](https://github.com/e-kosten/pf2e-atlas/issues).

## Paizo Community Use Notice

Acquired reference content has separate content licenses from Atlas's Apache 2.0
code. See the [trait description provenance review](docs/research/trait-description-provenance.md)
for pinned source evidence and the remaining paragraph-level attribution work.

PF2e Atlas uses trademarks and/or copyrights owned by Paizo Inc., used under Paizo's Community Use Policy (paizo.com/licenses/communityuse). We are expressly prohibited from charging you to use or access this content. PF2e Atlas is not published, endorsed, or specifically approved by Paizo. For more information about Paizo Inc. and Paizo products, visit paizo.com.
