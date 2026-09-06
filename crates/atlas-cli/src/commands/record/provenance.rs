use atlas_record::{
    CreatureAvailabilityEvidenceJson, CreatureContentJson, CreatureContentOwnerJson,
    CreatureContentProvenanceJson, CreatureFactProvenanceJson, CreatureOccurrenceProvenanceJson,
    CreatureProvenanceJson, HazardProvenanceJson, RecordEditionContextJson,
    RecordEditionCounterpartLookupJson, RecordEditionCounterpartRoleJson, RecordEditionStatusJson,
    RecordJson, RecordPresentationJson, SpellProvenanceJson,
};
use atlas_search::GraphContextResult;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct RecordProvenanceData {
    pub key: String,
    pub name: String,
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_provenance: Option<CreatureProvenanceJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hazard_provenance: Option<HazardProvenanceJson>,
    pub spell_provenance: Option<SpellProvenanceJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<RecordEditionContextJson>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub occurrences: Vec<OccurrenceProvenance>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub prepared_spells: Vec<PreparedSpellProvenance>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unmodeled_skills: Vec<UnmodeledSkillProvenance>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<ContentOwnershipProvenance>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub embedded_relationships: Vec<atlas_record::CreatureRelationshipJson>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub availability_evidence: Vec<CreatureAvailabilityEvidenceJson>,
    pub references: ReferenceProvenance,
}

#[derive(Debug, Serialize)]
pub(super) struct UnmodeledSkillProvenance {
    pub component_id: String,
    pub authored_order: u32,
    pub authored_key: String,
    pub base: atlas_record::CreatureIntegerPresenceJson,
    pub reason: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub source_entries: Vec<atlas_record::CreatureSkillSourceEntryJson>,
}

#[derive(Debug, Serialize)]
pub(super) struct OccurrenceProvenance {
    pub family: &'static str,
    pub id: String,
    pub order: u32,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_record_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_entity_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_entry_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepared_slot_locator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<CreatureOccurrenceProvenanceJson>,
}

#[derive(Debug, Serialize)]
pub(super) struct PreparedSpellProvenance {
    pub entry_occurrence_id: String,
    pub rank: i64,
    pub authored_order: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_item_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expended: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepared: Option<bool>,
}

#[derive(Debug, Serialize)]
pub(super) struct ContentOwnershipProvenance {
    pub content_key: String,
    pub owner: CreatureContentOwnerJson,
    pub role: &'static str,
    pub authored_order: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub content_hash: String,
    pub visibility: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<CreatureContentProvenanceJson>,
}

#[derive(Debug, Default, Serialize)]
pub(super) struct ReferenceProvenance {
    pub lookup_performed: bool,
    pub outgoing_truncated: bool,
    pub backlinks_truncated: bool,
    pub outgoing_total: usize,
    pub backlinks_total: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<ReferenceProvenanceEdge>,
}

#[derive(Debug, Serialize)]
pub(super) struct ReferenceProvenanceEdge {
    pub direction: &'static str,
    pub from_record_key: String,
    pub to_record_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_text: Option<String>,
    pub reference_text: String,
    pub source_kind: String,
    pub visibility: String,
    pub relation_kind: String,
}

