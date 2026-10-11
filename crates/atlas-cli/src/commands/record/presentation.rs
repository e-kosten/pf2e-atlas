//! Terminal layout over the app-service's selected semantic facts.
use super::{args::TerminalDetail, terminal::render_html};
use atlas_app_model::{
    DamageComponentView, FactView, NumberFactView, PreparedFieldBodyView, RecordBodyView,
    RecordDetailView, RecordNavigationView, RecordSummaryView,
};
use atlas_domain::QueryFieldState;

mod actor;
mod spell;

pub(super) fn render_record(
    record: &RecordDetailView,
    detail: TerminalDetail,
    width: usize,
) -> Result<String, String> {
    let mut out = String::new();
    let presentation = &record.presentation;
    identity(&mut out, &presentation.identity);
    if !record.selected.owners.is_empty() {
        line(
            &mut out,
            format!(
                "From {} ({})",
                record.record.title, record.record.record_key
            ),
        );
    }
    if !record.selected.owners.is_empty() || record.selected.field.is_some() {
        navigation(&mut out, &record.selected)?;
    }
    if detail == TerminalDetail::Summary {
        return Ok(out);
    }
    line(&mut out, "");
    match &presentation.body {
        RecordBodyView::Creature(body) => {
            if presentation.identity.level_label.is_none() {
                number(&mut out, "Level", &body.level, false, "");
            }
            actor::render(&mut out, body)?;
        }
        RecordBodyView::Hazard(body) => {
            if presentation.identity.level_label.is_none() {
                number(&mut out, "Level", &body.actor.level, false, "");
            }
            fact(&mut out, "Complex", &body.complex, |v| v.to_string());
            number(&mut out, "Stealth", &body.stealth, true, "");
            number(&mut out, "Hardness", &body.hardness, false, "");
            actor::render(&mut out, &body.actor)?;
        }
        RecordBodyView::Spell(body) => {
            if presentation.identity.level_label.is_none() {
                number(&mut out, "Rank", &body.rank, false, "");
            }
            spell::render(&mut out, body);
        }
        RecordBodyView::Activity(body) => actor::activity_facts(&mut out, body, false)?,
        RecordBodyView::PhysicalReference(body) => {
            optional_text(&mut out, "Usage", &body.usage);
            number(&mut out, "Bulk", &body.bulk, false, "");
            optional_text(&mut out, "Price", &body.price);
            optional_number(&mut out, "Price per", &body.price_per, false, "");
        }
        RecordBodyView::RollTable(body) => text(&mut out, "Formula", &body.formula),
        RecordBodyView::TableResult(body) => {
            text(&mut out, "Range", &body.range);
            number(&mut out, "Weight", &body.weight, false, "");
            text(&mut out, "Result type", &body.result_type);
        }
        RecordBodyView::Content => {}
    }
    for field in &presentation.content {
        let rendered = match &field.body {
            PreparedFieldBodyView::Html { html, .. } => render_html(html, width)?,
            PreparedFieldBodyView::Plain { text } => text.clone(),
            PreparedFieldBodyView::Unavailable {
                state: QueryFieldState::NotApplicable,
            } => continue,
            PreparedFieldBodyView::Unavailable { state } => unavailable(*state),
        };
        if rendered.trim().is_empty() {
            continue;
        }
        line(&mut out, "");
        // These semantic roles are assigned by the content owner, not parsed from prose.
        let label = match field.role.as_str() {
            "description" => "Description",
            "disable" => "Disable",
            "routine" => "Routine",
            "reset" => "Reset",
            "stealth" => "Detection",
            "notes" => "Notes",
            "biography" => "Biography",
            "caption" => "Caption",
            "tableresult" => "Result",
            "text" => "Text",
            _ => &field.role,
        };
        line(&mut out, label);
        line(&mut out, rendered.trim_end());
    }
    if !matches!(
        presentation.body,
        RecordBodyView::Creature(_) | RecordBodyView::Hazard(_)
    ) {
        for owned in &presentation.owned {
            line(
                &mut out,
                format!(
                    "\n{} — {}{}",
                    owned.title,
                    owned.family,
                    owned
                        .usage
                        .as_ref()
                        .map(|v| format!("; {v}"))
                        .unwrap_or_default()
                ),
            );
            navigation(&mut out, &owned.navigation)?;
        }
    }
    Ok(out)
}

