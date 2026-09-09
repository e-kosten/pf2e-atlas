use std::collections::{BTreeMap, BTreeSet};

use atlas_record::{
    FactValue, HazardOccurrenceIdentityStability, HazardRelationshipKind, HazardRelationshipTarget,
    MetricValue, RecordBody, project_creature_facts, project_hazard_facts,
};
use rusqlite::Connection;
use rusqlite::types::ValueRef;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::artifact::canonical_json;
use crate::{ArtifactValidationDiagnostic, ArtifactValidationFamily, IndexValidationError};

use super::artifact_validation_diagnostic;

#[derive(Default)]
struct ProjectionRows {
    tables: BTreeMap<&'static str, BTreeMap<String, BTreeSet<[u8; 32]>>>,
}

impl ProjectionRows {
    fn load(connection: &Connection) -> Result<Self, IndexValidationError> {
        let mut rows = Self::default();
        for &(table, columns) in PROJECTION_TABLES {
            rows.load_table(connection, table, columns)?;
        }
        Ok(rows)
    }

    fn load_table(
        &mut self,
        connection: &Connection,
        table: &'static str,
        columns: &str,
    ) -> Result<(), IndexValidationError> {
        let sql = format!("SELECT {columns} FROM {table}");
        let mut statement = connection.prepare(&sql).map_err(query_failed)?;
        let column_count = statement.column_count();
        let values = statement
            .query_map([], |row| {
                let record = row.get::<_, String>(0)?;
                let mut values = Vec::with_capacity(column_count);
                for index in 0..column_count {
                    values.push(sql_value(row.get_ref(index)?));
                }
                Ok((record, digest_json(&Value::Array(values))))
            })
            .map_err(query_failed)?;
        for value in values {
            let (record, value) = value.map_err(query_failed)?;
            self.tables
                .entry(table)
                .or_default()
                .entry(record)
                .or_default()
                .insert(value);
        }
        Ok(())
    }

    fn set(&self, table: &'static str, record: &str) -> BTreeSet<[u8; 32]> {
        self.tables
            .get(table)
            .and_then(|rows| rows.get(record))
            .cloned()
            .unwrap_or_default()
    }
}

const PROJECTION_TABLES: &[(&str, &str)] = &[
    (
        "canonical_creature_resources",
        "record_key,resource_id,authored_order,resource_kind,resource_json",
    ),
    (
        "canonical_creature_entities",
        "record_key,entity_id,family,label,source_identity_json",
    ),
    (
        "canonical_creature_occurrences",
        "record_key,occurrence_id,identity_stability,family,authored_order,source_sort_json,source_folder_json,source_identity_json,parent_kind,parent_occurrence_id,parent_occurrence_authored_order,target_kind,target_record_key,target_entity_id,context_json,capability_json,deltas_json",
    ),
    (
        "canonical_creature_relationships",
        "record_key,relationship_order,source_occurrence_id,source_occurrence_authored_order,relationship_kind,target_kind,target_occurrence_id,target_occurrence_authored_order,target_source_id,source_path,contextual_label_json,lifecycle_json,execution",
    ),
    (
        "canonical_hazard_entities",
        "record_key,entity_id,family,label,image_json,source_identity_json,capability_json",
    ),
    (
        "canonical_hazard_occurrences",
        "record_key,occurrence_id,entity_id,identity_stability,family,authored_order,source_sort_json,source_folder_json,source_ordinal,contextual_label_json",
    ),
    (
        "canonical_hazard_relationships",
        "record_key,relationship_id,authored_order,source_occurrence_id,source_occurrence_authored_order,relationship_kind,target_kind,target_entity_id,target_occurrence_id,target_occurrence_authored_order",
    ),
    (
        "record_content",
        "record_key,content_key,authored_order,identity_stability,owner_kind,owner_record_key,owner_entity_id,owner_occurrence_id,owner_occurrence_authored_order,owner_hazard_entity_id,owner_hazard_occurrence_id,owner_hazard_occurrence_authored_order,role,origin_json,visibility,provenance_json,source_kind,contributes_to_search,contributes_to_references,label,content_json,content_hash,duplicate_status_json,diagnostics_json",
    ),
    (
        "reference_occurrences",
        "record_key,content_key,content_authored_order,occurrence_ordinal,owner_kind,owner_record_key,owner_entity_id,owner_occurrence_id,owner_occurrence_authored_order,owner_hazard_entity_id,owner_hazard_occurrence_id,owner_hazard_occurrence_authored_order,role,origin_json,visibility,provenance_json,target_kind,target_record_key,target_json,label,relation_kind",
    ),
    (
        "record_content_exclusions",
        "record_key,content_key,relative_source_path,label,reason",
    ),
    (
        "record_metrics",
        "record_key,ordinal,metric_domain,metric_key,value_type,number_value,text_value,bool_value",
    ),
];

fn sql_value(value: ValueRef<'_>) -> Value {
    match value {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(value) => value.into(),
        ValueRef::Real(value) => serde_json::Number::from_f64(value)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        ValueRef::Text(value) => String::from_utf8_lossy(value).into_owned().into(),
        ValueRef::Blob(_) => Value::String("<blob>".to_string()),
    }
}

fn digest_json(value: &Value) -> [u8; 32] {
    Sha256::digest(value.to_string().as_bytes()).into()
}

