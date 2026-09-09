use std::collections::{BTreeMap, BTreeSet};

use atlas_domain::RecordKey;
use atlas_record::{
    AtlasRecord, ConsumableSpellChild, ContentIdentityStability, ContentOwner, ContentRole,
    FactValue, OwnedRichContent, RecordBody, RichLinkTarget, SpellStandaloneTarget,
};
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::{Queryable, Selectable, SelectableHelper, SqliteConnection};

use crate::artifact::canonical_json;
use crate::schema::{
    canonical_consumable_spell_children, canonical_creature_records, canonical_hazard_records,
    canonical_journal_records, canonical_roll_table_records, canonical_spell_records,
    record_content, reference_occurrences, spell_damage_types, spell_records, spell_traditions,
};
use crate::spell_query::{
    SpellDamageTypeProjection, SpellQueryProjection, SpellTraditionProjection,
};

use super::RecordLoadError;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = canonical_creature_records)]
#[diesel(check_for_backend(Sqlite))]
struct CanonicalCreatureRow {
    record_key: String,
    canonical_json: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = canonical_hazard_records)]
#[diesel(check_for_backend(Sqlite))]
struct CanonicalHazardRecordRow {
    record_key: String,
    canonical_json: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = canonical_spell_records)]
#[diesel(check_for_backend(Sqlite))]
struct CanonicalSpellRow {
    record_key: String,
    source_id: String,
    name: String,
    canonical_json: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = canonical_journal_records)]
#[diesel(check_for_backend(Sqlite))]
struct CanonicalJournalRow {
    record_key: String,
    source_id: String,
    name: String,
    canonical_json: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = canonical_roll_table_records)]
#[diesel(check_for_backend(Sqlite))]
struct CanonicalRollTableRow {
    record_key: String,
    source_id: String,
    name: String,
    canonical_json: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = canonical_consumable_spell_children)]
#[diesel(check_for_backend(Sqlite))]
struct CanonicalSpellChildRow {
    parent_record_key: String,
    child_id: String,
    authored_order: i64,
    standalone_target_record_key: Option<String>,
    canonical_json: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = spell_records)]
#[diesel(check_for_backend(Sqlite))]
struct SpellQueryRow {
    record_key: String,
    traditions_json: String,
    spell_kinds_json: String,
    range_text: Option<String>,
    range_value: Option<f64>,
    target_text: Option<String>,
    area_type: Option<String>,
    area_value: Option<f64>,
    save_type: Option<String>,
    sustained: bool,
    basic_save: Option<bool>,
    damage_types_json: String,
    rank: Option<i64>,
    range_kind: Option<String>,
    range_rule: Option<String>,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = spell_traditions)]
#[diesel(check_for_backend(Sqlite))]
struct SpellTraditionRow {
    record_key: String,
    authored_order: i64,
    tradition: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = spell_damage_types)]
#[diesel(check_for_backend(Sqlite))]
struct SpellDamageTypeRow {
    record_key: String,
    damage_key: String,
    damage_authored_order: i64,
    type_authored_order: i64,
    damage_type: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = record_content)]
#[diesel(check_for_backend(Sqlite))]
struct SpellContentRow {
    record_key: String,
    content_key: String,
    authored_order: i64,
    identity_stability: String,
    owner_kind: String,
    owner_record_key: Option<String>,
    owner_entity_id: Option<String>,
    owner_occurrence_id: Option<String>,
    owner_occurrence_authored_order: Option<i64>,
    role: String,
    origin_json: String,
    visibility: String,
    provenance_json: String,
    source_kind: String,
    contributes_to_search: bool,
    contributes_to_references: bool,
    label: Option<String>,
    content_json: String,
    content_hash: String,
    duplicate_status_json: String,
    diagnostics_json: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = reference_occurrences)]
#[diesel(check_for_backend(Sqlite))]
struct SpellReferenceRow {
    record_key: String,
    content_key: String,
    content_authored_order: i64,
    occurrence_ordinal: i64,
    owner_kind: String,
    owner_record_key: Option<String>,
    owner_entity_id: Option<String>,
    owner_occurrence_id: Option<String>,
    owner_occurrence_authored_order: Option<i64>,
    role: String,
    origin_json: String,
    visibility: String,
    provenance_json: String,
    target_kind: String,
    target_record_key: Option<String>,
    target_json: String,
    label: Option<String>,
    relation_kind: String,
}

