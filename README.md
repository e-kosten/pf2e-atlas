# PF2e Atlas

PF2e Atlas is a local Pathfinder Second Edition reference and encounter manager.
Find creatures, spells, equipment, and rules by name or by describing what you
need. Save finds in lists, build encounters for your party, and track initiative,
HP, and conditions during play.

## Quick Start

### Desktop app

Download and install the latest stable desktop app for your system.

- [Apple Silicon Mac](https://github.com/e-kosten/pf2e-atlas/releases/latest/download/atlas-desktop-aarch64-apple-darwin.dmg) (macOS Tahoe 26 or later) · [Installation help](docs/installation.md#macos-desktop-app)
- [Windows x64](https://github.com/e-kosten/pf2e-atlas/releases/latest/download/atlas-desktop-x86_64-pc-windows-msvc-setup.exe) · [Installation help](docs/installation.md#windows-desktop-app)

Open **PF2e Atlas** and choose **Set up Atlas**. Setup downloads the reference
content and search data so Atlas can search locally.

### Linux or browser use

On Linux x64 or ARM64, or to use a browser on a supported Mac or Windows PC,
use the [standalone CLI installer](docs/installation.md#standalone-cli).
Then prepare Atlas and open it in your browser.

```bash
atlas setup
atlas web --open
```

Keep the terminal command running while you use Atlas. If the browser does not
open, visit the local address shown in the terminal.

### Try your first search

Search for “a guardian for an enchanted forest,” then narrow the results by level
or trait. The [Getting started guide](docs/getting-started.md) walks you through
reading results, saving a list, and running your first encounter.

## Use the CLI or an agent

Install the [standalone CLI](docs/installation.md#standalone-cli) or add the
desktop app's bundled command to your PATH. See the terminal access instructions
for [macOS](docs/installation.md#macos-desktop-app) or [Windows](docs/installation.md#windows-desktop-app).
Once `atlas` is on your PATH, try a search from the terminal.

```bash
atlas search "low level healing spell" --kind spell --limit 5
```

To let an AI agent use Atlas for Pathfinder lookups and session prep, run
`atlas agent skills install` and choose your agent and installation scope.
See [agent skill installation](docs/installation.md#agent-skill) for details.

## Documentation

| I want to… | Read |
| --- | --- |
| Explore related records or change search settings | [Search guide](docs/search.md) |
| Update Atlas or install a different version | [Installation](docs/installation.md) |
| Preview upcoming changes | [Nightly builds](docs/installation.md#nightly-builds) |
| Troubleshoot setup, back up saved work, or manage downloaded files | [Setup and maintenance](docs/setup-and-maintenance.md) |
| Use more commands or work with JSON output | [CLI examples](docs/cli-examples.md) |
| Contribute code or documentation | [Contributing](CONTRIBUTING.md) |

Questions or problems? Use the [issue tracker](https://github.com/e-kosten/pf2e-atlas/issues).

Atlas uses reference content from the [Foundry PF2e project](https://github.com/foundryvtt/pf2e).
Setup downloads the reference version supported by your Atlas release.

## Paizo Community Use Notice

PF2e Atlas uses trademarks and/or copyrights owned by Paizo Inc., used under Paizo's Community Use Policy (paizo.com/licenses/communityuse). We are expressly prohibited from charging you to use or access this content. PF2e Atlas is not published, endorsed, or specifically approved by Paizo. For more information about Paizo Inc. and Paizo products, visit paizo.com.
