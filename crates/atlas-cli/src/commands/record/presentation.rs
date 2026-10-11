//! Terminal layout over the app-service's selected semantic facts.
use super::{args::TerminalDetail, terminal::render_html};
use atlas_app_model::{
    DamageComponentView, FactView, NumberFactView, OwnedReadingEntryView, PreparedContentFieldView,
    PreparedFieldBodyView, RecordBodyView, RecordDetailView, RecordSummaryView,
};
use atlas_domain::QueryFieldState;

mod actor;
mod spell;

pub(super) fn render_record(
    record: &RecordDetailView,
    detail: TerminalDetail,
    width: usize,
    reading: &[OwnedReadingEntryView],
) -> Result<String, String> {
    let mut out = String::new();
    let presentation = &record.presentation;
    identity(&mut out, &presentation.identity);
    if !record.selected.owners.is_empty() {
        line(&mut out, format!("From {}", record.record.title));
    }
    if detail == TerminalDetail::Summary {
        return Ok(out);
    }
    if record.selected.field.is_some() {
        for field in &presentation.content {
            content_field(&mut out, field, width, true)?;
        }
        return Ok(out);
    }
    line(&mut out, "");
    for field in presentation.content.iter().filter(|field| is_intro(field)) {
        content_field(&mut out, field, width, true)?;
    }
    line(&mut out, "");
    match &presentation.body {
        RecordBodyView::Creature(body) => {
            if presentation.identity.level_label.is_none() {
                number(&mut out, "Level", &body.level, false, "");
            }
            actor::render(&mut out, body, true, true, reading, width, detail)?;
        }
        RecordBodyView::Hazard(body) => {
            if presentation.identity.level_label.is_none() {
                number(&mut out, "Level", &body.actor.level, false, "");
            }
            fact(&mut out, "Hazard", &body.complex, |v| {
                if *v { "Complex" } else { "Simple" }.into()
            });
            line(&mut out, "\nDetection");
            number(&mut out, "Stealth", &body.stealth, true, "");
            for field in presentation.content.iter().filter(|f| f.role == "stealth") {
                content_field(&mut out, field, width, false)?;
            }
            for field in presentation.content.iter().filter(|f| f.role == "disable") {
                content_field(&mut out, field, width, true)?;
            }
            let defenses = known_value(&body.defense_block) != Some(&false);
            if defenses {
                number(&mut out, "Hardness", &body.hardness, false, "");
            }
            actor::render(
                &mut out,
                &body.actor,
                defenses,
                body.show_saves,
                reading,
                width,
                detail,
            )?;
        }
        RecordBodyView::Spell(body) => {
            if presentation.identity.level_label.is_none() {
                number(&mut out, "Rank", &body.rank, false, "");
            }
            spell::render(&mut out, body);
        }
        RecordBodyView::Activity(body) => actor::activity_facts(&mut out, body, false),
        RecordBodyView::PhysicalReference(body) => {
            optional_text(&mut out, "Usage", &body.usage);
            number(&mut out, "Bulk", &body.bulk, false, "");
            fact(&mut out, "Price", &body.price, |price| {
                let coins = [
                    ("pp", &price.pp),
                    ("gp", &price.gp),
                    ("sp", &price.sp),
                    ("cp", &price.cp),
                ]
                .into_iter()
                .filter(|(_, value)| {
                    !matches!(
                        value.state,
                        QueryFieldState::Missing | QueryFieldState::NotApplicable
                    )
                })
                .map(|(unit, value)| format!("{} {unit}", number_text(value, false, "")))
                .collect::<Vec<_>>();
                if coins.is_empty() {
                    "Empty".into()
                } else {
                    coins.join(", ")
                }
            });
            optional_number(&mut out, "Price per", &body.price_per, false, "");
        }
        RecordBodyView::RollTable(body) => {
            required_text(&mut out, "Formula", &body.formula);
            if body.results.state != QueryFieldState::Value {
                fact(&mut out, "Results", &body.results, |_| String::new());
            }
        }
        RecordBodyView::TableResult(body) => {
            required_text(&mut out, "Range", &body.range);
            number(&mut out, "Weight", &body.weight, false, "");
            required_text(&mut out, "Result type", &body.result_type);
        }
        RecordBodyView::Content => {}
    }
    for field in presentation.content.iter().filter(|field| {
        !is_intro(field)
            && !(matches!(presentation.body, RecordBodyView::Hazard(_))
                && matches!(field.role.as_str(), "stealth" | "disable"))
    }) {
        content_field(&mut out, field, width, true)?;
    }
    if detail != TerminalDetail::Full
        && !matches!(
            presentation.body,
            RecordBodyView::Creature(_) | RecordBodyView::Hazard(_)
        )
    {
        let owned = match &presentation.body {
            RecordBodyView::RollTable(table) => table.results.value.as_deref().unwrap_or_default(),
            _ => &presentation.owned,
        };
        for owned in owned {
            line(
                &mut out,
                format!(
                    "\n{} — {}{}",
                    owned.title,
                    owned.family,
                    owned
                        .action_cost
                        .value
                        .as_ref()
                        .map(|v| format!("; {}", v.label))
                        .or_else(|| owned.usage.as_ref().map(|v| format!("; {v}")))
                        .unwrap_or_default()
                ),
            );
        }
    }
    Ok(out.lines().fold(String::new(), |mut rendered, line| {
        if !line.is_empty() || !rendered.ends_with("\n\n") {
            rendered.push_str(line);
            rendered.push('\n');
        }
        rendered
    }))
}