pub(super) fn read_canonical_record_bodies(
    connection: &mut SqliteConnection,
) -> Result<Vec<RecordBody>, RecordLoadError> {
    let creature_rows = canonical_creature_records::table
        .select(CanonicalCreatureRow::as_select())
        .order(canonical_creature_records::record_key.asc())
        .load::<CanonicalCreatureRow>(connection)
        .map_err(query_failed)?;
    let hazard_rows = canonical_hazard_records::table
        .select(CanonicalHazardRecordRow::as_select())
        .order(canonical_hazard_records::record_key.asc())
        .load::<CanonicalHazardRecordRow>(connection)
        .map_err(query_failed)?;
    let spell_rows = canonical_spell_records::table
        .select(CanonicalSpellRow::as_select())
        .order(canonical_spell_records::record_key.asc())
        .load::<CanonicalSpellRow>(connection)
        .map_err(query_failed)?;
    let journal_rows = canonical_journal_records::table
        .select(CanonicalJournalRow::as_select())
        .order(canonical_journal_records::record_key.asc())
        .load::<CanonicalJournalRow>(connection)
        .map_err(query_failed)?;
    let table_rows = canonical_roll_table_records::table
        .select(CanonicalRollTableRow::as_select())
        .order(canonical_roll_table_records::record_key.asc())
        .load::<CanonicalRollTableRow>(connection)
        .map_err(query_failed)?;
    decode_body_rows(
        creature_rows,
        hazard_rows,
        spell_rows,
        journal_rows,
        table_rows,
    )
}

pub(super) fn read_canonical_record_bodies_by_key(
    connection: &mut SqliteConnection,
    keys: &[RecordKey],
) -> Result<Vec<RecordBody>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(Vec::new());
    }
    let keys = key_strings(keys);
    let creature_rows = canonical_creature_records::table
        .filter(canonical_creature_records::record_key.eq_any(&keys))
        .select(CanonicalCreatureRow::as_select())
        .order(canonical_creature_records::record_key.asc())
        .load::<CanonicalCreatureRow>(connection)
        .map_err(query_failed)?;
    let hazard_rows = canonical_hazard_records::table
        .filter(canonical_hazard_records::record_key.eq_any(&keys))
        .select(CanonicalHazardRecordRow::as_select())
        .order(canonical_hazard_records::record_key.asc())
        .load::<CanonicalHazardRecordRow>(connection)
        .map_err(query_failed)?;
    let spell_rows = canonical_spell_records::table
        .filter(canonical_spell_records::record_key.eq_any(&keys))
        .select(CanonicalSpellRow::as_select())
        .order(canonical_spell_records::record_key.asc())
        .load::<CanonicalSpellRow>(connection)
        .map_err(query_failed)?;
    let journal_rows = canonical_journal_records::table
        .filter(canonical_journal_records::record_key.eq_any(&keys))
        .select(CanonicalJournalRow::as_select())
        .order(canonical_journal_records::record_key.asc())
        .load::<CanonicalJournalRow>(connection)
        .map_err(query_failed)?;
    let table_rows = canonical_roll_table_records::table
        .filter(canonical_roll_table_records::record_key.eq_any(&keys))
        .select(CanonicalRollTableRow::as_select())
        .order(canonical_roll_table_records::record_key.asc())
        .load::<CanonicalRollTableRow>(connection)
        .map_err(query_failed)?;
    decode_body_rows(
        creature_rows,
        hazard_rows,
        spell_rows,
        journal_rows,
        table_rows,
    )
}

pub(super) fn read_spell_children(
    connection: &mut SqliteConnection,
) -> Result<Vec<ConsumableSpellChild>, RecordLoadError> {
    let rows = canonical_consumable_spell_children::table
        .select(CanonicalSpellChildRow::as_select())
        .order((
            canonical_consumable_spell_children::parent_record_key.asc(),
            canonical_consumable_spell_children::authored_order.asc(),
        ))
        .load::<CanonicalSpellChildRow>(connection)
        .map_err(query_failed)?;
    let children = decode_child_rows(rows)?;
    validate_resolved_spell_targets(connection, &children)?;
    Ok(children)
}