fn identity(out: &mut String, identity: &RecordSummaryView) {
    line(
        out,
        format!(
            "{} — {}{}",
            identity.title,
            identity.kind_label,
            identity
                .level_label
                .as_ref()
                .map(|v| format!(" {v}"))
                .unwrap_or_default()
        ),
    );
    line(out, format!("Record: {}", identity.record_key));
    if let Some(rarity) = &identity.rarity {
        line(out, format!("Rarity: {rarity}"));
    }
    if !identity.traits.is_empty() {
        line(
            out,
            format!(
                "Traits: {}",
                identity
                    .traits
                    .iter()
                    .map(|v| v.label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
    }
    if let Some(publication) = &identity.publication
        && !publication.trim().is_empty()
    {
        line(out, format!("Source: {publication}"));
    }
    if let Some(pack) = &identity.pack
        && !pack.trim().is_empty()
    {
        line(out, format!("Pack: {pack}"));
    }
}
fn line(out: &mut String, text: impl AsRef<str>) {
    out.push_str(text.as_ref());
    out.push('\n');
}
fn unavailable(state: QueryFieldState) -> String {
    format!("[{}]", state.as_str())
}
fn fact_string<T>(value: &FactView<T>, format: impl FnOnce(&T) -> String) -> String {
    match (value.state, &value.value) {
        (QueryFieldState::Value, Some(v)) => format(v),
        (state, _) => unavailable(state),
    }
}
fn known_value<T>(fact: &FactView<T>) -> Option<&T> {
    if fact.state == QueryFieldState::Value {
        fact.value.as_ref()
    } else {
        None
    }
}
fn fact<T>(out: &mut String, label: &str, value: &FactView<T>, format: impl FnOnce(&T) -> String) {
    if value.state == QueryFieldState::NotApplicable {
        return;
    }
    line(out, format!("{label}: {}", fact_string(value, format)));
}
fn text(out: &mut String, label: &str, value: &FactView<String>) {
    if known_value(value).is_some_and(|v| v.trim().is_empty()) {
        return;
    }
    fact(out, label, value, Clone::clone);
}
fn optional_text(out: &mut String, label: &str, value: &FactView<String>) {
    if !matches!(
        value.state,
        QueryFieldState::Missing | QueryFieldState::NotApplicable
    ) {
        text(out, label, value);
    }
}
fn list_text(items: &[String]) -> String {
    if items.is_empty() {
        "none".into()
    } else {
        items.join(", ")
    }
}
fn number_text(value: &NumberFactView, signed: bool, unit: &str) -> String {
    match (value.state, &value.value) {
        (QueryFieldState::Value, Some(v)) => {
            let text = v.to_string();
            let sign = if signed && !text.starts_with('-') {
                "+"
            } else {
                ""
            };
            let mut text = format!("{sign}{text}{unit}");
            if let Some(adjustment) = &value.adjustment {
                let mut explanations = vec![format!("authored {}", adjustment.authored)];
                for modifier in &adjustment.applied {
                    explanations.push(format!("{} {:+}", modifier.label, modifier.value));
                }
                for modifier in &adjustment.suppressed {
                    explanations.push(format!(
                        "{} {:+} suppressed",
                        modifier.label, modifier.value
                    ));
                }
                for note in &adjustment.notes {
                    explanations.push(format!("{}: {}", note.label, note.reason));
                }
                text.push_str(&format!(" ({})", explanations.join("; ")));
            }
            text
        }
        (state, _) => unavailable(state),
    }
}
fn number(out: &mut String, label: &str, value: &NumberFactView, signed: bool, unit: &str) {
    if value.state == QueryFieldState::NotApplicable {
        return;
    }
    line(
        out,
        format!("{label}: {}", number_text(value, signed, unit)),
    );
}
fn optional_number(
    out: &mut String,
    label: &str,
    value: &NumberFactView,
    signed: bool,
    unit: &str,
) {
    if !matches!(
        value.state,
        QueryFieldState::Missing | QueryFieldState::NotApplicable
    ) {
        number(out, label, value, signed, unit);
    }
}
fn navigation(out: &mut String, navigation: &RecordNavigationView) -> Result<(), String> {
    let mut command = format!("atlas record get {}", quote(&navigation.record_key));
    if !navigation.owners.is_empty() {
        command.push_str(&format!(
            " --owners {}",
            quote(&serde_json::to_string(&navigation.owners).map_err(|e| e.to_string())?)
        ));
    }
    if let Some(field) = &navigation.field {
        command.push_str(&format!(" --field {}", quote(field)));
    }
    if let Some(passage) = &navigation.passage {
        command.push_str(&format!(
            " --passage {}",
            quote(&serde_json::to_string(passage).map_err(|e| e.to_string())?)
        ));
    }
    if let Some(fingerprint) = &navigation.source_fingerprint {
        command.push_str(&format!(" --source-fingerprint {}", quote(fingerprint)));
    }
    line(out, format!("Open: {command}"));
    Ok(())
}
fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
fn damage(out: &mut String, value: &FactView<Vec<DamageComponentView>>) {
    damage_components(out, value, false);
}
fn damage_changes(out: &mut String, value: &FactView<Vec<DamageComponentView>>) {
    damage_components(out, value, true);
}
fn damage_components(out: &mut String, value: &FactView<Vec<DamageComponentView>>, changes: bool) {
    if value.state == QueryFieldState::NotApplicable
        || (changes && value.state == QueryFieldState::Missing)
    {
        return;
    }
    if changes && value.state == QueryFieldState::Null {
        line(out, "Damage: cleared (null)");
        return;
    }
    if value.state != QueryFieldState::Value {
        line(out, format!("Damage: {}", unavailable(value.state)));
        return;
    }
    let Some(components) = &value.value else {
        line(out, "Damage: [value unavailable]");
        return;
    };
    if components.is_empty() {
        line(out, "Damage: none");
    }
    for component in components {
        let mut parts = Vec::new();
        for (label, value) in [
            ("formula", &component.formula),
            ("damage type", &component.damage_type),
        ] {
            if changes
                && matches!(
                    value.state,
                    QueryFieldState::Missing | QueryFieldState::NotApplicable
                )
            {
                continue;
            }
            if changes && value.state == QueryFieldState::Null {
                parts.push(format!("{label}: cleared (null)"));
            } else {
                parts.push(fact_string(value, Clone::clone));
            }
        }
        for (label, value) in [
            ("kinds", &component.kinds),
            ("materials", &component.materials),
        ] {
            if matches!(
                value.state,
                QueryFieldState::Missing | QueryFieldState::NotApplicable
            ) {
                continue;
            }
            if changes && value.state == QueryFieldState::Null {
                parts.push(format!("{label}: cleared (null)"));
                continue;
            }
            if !changes
                && known_value(value)
                    .is_some_and(|v| v.is_empty() || (label == "kinds" && v == &["damage"]))
            {
                continue;
            }
            parts.push(format!("{label}: {}", fact_string(value, |v| list_text(v))));
        }
        if !matches!(
            component.category.state,
            QueryFieldState::Missing | QueryFieldState::NotApplicable
        ) {
            if changes && component.category.state == QueryFieldState::Null {
                parts.push("category: cleared (null)".into());
            } else if component.category.state != QueryFieldState::Null
                && !known_value(&component.category).is_some_and(|v| v.trim().is_empty())
            {
                parts.push(format!(
                    "category: {}",
                    fact_string(&component.category, Clone::clone)
                ));
            }
        }
        if !matches!(
            component.apply_modifier.state,
            QueryFieldState::Missing | QueryFieldState::NotApplicable
        ) {
            if changes && component.apply_modifier.state == QueryFieldState::Null {
                parts.push("apply modifier: cleared (null)".into());
            } else if changes || known_value(&component.apply_modifier) != Some(&false) {
                parts.push(format!(
                    "apply modifier: {}",
                    fact_string(&component.apply_modifier, |v| v.to_string())
                ));
            }
        }
        if parts.is_empty() {
            continue;
        }
        let label = if changes {
            "Damage component changes"
        } else if known_value(&component.kinds).is_some_and(|v| v == &["healing"]) {
            "Healing"
        } else if known_value(&component.kinds)
            .is_some_and(|v| v.contains(&"healing".into()) && v.contains(&"damage".into()))
        {
            "Damage/healing"
        } else {
            "Damage"
        };
        line(
            out,
            format!("{label} [{}]: {}", component.id, parts.join("; ")),
        );
    }
}

#[cfg(test)]
mod tests;
