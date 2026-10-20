mod conditions;
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
mod turns;
mod workflow;

mod spell_catalog;

#[cfg(test)]
mod spell_tests;
mod spells;

mod persistent_damage;

mod condition_events;

mod hp_events;

mod persistent_damage_resolution;

#[cfg(test)]
mod persistent_damage_tests;

mod action_attempts;
mod action_preview;
mod strike_attempts;

mod condition_recovery;

pub(crate) use rules::attack_progression;

mod relationship_views;
mod relationships;

mod detection_events;

#[cfg(test)]
mod relationship_tests;

mod attitude_events;
