//! Runtime-owned release schema and authenticated upgrade baseline.
use atlas_storage::{DataBackfill, Migration, MigrationHistory, MigrationPredecessor};

pub(crate) const RELEASE_HISTORY: MigrationHistory<'static> = MigrationHistory {
    migrations: &[
        Migration {
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
        },
        Migration {
            version: 43,
            name: "verified-reference-duplicates",
            sql: include_str!("../../atlas-index/src/verified_duplicates.sql"),
            backfill: None,
        },
    ],
    predecessor: Some(MigrationPredecessor {
        // Schema 41 history at b17f8482d, from an executable-created workspace.
        ledger_sha256: "063c58315d7841c8dff56ed43ddc6a9465d92e1ff9924e59f250879d9815dd85",
        bridge: DataBackfill {
            reconstruct_tables: true,
            id: "release-explicit-saved-owners/v1",
            apply: atlas_local_state::upgrade_release_owners,
        },
    }),
};