fn is_intro(field: &PreparedContentFieldView) -> bool {
    matches!(field.role.as_str(), "description" | "notes" | "biography")
}

fn content_field(
    out: &mut String,
    field: &PreparedContentFieldView,
    width: usize,
    heading: bool,
) -> Result<(), String> {
    let rendered = match &field.body {
        PreparedFieldBodyView::Html { html, .. } => render_html(html, width)?,
        PreparedFieldBodyView::Plain { text } => text.clone(),
        PreparedFieldBodyView::Bounded { reason } => reason.clone(),
        PreparedFieldBodyView::Unavailable {
            state: QueryFieldState::NotApplicable,
        } => return Ok(()),
        PreparedFieldBodyView::Unavailable { state } => unavailable(*state),
    };
    if rendered.trim().is_empty() {
        return Ok(());
    }
    line(out, "");
    if heading {
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
        line(out, label);
    }
    line(out, rendered.trim_end());
    Ok(())
}

pub(super) fn identity(out: &mut String, identity: &RecordSummaryView) {
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
    let text = fact_string(value, format);
    if value.state == QueryFieldState::Value && text.trim().is_empty() {
        return;
    }
    line(out, format!("{label}: {text}"));
}
fn text(out: &mut String, label: &str, value: &FactView<String>) {
    if known_value(value).is_some_and(|v| v.trim().is_empty()) {
        return;
    }
    fact(out, label, value, Clone::clone);
}
fn required_text(out: &mut String, label: &str, value: &FactView<String>) {
    fact(out, label, value, |value| {
        if value.trim().is_empty() {
            "Empty".into()
        } else {
            value.clone()
        }
    });
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
        String::new()
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
        if changes {
            line(out, "Damage: Empty");
        }
        return;
    }
    for (index, component) in components.iter().enumerate() {
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
                parts.push(fact_string(value, |value| {
                    if value.trim().is_empty() {
                        format!("{label}: Empty")
                    } else {
                        value.clone()
                    }
                }));
            }
        }
        for (label, value) in [
            ("kinds", &component.kinds),
            ("materials", &component.materials),
        ] {
            if value.state == QueryFieldState::NotApplicable
                || (changes && value.state == QueryFieldState::Missing)
            {
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
            parts.push(format!(
                "{label}: {}",
                fact_string(value, |v| {
                    if changes && v.is_empty() {
                        "Empty".into()
                    } else {
                        list_text(v)
                    }
                })
            ));
        }
        if !matches!(
            component.category.state,
            QueryFieldState::Missing | QueryFieldState::NotApplicable
        ) {
            if changes && component.category.state == QueryFieldState::Null {
                parts.push("category: cleared (null)".into());
            } else if component.category.state != QueryFieldState::Null
                && (changes
                    || !known_value(&component.category).is_some_and(|v| v.trim().is_empty()))
            {
                parts.push(format!(
                    "category: {}",
                    fact_string(&component.category, |v| if changes && v.trim().is_empty() {
                        "Empty".into()
                    } else {
                        v.clone()
                    })
                ));
            }
        }
        if component.apply_modifier.state != QueryFieldState::NotApplicable
            && !(changes && component.apply_modifier.state == QueryFieldState::Missing)
        {
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
            format!(
                "{label}{}: {}",
                if components.len() > 1 {
                    format!(" {}", index + 1)
                } else {
                    String::new()
                },
                parts.join("; ")
            ),
        );
    }
}

#[cfg(test)]
mod tests;
