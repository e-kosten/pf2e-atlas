pub(crate) const CREATE_ARTIFACT_SCHEMA_SQL: &str = concat!(
    include_str!("../../migrations/00000000000001_create_artifact/up.sql"),
    "\n",
    include_str!("../../migrations/00000000000002_atomic_canonical_records/up.sql"),
    "\n",
    include_str!("../../migrations/00000000000003_hazard_records/up.sql"),
    "\n",
    include_str!("../../migrations/00000000000004_consumable_records/up.sql")
);
