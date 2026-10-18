# Embedding reuse across preparation upgrades

Status: proposed
Priority: soon
Owner: unassigned

Review this within the [pre-1.0 database review](database-release-readiness.md)
before choosing an independent cache or reuse-reader implementation.

## Outcome

Avoid repeated inference when rebuilding an artifact after a preparation-policy
change that leaves embedding inputs identical. Ordinary readers must continue to
reject incompatible artifacts; reuse is a separate build concern.

## Scope

The current build opens its reuse source through the normal artifact reader.
That reader requires the executable's content-interpretation version, so a change
to reference metadata can prevent reuse even when model inputs are unchanged.

Design a supported reuse boundary that verifies complete model/input identity,
old input association and vector integrity without exposing stale prepared
content to product readers. Do not silently relax normal validation, trust only
stored hashes, or add per-version product rendering paths. Evaluate a durable
verified embedding cache versus a bounded rebuild-only reader before choosing.

## Acceptance

- Unchanged model inputs reuse vectors across preparation-version changes.
- Changed text, tokenization, prefix or model identity requires fresh inference.
- Corrupt associations or vectors fail verification with clear diagnostics.
- Rebuild publication remains atomic and preserves saved lists/encounters.
- Report inferred and reused counts; test a metadata-only change and an actual
  text change. A local preview refresh is not evidence of a shipped upgrade path.