pub(super) fn provenance_data(
    record: &RecordJson,
    graph: Option<&GraphContextResult>,
) -> RecordProvenanceData {
    let mut data = RecordProvenanceData {
        key: record.key.clone(),
        name: record.name.clone(),
        kind: record.kind,
        record_provenance: None,
        hazard_provenance: None,
        spell_provenance: None,
        edition: None,
        occurrences: Vec::new(),
        prepared_spells: Vec::new(),
        unmodeled_skills: Vec::new(),
        content: Vec::new(),
        embedded_relationships: Vec::new(),
        availability_evidence: Vec::new(),
        references: reference_provenance(graph),
    };
    if let RecordPresentationJson::Hazard {
        provenance,
        edition,
        ..
    } = &record.presentation
    {
        data.hazard_provenance = provenance.clone();
        data.edition = edition.clone();
        return data;
    }
    if let RecordPresentationJson::Spell { spell, edition, .. } = &record.presentation {
        data.spell_provenance = spell.provenance.clone();
        data.edition = edition.clone();
        return data;
    }
    let RecordPresentationJson::Creature {
        strikes,
        actions,
        spellcasting,
        skills,
        equipment,
        lore,
        content,
        relationships,
        provenance,
        edition,
        availability_evidence,
        ..
    } = &record.presentation
    else {
        return data;
    };
    data.record_provenance = provenance.clone();
    data.edition = edition.clone();
    data.availability_evidence = availability_evidence.clone().unwrap_or_default();
    data.embedded_relationships = relationships.clone().unwrap_or_default();
    data.content
        .extend(content.iter().flatten().map(content_provenance));
    for skill in skills.iter().flatten() {
        if let Some(unmodeled) = &skill.unmodeled {
            data.unmodeled_skills.push(UnmodeledSkillProvenance {
                component_id: skill.id.clone(),
                authored_order: skill.order,
                authored_key: unmodeled.authored_key.clone(),
                base: unmodeled.base.clone(),
                reason: unmodeled.reason,
                source_entries: skill.source_entries.clone(),
            });
        }
    }

    for row in strikes.iter().flatten() {
        data.occurrences.push(occurrence(
            "strike",
            &row.id,
            row.order,
            &row.label,
            row.target_record_key.clone(),
            row.target_entity_id.clone(),
            None,
            None,
            row.provenance.clone(),
        ));
        data.content
            .extend(row.content.iter().flatten().map(content_provenance));
    }
    for row in actions.iter().flatten() {
        data.occurrences.push(occurrence(
            "activity",
            &row.id,
            row.order,
            &row.label,
            row.target_record_key.clone(),
            row.target_entity_id.clone(),
            None,
            None,
            row.provenance.clone(),
        ));
        data.content
            .extend(row.content.iter().flatten().map(content_provenance));
    }
    if let Some(spellcasting) = spellcasting {
        for entry in &spellcasting.entries {
            data.occurrences.push(occurrence(
                "spellcasting_entry",
                &entry.id,
                entry.order,
                &entry.label,
                entry.target_record_key.clone(),
                entry.target_entity_id.clone(),
                None,
                None,
                entry.provenance.clone(),
            ));
            data.content
                .extend(entry.content.iter().flatten().map(content_provenance));
            for slot in entry.slots.iter().flatten() {
                for prepared in slot.prepared.iter().flatten() {
                    data.prepared_spells.push(PreparedSpellProvenance {
                        entry_occurrence_id: entry.id.clone(),
                        rank: slot.rank,
                        authored_order: prepared.order,
                        source_item_id: prepared.id.clone(),
                        name: prepared.name.clone(),
                        expended: prepared.expended,
                        prepared: prepared.prepared,
                    });
                }
            }
            for spell in &entry.spells {
                push_spell(&mut data, spell);
            }
        }
        for spell in &spellcasting.standalone_spells {
            push_spell(&mut data, spell);
        }
    }
    for row in equipment.iter().flatten() {
        data.occurrences.push(occurrence(
            "equipment",
            &row.id,
            row.order,
            &row.label,
            row.target_record_key.clone(),
            row.target_entity_id.clone(),
            None,
            None,
            row.provenance.clone(),
        ));
        data.content
            .extend(row.content.iter().flatten().map(content_provenance));
    }
    for row in lore.iter().flatten() {
        data.occurrences.push(occurrence(
            "lore",
            &row.id,
            row.order,
            &row.label,
            row.target_record_key.clone(),
            row.target_entity_id.clone(),
            None,
            None,
            row.provenance.clone(),
        ));
        data.content
            .extend(row.content.iter().flatten().map(content_provenance));
    }
    data.occurrences
        .sort_by_key(|row| (row.order, row.family, row.id.clone()));
    data.unmodeled_skills
        .sort_by_key(|row| (row.authored_order, row.component_id.clone()));
    data.prepared_spells.sort_by_key(|row| {
        (
            row.entry_occurrence_id.clone(),
            row.rank,
            row.authored_order,
            row.source_item_id.clone(),
        )
    });
    data.content
        .sort_by_key(|row| (row.authored_order, row.content_key.clone()));
    data
}

