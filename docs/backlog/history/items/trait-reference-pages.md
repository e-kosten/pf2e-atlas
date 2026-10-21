# Trait reference pages

Status: done
Completed: 2026-10-20
Delivery: [PR93](https://github.com/e-kosten/pf2e-atlas/pull/93)
Decision: [ADR0056](../../../architecture/decisions/0056-trait-reference-entities.md)

## Delivered

Traits have exact typed `trait:<identifier>` identities, addressable definitions,
authored memberships/references and stored-vector Similar into normal records.
Parameterized variants stay distinct even when their definition keys match.
Unknown identifiers and missing descriptions remain explicit; rarity/otherTags
and existing trait filter semantics stay separate.

One pinned generated catalog supplies metadata. Ingest resolves locale/fallback
and shared prepared content; checked publication is atomic with the reference
generation. Defined traits require seed coverage in each configured profile.
Metadata-only publication remains valid. Readers work without source files and
preserve exact embedded-owner navigation. Web/CLI reuse the shared application
contracts; trait and record pages share reading chrome and title-cased kinds.

## Validation

All applicable PR-head CI passed at `010c3caf84f6f6ccfa33b8c017d7b3ae0e99a0ad`;
the merge commit is `20c9b57b6d8d45e561ca324cd973e61774ca9eb8`. Local source-contract,
Rust, frontend and release-tooling checks passed, including migration preservation,
source-free reads, native seed integrity, complete profile coverage and failed
replacement rollback. Frontend coverage included 718 unit tests, six full-corpus
browser cases and 32 device-emulation cases. Independent backend and Astra browser
reviews passed; the user accepted the revised presentation. Emulation is not
physical-device certification.

The full-corpus checkpoint published 784 traits, 664 nonempty descriptions,
244,543 exact-owner memberships and 32 explicit reference occurrences. Both search
profiles had 784 stored trait seeds. The actual seeded preview's 11 pre-existing
saved-table snapshots were preserved; populated saved-state preservation was also
covered by historical migration fixtures.

## Separate open work

- [NPC glossary and prose rules destinations](../../items/rust-localization-backed-rules-terms.md).
- [Description content rights and attribution](../../items/trait-description-content-rights.md),
  grounded in the [pinned provenance review](../../../research/trait-description-provenance.md).
- [Trait-driven encounter effects](../../items/rust-encounter-trait-runtime-effects.md).

General search does not gain trait candidates implicitly. Rarity reference pages,
external citations, hover-card extensions and arbitrary prose linking were not
part of this delivery. No content-license clearance is inferred from implementation
or validation success.
