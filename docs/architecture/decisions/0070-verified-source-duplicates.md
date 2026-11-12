# ADR 0070: Verified source duplicates

Status: accepted.

## Context

A scenario can republish a bestiary creature verbatim under a source-specific
title. Both keys matter for published references and saved entries. Both
results occupy discovery slots even when a reader gains no additional rules
or encounter guidance from the copy.

## Decision

Ingest accepts reviewed same-edition NPC pairs with exact source keys, titles
and source hashes. Review covers the whole stat block and the published
encounter context. The declaration chooses a canonical record explicitly.
Missing or changed source evidence deactivates the relationship and emits a
developer diagnostic. The copy remains eligible for discovery when present.
Distinct mechanics and cross-edition counterparts retain independent results.

Reference publication owns the relation and the canonical title alias. It
reconciles and validates both with the source generation. A digest of the
active pairs binds the artifact context to the relation. The declaration bytes
and verification policy participate in reference freshness.

Global discovery evaluates the complete filter predicate against each exact
source record, projects qualifying records to canonical keys, and deduplicates
before ranking, counting and paging. Publication and pack facets count a
canonical record once per qualifying value. Ranked witnesses use canonical
content. A verified old title resolves to the canonical key, including in
vector-only search. Similar excludes the seed's whole duplicate group.

Exact links, source inspection, saved entries and authored relationships keep
the original source identity. A copied record's header links to the canonical
record. The canonical reader lists the copy's publication as an additional
source. [ADR0067](0067-remaster-relationships.md) governs legacy and remaster
counterparts.