#[allow(clippy::too_many_arguments)]
fn occurrence(
    family: &'static str,
    id: &str,
    order: u32,
    label: &str,
    target_record_key: Option<String>,
    target_entity_id: Option<String>,
    parent_entry_id: Option<String>,
    prepared_slot_locator: Option<String>,
    provenance: Option<CreatureOccurrenceProvenanceJson>,
) -> OccurrenceProvenance {
    OccurrenceProvenance {
        family,
        id: id.to_string(),
        order,
        label: label.to_string(),
        target_record_key,
        target_entity_id,
        parent_entry_id,
        prepared_slot_locator,
        provenance,
    }
}

fn push_spell(data: &mut RecordProvenanceData, spell: &atlas_record::CreatureSpellJson) {
    data.occurrences.push(occurrence(
        "spell",
        &spell.id,
        spell.order,
        &spell.label,
        spell.target_record_key.clone(),
        spell.target_entity_id.clone(),
        spell.parent_entry_id.clone(),
        spell.context.slot.clone(),
        spell.provenance.clone(),
    ));
    data.content
        .extend(spell.content.iter().flatten().map(content_provenance));
}

fn content_provenance(row: &CreatureContentJson) -> ContentOwnershipProvenance {
    ContentOwnershipProvenance {
        content_key: row.content_key.clone(),
        owner: row.owner.clone(),
        role: row.role,
        authored_order: row.authored_order,
        label: row.label.clone(),
        content_hash: row.content_hash.clone(),
        visibility: row.visibility,
        provenance: row.provenance.clone(),
    }
}

fn reference_provenance(graph: Option<&GraphContextResult>) -> ReferenceProvenance {
    let Some(graph) = graph else {
        return ReferenceProvenance::default();
    };
    let mut references = ReferenceProvenance {
        lookup_performed: true,
        outgoing_truncated: graph.outgoing.truncated,
        backlinks_truncated: graph.backlinks.truncated,
        outgoing_total: graph.outgoing.total_edges,
        backlinks_total: graph.backlinks.total_edges,
        edges: Vec::new(),
    };
    for (direction, section) in [
        ("outgoing", &graph.outgoing),
        ("backlink", &graph.backlinks),
    ] {
        references
            .edges
            .extend(section.edges.iter().map(|edge| ReferenceProvenanceEdge {
                direction,
                from_record_key: edge.from.to_string(),
                to_record_key: edge.to.to_string(),
                display_text: edge.display_text.clone(),
                reference_text: edge.reference_text.clone(),
                source_kind: edge.source.kind.clone(),
                visibility: edge.source.visibility.clone(),
                relation_kind: edge.source.relation_kind.clone(),
            }));
    }
    references
}

fn spell_fact_text<T: std::fmt::Display>(value: &atlas_record::SpellFactJson<T>) -> String {
    match value {
        atlas_record::SpellFactJson::Missing => "missing".to_string(),
        atlas_record::SpellFactJson::Null => "null".to_string(),
        atlas_record::SpellFactJson::Known(value) => value.to_string(),
        atlas_record::SpellFactJson::Unsupported(value) => {
            format!("unsupported {} ({})", value.value, value.reason)
        }
    }
}