pub(super) fn read_spell_children_by_parent_key(
    connection: &mut SqliteConnection,
    keys: &[RecordKey],
) -> Result<Vec<ConsumableSpellChild>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(Vec::new());
    }
    let rows = canonical_consumable_spell_children::table
        .filter(canonical_consumable_spell_children::parent_record_key.eq_any(key_strings(keys)))
        .select(CanonicalSpellChildRow::as_select())
        .order((
            canonical_consumable_spell_children::parent_record_key.asc(),
            canonical_consumable_spell_children::authored_order.asc(),
        ))
        .load::<CanonicalSpellChildRow>(connection)
        .map_err(query_failed)?;
    let children = decode_child_rows(rows)?;
    validate_resolved_spell_targets(connection, &children)?;
    Ok(children)
}

pub(super) fn bodies_by_key(
    bodies: Vec<RecordBody>,
) -> Result<BTreeMap<RecordKey, RecordBody>, RecordLoadError> {
    let mut by_key = BTreeMap::new();
    for body in bodies {
        let key = body.record_key().clone();
        if by_key.insert(key.clone(), body).is_some() {
            return Err(RecordLoadError::InvalidData(format!(
                "canonical record `{key}` has more than one body row"
            )));
        }
    }
    Ok(by_key)
}

pub(super) fn spell_children_by_parent(
    children: Vec<ConsumableSpellChild>,
) -> Result<BTreeMap<RecordKey, Vec<ConsumableSpellChild>>, RecordLoadError> {
    let mut by_parent = BTreeMap::<RecordKey, Vec<ConsumableSpellChild>>::new();
    let mut identities = BTreeSet::new();
    for child in children {
        let identity = (child.parent_record_key.clone(), child.child_id.clone());
        if !identities.insert(identity) {
            return Err(RecordLoadError::InvalidData(format!(
                "consumable `{}` has duplicate spell child `{}`",
                child.parent_record_key,
                child.child_id.as_str()
            )));
        }
        by_parent
            .entry(child.parent_record_key.clone())
            .or_default()
            .push(child);
    }
    for children in by_parent.values_mut() {
        children.sort_by_key(|child| child.authored_order);
        for (expected, child) in children.iter().enumerate() {
            if usize::try_from(child.authored_order) != Ok(expected) {
                return Err(RecordLoadError::InvalidData(format!(
                    "consumable `{}` spell child `{}` has non-contiguous authored order {}",
                    child.parent_record_key,
                    child.child_id.as_str(),
                    child.authored_order
                )));
            }
        }
    }
    Ok(by_parent)
}