pub(crate) fn validate_canonical_records(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    validate_enums(connection, diagnostics)?;
    if !diagnostics.is_empty() {
        return Ok(());
    }
    validate_canonical_side_row_ownership(connection, diagnostics)?;
    if !diagnostics.is_empty() {
        return Ok(());
    }
    validate_record_mechanics(connection, diagnostics)?;
    if !diagnostics.is_empty() {
        return Ok(());
    }
    let projections = ProjectionRows::load(connection)?;
    let expected_keys = text_column(
        connection,
        "SELECT record_key FROM records WHERE foundry_record_type = 'npc' ORDER BY record_key",
    )?;
    let mut statement = connection.prepare("SELECT record_key, source_id, name, family, canonical_json FROM canonical_creature_records ORDER BY record_key").map_err(query_failed)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(query_failed)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(query_failed)?;
    let actual_keys = rows.iter().map(|row| row.0.clone()).collect::<Vec<_>>();
    if actual_keys != expected_keys {
        mismatch(
            diagnostics,
            "canonical creature coverage diverges from source NPC records",
            "canonical_creature_records.coverage",
            expected_keys.len().to_string(),
            actual_keys.len().to_string(),
        );
    }
    for (record_key, source_id, name, family, json) in rows {
        let diagnostic_count = diagnostics.len();
        let path = format!("canonical_creature_records[{record_key}].canonical_json");
        let body = match canonical_json::decode::<RecordBody>(&json, &path) {
            Ok(body) if canonical_json::encode(&body).is_ok_and(|encoded| encoded == json) => body,
            Ok(_) => {
                invalid(
                    diagnostics,
                    "typed JSON is not in canonical deterministic form",
                    &path,
                );
                continue;
            }
            Err(error) => {
                invalid(diagnostics, &error, &path);
                continue;
            }
        };
        let RecordBody::Creature(creature) = body else {
            invalid(
                diagnostics,
                "canonical creature row contains a non-creature body",
                &path,
            );
            continue;
        };
        if creature.identity.record_key.to_string() != record_key
            || creature.identity.source_id.as_str() != source_id
            || creature.identity.name != name
            || family != "npc"
        {
            mismatch(
                diagnostics,
                "canonical record identity diverges from its relational row",
                &record_key,
                "hydrated identity".to_string(),
                "relational identity".to_string(),
            );
        }
        validate_unique_ids(&creature, diagnostics);
        reconcile(&projections, &record_key, &creature, diagnostics);
        if diagnostics.len() != diagnostic_count {
            return Ok(());
        }
    }
    validate_hazard_records(connection, &projections, diagnostics)?;
    if diagnostics.is_empty() {
        validate_h8_records(
            connection,
            "JournalEntry",
            "canonical_journal_records",
            "journal",
            diagnostics,
        )?;
    }
    if diagnostics.is_empty() {
        validate_h8_records(
            connection,
            "RollTable",
            "canonical_roll_table_records",
            "roll_table",
            diagnostics,
        )?;
    }
    Ok(())
}

fn validate_h8_records(
    connection: &Connection,
    document_type: &str,
    table: &str,
    body_kind: &str,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let expected_keys = text_column(
        connection,
        &format!(
            "SELECT record_key FROM records WHERE foundry_document_type = '{}' ORDER BY record_key",
            document_type.replace('\'', "''")
        ),
    )?;
    let sql =
        format!("SELECT record_key,source_id,name,canonical_json FROM {table} ORDER BY record_key");
    let mut statement = connection.prepare(&sql).map_err(query_failed)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(query_failed)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(query_failed)?;
    let actual_keys = rows.iter().map(|row| row.0.clone()).collect::<Vec<_>>();
    if actual_keys != expected_keys {
        mismatch(
            diagnostics,
            &format!("canonical {body_kind} coverage diverges from source records"),
            &format!("{table}.coverage"),
            expected_keys.len().to_string(),
            actual_keys.len().to_string(),
        );
        return Ok(());
    }
    for (record_key, source_id, name, json) in rows {
        let path = format!("{table}[{record_key}].canonical_json");
        let body = match canonical_json::decode::<RecordBody>(&json, &path) {
            Ok(body) if canonical_json::encode(&body).is_ok_and(|encoded| encoded == json) => body,
            Ok(_) => {
                invalid(
                    diagnostics,
                    "typed JSON is not in canonical deterministic form",
                    &path,
                );
                continue;
            }
            Err(error) => {
                invalid(diagnostics, &error, &path);
                continue;
            }
        };
        let identity = match &body {
            RecordBody::Journal(journal) if body_kind == "journal" => &journal.identity,
            RecordBody::RollTable(table) if body_kind == "roll_table" => &table.identity,
            _ => {
                invalid(
                    diagnostics,
                    "canonical H8 table contains the wrong body family",
                    &path,
                );
                continue;
            }
        };
        if identity.record_key.to_string() != record_key
            || identity.source_id.as_str() != source_id
            || identity.name != name
        {
            mismatch(
                diagnostics,
                "canonical H8 identity diverges from its relational row",
                &record_key,
                "hydrated identity".to_string(),
                "relational identity".to_string(),
            );
        }
    }
    Ok(())
}

fn validate_hazard_records(
    connection: &Connection,
    projections: &ProjectionRows,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let expected_keys = text_column(
        connection,
        "SELECT record_key FROM records WHERE foundry_record_type = 'hazard' ORDER BY record_key",
    )?;
    let mut statement = connection
        .prepare("SELECT record_key,source_id,name,family,canonical_json FROM canonical_hazard_records ORDER BY record_key")
        .map_err(query_failed)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(query_failed)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(query_failed)?;
    let actual_keys = rows.iter().map(|row| row.0.clone()).collect::<Vec<_>>();
    if actual_keys != expected_keys {
        mismatch(
            diagnostics,
            "canonical hazard coverage diverges from source hazard records",
            "canonical_hazard_records.coverage",
            expected_keys.len().to_string(),
            actual_keys.len().to_string(),
        );
    }
    for (record_key, source_id, name, family, json) in rows {
        let path = format!("canonical_hazard_records[{record_key}].canonical_json");
        let body = match canonical_json::decode::<RecordBody>(&json, &path) {
            Ok(body) if canonical_json::encode(&body).is_ok_and(|encoded| encoded == json) => body,
            Ok(_) => {
                invalid(
                    diagnostics,
                    "typed JSON is not in canonical deterministic form",
                    &path,
                );
                continue;
            }
            Err(error) => {
                invalid(diagnostics, &error, &path);
                continue;
            }
        };
        let RecordBody::Hazard(hazard) = body else {
            invalid(
                diagnostics,
                "canonical hazard table contains a non-hazard body",
                &path,
            );
            continue;
        };
        if hazard.identity.record_key.to_string() != record_key
            || hazard.identity.source_id.as_str() != source_id
            || hazard.identity.name != name
            || family != "hazard"
        {
            mismatch(
                diagnostics,
                "canonical hazard identity diverges from its relational row",
                &record_key,
                "hydrated identity".to_string(),
                "relational identity".to_string(),
            );
        }
        validate_unique_hazard_ids(&hazard, diagnostics);
        reconcile_hazard(projections, &record_key, &hazard, diagnostics);
        if !diagnostics.is_empty() {
            return Ok(());
        }
    }
    Ok(())
}

