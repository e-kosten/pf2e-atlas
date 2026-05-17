use atlas_artifact::schema::{
    actor_record_select_sql, item_record_select_sql, persisted_record_select_sql,
    record_alias_select_sql, record_content_select_sql, record_metric_select_sql,
    reference_edge_select_sql, remaster_link_select_sql, spell_record_select_sql,
};
use atlas_domain::{
    MetricDomain, MetricValueType, PackName, PublicationFamily, RecordFamily, RecordId, RecordKey,
    RemasterLinkSource, TimeKind, TimeUnit,
};
use atlas_record::{
    ActorSideData, AliasSource, ContentDocument, ContentSourceKind, ContentVisibility,
    ItemSideData, MetricRow, MetricValue, NormalizedTime, PersistedRecord, PersistedRecordSet,
    RecordAlias, ReferenceEdge, RemasterLink, SpellSideData, SupplementalContentDocument,
};
use rusqlite::{Connection, Row};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RecordLoadError {
    #[error("index is unavailable: {0}")]
    Unavailable(String),
    #[error("record query failed: {0}")]
    QueryFailed(String),
    #[error("record data is invalid: {0}")]
    InvalidData(String),
}

pub fn load_persisted_record_set_from_connection(
    connection: &Connection,
) -> Result<PersistedRecordSet, RecordLoadError> {
    Ok(PersistedRecordSet {
        records: load_persisted_records_from_connection(connection)?,
        reference_edges: read_reference_edges(connection)?,
        aliases: read_aliases(connection)?,
        remaster_links: read_remaster_links(connection)?,
    })
}

pub fn load_persisted_records_from_connection(
    connection: &Connection,
) -> Result<Vec<PersistedRecord>, RecordLoadError> {
    let mut records = read_record_rows(connection)?;
    let metrics = read_metrics(connection)?;
    let actor_data = read_actor_data(connection)?;
    let item_data = read_item_data(connection)?;
    let spell_data = read_spell_data(connection)?;
    let supplemental_content = read_record_content(connection)?;

    for record in &mut records {
        let key = record.key.to_string();
        record.metrics = metrics.get(&key).cloned().unwrap_or_default();
        record.actor_data = actor_data.get(&key).cloned();
        record.item_data = item_data.get(&key).cloned();
        record.spell_data = spell_data.get(&key).cloned();
        record.supplemental_content = supplemental_content.get(&key).cloned().unwrap_or_default();
    }

    Ok(records)
}

fn read_record_rows(connection: &Connection) -> Result<Vec<PersistedRecord>, RecordLoadError> {
    let persisted_record_select_sql = persisted_record_select_sql();
    let mut statement = connection
        .prepare(&persisted_record_select_sql)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut records = Vec::new();
    let mut rows = statement
        .query([])
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        records.push(record_from_row(row)?);
    }
    Ok(records)
}

