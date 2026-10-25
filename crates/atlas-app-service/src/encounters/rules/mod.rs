//! Shared pure encounter rules. Durable state and presentation are adapted at the edges.
pub(super) mod capabilities;
pub(crate) mod modifiers;
mod projection;
pub(in crate::encounters) mod senses;
pub(super) use projection::{
    effective_hp_maximum, effective_participant_level, hp_maximum_stat, participant_runtime_block,
    project_initiative, project_participant, resolve_condition_subject_hints,
};
mod applicability;
pub(super) use applicability::{
    ConditionApplicabilityFacts, ConditionImmunity, ConditionImmunityFacts, ImmunityCoverage,
    condition_applicability,
};
mod source_facts;
pub(super) use source_facts::{authored_stat_baseline, source_applicability};
pub(super) mod source_action_conditions;

pub(in crate::encounters) mod causes;

pub(super) mod action_context;

pub(super) mod persistent_damage;

pub(super) mod events;
pub(super) mod stunned_arrival;

pub(super) mod relationships;

#[cfg(test)]
mod numerical_tests;

#[cfg(test)]
mod spell_damage_tests;

pub(super) mod hp_events;

pub(super) mod damage;

pub(super) mod lifecycle;

pub(crate) use modifiers::attack_progression;

pub(in crate::encounters) mod sensory_context;

mod condition_effects;

pub(super) use condition_effects::hp_loss as condition_hp_loss;

mod effective_immunities;