/// Validates global canonical storage invariants without hydrating every body or
/// reconciling every relational projection back to its canonical JSON source.
/// Complete decoding and attachment validation remain strict on requested
/// records; [`validate_canonical_records`] is the explicit broad diagnostic.
pub(super) fn validate_canonical_structure(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    validate_enums(connection, diagnostics)?;
    if !diagnostics.is_empty() {
        return Ok(());
    }
    validate_canonical_side_row_ownership(connection, diagnostics)?;
    if !diagnostics.is_empty() {
        return Ok(());
    }
    validate_record_mechanics(connection, diagnostics)?;
    if !diagnostics.is_empty() {
        return Ok(());
    }

    for (key, sql, message) in [
        (
            "records.canonical_family_mismatch",
            "SELECT COUNT(*)
             FROM records r
             WHERE (r.record_kind = 'creature' AND r.foundry_record_type <> 'npc')
                OR (r.record_kind <> 'creature' AND r.foundry_record_type = 'npc')
                OR (r.record_kind = 'hazard' AND r.foundry_record_type <> 'hazard')
                OR (r.record_kind <> 'hazard' AND r.foundry_record_type = 'hazard')
                OR (r.record_kind = 'spell' AND r.foundry_record_type <> 'spell')
                OR (r.record_kind <> 'spell' AND r.foundry_record_type = 'spell')
                OR (r.record_kind = 'journal' AND r.foundry_document_type <> 'JournalEntry')
                OR (r.record_kind <> 'journal' AND r.foundry_document_type = 'JournalEntry')
                OR (r.record_kind = 'roll_table' AND r.foundry_document_type <> 'RollTable')
                OR (r.record_kind <> 'roll_table' AND r.foundry_document_type = 'RollTable')",
            "canonical record kind and Foundry type must identify the same body family",
        ),
        (
            "canonical_creature_records.missing_npc_body",
            "SELECT COUNT(*)
             FROM records r
             LEFT JOIN canonical_creature_records c ON c.record_key = r.record_key
             WHERE r.record_kind = 'creature'
               AND r.foundry_record_type = 'npc'
               AND c.record_key IS NULL",
            "every NPC record must have one canonical creature body",
        ),
        (
            "canonical_creature_records.non_npc_body",
            "SELECT COUNT(*)
             FROM canonical_creature_records c
             JOIN records r ON r.record_key = c.record_key
             WHERE r.record_kind <> 'creature' OR r.foundry_record_type <> 'npc'",
            "non-NPC records must not have canonical creature bodies",
        ),
        (
            "canonical_hazard_records.missing_hazard_body",
            "SELECT COUNT(*)
             FROM records r
             LEFT JOIN canonical_hazard_records h ON h.record_key = r.record_key
             WHERE r.record_kind = 'hazard'
               AND r.foundry_record_type = 'hazard'
               AND h.record_key IS NULL",
            "every hazard record must have one canonical hazard body",
        ),
        (
            "canonical_hazard_records.non_hazard_body",
            "SELECT COUNT(*)
             FROM canonical_hazard_records h
             JOIN records r ON r.record_key = h.record_key
             WHERE r.record_kind <> 'hazard' OR r.foundry_record_type <> 'hazard'",
            "non-hazard records must not have canonical hazard bodies",
        ),
        (
            "record_content_exclusions.noncanonical_owner",
            "SELECT COUNT(*)
             FROM record_content_exclusions e
             WHERE NOT EXISTS (SELECT 1 FROM canonical_creature_records c WHERE c.record_key=e.record_key)
               AND NOT EXISTS (SELECT 1 FROM canonical_hazard_records h WHERE h.record_key=e.record_key)
               AND NOT EXISTS (SELECT 1 FROM canonical_journal_records j WHERE j.record_key=e.record_key)
               AND NOT EXISTS (SELECT 1 FROM canonical_roll_table_records t WHERE t.record_key=e.record_key)",
            "content exclusions must belong to a canonical record body",
        ),
        (
            "canonical_spell_records.missing_spell_body",
            "SELECT COUNT(*)
             FROM records r
             LEFT JOIN canonical_spell_records s ON s.record_key = r.record_key
             WHERE r.foundry_record_type = 'spell' AND s.record_key IS NULL",
            "every spell record must have one canonical spell body",
        ),
        (
            "canonical_spell_records.non_spell_body",
            "SELECT COUNT(*)
             FROM canonical_spell_records s
             JOIN records r ON r.record_key = s.record_key
             WHERE r.foundry_record_type <> 'spell' OR r.record_kind <> 'spell'",
            "non-spell records must not have canonical spell bodies",
        ),
        (
            "canonical_journal_records.missing_journal_body",
            "SELECT COUNT(*) FROM records r
             LEFT JOIN canonical_journal_records j ON j.record_key = r.record_key
             WHERE r.record_kind = 'journal' AND r.foundry_document_type = 'JournalEntry'
               AND j.record_key IS NULL",
            "every JournalEntry record must have one canonical journal body",
        ),
        (
            "canonical_journal_records.non_journal_body",
            "SELECT COUNT(*) FROM canonical_journal_records j
             JOIN records r ON r.record_key = j.record_key
             WHERE r.record_kind <> 'journal' OR r.foundry_document_type <> 'JournalEntry'",
            "non-JournalEntry records must not have canonical journal bodies",
        ),
        (
            "canonical_roll_table_records.missing_roll_table_body",
            "SELECT COUNT(*) FROM records r
             LEFT JOIN canonical_roll_table_records t ON t.record_key = r.record_key
             WHERE r.record_kind = 'roll_table' AND r.foundry_document_type = 'RollTable'
               AND t.record_key IS NULL",
            "every RollTable record must have one canonical roll-table body",
        ),
        (
            "canonical_roll_table_records.non_roll_table_body",
            "SELECT COUNT(*) FROM canonical_roll_table_records t
             JOIN records r ON r.record_key = t.record_key
             WHERE r.record_kind <> 'roll_table' OR r.foundry_document_type <> 'RollTable'",
            "non-RollTable records must not have canonical roll-table bodies",
        ),
        (
            "canonical_spell_records.missing_query_projection",
            "SELECT COUNT(*)
             FROM canonical_spell_records s
             LEFT JOIN spell_records q ON q.record_key = s.record_key
             WHERE q.record_key IS NULL",
            "every canonical spell body must have one spell query projection",
        ),
        (
            "spell_records.extra_query_projection",
            "SELECT COUNT(*)
             FROM spell_records q
             LEFT JOIN canonical_spell_records s ON s.record_key = q.record_key
             WHERE s.record_key IS NULL",
            "spell query projections must be owned by canonical spell bodies",
        ),
        (
            "canonical_consumable_spell_children.non_consumable_parent",
            "SELECT COUNT(*)
             FROM canonical_consumable_spell_children c
             JOIN records r ON r.record_key = c.parent_record_key
             WHERE r.foundry_record_type <> 'consumable'",
            "canonical consumable spell children must belong to consumable records",
        ),
        (
            "canonical_consumable_spell_children.non_contiguous_order",
            "SELECT COUNT(*) FROM (
               SELECT parent_record_key
               FROM canonical_consumable_spell_children
               GROUP BY parent_record_key
               HAVING MIN(authored_order) <> 0 OR MAX(authored_order) <> COUNT(*) - 1
             )",
            "consumable spell child authored order must be contiguous from zero",
        ),
    ] {
        let invalid: i64 = connection
            .query_row(sql, [], |row| row.get(0))
            .map_err(query_failed)?;
        if invalid != 0 {
            mismatch(
                diagnostics,
                message,
                key,
                "0 invalid rows".to_string(),
                invalid.to_string(),
            );
        }
    }
    Ok(())
}

