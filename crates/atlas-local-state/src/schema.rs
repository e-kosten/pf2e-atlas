pub const PARTIES_SCHEMA_SQL: &str = include_str!("parties.sql");

/// Mutable domain baseline, composed into the workspace migration by runtime.
pub const LOCAL_STATE_SCHEMA_SQL: &str = include_str!("schema.sql");

/// Independent hazard participation state; existing hazards remain active.
pub const ENCOUNTER_STATUS_SCHEMA_SQL: &str = include_str!("encounter_status.sql");

pub const HAZARD_PARTICIPATION_SCHEMA_SQL: &str = include_str!("hazard_participation.sql");
