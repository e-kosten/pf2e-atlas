# PF2e 8.6.1 source and model upgrade

Status: source/model migration and local installed acceptance complete.

## Identity and scope

The supported source is [PF2e 8.6.1](https://github.com/foundryvtt/pf2e/releases/tag/pf2e-8.6.1),
commit `28d9d156f252d084371d994dc359509b49eadf18`. The canonical pin is
`dev-tools/source-contracts/source-pin.json`; the declaration digest is
`c1ee148d16a58ace19114544105ccd50c625ba356d5a79fffd40f49165b68d3b`.
The portfolio has 45 roots, 25 Item families, eight Actor families and 40 specific
rule models. The checked source contains 29,894 root records and 98,228 immediate
actor items. Source packs are selected from `packs/pf2e` using the PF2e manifest;
Starfinder packs are excluded.

This replaces the 6.12.4 model. It does not support simultaneous upstream model
versions. Existing workspaces with the exact predecessor contract must rebuild
reference data from the new source. Saved lists, encounters, record identities and
compatible exact-input embedding checkpoints remain owned by the existing workspace.
See ADR0052 for reader rejection, maintenance recovery and failed-publication policy.

## Upstream changes handled

- Manifest and Foundry declaration entry points moved to `system.pf2e.json` and
  `.d.mts` modules. Authored selector forms follow the declared `ArrayField` and
  `SetField` owners; `StrictArrayField` remains array-only.
- Ammunition is a physical Item family with physical query rows and equipment
  presentation. Treasure traits now accept strings.
- NPC skills are keyed source entries. Character movement is runtime-derived and
  unavailable to source-backed movement filters. NPC attack range comes from the
  authored `range` field; missing or invalid range does not imply melee.
- Roll-table result prose uses `description`; references use explicit
  `documentUuid` values. World and malformed references remain unresolved.
- Spell override patches omit `overlays`, while fixed heightening retains the full
  patch domain. Generated duration, coin, damage and flag-namespace types follow
  the new declarations. Striking and WeaponPotency are no longer registered rule
  roots.

Query catalog 6, `authored-query/v3`, `structural-source-sections/v4` and
`typed-authored-occurrences/v5` identify the rebuilt derived outputs.

## Validation

Use the repository's Rust 1.95 toolchain, not an unpinned system Cargo.

- Generator fixture/typecheck suite: 104 tests.
- Independent corpus query oracle: 978,569 known value checks, 39,112 unavailable
  terminal checks and 29 family SQL oracles; all passed.
- Frontend verification: formatting, lint, styles, typecheck, 691 tests across
  70 files, and production build passed.
- Full Rust fmt, both Clippy gates, workspace tests and build passed.
- Snapshot round-trip: all 29,894 roots across 98 packs passed; 352,598,041
  encoded snapshot bytes, 237 partial roots and 305 field diagnostics.
- Final corpus admission: 168,127 occurrences retained, 168,115 modeled,
  167,758 fully typed, 357 partially typed, 12 retained raw rule occurrences,
  zero rejected and zero measured fidelity failures. All 29,894 roots have models;
  237 roots have diagnostics on specific fields. Strict parsing rejects 369
  occurrences rather than coercing malformed authored values. Baselines record
  these outcomes explicitly; retaining raw values does not make them typed.
- Pinned generation freshness: all nine integration tests passed.

The independent corpus oracle can be repeated with:

```sh
ATLAS_PF2E_SOURCE=/absolute/path/to/pf2e-8.6.1 \
  cargo test -p atlas-index pinned_corpus_item_and_common_projection_oracle -- --ignored --nocapture
```

The maintained comparison commands are documented in
`dev-tools/source-contracts/README.md`. Baseline JSON files contain measured
source/corpus/outcome identities, not acceptance exceptions.

## Installed acceptance

The repository vendor checkout is at the exact 8.6.1 commit. Its complete preceding
7.12.2 checkout was preserved under the ignored upgrade backup directory; an atomic
directory exchange avoided a partially checked-out shared tree.

The installed executable and global workspace now use the exact 8.6.1 source.
The local migration build identifies itself as Atlas `0.1.0-rc.6`; 8.6.1 is the
PF2e data version. This is a local build, not a published Atlas release.

Both configured profiles, BGE Base and EmbeddingGemma, are ready. The first
migration reused 117,631 and generated 43,084 unique inputs per profile. The final
candidate rebuild reused all 160,715 unique inputs per profile. Deep validation
checked 29,894 snapshots, 609,644 projection rows, 206,917 relationships and
323,462 semantic units. Both profile searches returned results; the ammunition
filter found 216 records. The packaged web smoke verified embedded assets,
readiness, ammunition and roll-table records from outside the checkout.

Activation checked the live workspace against its original consistent backup,
held the stable update/access locks, restored the validated database in place and
atomically replaced the executable. The workspace key, principals and all saved
list/encounter tables were preserved; reference revision advanced from 1 to 3.
The identified installed server alone was restarted on port 4728. Installed
readiness, a semantic query and the live ammunition record passed afterward.
The configured source and model paths are global and do not depend on the worktree.

Local evidence is retained under `scratch/8.6.1-migration/evidence/` in the upgrade
worktree. The primary checkout retains the preceding executable and consistent
SQLite backups under `scratch/pf2e-8.6.1-upgrade/backup/`, and the complete prior
vendor checkout under `scratch/pf2e-8.6.1-upgrade/vendor-before-8.6.1/`.
The shared checkout's tracked files and branch were not edited by this task.
