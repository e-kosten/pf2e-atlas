//! CLI projections of checked reference content for a single selection.
use super::{
    args::{RecordSection, TerminalDetail},
    navigation::Locator,
    sections,
    terminal::{TerminalLayout, render_html},
};
use atlas_app_model::{
    EmbeddedReadingEntryView, PreparedContentFieldView, PreparedFieldBodyView, RecordBodyView,
    RecordDetailView, RecordNavigationView, RecordRelationshipView, RecordSummaryView,
};
use serde_json::{Value, json};

pub(super) fn header(
    identity: &RecordSummaryView,
    generation: &str,
    selected: &RecordNavigationView,
) -> Value {
    let link = atlas_desktop_protocol::EntityLink::from_key("record", &selected.record_key)
        .ok()
        .map(|e| e.uri());
    json!({
        "identity": identity,
        "citation": {"publication":identity.publication,"root_uri":link},
        "selected": Locator::new(generation,selected),
    })
}

pub(super) fn rules(record: &RecordDetailView, excerpt: Option<&str>) -> Result<Value, String> {
    rule_fields(
        &record.presentation.content,
        &record.selected,
        &record.artifact_generation,
        &record.relationships,
        record.relationships_truncated,
        excerpt,
    )
}

pub(super) fn rule_fields(
    content: &[PreparedContentFieldView],
    selection: &RecordNavigationView,
    generation: &str,
    relationships: &[RecordRelationshipView],
    truncated: bool,
    excerpt: Option<&str>,
) -> Result<Value, String> {
    let mut fields = Vec::new();
    for field in content {
        let mut address = selection.clone();
        address.record_key = field.locator.record.to_string();
        address.owners = field.locator.owners.clone();
        address.field = Some(field.locator.field.clone());
        address.source_fingerprint = field.source_fingerprint.clone();
        let mut row =
            json!({"role":field.role,"navigation":Locator::new(generation,&address).compact()});
        if let Some(text) = excerpt {
            row["text"] = json!(text);
        } else {
            match &field.body {
                PreparedFieldBodyView::Html { html, .. } => {
                    row["text"] = json!(
                        render_html(
                            html,
                            TerminalLayout {
                                width: 100,
                                styled: false
                            }
                        )?
                        .trim_end()
                    );
                }
                PreparedFieldBodyView::Plain { text } => row["text"] = json!(text),
                PreparedFieldBodyView::Unavailable { state } => row["state"] = json!(state),
                PreparedFieldBodyView::Bounded { reason } => {
                    row["unavailable_reason"] = json!(reason)
                }
            }
            let links = super::links::visible(field, relationships)?
                .iter()
                .map(|link| link.json(generation))
                .collect::<Vec<_>>();
            if !links.is_empty() {
                row["links"] = json!(links);
                row["links_truncated"] = json!(truncated);
            }
        }
        fields.push(row);
    }
    Ok(Value::Array(fields))
}

pub(super) fn selected(
    record: &RecordDetailView,
    detail: TerminalDetail,
    wanted: &[RecordSection],
    reading: &[EmbeddedReadingEntryView],
    excerpt: Option<&str>,
) -> Result<Value, String> {
    let mut value = reference(record, detail, wanted, reading, excerpt)?;
    super::navigation::bind_nested(&mut value, &record.artifact_generation);
    Ok(value)
}

fn reference(
    record: &RecordDetailView,
    detail: TerminalDetail,
    wanted: &[RecordSection],
    reading: &[EmbeddedReadingEntryView],
    excerpt: Option<&str>,
) -> Result<Value, String> {
    let p = &record.presentation;
    let mut output = header(&p.identity, &record.artifact_generation, &record.selected);
    if !record.selected.owners.is_empty() {
        output["parent"] = json!(&record.record);
    }
    if record.selected.field.is_some() {
        output["rules"] = rules(record, excerpt)?;
        if detail == TerminalDetail::Full && excerpt.is_none() {
            output["prepared_fields"] = json!(p.content);
        }
        return Ok(output);
    }
    if record.selected.passage.is_some() || (detail == TerminalDetail::Summary && wanted.is_empty())
    {
        return Ok(output);
    }
    if !wanted.is_empty() {
        for section in wanted {
            output["sections"][section.name()] = sections::json(*section, record, reading)?;
        }
        if detail == TerminalDetail::Full && wanted.contains(&RecordSection::Rules) {
            output["prepared_fields"] = json!(p.content);
        }
        return Ok(output);
    }
    if let Some(canonical) = &record.canonical_record {
        output["canonical_record"] = json!(canonical);
    }
    if let Some(links) = &record.named_creature_links {
        output["named_creature_links"] = json!(links);
    }
    output["remaster_relationships"] = json!(record.remaster_relationships);
    if let Some(context) = &record.source_remaster_context {
        output["source_remaster_context"] = json!(context);
    }
    if detail == TerminalDetail::Full {
        output["content"] = json!({
            "classification": p.classification,
            "fields": p.content,
            "embedded": p.owned,
            "body": p.body,
            "grants": p.grants,
        });
        output["relationships"] = json!(record.relationships);
        output["relationships_truncated"] = json!(record.relationships_truncated);
        if let Some(placement) = &record.creature_link_placement {
            output["creature_link_placement"] = json!(placement);
        }
        if let Some(family) = &record.creature_family {
            output["creature_family"] = json!(family);
        }
        if let Some(family) = &record.equipment_family {
            output["equipment_family"] = json!(family);
        }
        if let Some(adjustment) = &record.encounter_adjustment {
            output["encounter_adjustment"] = json!(adjustment);
        }
        if !record.creature_overrides.is_empty() {
            output["creature_overrides"] = json!(record.creature_overrides);
        }
        return Ok(output);
    }
    output["rules"] = rules(record, None)?;
    if let Some(actor) = sections::actor(&p.body) {
        for section in [
            RecordSection::Defenses,
            RecordSection::Strikes,
            RecordSection::Abilities,
            RecordSection::Spellcasting,
        ] {
            output["sections"][section.name()] = sections::json(section, record, &[])?;
        }
        output["perception"] = json!(actor.perception);
        output["speed"] = json!(actor.land_speed);
        if let RecordBodyView::Hazard(h) = &p.body {
            output["hazard"] =
                json!({"complex":h.complex,"stealth":h.stealth,"detection_dc":h.detection_dc});
        }
    } else {
        output["facts"] = match &p.body {
            RecordBodyView::Activity(a) => {
                json!({
                    "cost": a.action_cost,
                    "usage": a.usage,
                    "frequency": a.frequency,
                    "attack": a.attack,
                    "damage": a.damage,
                    "dc": a.difficulty_class,
                })
            }
            RecordBodyView::Spell(s) => {
                json!({
                    "rank": s.rank,
                    "base_rank": s.base_rank,
                    "rank_changes": s.rank_changes,
                    "rank_note": s.rank_note,
                    "cast": s.cast,
                    "traditions": s.traditions,
                    "range": s.range,
                    "target": s.target,
                    "area": s.area,
                    "defense": s.defense,
                    "duration": s.duration,
                    "damage": s.damage,
                    "heightening": s.heightening,
                    "forms": s.forms,
                    "requirements": s.requirements,
                    "cost": s.cost,
                })
            }
            _ => json!(p.body),
        };
    }
    Ok(output)
}