fn validate_canonical_side_row_ownership(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    for (key, sql, message) in [
        (
            "actor_records.canonical_body_owner",
            "SELECT COUNT(*)
             FROM actor_records a
             WHERE EXISTS (SELECT 1 FROM canonical_creature_records c WHERE c.record_key = a.record_key)
                OR EXISTS (SELECT 1 FROM canonical_hazard_records h WHERE h.record_key = a.record_key)
                OR EXISTS (SELECT 1 FROM canonical_spell_records s WHERE s.record_key = a.record_key)
                OR EXISTS (SELECT 1 FROM canonical_journal_records j WHERE j.record_key = a.record_key)
                OR EXISTS (SELECT 1 FROM canonical_roll_table_records t WHERE t.record_key = a.record_key)",
            "canonical records must not own generic actor rows",
        ),
        (
            "item_records.canonical_body_owner",
            "SELECT COUNT(*)
             FROM item_records i
             WHERE EXISTS (SELECT 1 FROM canonical_creature_records c WHERE c.record_key = i.record_key)
                OR EXISTS (SELECT 1 FROM canonical_hazard_records h WHERE h.record_key = i.record_key)
                OR EXISTS (SELECT 1 FROM canonical_spell_records s WHERE s.record_key = i.record_key)
                OR EXISTS (SELECT 1 FROM canonical_journal_records j WHERE j.record_key = i.record_key)
                OR EXISTS (SELECT 1 FROM canonical_roll_table_records t WHERE t.record_key = i.record_key)",
            "canonical records must not own generic item rows",
        ),
        (
            "record_metrics.canonical_spell_owner",
            "SELECT COUNT(*)
             FROM record_metrics m
             WHERE EXISTS (SELECT 1 FROM canonical_spell_records s WHERE s.record_key = m.record_key)",
            "canonical spell records must not own generic metric rows",
        ),
    ] {
        let invalid: i64 = connection
            .query_row(sql, [], |row| row.get(0))
            .map_err(query_failed)?;
        if invalid != 0 {
            mismatch(
                diagnostics,
                message,
                key,
                "0 invalid rows".to_string(),
                invalid.to_string(),
            );
        }
    }
    Ok(())
}

