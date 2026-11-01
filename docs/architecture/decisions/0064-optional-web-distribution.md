# ADR 0064: Optional web distribution

## Status

Accepted.

## Context

CLI and agent users need the shared reference/search runtime without compiling or
installing web hosting and frontend assets. Most users are expected to want both
interfaces. Atlas already separates web transport from application workflows.

## Decision

Keep one executable named `atlas`. Its `web` Cargo feature is enabled by default
and owns the optional `atlas-web`, Axum and CLI Tokio dependencies. CLI-only
builds disable default features and omit the entire web command tree. Shared
app-service/model/runtime crates and the bundled agent skill remain available.
Models, reference preparation and workspace migrations remain independent of the
installed presentation surfaces.

Release each supported platform with `full` and `cli-only` variants at the same
version. Retain full archive names `atlas-cli-<target>.*`; name CLI-only archives
`atlas-cli-only-<target>.*`. Release-manifest assets identify their variant and
place full assets first so existing target-only installers still select full.
New installers interpret old entries without a variant as full, and require one
matching target/variant asset. Missing CLI-only assets fail explicitly.

Fresh installs default to full. Omitted selection on update preserves the variant
reported by the executable at the installation destination; old binaries lacking
build-info are full. Explicit with-web/without-web options replace that executable
with the chosen variant. Both variants use the same workspace and caches.
Malformed existing metadata requires an explicit choice before replacement.

`atlas build-info --json` returns the CLI's standard success envelope with
`data.version` and `data.variant`. It needs no setup, source, database or model
access. Keep `atlas --version` unchanged for existing consumers.

## Consequences

CLI-only source builds need no frontend assets or Node.js. Installed full builds
continue embedding a compatible frontend/backend pair. Switching variants within
one release replaces the executable without changing runtime data.

Packaging validates both variants on all supported platforms, with shared CLI
smoke checks and additional embedded-web checks for full builds. Local/CI Rust
validation exercises the CLI-only feature configuration before frontend
preparation, as well as the default workspace configuration. Stable and nightly
publication require both variants; nightly downloaded-install checks cover both
before advancing the channel.
