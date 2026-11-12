//! Runtime-owned release schema and authenticated upgrade baseline.
use atlas_storage::{BaselineProvenance, DataBackfill, Migration, MigrationHistory};

pub(crate) const RELEASE_HISTORY: MigrationHistory<'static> = MigrationHistory {
    migrations: &[
        BASELINE,
        Migration {
            version: 43,
            name: "workspace-settings-and-trait-memberships",
            sql: include_str!("workspace_settings.sql"),
            backfill: Some(DataBackfill {
                reconstruct_tables: false,
                id: "trait-membership-attestation/v1",
                apply: super::migrate_trait_membership_digests,
            }),
        },
    ],
    baseline_provenance: Some(PROVENANCE),
};

const BASELINE: Migration = Migration {
    version: 42,
    name: "release-baseline",
    sql: concat!(
        include_str!("../../atlas-index/src/release_schema.sql"),
        include_str!("../../atlas-local-state/src/release/saved_lists.sql"),
        include_str!("../../atlas-local-state/src/release/parties.sql"),
        include_str!("../../atlas-local-state/src/release/encounters.sql"),
        include_str!("../../atlas-local-state/src/release/schema.sql"),
    ),
    backfill: None,
};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    // Schema 41 history at b17f8482d, from an executable-created workspace.
    ledger_sha256: "063c58315d7841c8dff56ed43ddc6a9465d92e1ff9924e59f250879d9815dd85",
    reconstruct_tables: true,
    bridge_id: "release-explicit-saved-owners/v1",
};