fn validate_resolved_spell_targets(
    connection: &mut SqliteConnection,
    children: &[ConsumableSpellChild],
) -> Result<(), RecordLoadError> {
    let target_keys = children
        .iter()
        .filter_map(|child| match &child.standalone_target {
            FactValue::Value(SpellStandaloneTarget::Resolved(key)) => Some(key.clone()),
            FactValue::Missing
            | FactValue::Null
            | FactValue::Value(SpellStandaloneTarget::Unresolved(_)) => None,
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if target_keys.is_empty() {
        return Ok(());
    }
    let records = super::rows::read_record_rows_by_keys(connection, &target_keys)?
        .into_iter()
        .map(|record| (record.identity.key.clone(), record))
        .collect::<BTreeMap<RecordKey, AtlasRecord>>();
    let bodies = bodies_by_key(read_canonical_record_bodies_by_key(
        connection,
        &target_keys,
    )?)?;
    for key in target_keys {
        let Some(record) = records.get(&key) else {
            return Err(invalid_resolved_spell_target(&key));
        };
        let Some(RecordBody::Spell(spell)) = bodies.get(&key) else {
            return Err(invalid_resolved_spell_target(&key));
        };
        if record.classification.kind != atlas_domain::RecordKind::Spell
            || record.foundry.record_type != atlas_record::FoundryRecordType::Spell
            || super::validate_spell_body_owner(record, spell).is_err()
        {
            return Err(invalid_resolved_spell_target(&key));
        }
    }
    Ok(())
}

fn invalid_resolved_spell_target(key: &RecordKey) -> RecordLoadError {
    RecordLoadError::InvalidData(format!(
        "resolved consumable spell target `{key}` is not a canonical spell record"
    ))
}

pub(super) fn reconcile_spell_query_projections(
    connection: &mut SqliteConnection,
    bodies: &BTreeMap<RecordKey, RecordBody>,
    requested_keys: Option<&[RecordKey]>,
) -> Result<(), RecordLoadError> {
    if requested_keys.is_some_and(<[RecordKey]>::is_empty) {
        return Ok(());
    }
    let requested = requested_keys.map(key_strings);
    let rows = if let Some(keys) = &requested {
        spell_records::table
            .filter(spell_records::record_key.eq_any(keys))
            .select(SpellQueryRow::as_select())
            .load::<SpellQueryRow>(connection)
    } else {
        spell_records::table
            .select(SpellQueryRow::as_select())
            .load::<SpellQueryRow>(connection)
    }
    .map_err(query_failed)?;
    let mut query_rows = BTreeMap::new();
    for row in rows {
        if query_rows.insert(row.record_key.clone(), row).is_some() {
            return Err(RecordLoadError::InvalidData(
                "duplicate spell query projection".to_string(),
            ));
        }
    }
    let tradition_rows = load_traditions(connection, requested.as_deref())?;
    let damage_rows = load_damage_types(connection, requested.as_deref())?;
    let traditions = group_traditions(tradition_rows);
    let damage_types = group_damage_types(damage_rows);
    for (key, body) in bodies {
        let RecordBody::Spell(spell) = body else {
            continue;
        };
        let key_string = key.to_string();
        let row = query_rows.remove(&key_string).ok_or_else(|| {
            RecordLoadError::InvalidData(format!(
                "canonical spell `{key}` is missing its required spell query projection"
            ))
        })?;
        let expected =
            SpellQueryProjection::from_spell(spell).map_err(RecordLoadError::InvalidData)?;
        let compact_traditions =
            decode_json_list(&row.traditions_json, &key_string, "traditions_json")?;
        let compact_damage =
            decode_json_list(&row.damage_types_json, &key_string, "damage_types_json")?;
        let spell_kinds = decode_json_list(&row.spell_kinds_json, &key_string, "spell_kinds_json")?;
        let actual_traditions = traditions.get(&key_string).cloned().unwrap_or_default();
        let actual_damage = damage_types.get(&key_string).cloned().unwrap_or_default();
        if row.rank != expected.rank
            || compact_traditions != expected.tradition_values()
            || actual_traditions.as_slice() != expected.traditions.as_slice()
            || spell_kinds != expected.spell_kinds
            || row.range_text != expected.range_text
            || row.range_value != expected.range_value
            || row.range_kind != expected.range_kind
            || row.range_rule != expected.range_rule
            || row.target_text != expected.target_text
            || row.area_type != expected.area_type
            || row.area_value != expected.area_value
            || row.save_type != expected.save_type
            || row.sustained != expected.sustained
            || row.basic_save != expected.basic_save
            || compact_damage != expected.damage_type_values()
            || actual_damage.as_slice() != expected.damage_types.as_slice()
        {
            return Err(RecordLoadError::InvalidData(format!(
                "spell query projection for `{key}` does not match its canonical spell body"
            )));
        }
    }
    if let Some((key, _)) = query_rows.into_iter().next() {
        return Err(RecordLoadError::InvalidData(format!(
            "spell query projection `{key}` has no canonical spell body"
        )));
    }
    Ok(())
}

pub(super) fn reconcile_canonical_owned_projections(
    connection: &mut SqliteConnection,
    bodies: &BTreeMap<RecordKey, RecordBody>,
    children: &BTreeMap<RecordKey, Vec<ConsumableSpellChild>>,
) -> Result<(), RecordLoadError> {
    let mut keys = bodies
        .iter()
        .filter(|(_, body)| {
            matches!(
                body,
                RecordBody::Spell(_) | RecordBody::Journal(_) | RecordBody::RollTable(_)
            )
        })
        .map(|(key, _)| key.to_string())
        .collect::<BTreeSet<_>>();
    keys.extend(children.keys().map(ToString::to_string));
    if keys.is_empty() {
        return Ok(());
    }
    let keys = keys.into_iter().collect::<Vec<_>>();
    let content_rows = record_content::table
        .filter(record_content::record_key.eq_any(&keys))
        .select(SpellContentRow::as_select())
        .load::<SpellContentRow>(connection)
        .map_err(query_failed)?;
    let reference_rows = reference_occurrences::table
        .filter(reference_occurrences::record_key.eq_any(&keys))
        .select(SpellReferenceRow::as_select())
        .load::<SpellReferenceRow>(connection)
        .map_err(query_failed)?;

    for (key, body) in bodies {
        if let RecordBody::Spell(spell) = body {
            reconcile_owned_content(
                &key.to_string(),
                &spell.definition.content,
                &content_rows,
                &reference_rows,
            )?;
        } else if let RecordBody::Journal(journal) = body {
            reconcile_owned_content(
                &key.to_string(),
                &journal.content,
                &content_rows,
                &reference_rows,
            )?;
        } else if let RecordBody::RollTable(table) = body {
            reconcile_owned_content(
                &key.to_string(),
                &table.content,
                &content_rows,
                &reference_rows,
            )?;
        }
    }
    for (key, values) in children {
        for child in values {
            reconcile_owned_content(
                &key.to_string(),
                &child.definition.content,
                &content_rows,
                &reference_rows,
            )?;
        }
    }
    Ok(())
}

fn reconcile_owned_content(
    record_key: &str,
    owned: &OwnedRichContent,
    rows: &[SpellContentRow],
    references: &[SpellReferenceRow],
) -> Result<(), RecordLoadError> {
    for document in &owned.documents {
        let content_key = document.id.content_key.as_str();
        let authored_order = i64::from(document.authored_order);
        let matching = rows
            .iter()
            .filter(|row| {
                row.record_key == record_key
                    && row.content_key == content_key
                    && row.authored_order == authored_order
            })
            .collect::<Vec<_>>();
        if matching.len() != 1 {
            return Err(RecordLoadError::InvalidData(format!(
                "owned spell content `{record_key}:{content_key}:{authored_order}` has {} relational rows",
                matching.len()
            )));
        }
        let row = matching[0];
        let content_json =
            canonical_json::encode(&document.document).map_err(RecordLoadError::InvalidData)?;
        let origin_json =
            canonical_json::encode(&document.origin).map_err(RecordLoadError::InvalidData)?;
        let provenance_json =
            canonical_json::encode(&document.provenance).map_err(RecordLoadError::InvalidData)?;
        let duplicate_status_json = canonical_json::encode(&document.duplicate_status)
            .map_err(RecordLoadError::InvalidData)?;
        let diagnostics_json =
            canonical_json::encode(&document.diagnostics).map_err(RecordLoadError::InvalidData)?;
        let expected_owner_kind = match &document.owner {
            ContentOwner::Record(key) if key.to_string() == record_key => "record",
            ContentOwner::Child(locator) if locator.parent.to_string() == record_key => "child",
            _ => {
                return Err(RecordLoadError::InvalidData(format!(
                    "owned canonical content `{record_key}:{content_key}:{authored_order}` has an invalid owner"
                )));
            }
        };
        if row.identity_stability != content_stability(document.identity_stability)
            || row.owner_kind != expected_owner_kind
            || row.owner_record_key.as_deref() != Some(record_key)
            || row.owner_entity_id.is_some()
            || row.owner_occurrence_id.is_some()
            || row.owner_occurrence_authored_order.is_some()
            || row.role != content_role(document.role)
            || row.origin_json != origin_json
            || row.visibility != document.visibility.as_str()
            || row.provenance_json != provenance_json
            || row.source_kind != document.source_kind.as_str()
            || row.contributes_to_search != document.source_kind.default_contributes_to_search()
            || row.contributes_to_references
                != document
                    .source_kind
                    .default_contributes_to_reference_occurrences()
            || row.label != document.label
            || row.content_json != content_json
            || row.content_hash != document.content_hash.as_str()
            || row.duplicate_status_json != duplicate_status_json
            || row.diagnostics_json != diagnostics_json
        {
            return Err(RecordLoadError::InvalidData(format!(
                "owned spell content projection `{record_key}:{content_key}:{authored_order}` does not match its canonical owner"
            )));
        }

        let actual_references = references
            .iter()
            .filter(|row| {
                row.record_key == record_key
                    && row.content_key == content_key
                    && row.content_authored_order == authored_order
            })
            .collect::<Vec<_>>();
        if actual_references.len() != document.reference_occurrences.len() {
            return Err(RecordLoadError::InvalidData(format!(
                "spell reference projection `{record_key}:{content_key}:{authored_order}` has the wrong occurrence count"
            )));
        }
        for expected in &document.reference_occurrences {
            let ordinal = i64::from(expected.ordinal);
            let Some(actual) = actual_references
                .iter()
                .find(|row| row.occurrence_ordinal == ordinal)
            else {
                return Err(RecordLoadError::InvalidData(format!(
                    "spell reference projection `{record_key}:{content_key}:{authored_order}` is missing ordinal {ordinal}"
                )));
            };
            let target_json = serde_json::to_string(&expected.target)
                .map_err(|error| RecordLoadError::InvalidData(error.to_string()))?;
            let origin_json =
                canonical_json::encode(&expected.origin).map_err(RecordLoadError::InvalidData)?;
            let provenance_json = canonical_json::encode(&expected.provenance)
                .map_err(RecordLoadError::InvalidData)?;
            if actual.owner_kind != expected_owner_kind
                || actual.owner_record_key.as_deref() != Some(record_key)
                || actual.owner_entity_id.is_some()
                || actual.owner_occurrence_id.is_some()
                || actual.owner_occurrence_authored_order.is_some()
                || actual.role != content_role(expected.role)
                || actual.origin_json != origin_json
                || actual.visibility != expected.visibility.as_str()
                || actual.provenance_json != provenance_json
                || actual.target_kind != target_kind(&expected.target)
                || actual.target_record_key != target_record(&expected.target)
                || actual.target_json != target_json
                || actual.label != expected.label
                || actual.relation_kind != expected.relation_kind.as_str()
            {
                return Err(RecordLoadError::InvalidData(format!(
                    "spell reference projection `{record_key}:{content_key}:{authored_order}:{ordinal}` does not match its canonical owner"
                )));
            }
        }
    }
    Ok(())
}

fn content_stability(value: ContentIdentityStability) -> &'static str {
    match value {
        ContentIdentityStability::StableSourceIdentity => "stable_source_identity",
        ContentIdentityStability::UnstableAuthoredOrdinal => "unstable_authored_ordinal",
    }
}

fn content_role(value: ContentRole) -> &'static str {
    match value {
        ContentRole::PrimaryDescription => "primary_description",
        ContentRole::Summary => "summary",
        ContentRole::SupplementalRules => "supplemental_rules",
        ContentRole::EmbeddedCapability => "embedded_capability",
        ContentRole::JournalPage => "journal_page",
        ContentRole::TableResult => "table_result",
        ContentRole::GeneratedNarrative => "generated_narrative",
        ContentRole::Provenance => "provenance",
    }
}