pub(super) fn render_provenance(data: &RecordProvenanceData) -> String {
    let mut lines = vec![
        data.name.clone(),
        format!("  Type: {}", data.kind),
        format!("  Key: {}", data.key),
    ];
    if let Some(provenance) = &data.record_provenance {
        lines.extend([
            String::new(),
            "Record provenance".to_string(),
            format!("  Source path: {}", provenance.source_path),
            format!("  Contract: {}", provenance.source_contract_version),
            format!("  System: {}", provenance.source_system_version),
            format!("  Upstream commit: {}", provenance.source_upstream_commit),
            "  Fact sources".to_string(),
        ]);
        for (label, fact) in fact_rows(provenance) {
            lines.push(format!("    {label}: {}", fact_text(fact)));
        }
    }
    if let Some(provenance) = &data.hazard_provenance {
        lines.extend([
            String::new(),
            "Hazard provenance".to_string(),
            format!("  Source path: {}", provenance.source_path),
            format!("  Contract: {}", provenance.source_contract_version),
            format!("  System: {}", provenance.source_system_version),
            format!("  Upstream commit: {}", provenance.source_upstream_commit),
            format!(
                "  Convenience projection: {} v{}",
                provenance.convenience_rule_id, provenance.convenience_rule_version
            ),
        ]);
        if let Some(license) = &provenance.publication_license {
            lines.push(format!("  Publication license: {license}"));
        }
        if !provenance.occurrences.is_empty() {
            lines.push("  Occurrences".to_string());
            for occurrence in &provenance.occurrences {
                lines.push(format!(
                    "    {}: {} -> {} (order {}, source ordinal {}, {})",
                    occurrence.family,
                    occurrence.id,
                    occurrence.entity_id,
                    occurrence.authored_order,
                    occurrence.source_ordinal,
                    occurrence.identity_stability
                ));
            }
        }
        if !provenance.content.is_empty() {
            lines.push("  Content ownership".to_string());
            for content in &provenance.content {
                lines.push(format!(
                    "    {}: {} ({}, {}, {})",
                    content.content_key,
                    content.owner,
                    content.role,
                    content.visibility,
                    content.content_hash
                ));
            }
        }
        if !provenance.unsupported_fields.is_empty() {
            lines.push(format!(
                "  Unsupported source fields: {}",
                provenance.unsupported_fields.len()
            ));
        }
    }
    if let Some(provenance) = &data.spell_provenance {
        lines.extend([
            String::new(),
            "Spell provenance".to_string(),
            format!("  Source ID: {}", provenance.source_id),
            format!("  Source path: {}", provenance.source_path),
            format!("  Contract: {}", provenance.source_contract_version),
            format!("  System: {}", provenance.source_system_version),
            format!("  Upstream commit: {}", provenance.source_upstream_commit),
        ]);
        lines.push(format!("  Image: {}", spell_fact_text(&provenance.image)));
        lines.push(format!(
            "  Publication license: {}",
            spell_fact_text(&provenance.publication_license)
        ));
        lines.push(format!(
            "  Standalone location: {}",
            spell_fact_text(&provenance.standalone_location)
        ));
        for member in &provenance.members {
            lines.push(format!(
                "  {} {} at {} (order {}, {} state)",
                member.field, member.authored_key, member.source_path, member.order, member.state
            ));
        }
        for fact in &provenance.unsupported {
            lines.push(format!(
                "  Unsupported {} at {} ({}): {}",
                fact.field, fact.source_path, fact.authored_key, fact.value.value
            ));
        }
    }
    if let Some(edition) = &data.edition {
        lines.extend([String::new(), "Edition".to_string()]);
        lines.push(format!(
            "  Status: {}",
            match edition.status {
                RecordEditionStatusJson::Legacy => "legacy",
                RecordEditionStatusJson::Remaster => "remaster",
            }
        ));
        match &edition.counterpart_lookup {
            RecordEditionCounterpartLookupJson::NotPerformed => {
                lines.push("  Counterpart lookup: not performed".to_string());
            }
            RecordEditionCounterpartLookupJson::Verified { counterparts } => {
                lines.push("  Counterpart lookup: verified".to_string());
                for row in counterparts {
                    let role = match row.role {
                        RecordEditionCounterpartRoleJson::LegacyCounterpart => "legacy",
                        RecordEditionCounterpartRoleJson::RemasteredCounterpart => "remastered",
                    };
                    lines.push(format!("    {role}: {} ({})", row.title, row.record_key));
                }
            }
        }
    }
    if !data.occurrences.is_empty() {
        lines.extend([String::new(), "Occurrences".to_string()]);
        for row in &data.occurrences {
            lines.push(format!("  {}: {}", row.family, row.label));
            lines.push(format!("    ID: {}", row.id));
            lines.push(format!("    Order: {}", row.order));
            if let Some(key) = &row.target_record_key {
                lines.push(format!("    Target record: {key}"));
            }
            if let Some(id) = &row.target_entity_id {
                lines.push(format!("    Target entity: {id}"));
            }
            if let Some(id) = &row.parent_entry_id {
                lines.push(format!("    Parent entry: {id}"));
            }
            if let Some(slot) = &row.prepared_slot_locator {
                lines.push(format!("    Prepared slot locator: {slot}"));
            }
            if let Some(provenance) = &row.provenance {
                lines.push(format!(
                    "    Identity stability: {}",
                    provenance.identity_stability
                ));
                if let Some(source_id) = &provenance.nested_source_id {
                    lines.push(format!("    Nested source ID: {source_id}"));
                }
                if let Some(locator) = &provenance.stable_source_locator {
                    lines.push(format!("    Stable source locator: {locator}"));
                }
                for locator in &provenance.source_locators {
                    lines.push(format!("    Source locator: {locator}"));
                }
            }
        }
    }
    if !data.unmodeled_skills.is_empty() {
        lines.extend([String::new(), "Unmodeled skills".to_string()]);
        for row in &data.unmodeled_skills {
            lines.push(format!("  {}", row.authored_key));
            lines.push(format!("    Component ID: {}", row.component_id));
            lines.push(format!("    Authored order: {}", row.authored_order));
            lines.push(format!("    Base: {}", integer_presence_text(&row.base)));
            lines.push(format!("    Reason: {}", row.reason));
            for source in &row.source_entries {
                lines.push(format!(
                    "    Source entry: {} ({})",
                    source.authored_key,
                    integer_presence_text(&source.modifier)
                ));
            }
        }
    }
    if !data.prepared_spells.is_empty() {
        lines.extend([String::new(), "Prepared spell slots".to_string()]);
        for row in &data.prepared_spells {
            lines.push(format!(
                "  Entry occurrence ID: {}",
                row.entry_occurrence_id
            ));
            lines.push(format!("    Rank: {}", row.rank));
            lines.push(format!("    Authored order: {}", row.authored_order));
            if let Some(id) = &row.source_item_id {
                lines.push(format!("    Source item ID: {id}"));
            }
            if let Some(name) = &row.name {
                lines.push(format!("    Name: {name}"));
            }
            if let Some(expended) = row.expended {
                lines.push(format!("    Expended: {expended}"));
            }
            if let Some(prepared) = row.prepared {
                lines.push(format!("    Prepared: {prepared}"));
            }
        }
    }
    if !data.content.is_empty() {
        lines.extend([String::new(), "Content ownership".to_string()]);
        for row in &data.content {
            lines.push(format!("  {}", row.label.as_deref().unwrap_or(row.role)));
            lines.push(format!("    Content key: {}", row.content_key));
            lines.push(format!("    Owner: {}", owner_text(&row.owner)));
            lines.push(format!("    Role: {}", row.role));
            lines.push(format!("    Authored order: {}", row.authored_order));
            lines.push(format!("    Visibility: {}", row.visibility));
            lines.push(format!("    Hash: {}", row.content_hash));
            if let Some(provenance) = &row.provenance {
                lines.push(format!(
                    "    Source record: {}",
                    provenance.source_record_key
                ));
                lines.push(format!(
                    "    Relative source path: {}",
                    provenance.relative_source_path
                ));
                lines.push(format!("    Field family: {}", provenance.field_family));
                if let Some(source_id) = &provenance.nested_source_id {
                    lines.push(format!("    Nested source ID: {source_id}"));
                }
            }
        }
    }
    if !data.embedded_relationships.is_empty() {
        lines.extend([String::new(), "Embedded relationships".to_string()]);
        for row in &data.embedded_relationships {
            lines.push(format!("  {}", row.kind));
            lines.push(format!(
                "    Source occurrence: {}",
                row.source_occurrence_id
            ));
            lines.push(format!("    Target: {}", embedded_target_text(&row.target)));
            lines.push(format!("    Source path: {}", row.source_path));
        }
    }
    if !data.availability_evidence.is_empty() {
        lines.extend([String::new(), "Availability evidence".to_string()]);
        for row in &data.availability_evidence {
            lines.push(format!("  {}: {:?}", row.field.as_str(), row.state).to_lowercase());
            if let Some(component_id) = &row.component_id {
                lines.push(format!("    Component ID: {component_id}"));
            }
            if let Some(authored_key) = &row.authored_key {
                lines.push(format!("    Authored key: {authored_key}"));
            }
            if let Some(source_value) = &row.source_value {
                lines.push(format!("    Source value: {source_value}"));
            }
            if let Some(source_shape) = row.source_shape {
                lines.push(format!("    Source shape: {source_shape}"));
            }
            if let Some(source_reason) = row.source_reason {
                lines.push(format!("    Source reason: {source_reason}"));
            }
            if let Some(source_path) = &row.source_path {
                lines.push(format!("    Source path: {source_path}"));
            }
            lines.push(format!("    Message: {}", row.message));
        }
    }
    lines.extend([String::new(), "References".to_string()]);
    if !data.references.lookup_performed {
        lines.push("  Lookup not performed".to_string());
    } else if data.references.edges.is_empty() {
        lines.push("  Verified: none".to_string());
    } else {
        lines.push(format!(
            "  Outgoing: {}{}",
            data.references.outgoing_total,
            if data.references.outgoing_truncated {
                " (truncated)"
            } else {
                ""
            }
        ));
        lines.push(format!(
            "  Backlinks: {}{}",
            data.references.backlinks_total,
            if data.references.backlinks_truncated {
                " (truncated)"
            } else {
                ""
            }
        ));
        for row in &data.references.edges {
            lines.push(format!(
                "  {}: {} -> {}",
                row.direction, row.from_record_key, row.to_record_key
            ));
            if let Some(label) = &row.display_text {
                lines.push(format!("    Label: {label}"));
            }
            lines.push(format!("    Reference text: {}", row.reference_text));
            lines.push(format!("    Relation kind: {}", row.relation_kind));
            lines.push(format!("    Source kind: {}", row.source_kind));
            lines.push(format!("    Visibility: {}", row.visibility));
        }
    }
    lines.push(String::new());
    lines.join("\n")
}