fn record_from_row(row: &Row<'_>) -> Result<PersistedRecord, RecordLoadError> {
    let record_key = required_string(row, "record_key")?;
    let id = required_string(row, "id")?;
    let record_family = required_string(row, "record_family")?;
    let pack_name = required_string(row, "pack_name")?;
    let traits_json = required_string(row, "traits_json")?;
    let activation_time_kind = optional_string(row, "activation_time_kind")?;
    let duration_kind = optional_string(row, "duration_kind")?;
    let publication_family = required_string(row, "publication_family")?;
    let taxonomy_families_json = required_string(row, "taxonomy_families_json")?;
    let variant_axes_json = required_string(row, "variant_axes_json")?;

    Ok(PersistedRecord {
        key: parse_record_key(&record_key)?,
        id: RecordId::new(id).map_err(invalid_parse("id"))?,
        name: required_string(row, "name")?,
        normalized_name: required_string(row, "normalized_name")?,
        record_family: parse_record_family(&record_family)?,
        pack_name: PackName::new(pack_name).map_err(invalid_parse("pack_name"))?,
        pack_label: required_string(row, "pack_label")?,
        foundry_document_type: required_string(row, "foundry_document_type")?,
        foundry_record_type: required_string(row, "foundry_record_type")?,
        level: optional_i64(row, "level")?,
        rarity: optional_string(row, "rarity")?,
        traits: json_string_array("records.traits_json", &traits_json)?,
        system_category: optional_string(row, "system_category")?,
        system_group: optional_string(row, "system_group")?,
        system_base_item: optional_string(row, "system_base_item")?,
        system_usage: optional_string(row, "system_usage")?,
        system_price_json: optional_string(row, "system_price_json")?,
        system_actions_value: optional_i64(row, "system_actions_value")?,
        system_time_value: optional_string(row, "system_time_value")?,
        system_duration_value: optional_string(row, "system_duration_value")?,
        price_cp: optional_i64(row, "price_cp")?,
        activation_time: normalized_time(
            "activation_time",
            activation_time_kind,
            optional_i64(row, "activation_time_actions")?,
            optional_i64(row, "activation_time_duration_value")?,
            optional_string(row, "activation_time_duration_unit")?,
            optional_string(row, "activation_time_text")?,
        )?,
        duration: normalized_time(
            "duration",
            duration_kind,
            None,
            optional_i64(row, "duration_value")?,
            optional_string(row, "duration_unit")?,
            optional_string(row, "duration_text")?,
        )?,
        metrics: Vec::new(),
        actor_data: None,
        item_data: None,
        spell_data: None,
        publication_title: optional_string(row, "publication_title")?,
        publication_remaster: bool_column(
            "records.publication_remaster",
            row,
            "publication_remaster",
        )?,
        description: optional_content_document(
            "records.description_json",
            row,
            "description_json",
        )?,
        blurb: optional_content_document("records.blurb_json", row, "blurb_json")?,
        supplemental_content: Vec::new(),
        publication_family: parse_publication_family(&publication_family)?,
        folder_id: optional_string(row, "folder_id")?,
        taxonomy_families: json_string_array(
            "records.taxonomy_families_json",
            &taxonomy_families_json,
        )?,
        variant_group_key: optional_string(row, "variant_group_key")?,
        variant_base_name: optional_string(row, "variant_base_name")?,
        variant_label: optional_string(row, "variant_label")?,
        variant_axes: json_string_array("records.variant_axes_json", &variant_axes_json)?,
        variant_confidence: optional_f64(row, "variant_confidence")?,
        variant_source: required_string(row, "variant_source")?,
        source_path: required_string(row, "source_path")?,
        is_default_visible: bool_column("records.is_default_visible", row, "is_default_visible")?,
        raw_json: required_string(row, "raw_json")?,
    })
}

fn read_record_content(
    connection: &Connection,
) -> Result<BTreeMap<String, Vec<SupplementalContentDocument>>, RecordLoadError> {
    let mut statement = connection
        .prepare(&record_content_select_sql())
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut rows = statement
        .query([])
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut values: BTreeMap<String, Vec<SupplementalContentDocument>> = BTreeMap::new();
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        let record_key = required_string(row, "record_key")?;
        let source_kind = parse_content_source_kind(&required_string(row, "source_kind")?)?;
        let visibility = parse_content_visibility(&required_string(row, "visibility")?)?;
        let content_json = required_string(row, "content_json")?;
        values
            .entry(record_key)
            .or_default()
            .push(SupplementalContentDocument {
                source_kind,
                visibility,
                contributes_to_search: required_bool(row, "contributes_to_search")?,
                contributes_to_references: required_bool(row, "contributes_to_references")?,
                label: optional_string(row, "label")?,
                document: content_document("record_content.content_json", &content_json)?,
            });
    }
    Ok(values)
}