fn target_kind(target: &RichLinkTarget) -> &'static str {
    match target {
        RichLinkTarget::Record { .. } => "record",
        RichLinkTarget::RecordChild { .. } => "record_child",
        RichLinkTarget::LocalContent { .. } => "local_content",
        RichLinkTarget::External { .. } => "external",
        RichLinkTarget::Unresolved { .. } => "unresolved",
    }
}

fn target_record(target: &RichLinkTarget) -> Option<String> {
    target.record_key().map(ToString::to_string)
}

fn load_traditions(
    connection: &mut SqliteConnection,
    keys: Option<&[String]>,
) -> Result<Vec<SpellTraditionRow>, RecordLoadError> {
    let query = spell_traditions::table
        .select(SpellTraditionRow::as_select())
        .order((
            spell_traditions::record_key.asc(),
            spell_traditions::authored_order.asc(),
        ));
    if let Some(keys) = keys {
        query
            .filter(spell_traditions::record_key.eq_any(keys))
            .load(connection)
            .map_err(query_failed)
    } else {
        query.load(connection).map_err(query_failed)
    }
}

fn load_damage_types(
    connection: &mut SqliteConnection,
    keys: Option<&[String]>,
) -> Result<Vec<SpellDamageTypeRow>, RecordLoadError> {
    let query = spell_damage_types::table
        .select(SpellDamageTypeRow::as_select())
        .order((
            spell_damage_types::record_key.asc(),
            spell_damage_types::damage_authored_order.asc(),
            spell_damage_types::type_authored_order.asc(),
        ));
    if let Some(keys) = keys {
        query
            .filter(spell_damage_types::record_key.eq_any(keys))
            .load(connection)
            .map_err(query_failed)
    } else {
        query.load(connection).map_err(query_failed)
    }
}