fn fact_rows(
    provenance: &CreatureProvenanceJson,
) -> [(&'static str, &CreatureFactProvenanceJson); 16] {
    [
        ("Level", &provenance.facts.level),
        ("Rarity", &provenance.facts.rarity),
        ("Traits", &provenance.facts.traits),
        ("Size", &provenance.facts.size),
        ("Publication", &provenance.facts.publication),
        ("Adjustment", &provenance.facts.adjustment),
        ("Source alliance", &provenance.facts.source_alliance),
        ("Perception", &provenance.facts.perception),
        ("Initiative", &provenance.facts.initiative),
        ("Languages", &provenance.facts.languages),
        ("Skills", &provenance.facts.skills),
        ("Abilities", &provenance.facts.abilities),
        ("Defenses", &provenance.facts.defenses),
        ("Movement", &provenance.facts.movement),
        ("Resources", &provenance.facts.resources),
        ("Embedded entities", &provenance.facts.embedded_entities),
    ]
}

fn fact_text(fact: &CreatureFactProvenanceJson) -> String {
    match fact {
        CreatureFactProvenanceJson::Source { field } => format!("source field {field}"),
        CreatureFactProvenanceJson::Derived { derivation } => format!("derived: {derivation}"),
    }
}

fn owner_text(owner: &CreatureContentOwnerJson) -> String {
    match owner {
        CreatureContentOwnerJson::Record { record_key } => format!("record {record_key}"),
        CreatureContentOwnerJson::Entity { entity_id } => format!("entity {entity_id}"),
        CreatureContentOwnerJson::Occurrence { occurrence_id } => {
            format!("occurrence {occurrence_id}")
        }
    }
}

fn embedded_target_text(target: &atlas_record::CreatureRelationshipTargetJson) -> String {
    match target {
        atlas_record::CreatureRelationshipTargetJson::Occurrence { occurrence_id } => {
            format!("occurrence {occurrence_id}")
        }
        atlas_record::CreatureRelationshipTargetJson::UnresolvedNestedSource { source_id } => {
            format!("unresolved nested source {source_id}")
        }
    }
}

fn integer_presence_text(value: &atlas_record::CreatureIntegerPresenceJson) -> String {
    match value {
        atlas_record::CreatureIntegerPresenceJson::Missing => "missing".to_string(),
        atlas_record::CreatureIntegerPresenceJson::Null => "null".to_string(),
        atlas_record::CreatureIntegerPresenceJson::Value(value) => format!("value {value:+}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_domain::{RecordKey, RecordKind};
    use atlas_record::{
        AtlasRecord, FoundryDocumentType, FoundryRecordInfo, FoundryRecordType,
        RecordClassification, RecordIdentity, RecordProvenance, RetrievedRecord,
    };
    use atlas_search::{GraphContextEdge, GraphContextEdgeSource, GraphContextSection};

    #[test]
    fn text_provenance_exposes_typed_internal_identity_and_reference_context() {
        let record = super::super::render::tests::review_fixture(atlas_domain::DetailLevel::Full);
        let graph = graph_fixture();
        let data = provenance_data(&record, Some(&graph));
        let text = render_provenance(&data);
        assert!(text.contains("Fact sources"));
        assert!(text.contains("ID: dream-message-rank-5-first"));
        assert!(text.contains("Prepared slot locator: slot5:0"));
        assert!(text.contains("Entry occurrence ID: innate-spells"));
        assert!(text.contains("Source item ID: dream-message-source-item"));
        assert!(text.contains("Owner: occurrence change-shape"));
        assert!(text.contains("Counterpart lookup: verified"));
        assert!(text.contains("outgoing: bestiary:Night-Hag -> spells:Dream-Message"));
        assert!(text.contains("Label: Dream Message"));
        assert!(text.contains("Relation kind: reference"));
        assert!(text.contains("Unmodeled skills"));
        assert!(text.contains("synthetic-review-skill"));
        assert!(text.contains("Base: value +17"));
        assert!(text.contains("Availability evidence"));
        assert!(text.contains("Component ID: focus-resource"));
        assert!(text.contains("Source value: {\"value\":0}"));
        assert!(!text.contains("source_json"));
        write_review_sample("night-hag-injected-provenance.txt", &text);
    }

    #[test]
    fn json_provenance_is_named_typed_data_without_raw_source() {
        let record = super::super::render::tests::review_fixture(atlas_domain::DetailLevel::Full);
        let graph = graph_fixture();
        let data = provenance_data(&record, Some(&graph));
        let json = serde_json::to_value(&data).expect("provenance JSON");
        assert_eq!(json["occurrences"][0]["order"], 0);
        assert!(json["occurrences"].as_array().is_some_and(|rows| {
            rows.iter().any(|row| {
                row["id"] == "dream-message-rank-5-first"
                    && row["target_entity_id"] == "dream-message-entity"
                    && row["parent_entry_id"] == "innate-spells"
                    && row["prepared_slot_locator"] == "slot5:0"
            })
        }));
        assert_eq!(
            json["prepared_spells"][0]["entry_occurrence_id"],
            "innate-spells"
        );
        assert_eq!(
            json["prepared_spells"][0]["source_item_id"],
            "dream-message-source-item"
        );
        assert!(json["record_provenance"]["facts"].is_object());
        assert_eq!(json["references"]["lookup_performed"], true);
        assert_eq!(
            json["references"]["edges"][0]["to_record_key"],
            "spells:Dream-Message"
        );
        assert_eq!(
            json["references"]["edges"][0]["from_record_key"],
            "bestiary:Night-Hag"
        );
        assert_eq!(json["references"]["edges"][0]["source_kind"], "description");
        assert_eq!(json["references"]["edges"][0]["visibility"], "public");
        assert!(json["content"].as_array().is_some_and(|rows| {
            rows.iter().any(|row| {
                row["owner"]["owner_type"] == "occurrence"
                    && row["owner"]["occurrence_id"] == "change-shape"
            })
        }));
        assert!(json["edition"]["counterpart_lookup"]["counterparts"].is_array());
        assert_eq!(
            json["unmodeled_skills"][0]["authored_key"],
            "synthetic-review-skill"
        );
        assert_eq!(json["unmodeled_skills"][0]["base"]["state"], "value");
        assert_eq!(json["unmodeled_skills"][0]["base"]["value"], 17);
        assert_eq!(
            json["availability_evidence"][0]["field"],
            "resource_serialized_value"
        );
        assert_eq!(
            json["availability_evidence"][0]["component_id"],
            "focus-resource"
        );
        assert_eq!(
            json["availability_evidence"][0]["source_value"],
            "{\"value\":0}"
        );
        assert!(json.get("source_json").is_none());
        let envelope = serde_json::json!({"status": "ok", "data": &data});
        write_review_sample(
            "night-hag-injected-provenance.json",
            &format!(
                "{}\n",
                serde_json::to_string_pretty(&envelope).expect("provenance envelope")
            ),
        );
    }

    #[test]
    fn json_provenance_preserves_unmodeled_skill_missing_and_null() {
        for (base, expected_state) in [
            (
                atlas_record::CreatureIntegerPresenceJson::Missing,
                "missing",
            ),
            (atlas_record::CreatureIntegerPresenceJson::Null, "null"),
        ] {
            let mut record =
                super::super::render::tests::review_fixture(atlas_domain::DetailLevel::Full);
            let RecordPresentationJson::Creature { skills, .. } = &mut record.presentation else {
                panic!("creature fixture");
            };
            skills
                .as_mut()
                .and_then(|skills| skills.first_mut())
                .and_then(|skill| skill.unmodeled.as_mut())
                .expect("unmodeled fact")
                .base = base;
            let data = provenance_data(&record, None);
            let text = render_provenance(&data);
            assert!(text.contains(&format!("Base: {expected_state}")));
            let json = serde_json::to_value(data).expect("provenance JSON");
            assert_eq!(json["unmodeled_skills"][0]["base"]["state"], expected_state);
            assert!(json["unmodeled_skills"][0]["base"].get("value").is_none());
        }
    }

    fn graph_fixture() -> GraphContextResult {
        let seed = retrieved_record("bestiary:Night-Hag", "Night Hag");
        let spell = retrieved_record("spells:Dream-Message", "Dream Message");
        GraphContextResult {
            seed: seed.clone(),
            outgoing: GraphContextSection {
                records: vec![spell],
                edges: vec![GraphContextEdge {
                    from: seed.record.identity.key.clone(),
                    to: RecordKey::parse("spells:Dream-Message").expect("target key"),
                    display_text: Some("Dream Message".into()),
                    reference_text: "@UUID[Compendium.spells.Dream-Message]".into(),
                    source: GraphContextEdgeSource {
                        kind: "description".into(),
                        visibility: "public".into(),
                        relation_kind: "reference".into(),
                    },
                }],
                total_records: 1,
                total_edges: 1,
                truncated: false,
            },
            backlinks: GraphContextSection {
                records: Vec::new(),
                edges: Vec::new(),
                total_records: 0,
                total_edges: 0,
                truncated: false,
            },
        }
    }

    fn retrieved_record(key: &str, name: &str) -> RetrievedRecord {
        RetrievedRecord {
            record: AtlasRecord::new(
                RecordIdentity::new(RecordKey::parse(key).expect("record key"), name),
                RecordClassification::new(RecordKind::Rule),
                FoundryRecordInfo::new(
                    "Fixture",
                    FoundryDocumentType::Item,
                    FoundryRecordType::Action,
                ),
                RecordProvenance::new(format!("fixtures/{key}.json")),
            ),
            body: None,
            spell_children: Vec::new(),
        }
    }

    fn write_review_sample(name: &str, contents: &str) {
        let Some(dir) = std::env::var_os("ATLAS_CLI_REVIEW_SAMPLE_DIR") else {
            return;
        };
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).expect("create review sample directory");
        std::fs::write(dir.join(name), contents).expect("write review sample");
    }
}