fn reconcile(
    rows: &ProjectionRows,
    record_key: &str,
    creature: &atlas_record::CreatureRecord,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) {
    let mut expected = BTreeMap::<&'static str, BTreeSet<[u8; 32]>>::new();
    let resources = creature
        .resources
        .value
        .as_value()
        .map_or(&[][..], Vec::as_slice);
    for resource in resources {
        add_expected(
            &mut expected,
            "canonical_creature_resources",
            json!([
                record_key,
                resource.id.as_str(),
                i64::from(resource.authored_order),
                resource.kind.as_str(),
                canonical_json::encode(resource).unwrap_or_default()
            ]),
        );
    }
    let embedded = creature.embedded_entities.value.as_value();
    let entities = embedded.map_or(&[][..], |value| value.entities.as_slice());
    for entity in entities {
        add_expected(
            &mut expected,
            "canonical_creature_entities",
            json!([
                record_key,
                entity.id.as_str(),
                entity.family.as_str(),
                entity.label,
                canonical_json::encode(&entity.source_identity).unwrap_or_default()
            ]),
        );
    }
    let occurrences = embedded.map_or(&[][..], |value| value.occurrences.as_slice());
    let mut occurrence_orders = BTreeMap::<String, Vec<i64>>::new();
    for occurrence in occurrences {
        occurrence_orders
            .entry(occurrence.id.as_str().to_string())
            .or_default()
            .push(i64::from(occurrence.authored_order));
    }
    for occurrence in occurrences {
        let (parent_kind, parent_id, parent_order) = match &occurrence.parent {
            atlas_record::CreatureOccurrenceParent::Creature => ("creature", None, None),
            atlas_record::CreatureOccurrenceParent::SpellcastingEntry(id) => (
                "spellcasting_entry",
                Some(id.as_str()),
                occurrence_order(&occurrence_orders, id.as_str(), None),
            ),
        };
        let (target_kind, target_record, target_entity) = match &occurrence.target {
            atlas_record::CreatureEntityTarget::CanonicalRecord(key) => {
                ("canonical_record", Some(key.to_string()), None)
            }
            atlas_record::CreatureEntityTarget::ActorOwned(id) => {
                ("actor_owned", None, Some(id.as_str().to_string()))
            }
        };
        add_expected(
            &mut expected,
            "canonical_creature_occurrences",
            json!([
                record_key,
                occurrence.id.as_str(),
                occurrence_stability(occurrence.identity_stability),
                occurrence.family.as_str(),
                i64::from(occurrence.authored_order),
                canonical_json::encode(&occurrence.source_sort).unwrap_or_default(),
                canonical_json::encode(&occurrence.source_folder).unwrap_or_default(),
                canonical_json::encode(&occurrence.source_identity).unwrap_or_default(),
                parent_kind,
                parent_id,
                parent_order,
                target_kind,
                target_record,
                target_entity,
                canonical_json::encode(&occurrence.context).unwrap_or_default(),
                canonical_json::encode(&occurrence.capability).unwrap_or_default(),
                canonical_json::encode(&occurrence.deltas).unwrap_or_default()
            ]),
        );
    }
    if let Some(embedded) = embedded {
        for (order, relationship) in embedded.relationships.iter().enumerate() {
            let source_order =
                occurrence_order(&occurrence_orders, relationship.source.as_str(), None);
            let (target_kind, target_occurrence, target_order, target_source) =
                match &relationship.target {
                    atlas_record::CreatureRelationshipTarget::Occurrence(id) => (
                        "occurrence",
                        Some(id.as_str()),
                        occurrence_order(&occurrence_orders, id.as_str(), None),
                        None,
                    ),
                    atlas_record::CreatureRelationshipTarget::UnresolvedNestedSourceId(id) => {
                        ("unresolved_nested_source_id", None, None, Some(id.as_str()))
                    }
                };
            add_expected(
                &mut expected,
                "canonical_creature_relationships",
                json!([
                    record_key,
                    i64::try_from(order).unwrap_or(i64::MAX),
                    relationship.source.as_str(),
                    source_order,
                    relationship_kind(relationship.kind),
                    target_kind,
                    target_occurrence,
                    target_order,
                    target_source,
                    relationship.source_path,
                    canonical_json::encode(&relationship.contextual_label).unwrap_or_default(),
                    canonical_json::encode(&relationship.lifecycle).unwrap_or_default(),
                    relationship_execution(relationship.execution)
                ]),
            );
        }
    }
    for document in &creature.content.documents {
        let (kind, owner_record, owner_entity, owner_occurrence) = match &document.owner {
            atlas_record::ContentOwner::Record(key) => {
                ("record".to_string(), Some(key.to_string()), None, None)
            }
            atlas_record::ContentOwner::CreatureEntity(id) => (
                "creature_entity".to_string(),
                None,
                Some(id.as_str().to_string()),
                None,
            ),
            atlas_record::ContentOwner::CreatureOccurrence(id) => (
                "creature_occurrence".to_string(),
                None,
                None,
                Some(id.as_str().to_string()),
            ),
            atlas_record::ContentOwner::HazardEntity(id) => (
                "hazard_entity".to_string(),
                None,
                Some(id.as_str().to_string()),
                None,
            ),
            atlas_record::ContentOwner::HazardOccurrence(id) => (
                "hazard_occurrence".to_string(),
                None,
                None,
                Some(id.as_str().to_string()),
            ),
            atlas_record::ContentOwner::Child(_) => {
                invalid(
                    diagnostics,
                    "creature content has a cross-family child owner",
                    record_key,
                );
                continue;
            }
        };
        let owner_order = owner_occurrence.as_deref().and_then(|id| {
            occurrence_order(
                &occurrence_orders,
                id,
                Some(i64::from(document.authored_order)),
            )
        });
        add_expected(
            &mut expected,
            "record_content",
            json!([
                record_key,
                document.id.content_key.as_str(),
                i64::from(document.authored_order),
                content_stability(document.identity_stability),
                kind,
                owner_record,
                owner_entity,
                owner_occurrence,
                owner_order,
                Value::Null,
                Value::Null,
                Value::Null,
                content_role(document.role),
                canonical_json::encode(&document.origin).unwrap_or_default(),
                document.visibility.as_str(),
                canonical_json::encode(&document.provenance).unwrap_or_default(),
                document.source_kind.as_str(),
                i64::from(document.source_kind.default_contributes_to_search()),
                i64::from(
                    document
                        .source_kind
                        .default_contributes_to_reference_occurrences()
                ),
                document.label,
                canonical_json::encode(&document.document).unwrap_or_default(),
                document.content_hash.as_str(),
                canonical_json::encode(&document.duplicate_status).unwrap_or_default(),
                canonical_json::encode(&document.diagnostics).unwrap_or_default()
            ]),
        );
        for occurrence in &document.reference_occurrences {
            add_expected(
                &mut expected,
                "reference_occurrences",
                json!([
                    record_key,
                    document.id.content_key.as_str(),
                    i64::from(document.authored_order),
                    i64::from(occurrence.ordinal),
                    kind,
                    owner_record,
                    owner_entity,
                    owner_occurrence,
                    owner_order,
                    Value::Null,
                    Value::Null,
                    Value::Null,
                    content_role(occurrence.role),
                    canonical_json::encode(&occurrence.origin).unwrap_or_default(),
                    occurrence.visibility.as_str(),
                    canonical_json::encode(&occurrence.provenance).unwrap_or_default(),
                    reference_target_kind(&occurrence.target),
                    occurrence.target.record_key().map(ToString::to_string),
                    serde_json::to_string(&occurrence.target).unwrap_or_default(),
                    occurrence.label,
                    occurrence.relation_kind.as_str()
                ]),
            );
        }
    }
    for exclusion in &creature.content.exclusions {
        add_expected(
            &mut expected,
            "record_content_exclusions",
            json!([
                record_key,
                exclusion.content_key.as_str(),
                exclusion.relative_source_path,
                exclusion.label,
                exclusion_reason(exclusion.reason)
            ]),
        );
    }
    for (ordinal, metric) in project_creature_facts(creature)
        .metrics
        .into_iter()
        .enumerate()
    {
        let (kind, number, text, boolean) = match metric.value {
            MetricValue::Number(value) => ("number", Some(value), None, None),
            MetricValue::Text(value) => ("text", None, Some(value), None),
            MetricValue::Boolean(value) => ("boolean", None, None, Some(i64::from(value))),
        };
        add_expected(
            &mut expected,
            "record_metrics",
            json!([
                record_key,
                i64::try_from(ordinal).unwrap_or(i64::MAX),
                metric.domain.as_str(),
                metric.key,
                kind,
                number,
                text,
                boolean
            ]),
        );
    }
    for &(table, _) in PROJECTION_TABLES {
        let actual = rows.set(table, record_key);
        let wanted = expected.remove(table).unwrap_or_default();
        if actual != wanted {
            mismatch(
                diagnostics,
                &format!(
                    "{table} exact relational projection diverges from hydrated canonical truth"
                ),
                record_key,
                format!("{} exact rows", wanted.len()),
                format!(
                    "{} rows; symmetric difference={}",
                    actual.len(),
                    actual.symmetric_difference(&wanted).count()
                ),
            );
        }
    }
}

