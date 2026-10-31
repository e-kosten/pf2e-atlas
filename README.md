# PF2e Atlas

**Find Pathfinder content by idea.**

PF2e Atlas helps you find Pathfinder Second Edition creatures, spells, equipment,
and rules by describing what you have in mind. Try “a guardian for an enchanted
forest” or “magic for a haunted-house adventure.” Search matches meaning as well
as names, and filters help you narrow the results by level, traits, and other
game details.

Explore the results in your browser, save options for a session, and build and run
encounters. Atlas also includes a command-line interface and a bundled skill that
lets AI agents search and reference PF2e content while helping you prepare.

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

Setup downloads the Pathfinder reference content and a search model, then prepares
them for local searching. This first run may take a while; later runs reuse work
already completed.

**3. Open Atlas.**

```bash
atlas web --open
```

Keep the command running while you use Atlas. If the browser does not open,
visit the local address printed in the terminal. Later sessions only need
`atlas web --open`.

Start with something you’re looking for—a particular spell, a creature concept,
or an idea for your next session. Explore the results, save useful finds in lists,
or bring creatures and hazards into an encounter.

See [Getting started](docs/getting-started.md) for your first list and encounter.
For alternate versions, supported platforms or a manual download, see
[Installation](docs/installation.md).

## Use the CLI or an agent

```bash
atlas search "low level healing spell" --kind spell --limit 5
atlas agent skills install
```

The skill installer lets you choose your agent and installation scope.
See [CLI examples](docs/cli-examples.md) for lookup, filters, lists and JSON,
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

PF2e Atlas uses trademarks and/or copyrights owned by Paizo Inc., used under Paizo's Community Use Policy (paizo.com/licenses/communityuse). We are expressly prohibited from charging you to use or access this content. PF2e Atlas is not published, endorsed, or specifically approved by Paizo. For more information about Paizo Inc. and Paizo products, visit paizo.com.