fn read_metrics(
    connection: &Connection,
) -> Result<BTreeMap<String, Vec<MetricRow>>, RecordLoadError> {
    let mut statement = connection
        .prepare(&record_metric_select_sql())
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut metrics: BTreeMap<String, Vec<MetricRow>> = BTreeMap::new();
    let mut rows = statement
        .query([])
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        let record_key = required_string(row, "record_key")?;
        let domain = parse_metric_domain(&required_string(row, "metric_domain")?)?;
        let key = required_string(row, "metric_key")?;
        let value_type = required_string(row, "value_type")?;
        let value = match parse_metric_value_type(&value_type)? {
            MetricValueType::Number => MetricValue::Number(required_f64(row, "number_value")?),
            MetricValueType::Text => MetricValue::Text(required_string(row, "text_value")?),
            MetricValueType::Boolean => {
                MetricValue::Boolean(bool_column("record_metrics.bool_value", row, "bool_value")?)
            }
        };
        let metric = MetricRow { domain, key, value };
        metrics.entry(record_key).or_default().push(metric);
    }
    Ok(metrics)
}

fn read_actor_data(
    connection: &Connection,
) -> Result<BTreeMap<String, ActorSideData>, RecordLoadError> {
    let mut statement = connection
        .prepare(&actor_record_select_sql())
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut values = BTreeMap::new();
    let mut rows = statement
        .query([])
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        let record_key = required_string(row, "record_key")?;
        values.insert(
            record_key,
            ActorSideData {
                size: optional_string(row, "size")?,
                languages: json_string_array(
                    "actor_records.languages_json",
                    &required_string(row, "languages_json")?,
                )?,
                speed_types: json_string_array(
                    "actor_records.speed_types_json",
                    &required_string(row, "speed_types_json")?,
                )?,
                senses: json_string_array(
                    "actor_records.senses_json",
                    &required_string(row, "senses_json")?,
                )?,
                immunities: json_string_array(
                    "actor_records.immunities_json",
                    &required_string(row, "immunities_json")?,
                )?,
                resistances: json_string_array(
                    "actor_records.resistances_json",
                    &required_string(row, "resistances_json")?,
                )?,
                weaknesses: json_string_array(
                    "actor_records.weaknesses_json",
                    &required_string(row, "weaknesses_json")?,
                )?,
                disable_text: optional_string(row, "disable_text")?,
                disable_skills: json_string_array(
                    "actor_records.disable_skills_json",
                    &required_string(row, "disable_skills_json")?,
                )?,
                is_complex: bool_column("actor_records.is_complex", row, "is_complex")?,
            },
        );
    }
    Ok(values)
}

fn read_item_data(
    connection: &Connection,
) -> Result<BTreeMap<String, ItemSideData>, RecordLoadError> {
    let mut statement = connection
        .prepare(&item_record_select_sql())
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut values = BTreeMap::new();
    let mut rows = statement
        .query([])
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        let record_key = required_string(row, "record_key")?;
        values.insert(
            record_key,
            ItemSideData {
                system_category: optional_string(row, "system_category")?,
                system_base_item: optional_string(row, "system_base_item")?,
                system_group: optional_string(row, "system_group")?,
                system_usage: optional_string(row, "system_usage")?,
                price_cp: optional_i64(row, "price_cp")?,
                bulk_value: optional_f64(row, "bulk_value")?,
                hands_requirement: optional_string(row, "hands_requirement")?,
                damage_types: json_string_array(
                    "item_records.damage_types_json",
                    &required_string(row, "damage_types_json")?,
                )?,
            },
        );
    }
    Ok(values)
}

fn read_spell_data(
    connection: &Connection,
) -> Result<BTreeMap<String, SpellSideData>, RecordLoadError> {
    let mut statement = connection
        .prepare(&spell_record_select_sql())
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut values = BTreeMap::new();
    let mut rows = statement
        .query([])
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        let record_key = required_string(row, "record_key")?;
        values.insert(
            record_key,
            SpellSideData {
                traditions: json_string_array(
                    "spell_records.traditions_json",
                    &required_string(row, "traditions_json")?,
                )?,
                spell_kinds: json_string_array(
                    "spell_records.spell_kinds_json",
                    &required_string(row, "spell_kinds_json")?,
                )?,
                range_text: optional_string(row, "range_text")?,
                range_value: optional_f64(row, "range_value")?,
                target_text: optional_string(row, "target_text")?,
                area_type: optional_string(row, "area_type")?,
                area_value: optional_f64(row, "area_value")?,
                save_type: optional_string(row, "save_type")?,
                sustained: bool_column("spell_records.sustained", row, "sustained")?,
                basic_save: bool_column("spell_records.basic_save", row, "basic_save")?,
                damage_types: json_string_array(
                    "spell_records.damage_types_json",
                    &required_string(row, "damage_types_json")?,
                )?,
            },
        );
    }
    Ok(values)
}