fn decode_body_rows(
    creature_rows: Vec<CanonicalCreatureRow>,
    hazard_rows: Vec<CanonicalHazardRecordRow>,
    spell_rows: Vec<CanonicalSpellRow>,
    journal_rows: Vec<CanonicalJournalRow>,
    table_rows: Vec<CanonicalRollTableRow>,
) -> Result<Vec<RecordBody>, RecordLoadError> {
    let mut bodies = Vec::with_capacity(
        creature_rows.len()
            + hazard_rows.len()
            + spell_rows.len()
            + journal_rows.len()
            + table_rows.len(),
    );
    for row in creature_rows {
        let path = format!(
            "canonical_creature_records[{}].canonical_json",
            row.record_key
        );
        let body = canonical_json::decode::<RecordBody>(&row.canonical_json, &path)
            .map_err(RecordLoadError::InvalidData)?;
        let RecordBody::Creature(creature) = &body else {
            return Err(RecordLoadError::InvalidData(format!(
                "{path}: canonical creature table contains a non-creature body"
            )));
        };
        require_key(&path, &row.record_key, &creature.identity.record_key)?;
        bodies.push(body);
    }
    for row in hazard_rows {
        let path = format!(
            "canonical_hazard_records[{}].canonical_json",
            row.record_key
        );
        let body = canonical_json::decode::<RecordBody>(&row.canonical_json, &path)
            .map_err(RecordLoadError::InvalidData)?;
        let RecordBody::Hazard(hazard) = &body else {
            return Err(RecordLoadError::InvalidData(format!(
                "{path}: canonical hazard table contains a non-hazard body"
            )));
        };
        require_key(&path, &row.record_key, &hazard.identity.record_key)?;
        bodies.push(body);
    }
    for row in spell_rows {
        let path = format!("canonical_spell_records[{}].canonical_json", row.record_key);
        let body = canonical_json::decode::<RecordBody>(&row.canonical_json, &path)
            .map_err(RecordLoadError::InvalidData)?;
        let RecordBody::Spell(spell) = &body else {
            return Err(RecordLoadError::InvalidData(format!(
                "{path}: canonical spell table contains a non-spell body"
            )));
        };
        require_key(&path, &row.record_key, &spell.identity.record_key)?;
        if row.source_id != spell.identity.source_id.as_str() || row.name != spell.identity.name {
            return Err(RecordLoadError::InvalidData(format!(
                "{path}: indexed spell identity columns do not match the canonical spell body"
            )));
        }
        bodies.push(body);
    }
    for row in journal_rows {
        let path = format!(
            "canonical_journal_records[{}].canonical_json",
            row.record_key
        );
        let body = canonical_json::decode::<RecordBody>(&row.canonical_json, &path)
            .map_err(RecordLoadError::InvalidData)?;
        let RecordBody::Journal(journal) = &body else {
            return Err(RecordLoadError::InvalidData(format!(
                "{path}: canonical journal table contains a non-journal body"
            )));
        };
        require_key(&path, &row.record_key, &journal.identity.record_key)?;
        if row.source_id != journal.identity.source_id.as_str() || row.name != journal.identity.name
        {
            return Err(RecordLoadError::InvalidData(format!(
                "{path}: indexed journal identity columns do not match the canonical body"
            )));
        }
        bodies.push(body);
    }
    for row in table_rows {
        let path = format!(
            "canonical_roll_table_records[{}].canonical_json",
            row.record_key
        );
        let body = canonical_json::decode::<RecordBody>(&row.canonical_json, &path)
            .map_err(RecordLoadError::InvalidData)?;
        let RecordBody::RollTable(table) = &body else {
            return Err(RecordLoadError::InvalidData(format!(
                "{path}: canonical roll-table table contains a non-roll-table body"
            )));
        };
        require_key(&path, &row.record_key, &table.identity.record_key)?;
        if row.source_id != table.identity.source_id.as_str() || row.name != table.identity.name {
            return Err(RecordLoadError::InvalidData(format!(
                "{path}: indexed roll-table identity columns do not match the canonical body"
            )));
        }
        bodies.push(body);
    }
    bodies.sort_by(|left, right| left.record_key().cmp(right.record_key()));
    Ok(bodies)
}

