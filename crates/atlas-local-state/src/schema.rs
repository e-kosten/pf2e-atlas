pub const PARTIES_SCHEMA_SQL: &str = include_str!("parties.sql");
pub const PARTY_LEVELS_SCHEMA_SQL: &str = include_str!("party_levels.sql");

/// Mutable domain baseline, composed into the workspace migration by runtime.
pub const LOCAL_STATE_SCHEMA_SQL: &str = include_str!("schema.sql");

/// Independent hazard participation state; existing hazards remain active.
pub const ENCOUNTER_STATUS_SCHEMA_SQL: &str = include_str!("encounter_status.sql");

pub const HAZARD_PARTICIPATION_SCHEMA_SQL: &str = include_str!("hazard_participation.sql");

/// Conditions, their gameplay events, and current-turn reminders.
pub const CONDITION_SYSTEM_SCHEMA_SQL: &str = include_str!("condition_system.sql");

/// Receipt-only persistent damage/recovery preparation claims.
pub const PERSISTENT_PREPARATION_SCHEMA_SQL: &str = include_str!("persistent_preparation.sql");

/// Authoritative manual action/reaction resources for an actual turn.
pub const TURN_RESOURCES_SCHEMA_SQL: &str = include_str!("turn_resources.sql");

/// Application identities that survive even when no condition row is inserted.
pub const CONSUMED_CONDITION_APPLICATIONS_SCHEMA_SQL: &str =
    include_str!("consumed_condition_applications.sql");

/// Optional encounter difficulty references; existing encounters follow their PCs.
pub const ENCOUNTER_DIFFICULTY_SCHEMA_SQL: &str = include_str!("encounter_difficulty.sql");

/// Source-defined switches, choices and ability expenditure.
pub const ABILITY_STATE_SCHEMA_SQL: &str = include_str!("ability_state.sql");

/// Remove obsolete display-only strike receipts while retaining durable gameplay receipts.
pub const STATELESS_STRIKE_ROLLS_SCHEMA_SQL: &str = include_str!("stateless_strike_rolls.sql");

/// Versioned participant creation baselines and recovery idempotency.
pub const ENCOUNTER_RECOVERY_SCHEMA_SQL: &str = include_str!("encounter_recovery.sql");

/// Lifetime turn request receipts and one current outgoing context per encounter.
pub const TURN_RECEIPTS_SCHEMA_SQL: &str = include_str!("turn_receipts.sql");
