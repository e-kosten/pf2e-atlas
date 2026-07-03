# Rust Optional PF2e Art Ingest

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-06-02

## Problem

The base PF2e source checkout carries record-level `img` fields and bundled system icons under `static/`, but richer creature portrait and token art is commonly supplied by separate Foundry modules rather than the base PF2e compendium JSON. The Rust ingest path currently focuses on rules/content/search data and does not model optional media assets as first-class record presentation data.

A future locally hosted Atlas web surface could benefit substantially from showing owned creature art alongside the existing rich search, graph, and record-detail functionality. Without an explicit optional art-ingest design, this work risks being conflated with HTML/content parsing or hard-coding assumptions about assets that are not actually present in the base PF2e repository.

## Desired Outcome

Design and eventually implement an optional local art enrichment path for record media assets, especially creature portraits and tokens.

The work should answer:

- how to ingest base record `img` paths from PF2e pack JSON and resolve `systems/pf2e/...` against the source checkout's `static/` tree
- how to represent Foundry core `icons/...` paths that are validated by PF2e but not shipped inside the PF2e GitHub checkout
- how to read installed module art mappings, including current `compendiumArtMappings` and legacy PF2e `pf2e-art` mappings
- how to associate art mappings with Atlas records through compendium identity and source document ids
- how to store record media metadata in the artifact without making art required for search, embeddings, graph, or CLI output
- how a local web app should serve media from approved local roots without embedding or redistributing premium art by default

## Constraints

- Treat rich PF2e art as optional local enrichment, not canonical normalized record content.
- Do not put record portrait/token art into `RichDocument`; it belongs in record-level media or presentation metadata.
- Do not block HTML parsing, CLI presentation, FTS, embedding, or reference work on art ingestion.
- Do not assume premium token-pack assets are present in the PF2e GitHub export.
- Preserve a useful artifact when no optional art modules are installed.
- Respect licensing and redistribution boundaries: local paths and credits may be indexed, but media blobs should not be copied into shareable artifacts by default.

## Notes

PF2e validates pack `img` paths during its build. System-relative paths such as `systems/pf2e/icons/...` resolve to files under the PF2e source checkout's `static/` directory. Foundry core paths such as `icons/...` are validated against PF2e's core-icon inventory but refer to Foundry-provided assets outside the PF2e repository.

PF2e also supports module-provided art mappings for compendium actors. Current Foundry module manifests can declare `compendiumArtMappings`; PF2e still has legacy support for module flags named `pf2e-art`. These mappings can provide actor portrait art, token textures, token scaling, random image settings, and dynamic-token subject art.

The likely artifact shape is an optional record media table keyed by `record_key`, with media kind, resolved or source path, source module, credit/license text, priority, and metadata JSON for token-specific settings.

## Related

- [Runtime architecture](../../architecture/runtime.md)
- [Rust artifact contract](../../architecture/artifact-contract.md)
- [Rust Foundry JSON field audit](./rust-foundry-json-field-audit.md)
- [Rust CLI content output formats](./rust-cli-content-output-formats.md)