fn decode_child_rows(
    rows: Vec<CanonicalSpellChildRow>,
) -> Result<Vec<ConsumableSpellChild>, RecordLoadError> {
    rows.into_iter()
        .map(|row| {
            let path = format!(
                "canonical_consumable_spell_children[{}:{}].canonical_json",
                row.parent_record_key, row.child_id
            );
            let child = canonical_json::decode::<ConsumableSpellChild>(&row.canonical_json, &path)
                .map_err(RecordLoadError::InvalidData)?;
            if child.parent_record_key.to_string() != row.parent_record_key
                || child.child_id.as_str() != row.child_id
                || i64::from(child.authored_order) != row.authored_order
            {
                return Err(RecordLoadError::InvalidData(format!(
                    "{path}: indexed child identity/order columns do not match the canonical child"
                )));
            }
            let target = match &child.standalone_target {
                FactValue::Value(SpellStandaloneTarget::Resolved(key)) => Some(key.to_string()),
                FactValue::Missing
                | FactValue::Null
                | FactValue::Value(SpellStandaloneTarget::Unresolved(_)) => None,
            };
            if target != row.standalone_target_record_key {
                return Err(RecordLoadError::InvalidData(format!(
                    "{path}: indexed standalone target does not match the canonical child"
                )));
            }
            Ok(child)
        })
        .collect()
}

