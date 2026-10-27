mod ability_state;
mod conditions;
mod difficulty;
mod hp;
#[cfg(test)]
mod hp_tests;
mod hydration;
mod initiative;
#[cfg(test)]
mod initiative_tests;
pub(crate) mod mechanics;
mod outgoing_turns;
mod projection;
mod record_addition;
mod reference;
#[cfg(test)]
mod reference_tests;
mod rules;
mod stat_block;
mod strike_roll;
#[cfg(test)]
mod tests;
mod timing;
#[cfg(test)]
mod timing_tests;
mod turn_resources;
mod turns;
mod workflow;

mod spell_catalog;

#[cfg(test)]
mod spell_tests;
mod spells;

mod persistent_damage;

mod condition_events;
mod event_execution;

mod hp_events;

mod persistent_damage_resolution;
mod persistent_preparation;

#[cfg(test)]
mod persistent_damage_tests;

mod action_projection;
mod casting_reminders;
mod retch;
mod strike_context;

mod condition_recovery;

pub(crate) use rules::attack_progression;

#[cfg(test)]
mod turn_resource_tests;