fn reconcile_hazard(
    rows: &ProjectionRows,
    record_key: &str,
    hazard: &atlas_record::HazardRecord,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) {
    let mut expected = BTreeMap::<&'static str, BTreeSet<[u8; 32]>>::new();
    let embedded = hazard.embedded_entities.typed();
    let occurrences = embedded.map_or(&[][..], |value| value.occurrences.as_slice());
    let occurrence_orders = occurrences
        .iter()
        .map(|occurrence| {
            (
                occurrence.id.as_str().to_string(),
                i64::from(occurrence.authored_order),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if let Some(embedded) = embedded {
        for entity in &embedded.entities {
            add_expected(
                &mut expected,
                "canonical_hazard_entities",
                json!([
                    record_key,
                    entity.id.as_str(),
                    entity.family.as_str(),
                    entity.label,
                    canonical_json::encode(&entity.image).unwrap_or_default(),
                    canonical_json::encode(&entity.source_identity).unwrap_or_default(),
                    canonical_json::encode(&entity.capability).unwrap_or_default()
                ]),
            );
        }
        for occurrence in &embedded.occurrences {
            add_expected(
                &mut expected,
                "canonical_hazard_occurrences",
                json!([
                    record_key,
                    occurrence.id.as_str(),
                    occurrence.entity_id.as_str(),
                    hazard_occurrence_stability(occurrence.identity_stability),
                    occurrence.family.as_str(),
                    i64::from(occurrence.authored_order),
                    canonical_json::encode(&occurrence.source_sort).unwrap_or_default(),
                    canonical_json::encode(&occurrence.source_folder).unwrap_or_default(),
                    i64::from(occurrence.source_ordinal),
                    canonical_json::encode(&occurrence.contextual_label).unwrap_or_default()
                ]),
            );
        }
    }
    for relationship in &hazard.relationships {
        let source_id = relationship
            .source_occurrence_id
            .as_ref()
            .map(|id| id.as_str());
        let source_order = source_id.and_then(|id| occurrence_orders.get(id).copied());
        let (target_kind, target_entity, target_occurrence, target_order) =
            match &relationship.target {
                HazardRelationshipTarget::Entity(id) => ("entity", Some(id.as_str()), None, None),
                HazardRelationshipTarget::Occurrence(id) => (
                    "occurrence",
                    None,
                    Some(id.as_str()),
                    occurrence_orders.get(id.as_str()).copied(),
                ),
            };
        add_expected(
            &mut expected,
            "canonical_hazard_relationships",
            json!([
                record_key,
                relationship.id.as_str(),
                i64::from(relationship.authored_order),
                source_id,
                source_order,
                hazard_relationship_kind(relationship.kind),
                target_kind,
                target_entity,
                target_occurrence,
                target_order
            ]),
        );
    }
    for document in &hazard.content.documents {
        let (kind, owner_record, owner_entity, owner_occurrence) = match &document.owner {
            atlas_record::ContentOwner::Record(key) => {
                ("record", Some(key.to_string()), None, None)
            }
            atlas_record::ContentOwner::HazardEntity(id) => {
                ("hazard_entity", None, Some(id.as_str().to_string()), None)
            }
            atlas_record::ContentOwner::HazardOccurrence(id) => (
                "hazard_occurrence",
                None,
                None,
                Some(id.as_str().to_string()),
            ),
            atlas_record::ContentOwner::CreatureEntity(_)
            | atlas_record::ContentOwner::CreatureOccurrence(_)
            | atlas_record::ContentOwner::Child(_) => {
                invalid(
                    diagnostics,
                    "hazard content has a cross-family owner",
                    record_key,
                );
                continue;
            }
        };
        let owner_order = owner_occurrence
            .as_deref()
            .and_then(|id| occurrence_orders.get(id).copied());
        add_expected(
            &mut expected,
            "record_content",
            json!([
                record_key,
                document.id.content_key.as_str(),
                i64::from(document.authored_order),
                content_stability(document.identity_stability),
                kind,
                owner_record,
                Value::Null,
                Value::Null,
                Value::Null,
                owner_entity,
                owner_occurrence,
                owner_order,
                content_role(document.role),
                canonical_json::encode(&document.origin).unwrap_or_default(),
                document.visibility.as_str(),
                canonical_json::encode(&document.provenance).unwrap_or_default(),
                document.source_kind.as_str(),
                i64::from(document.source_kind.default_contributes_to_search()),
                i64::from(
                    document
                        .source_kind
                        .default_contributes_to_reference_occurrences()
                ),
                document.label,
                canonical_json::encode(&document.document).unwrap_or_default(),
                document.content_hash.as_str(),
                canonical_json::encode(&document.duplicate_status).unwrap_or_default(),
                canonical_json::encode(&document.diagnostics).unwrap_or_default()
            ]),
        );
        for occurrence in &document.reference_occurrences {
            add_expected(
                &mut expected,
                "reference_occurrences",
                json!([
                    record_key,
                    document.id.content_key.as_str(),
                    i64::from(document.authored_order),
                    i64::from(occurrence.ordinal),
                    kind,
                    owner_record,
                    Value::Null,
                    Value::Null,
                    Value::Null,
                    owner_entity,
                    owner_occurrence,
                    owner_order,
                    content_role(occurrence.role),
                    canonical_json::encode(&occurrence.origin).unwrap_or_default(),
                    occurrence.visibility.as_str(),
                    canonical_json::encode(&occurrence.provenance).unwrap_or_default(),
                    reference_target_kind(&occurrence.target),
                    occurrence.target.record_key().map(ToString::to_string),
                    serde_json::to_string(&occurrence.target).unwrap_or_default(),
                    occurrence.label,
                    occurrence.relation_kind.as_str()
                ]),
            );
        }
    }
    for exclusion in &hazard.content.exclusions {
        add_expected(
            &mut expected,
            "record_content_exclusions",
            json!([
                record_key,
                exclusion.content_key.as_str(),
                exclusion.relative_source_path,
                exclusion.label,
                exclusion_reason(exclusion.reason)
            ]),
        );
    }
    for (ordinal, metric) in project_hazard_facts(hazard).metrics.into_iter().enumerate() {
        let (kind, number, text, boolean) = match metric.value {
            MetricValue::Number(value) => ("number", Some(value), None, None),
            MetricValue::Text(value) => ("text", None, Some(value), None),
            MetricValue::Boolean(value) => ("boolean", None, None, Some(i64::from(value))),
        };
        add_expected(
            &mut expected,
            "record_metrics",
            json!([
                record_key,
                i64::try_from(ordinal).unwrap_or(i64::MAX),
                metric.domain.as_str(),
                metric.key,
                kind,
                number,
                text,
                boolean
            ]),
        );
    }
    for &(table, _) in PROJECTION_TABLES {
        let actual = rows.set(table, record_key);
        let wanted = expected.remove(table).unwrap_or_default();
        if actual != wanted {
            mismatch(
                diagnostics,
                &format!(
                    "{table} exact relational projection diverges from hydrated canonical truth"
                ),
                record_key,
                format!("{} exact rows", wanted.len()),
                format!(
                    "{} rows; symmetric difference={}",
                    actual.len(),
                    actual.symmetric_difference(&wanted).count()
                ),
            );
        }
    }
}

fn validate_unique_hazard_ids(
    hazard: &atlas_record::HazardRecord,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) {
    if let Some(embedded) = hazard.embedded_entities.typed() {
        unique(
            embedded.entities.iter().map(|value| value.id.as_str()),
            "duplicate canonical hazard entity ID",
            diagnostics,
        );
        unique(
            embedded.occurrences.iter().map(|value| value.id.as_str()),
            "duplicate canonical hazard occurrence ID",
            diagnostics,
        );
        unique(
            embedded
                .occurrences
                .iter()
                .map(|value| value.authored_order.to_string()),
            "duplicate canonical hazard occurrence authored order",
            diagnostics,
        );
    }
    unique(
        hazard.relationships.iter().map(|value| value.id.as_str()),
        "duplicate canonical hazard relationship ID",
        diagnostics,
    );
    unique(
        hazard
            .relationships
            .iter()
            .map(|value| value.authored_order.to_string()),
        "duplicate canonical hazard relationship authored order",
        diagnostics,
    );
    unique(
        hazard
            .content
            .documents
            .iter()
            .map(|value| value.authored_order.to_string()),
        "duplicate canonical hazard content authored order",
        diagnostics,
    );
}

fn add_expected(
    expected: &mut BTreeMap<&'static str, BTreeSet<[u8; 32]>>,
    table: &'static str,
    value: Value,
) {
    expected
        .entry(table)
        .or_default()
        .insert(digest_json(&value));
}

fn occurrence_order(
    orders: &BTreeMap<String, Vec<i64>>,
    id: &str,
    preferred: Option<i64>,
) -> Option<i64> {
    let values = orders.get(id)?;
    preferred
        .filter(|value| values.contains(value))
        .or_else(|| (values.len() == 1).then_some(values[0]))
}

fn occurrence_stability(value: atlas_record::OccurrenceIdentityStability) -> &'static str {
    match value {
        atlas_record::OccurrenceIdentityStability::StableNestedSourceId => {
            "stable_nested_source_id"
        }
        atlas_record::OccurrenceIdentityStability::UnstableOwnerFamilyOrdinal => {
            "unstable_owner_family_ordinal"
        }
    }
}

fn hazard_occurrence_stability(value: HazardOccurrenceIdentityStability) -> &'static str {
    match value {
        HazardOccurrenceIdentityStability::StableSourceIdentity => "stable_source_identity",
        HazardOccurrenceIdentityStability::UnstableAuthoredOrdinal => "unstable_authored_ordinal",
    }
}

fn hazard_relationship_kind(value: HazardRelationshipKind) -> &'static str {
    match value {
        HazardRelationshipKind::Contains => "contains",
    }
}
fn relationship_kind(value: atlas_record::CreatureEntityRelationshipKind) -> &'static str {
    match value {
        atlas_record::CreatureEntityRelationshipKind::GrantedBy => "granted_by",
        atlas_record::CreatureEntityRelationshipKind::ItemGrant => "item_grant",
        atlas_record::CreatureEntityRelationshipKind::LinkedWeapon => "linked_weapon",
        atlas_record::CreatureEntityRelationshipKind::PreparedSpell => "prepared_spell",
    }
}
fn relationship_execution(value: atlas_record::CreatureRelationshipExecution) -> &'static str {
    match value {
        atlas_record::CreatureRelationshipExecution::ProvenanceOnly => "provenance_only",
    }
}
fn content_stability(value: atlas_record::ContentIdentityStability) -> &'static str {
    match value {
        atlas_record::ContentIdentityStability::StableSourceIdentity => "stable_source_identity",
        atlas_record::ContentIdentityStability::UnstableAuthoredOrdinal => {
            "unstable_authored_ordinal"
        }
    }
}
fn content_role(value: atlas_record::ContentRole) -> &'static str {
    match value {
        atlas_record::ContentRole::PrimaryDescription => "primary_description",
        atlas_record::ContentRole::Summary => "summary",
        atlas_record::ContentRole::SupplementalRules => "supplemental_rules",
        atlas_record::ContentRole::EmbeddedCapability => "embedded_capability",
        atlas_record::ContentRole::JournalPage => "journal_page",
        atlas_record::ContentRole::TableResult => "table_result",
        atlas_record::ContentRole::GeneratedNarrative => "generated_narrative",
        atlas_record::ContentRole::Provenance => "provenance",
    }
}
fn reference_target_kind(value: &atlas_record::RichLinkTarget) -> &'static str {
    match value {
        atlas_record::RichLinkTarget::Record { .. } => "record",
        atlas_record::RichLinkTarget::RecordChild { .. } => "record_child",
        atlas_record::RichLinkTarget::LocalContent { .. } => "local_content",
        atlas_record::RichLinkTarget::External { .. } => "external",
        atlas_record::RichLinkTarget::Unresolved { .. } => "unresolved",
    }
}
fn exclusion_reason(value: atlas_record::ContentExclusionReason) -> &'static str {
    match value {
        atlas_record::ContentExclusionReason::DeferredEntityFamily => "deferred_entity_family",
        atlas_record::ContentExclusionReason::MissingTypedOwner => "missing_typed_owner",
    }
}

