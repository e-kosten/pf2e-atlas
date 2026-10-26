# ADR 0059 Configurable stored vector similarity

Status: proposed; implementation in progress.

## Context

Record Similar currently queries from one root identity vector, while trait
Similar queries from a stored definition centroid. Both retrieve record passages
and collapse by the strongest hit. Existing descriptive and embedded passage
vectors support richer comparisons without another embedding inference pass.

## Decision

Expose three explicit methods through shared retrieval and app-service workflows:
Passage matches, Content average and `baseline`. Label the baseline Name and
identity for records and Trait definition for traits. Keep embedding-profile
selection independent. Preserve the old record/trait algorithms as baselines;
their different seed representations remain explicit.

Parts are existing source sections under exact record/owner/field addresses and
source fingerprints. Field/document selection expands to deduplicated logical
parts. Chunks subdivide parts and do not count as independent part contributions.
Pure source-backed policy maps typed content to cross-kind comparison roles;
index owns role metadata and restricts the candidate relation before KNN.

Passage scoring averages each seed chunk's best allowed candidate match within
its part, adjusts for overlapping source coverage, then combines sorted part
scores with normalized geometric decay. Decay zero selects the strongest logical
part; decay one averages all selected parts. New-method cosine evidence is floored
at zero; missing counterparts remain zero contributions rather than disappearing
from the selected-part denominator.

Content average compares centroids built from equally weighted logical parts.
Corresponding mode keeps separate role centroids and combines only permitted
role comparisons with explicit selected-seed-part weights. Global centroid cosine
is a different baseline from directed passage coverage; extra candidate content
can dilute it. Undefined centroids do not silently become identity vectors.

All new methods consume existing stored vectors and remain usable without model
weights. Metadata backfills use retained checked snapshots. The index owns SQL,
search owns candidate generation and scoring, app-service owns coherent snapshot
workflows and presentation, and CLI/web are adapters. The browser does not infer
roles, parse source prose, or calculate ranking.

One Search settings dialog commits model, mode-specific ranking and selection
changes as a single operation. Browser launch preferences persist in localStorage;
explicit URLs retain authority without implicitly replacing saved preferences.
Exact parts remain seed/generation-specific. Existing open tabs keep independent
effective settings; newly opened tabs use the saved defaults.

## Consequences and validation

Bounded candidate generation and final ranking must report separate evidence.
Evaluate candidate recall against a manageable exhaustive oracle, and compare
scorers over fixed pools before tuning the end-to-end retrieval policy. Any saved
derived metadata/cache includes generation, profile and recipe identity. Evidence
limits cannot alter ranking, and unsupported requests cannot silently lose parts.

Initial defaults remain experimental until source-grounded usefulness, measured
latency and human UX review support a shipping choice. Ordinary hybrid search
fusion, structured filters, authored section boundaries and exact trait
membership/reference workflows are unchanged.
