//! Shared pure encounter rules. Durable state and presentation are adapted at the edges.
pub(super) mod capabilities;
pub(crate) mod modifiers;
mod projection;
pub(in crate::encounters) mod senses;
pub(super) use projection::{
    effective_hp_maximum, participant_runtime_block, project_initiative, project_participant,
};
mod applicability;
pub(super) use applicability::{
    ConditionApplicabilityFacts, ConditionImmunity, condition_applicability,
};
mod source_facts;
pub(super) use source_facts::source_baseline;

pub(in crate::encounters) mod causes;

pub(super) mod action_context;

pub(super) mod persistent_damage;

pub(super) mod events;

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