fn require_key(path: &str, row_key: &str, body_key: &RecordKey) -> Result<(), RecordLoadError> {
    if body_key.to_string() != row_key {
        return Err(RecordLoadError::InvalidData(format!(
            "{path}: hydrated record key `{body_key}` does not match row key `{row_key}`"
        )));
    }
    Ok(())
}

fn group_traditions(
    rows: Vec<SpellTraditionRow>,
) -> BTreeMap<String, Vec<SpellTraditionProjection>> {
    let mut grouped = BTreeMap::<String, Vec<SpellTraditionProjection>>::new();
    for row in rows {
        grouped
            .entry(row.record_key)
            .or_default()
            .push(SpellTraditionProjection {
                authored_order: row.authored_order,
                tradition: row.tradition,
            });
    }
    grouped
}

fn group_damage_types(
    rows: Vec<SpellDamageTypeRow>,
) -> BTreeMap<String, Vec<SpellDamageTypeProjection>> {
    let mut grouped = BTreeMap::<String, Vec<SpellDamageTypeProjection>>::new();
    for row in rows {
        grouped
            .entry(row.record_key)
            .or_default()
            .push(SpellDamageTypeProjection {
                damage_key: row.damage_key,
                damage_authored_order: row.damage_authored_order,
                type_authored_order: row.type_authored_order,
                damage_type: row.damage_type,
            });
    }
    grouped
}

fn decode_json_list(
    value: &str,
    record_key: &str,
    column: &str,
) -> Result<Vec<String>, RecordLoadError> {
    serde_json::from_str(value).map_err(|error| {
        RecordLoadError::InvalidData(format!(
            "spell_records[{record_key}].{column}: invalid JSON: {error}"
        ))
    })
}

fn key_strings(keys: &[RecordKey]) -> Vec<String> {
    keys.iter().map(ToString::to_string).collect()
}

fn query_failed(error: diesel::result::Error) -> RecordLoadError {
    RecordLoadError::QueryFailed(error.to_string())
}
