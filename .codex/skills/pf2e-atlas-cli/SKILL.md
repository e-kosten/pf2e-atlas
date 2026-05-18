---
name: pf2e-atlas-cli
description: Use when answering PF2E questions through the Rust Atlas CLI, especially when discovering available filters, narrowing searches, resolving records, or fetching record details from the local artifact.
---

# PF2E Atlas CLI

Use the local `atlas` binary from `rust/` when the task needs current PF2E record data from the Rust artifact.

## Discovery First

When you do not know which filters or values are available, discover them before searching:

```bash
cargo run --bin atlas -- filters fields
cargo run --bin atlas -- filters fields --family spell
cargo run --bin atlas -- filters values --field traits --family spell
cargo run --bin atlas -- filters values --field metric --family creature --metric-prefix save.
```

Use the discovered field names and values with `atlas search`, `atlas record resolve`, or `--filter-json`. For partially built searches, pass the same filter flags to `filters values`; the command returns values constrained to that search space.

## Common Record Commands

Resolve by name when the record key is unknown:

```bash
cargo run --bin atlas -- record resolve "Treat Wounds" --family rule --json
```

Fetch known keys directly:

```bash
cargo run --bin atlas -- record get actions:treat-wounds --json
```

Search with discovered filters:

```bash
cargo run --bin atlas -- search "low level healing spell" --family spell --trait healing --json
cargo run --bin atlas -- search --family equipment --rarity uncommon --json
```

Prefer structured filters and discovered values over guessing PF2E category names or legacy TypeScript subcategory labels.
