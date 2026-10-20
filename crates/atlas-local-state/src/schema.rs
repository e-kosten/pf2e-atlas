pub const PARTIES_SCHEMA_SQL: &str = include_str!("parties.sql");

/// Mutable domain baseline, composed into the workspace migration by runtime.
pub const LOCAL_STATE_SCHEMA_SQL: &str = include_str!("schema.sql");

/// Independent hazard participation state; existing hazards remain active.
pub const ENCOUNTER_STATUS_SCHEMA_SQL: &str = include_str!("encounter_status.sql");

pub const HAZARD_PARTICIPATION_SCHEMA_SQL: &str = include_str!("hazard_participation.sql");

/// Typed contextual condition inputs independent of source and duration.
pub const CONDITION_CONTEXT_SCHEMA_SQL: &str = include_str!("condition_context.sql");

/// Explicit immunity override intent; explanations use the ordinary optional note.
pub const CONDITION_IMMUNITY_OVERRIDE_SCHEMA_SQL: &str =
    include_str!("condition_immunity_override.sql");

/// Source-specific condition inputs and atomic lifecycle events.
pub const CONDITION_EVENTS_SCHEMA_SQL: &str = include_str!("condition_events.sql");

pub const PERSISTENT_SELECTIONS_SCHEMA_SQL: &str = include_str!("persistent_selections.sql");

/// Saved casting checks and once-only resource debits.
pub const ACTION_ATTEMPTS_SCHEMA_SQL: &str = include_str!("encounter_attempts.sql");

/// Once-per-boundary gameplay event claims.
pub const ENCOUNTER_BOUNDARIES_SCHEMA_SQL: &str = include_str!("encounter_boundaries.sql");

/// Explicit participant facts, separate from derived condition statistics.
pub const PARTICIPANT_FACTS_SCHEMA_SQL: &str = include_str!("participant_facts.sql");

/// Directed relationship knowledge; missing-row policy remains app-service owned.
pub const ENCOUNTER_RELATIONSHIPS_SCHEMA_SQL: &str = include_str!("encounter_relationships.sql");

pub const OUTGOING_TURNS_SCHEMA_SQL: &str = include_str!("encounter_outgoing_turns.sql");

pub const TURN_NOTIFICATIONS_SCHEMA_SQL: &str = include_str!("encounter_turn_notifications.sql");
