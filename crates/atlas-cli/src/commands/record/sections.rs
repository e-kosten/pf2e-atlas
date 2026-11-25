//! Focused views from the same typed facts used by the reference reader.
use super::{
    args::{RecordSection, TerminalDetail},
    navigation::Locator,
    presentation,
    terminal::TerminalLayout,
};
use atlas_app_model::{
    ActorActivityView, ActorPresentationView, EmbeddedReadingEntryView, RecordBodyView,
    RecordDetailView,
};
use serde_json::{Value, json};

pub(super) fn actor(body: &RecordBodyView) -> Option<&ActorPresentationView> {
    match body {
        RecordBodyView::Creature(a) => Some(a),
        RecordBodyView::Hazard(h) => Some(&h.actor),
        _ => None,
    }
}

fn hazard(body: &RecordBodyView) -> Option<&atlas_app_model::HazardPresentationView> {
    match body {
        RecordBodyView::Hazard(hazard) => Some(hazard),
        _ => None,
    }
}

fn applies(section: RecordSection, a: &ActorActivityView) -> bool {
    section.read_section().includes_activity(&a.kind)
}

fn activity(section: RecordSection, a: &ActorActivityView, generation: &str) -> Value {
    let mut value = json!({
        "title": a.title,
        "navigation": Locator::new(generation,&a.navigation),
        "cost": a.action_cost,
        "traits": a.traits.iter().map(|v|&v.value).collect::<Vec<_>>(),
    });
    let mechanics = match section {
        RecordSection::Strikes => {
            json!({
                "attack": a.attack,
                "options": a.attack_options,
                "damage": a.damage,
                "range": a.strike_range,
                "effects": a.strike_effects,
            })
        }
        RecordSection::Spellcasting => {
            json!({
                "kind": a.kind,
                "rank": a.base_rank,
                "cast_rank": a.authored_cast_rank,
                "casting_ranks": a.casting_ranks,
                "tradition": a.casting_tradition,
                "preparation": a.preparation,
                "attack": a.attack,
                "dc": a.difficulty_class,
                "entry": a.casting_entry.as_ref().map(|v|Locator::new(generation,v)),
            })
        }
        _ => json!({"usage":a.usage,"frequency":a.frequency,"dc":a.difficulty_class}),
    };
    value["facts"] = mechanics;
    value
}

pub(super) fn json(
    section: RecordSection,
    record: &RecordDetailView,
    reading: &[EmbeddedReadingEntryView],
) -> Result<Value, String> {
    if section == RecordSection::Rules {
        return super::json::rules(record, None);
    }
    let body = &record.presentation.body;
    if section == RecordSection::Spellcasting
        && let RecordBodyView::Spell(spell) = body
    {
        return Ok(json!(spell));
    }
    if let RecordBodyView::Activity(a) = body
        && applies(section, a)
    {
        let mut row = activity(section, a, &record.artifact_generation);
        row["rules"] = super::json::rules(record, None)?;
        return Ok(row);
    }
    let Some(actor) = actor(body) else {
        return Ok(json!({"state":"not_applicable"}));
    };
    Ok(match section {
        RecordSection::Defenses => {
            let mut facts = json!({"notes": actor.defense_notes, "immunities": actor.immunities,
                "weaknesses": actor.weaknesses, "resistances": actor.resistances});
            let hazard = hazard(body);
            if let Some(h) = hazard {
                facts["defense_block"] = json!(h.defense_block);
                facts["show_saves"] = json!(h.show_saves);
            }
            if hazard.is_none_or(|h| h.defense_block.value != Some(false)) {
                facts["armor_class"] = json!(actor.armor_class);
                facts["maximum_hp"] = json!(actor.maximum_hp);
                if let Some(h) = hazard {
                    facts["broken_threshold"] = json!(h.broken_threshold);
                    facts["hardness"] = json!(h.hardness);
                } else {
                    facts["hardness"] = json!(actor.hardness);
                }
            }
            if hazard.is_none_or(|h| h.show_saves) {
                facts["saves"] = json!(actor.saves);
            }
            facts
        }
        _ => {
            let Some(activities) = &actor.activities.value else {
                return Ok(json!({"state": actor.activities.state}));
            };
            let mut entries = Vec::new();
            for a in activities.iter().filter(|a| applies(section, a)) {
                let mut row = activity(section, a, &record.artifact_generation);
                if let Some(entry) = reading
                    .iter()
                    .find(|entry| entry.navigation == a.navigation)
                {
                    row["rules"] = super::json::rule_fields(
                        &entry.content,
                        &entry.navigation,
                        &record.artifact_generation,
                        &entry.relationships,
                        false,
                        None,
                    )?;
                }
                entries.push(row);
            }
            json!({"state":actor.activities.state,"entries": entries})
        }
    })
}

pub(super) fn human(
    record: &RecordDetailView,
    wanted: &[RecordSection],
    reading: &[EmbeddedReadingEntryView],
    layout: TerminalLayout,
) -> Result<String, String> {
    let mut out = String::new();
    presentation::selected_identity(&mut out, record);
    for section in wanted {
        out.push_str(&format!("\n{section:?}\n"));
        if *section == RecordSection::Rules {
            for field in &record.presentation.content {
                presentation::content_field(&mut out, field, layout, true)?;
            }
            if record.presentation.content.is_empty() {
                out.push_str("No rules text supplied.\n");
            }
            continue;
        }
        let body = &record.presentation.body;
        if let RecordBodyView::Spell(spell) = body
            && *section == RecordSection::Spellcasting
        {
            presentation::spell::render(&mut out, spell);
            continue;
        }
        if let RecordBodyView::Activity(a) = body
            && applies(*section, a)
        {
            presentation::actor::activity_facts(&mut out, a, false);
            for field in &record.presentation.content {
                presentation::content_field(&mut out, field, layout, true)?;
            }
            continue;
        }
        let Some(actor) = actor(body) else {
            out.push_str("Not applicable.\n");
            continue;
        };
        if *section == RecordSection::Defenses {
            presentation::actor::defenses(&mut out, actor, hazard(body));
        } else if let Some(activities) = &actor.activities.value {
            if !activities.iter().any(|a| applies(*section, a)) {
                out.push_str("No entries.\n");
                continue;
            }
            if *section == RecordSection::Spellcasting {
                presentation::actor::spellcasting(&mut out, activities, false);
            } else {
                for a in activities.iter().filter(|a| applies(*section, a)) {
                    presentation::actor::activity_facts(&mut out, a, true);
                    if let Some(entry) = reading.iter().find(|e| e.navigation == a.navigation) {
                        for field in &entry.content {
                            presentation::content_field(
                                &mut out,
                                field,
                                layout,
                                field.role != "description",
                            )?;
                        }
                    }
                    out.push_str(&format!(
                        "Navigation: {}\n",
                        Locator::new(&record.artifact_generation, &a.navigation).compact()
                    ));
                }
            }
        } else {
            out.push_str(&format!(
                "Owned entries: {}\n",
                actor.activities.state.as_str()
            ));
        }
    }
    Ok(out)
}

pub(crate) fn render_encounter_actor(
    p: &atlas_app_model::RecordPresentationView,
) -> Result<String, String> {
    let mut out = String::new();
    let width = super::width();
    presentation::render_actor(
        &mut out,
        p,
        &[],
        TerminalLayout::current(width),
        TerminalDetail::Standard,
    )?;
    Ok(super::terminal::wrap_facts(&out, width))
}