fn validate_unique_ids(
    creature: &atlas_record::CreatureRecord,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) {
    if let FactValue::Value(resources) = &creature.resources.value {
        unique(
            resources.iter().map(|value| value.id.as_str()),
            "duplicate canonical resource ID",
            diagnostics,
        );
        unique(
            resources
                .iter()
                .map(|value| value.authored_order.to_string()),
            "duplicate canonical resource authored order",
            diagnostics,
        );
    }
    if let FactValue::Value(embedded) = &creature.embedded_entities.value {
        unique(
            embedded.entities.iter().map(|value| value.id.as_str()),
            "duplicate canonical entity ID",
            diagnostics,
        );
        unique(
            embedded
                .occurrences
                .iter()
                .map(|value| value.authored_order.to_string()),
            "duplicate canonical occurrence authored order",
            diagnostics,
        );
    }
    unique(
        creature
            .content
            .documents
            .iter()
            .map(|value| value.authored_order.to_string()),
        "duplicate canonical content authored order",
        diagnostics,
    );
}

fn unique<T: AsRef<str>>(
    values: impl IntoIterator<Item = T>,
    message: &str,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value.as_ref().to_string()) {
            invalid(diagnostics, message, value.as_ref());
        }
    }
}

fn validate_enums(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    for (key, sql) in [
        (
            "records.visibility_state",
            "SELECT COUNT(*) FROM records WHERE visibility_state NOT IN ('visible','hidden')",
        ),
        (
            "records.visibility_reason",
            "SELECT COUNT(*) FROM records WHERE visibility_reason NOT IN ('source_record','generated_canonical','generated_instance')",
        ),
        (
            "records.record_role",
            "SELECT COUNT(*) FROM records WHERE record_role NOT IN ('source','canonical','source_instance')",
        ),
        (
            "records.retrieval_disposition",
            "SELECT COUNT(*) FROM records WHERE retrieval_disposition NOT IN ('ordinary','direct_only','inspection_only')",
        ),
        (
            "records.retrieval_rationale",
            "SELECT COUNT(*) FROM records WHERE retrieval_rationale NOT IN ('source_record','generated_canonical','duplicate_source_instance','canonical_edition_duplicate','tooling_no_addressable_product_meaning')",
        ),
        (
            "records.retrieval_policy",
            "SELECT COUNT(*) FROM records WHERE (retrieval_disposition='ordinary') <> (is_default_visible=1)",
        ),
        (
            "records.retrieval_policy_tuple",
            "SELECT COUNT(*) FROM records WHERE NOT (
               (record_role='source' AND retrieval_disposition='ordinary' AND retrieval_rationale='source_record')
               OR (record_role='canonical' AND retrieval_disposition='ordinary' AND retrieval_rationale='generated_canonical')
               OR (record_role='source_instance' AND retrieval_disposition='direct_only' AND retrieval_rationale='duplicate_source_instance')
               OR (record_role='source' AND retrieval_disposition='direct_only' AND retrieval_rationale='canonical_edition_duplicate')
               OR (record_role='source' AND retrieval_disposition='inspection_only' AND retrieval_rationale='tooling_no_addressable_product_meaning')
             )",
        ),
        (
            "records.visibility_role_coherence",
            "SELECT COUNT(*) FROM records WHERE NOT (
               (visibility_reason='source_record' AND record_role='source')
               OR (visibility_reason='generated_canonical' AND record_role='canonical')
               OR (visibility_reason='generated_instance' AND record_role='source_instance')
             )",
        ),
        (
            "canonical_creature_occurrences.family",
            "SELECT COUNT(*) FROM canonical_creature_occurrences WHERE family NOT IN ('strike','action','spellcasting-entry','spell','equipment','lore','affliction','armor','backpack','book','condition','consumable','effect','shield','treasure','weapon','unsupported')",
        ),
        (
            "canonical_creature_relationships.kind",
            "SELECT COUNT(*) FROM canonical_creature_relationships WHERE relationship_kind NOT IN ('granted_by','item_grant','linked_weapon','prepared_spell') OR execution <> 'provenance_only'",
        ),
        (
            "canonical_hazard_occurrences.family",
            "SELECT COUNT(*) FROM canonical_hazard_occurrences WHERE family NOT IN ('action','strike','condition','effect','unsupported_child')",
        ),
        (
            "canonical_hazard_relationships.kind",
            "SELECT COUNT(*) FROM canonical_hazard_relationships WHERE relationship_kind <> 'contains' OR target_kind NOT IN ('entity','occurrence')",
        ),
        (
            "record_content.owner_kind",
            "SELECT COUNT(*) FROM record_content WHERE owner_kind NOT IN ('record','creature_entity','creature_occurrence','hazard_entity','hazard_occurrence','child')",
        ),
        (
            "reference_occurrences.owner_kind",
            "SELECT COUNT(*) FROM reference_occurrences WHERE owner_kind NOT IN ('record','creature_entity','creature_occurrence','hazard_entity','hazard_occurrence','child')",
        ),
    ] {
        let count: i64 = connection
            .query_row(sql, [], |row| row.get(0))
            .map_err(query_failed)?;
        if count != 0 {
            mismatch(
                diagnostics,
                "stored enum or retrieval policy contains an unsupported value",
                key,
                "0 invalid rows".to_string(),
                count.to_string(),
            );
        }
    }
    Ok(())
}

