# Source contract tooling

Offline developer tooling for investigating upstream PF2e declarations and authored metadata. These scripts do not run during normal Rust builds or change the ingest pipeline.

Use Node 22 or later. Install the pinned compiler and declaration dependencies, then run the fixture tests:

```sh
npm --prefix scripts/source-contracts ci --ignore-scripts
npm --prefix scripts/source-contracts test
```

`source-identity.mjs` identifies the source bytes independently of checkout location. It hashes `src`, `types`, the package manifest, compiler configuration and English localization. A real upstream checkout also reports its Git commit and dirty state; an exported archive reports no Git identity.

The declaration graph and trait catalog are separate implementation slices above this shared tooling base. Their combined command is a later layer. Rust generation and production adoption remain experiments tracked in the [source-contract backlog](../../docs/backlog/items/rust-source-contract-generation.md).