fn read_reference_edges(connection: &Connection) -> Result<Vec<ReferenceEdge>, RecordLoadError> {
    let mut statement = connection
        .prepare(&reference_edge_select_sql())
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut rows = statement
        .query([])
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut edges = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        edges.push(ReferenceEdge {
            from_record_key: parse_record_key(&required_string(row, "from_record_key")?)?,
            to_record_key: parse_record_key(&required_string(row, "to_record_key")?)?,
            display_text: optional_string(row, "display_text")?,
            reference_text: required_string(row, "reference_text")?,
            source_kind: parse_content_source_kind(&required_string(row, "source_kind")?)?,
            visibility: parse_content_visibility(&required_string(row, "visibility")?)?,
        });
    }
    Ok(edges)
}

fn read_aliases(connection: &Connection) -> Result<Vec<RecordAlias>, RecordLoadError> {
    let mut statement = connection
        .prepare(&record_alias_select_sql())
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut rows = statement
        .query([])
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut aliases = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        aliases.push(RecordAlias {
            canonical_record_key: parse_record_key(&required_string(row, "canonical_record_key")?)?,
            alias_text: required_string(row, "alias_text")?,
            normalized_alias: required_string(row, "normalized_alias")?,
            source: parse_alias_source(&required_string(row, "source_kind")?)?,
            source_ref: required_string(row, "source_ref")?,
        });
    }
    Ok(aliases)
}

fn read_remaster_links(connection: &Connection) -> Result<Vec<RemasterLink>, RecordLoadError> {
    let mut statement = connection
        .prepare(&remaster_link_select_sql())
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut rows = statement
        .query([])
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut links = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        links.push(RemasterLink {
            remaster_record_key: parse_record_key(&required_string(row, "remaster_record_key")?)?,
            legacy_record_key: parse_record_key(&required_string(row, "legacy_record_key")?)?,
            source: parse_remaster_link_source(&required_string(row, "source_kind")?)?,
            source_ref: required_string(row, "source_ref")?,
        });
    }
    Ok(links)
}

fn normalized_time(
    column_family: &'static str,
    kind: Option<String>,
    actions: Option<i64>,
    duration_value: Option<i64>,
    duration_unit: Option<String>,
    text: Option<String>,
) -> Result<Option<NormalizedTime>, RecordLoadError> {
    let Some(kind) = kind else {
        return Ok(None);
    };
    Ok(Some(NormalizedTime {
        kind: parse_time_kind(&kind)?,
        actions,
        duration_value,
        duration_unit: duration_unit.as_deref().map(parse_time_unit).transpose()?,
        text: text.ok_or_else(|| invalid_missing(format!("{column_family}_text")))?,
    }))
}

fn required_string(row: &Row<'_>, column: &'static str) -> Result<String, RecordLoadError> {
    row.get(column)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))
}

fn optional_string(row: &Row<'_>, column: &'static str) -> Result<Option<String>, RecordLoadError> {
    row.get(column)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))
}

fn optional_i64(row: &Row<'_>, column: &'static str) -> Result<Option<i64>, RecordLoadError> {
    row.get(column)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))
}

fn optional_f64(row: &Row<'_>, column: &'static str) -> Result<Option<f64>, RecordLoadError> {
    row.get(column)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))
}

fn required_f64(row: &Row<'_>, column: &'static str) -> Result<f64, RecordLoadError> {
    row.get(column)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))
}

fn required_bool(row: &Row<'_>, column: &'static str) -> Result<bool, RecordLoadError> {
    bool_column(column, row, column)
}