fn validate_record_mechanics(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let mut metrics = BTreeMap::<String, Vec<atlas_record::MetricRow>>::new();
    let mut statement = connection
        .prepare(
            "SELECT record_key,ordinal,metric_domain,metric_key,value_type,number_value,text_value,bool_value
             FROM record_metrics ORDER BY record_key,ordinal",
        )
        .map_err(query_failed)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<f64>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<i64>>(7)?,
            ))
        })
        .map_err(query_failed)?;
    for row in rows {
        let (record_key, ordinal, domain, key, value_type, number, text, boolean) =
            row.map_err(query_failed)?;
        if !expected_ordinal(
            &metrics,
            &record_key,
            ordinal,
            "record_metrics",
            diagnostics,
        ) {
            return Ok(());
        }
        let metric = match crate::read::records::children::metric_from_storage(
            &domain,
            key,
            &value_type,
            number,
            text,
            boolean.map(|value| value != 0),
        ) {
            Ok(metric) => metric,
            Err(error) => {
                invalid(
                    diagnostics,
                    &error,
                    &format!("record_metrics[{record_key}:{ordinal}]"),
                );
                return Ok(());
            }
        };
        metrics.entry(record_key).or_default().push(metric);
    }

    let mut statement = connection
        .prepare(
            "SELECT record_key,metric_count,metric_order_sha256
             FROM records ORDER BY record_key",
        )
        .map_err(query_failed)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(query_failed)?;
    for row in rows {
        let (record_key, metric_count, metric_digest) = row.map_err(query_failed)?;
        let metric_values = metrics
            .get(&record_key)
            .map(Vec::as_slice)
            .unwrap_or_default();
        validate_summary(
            diagnostics,
            &record_key,
            "record_metrics",
            metric_count,
            &metric_digest,
            metric_values.len(),
            crate::read::records::children::metric_order_digest(metric_values),
        );
        if !diagnostics.is_empty() {
            return Ok(());
        }
    }
    Ok(())
}

fn expected_ordinal<T>(
    rows: &BTreeMap<String, Vec<T>>,
    record_key: &str,
    actual: i64,
    table: &str,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> bool {
    let expected = rows.get(record_key).map_or(0, Vec::len);
    let expected = i64::try_from(expected).unwrap_or(i64::MAX);
    if actual == expected {
        true
    } else {
        mismatch(
            diagnostics,
            "ordered mechanics exact relational projection is not contiguous in canonical vector order",
            &format!("{table}[{record_key}].ordinal"),
            expected.to_string(),
            actual.to_string(),
        );
        false
    }
}

fn validate_summary(
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
    record_key: &str,
    table: &str,
    expected_count: i64,
    expected_digest: &str,
    actual_count: usize,
    actual_digest: Result<String, String>,
) {
    let actual_count = i64::try_from(actual_count).unwrap_or(i64::MAX);
    if expected_count != actual_count {
        mismatch(
            diagnostics,
            "mechanics child row count diverges from the writer-bound canonical vector",
            &format!("{table}[{record_key}].count"),
            expected_count.to_string(),
            actual_count.to_string(),
        );
        return;
    }
    match actual_digest {
        Ok(actual) if actual == expected_digest => {}
        Ok(actual) => mismatch(
            diagnostics,
            "mechanics child exact relational projection values or order diverge from the writer-bound canonical vector",
            &format!("{table}[{record_key}].order_sha256"),
            expected_digest.to_string(),
            actual,
        ),
        Err(error) => invalid(
            diagnostics,
            &error,
            &format!("{table}[{record_key}].order_sha256"),
        ),
    }
}

fn text_column(connection: &Connection, sql: &str) -> Result<Vec<String>, IndexValidationError> {
    let mut statement = connection.prepare(sql).map_err(query_failed)?;
    statement
        .query_map([], |row| row.get(0))
        .map_err(query_failed)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(query_failed)
}

fn invalid(diagnostics: &mut Vec<ArtifactValidationDiagnostic>, message: &str, key: &str) {
    diagnostics.push(artifact_validation_diagnostic(
        ArtifactValidationFamily::Data,
        message.to_string(),
        Some(key.to_string()),
        Some("valid canonical typed projection".to_string()),
        Some("invalid".to_string()),
    ));
}

fn mismatch(
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
    message: &str,
    key: &str,
    expected: String,
    actual: String,
) {
    diagnostics.push(artifact_validation_diagnostic(
        ArtifactValidationFamily::Data,
        message.to_string(),
        Some(key.to_string()),
        Some(expected),
        Some(actual),
    ));
}

fn query_failed(error: rusqlite::Error) -> IndexValidationError {
    IndexValidationError::QueryFailed(error.to_string())
}