fn bool_column(
    name: &'static str,
    row: &Row<'_>,
    column: &'static str,
) -> Result<bool, RecordLoadError> {
    match row
        .get::<_, i64>(column)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        0 => Ok(false),
        1 => Ok(true),
        value => Err(invalid_value(name, value.to_string())),
    }
}

fn json_string_array(name: &'static str, value: &str) -> Result<Vec<String>, RecordLoadError> {
    serde_json::from_str(value).map_err(|error| {
        RecordLoadError::InvalidData(format!("{name} must be a JSON string array: {error}"))
    })
}

fn optional_content_document(
    name: &'static str,
    row: &Row<'_>,
    column: &'static str,
) -> Result<Option<ContentDocument>, RecordLoadError> {
    optional_string(row, column)?
        .as_deref()
        .map(|value| content_document(name, value))
        .transpose()
}

fn content_document(name: &'static str, value: &str) -> Result<ContentDocument, RecordLoadError> {
    serde_json::from_str(value).map_err(|error| {
        RecordLoadError::InvalidData(format!("{name} must be a content document JSON: {error}"))
    })
}

fn parse_record_key(value: &str) -> Result<RecordKey, RecordLoadError> {
    RecordKey::parse(value).map_err(invalid_parse("record_key"))
}

fn parse_record_family(value: &str) -> Result<RecordFamily, RecordLoadError> {
    RecordFamily::from_canonical(value)
        .ok_or_else(|| invalid_value("records.record_family", value.to_string()))
}

fn parse_publication_family(value: &str) -> Result<PublicationFamily, RecordLoadError> {
    PublicationFamily::from_canonical(value)
        .ok_or_else(|| invalid_value("records.publication_family", value.to_string()))
}

fn parse_metric_domain(value: &str) -> Result<MetricDomain, RecordLoadError> {
    MetricDomain::from_canonical(value)
        .ok_or_else(|| invalid_value("record_metrics.metric_domain", value.to_string()))
}

fn parse_metric_value_type(value: &str) -> Result<MetricValueType, RecordLoadError> {
    MetricValueType::from_canonical(value)
        .ok_or_else(|| invalid_value("record_metrics.value_type", value.to_string()))
}

fn parse_alias_source(value: &str) -> Result<AliasSource, RecordLoadError> {
    AliasSource::from_canonical(value)
        .ok_or_else(|| invalid_value("record_aliases.source_kind", value.to_string()))
}

fn parse_content_source_kind(value: &str) -> Result<ContentSourceKind, RecordLoadError> {
    ContentSourceKind::from_canonical(value)
        .ok_or_else(|| invalid_value("content.source_kind", value.to_string()))
}

fn parse_content_visibility(value: &str) -> Result<ContentVisibility, RecordLoadError> {
    ContentVisibility::from_canonical(value)
        .ok_or_else(|| invalid_value("content.visibility", value.to_string()))
}

fn parse_remaster_link_source(value: &str) -> Result<RemasterLinkSource, RecordLoadError> {
    RemasterLinkSource::from_canonical(value)
        .ok_or_else(|| invalid_value("remaster_links.source_kind", value.to_string()))
}

fn parse_time_kind(value: &str) -> Result<TimeKind, RecordLoadError> {
    TimeKind::from_canonical(value).ok_or_else(|| invalid_value("time.kind", value.to_string()))
}

fn parse_time_unit(value: &str) -> Result<TimeUnit, RecordLoadError> {
    TimeUnit::from_canonical(value).ok_or_else(|| invalid_value("time.unit", value.to_string()))
}

fn invalid_parse<E: std::fmt::Display + 'static>(
    field: &'static str,
) -> impl FnOnce(E) -> RecordLoadError {
    move |error| RecordLoadError::InvalidData(format!("{field} is invalid: {error}"))
}

fn invalid_value(field: &'static str, value: String) -> RecordLoadError {
    RecordLoadError::InvalidData(format!("{field} has unsupported value `{value}`"))
}

fn invalid_missing(field: impl Into<String>) -> RecordLoadError {
    RecordLoadError::InvalidData(format!("required field `{}` is missing", field.into()))
}
