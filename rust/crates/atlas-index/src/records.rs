use std::collections::BTreeMap;
use std::path::Path;

use atlas_artifact::schema::persisted_record_select_sql;
use atlas_domain::{
    MetricDomain, PackName, PublicationFamily, RecordFamily, RecordId, RecordKey,
    RemasterLinkSource, TimeKind, TimeUnit,
};
use atlas_record::{
    ActorSideData, AliasSource, ItemSideData, MetricRow, MetricValue, NormalizedTime,
    PersistedRecord, PersistedRecordSet, RecordAlias, ReferenceEdge, RemasterLink, SpellSideData,
};
use rusqlite::{Connection, OpenFlags, Row};
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

pub fn load_persisted_records(
    path: impl AsRef<Path>,
) -> Result<Vec<PersistedRecord>, RecordLoadError> {
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| RecordLoadError::Unavailable(error.to_string()))?;
    load_persisted_records_from_connection(&connection)
}

pub fn load_persisted_record_set(
    path: impl AsRef<Path>,
) -> Result<PersistedRecordSet, RecordLoadError> {
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| RecordLoadError::Unavailable(error.to_string()))?;
    load_persisted_record_set_from_connection(&connection)
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

    for record in &mut records {
        let key = record.key.to_string();
        record.metrics = metrics.get(&key).cloned().unwrap_or_default();
        record.actor_data = actor_data.get(&key).cloned();
        record.item_data = item_data.get(&key).cloned();
        record.spell_data = spell_data.get(&key).cloned();
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
    let record_key = required_string(row, 0)?;
    let id = required_string(row, 1)?;
    let record_family = required_string(row, 4)?;
    let pack_name = required_string(row, 5)?;
    let traits_json = required_string(row, 11)?;
    let activation_time_kind = optional_string(row, 21)?;
    let duration_kind = optional_string(row, 26)?;
    let publication_family = required_string(row, 34)?;
    let taxonomy_families_json = required_string(row, 36)?;
    let variant_axes_json = required_string(row, 40)?;

    Ok(PersistedRecord {
        key: parse_record_key(&record_key)?,
        id: RecordId::new(id).map_err(invalid_parse("id"))?,
        name: required_string(row, 2)?,
        normalized_name: required_string(row, 3)?,
        record_family: parse_record_family(&record_family)?,
        pack_name: PackName::new(pack_name).map_err(invalid_parse("pack_name"))?,
        pack_label: required_string(row, 6)?,
        foundry_document_type: required_string(row, 7)?,
        foundry_record_type: required_string(row, 8)?,
        level: row
            .get(9)
            .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?,
        rarity: optional_string(row, 10)?,
        traits: json_string_array("records.traits_json", &traits_json)?,
        system_category: optional_string(row, 12)?,
        system_group: optional_string(row, 13)?,
        system_base_item: optional_string(row, 14)?,
        system_usage: optional_string(row, 15)?,
        system_price_json: optional_string(row, 16)?,
        system_actions_value: row
            .get(17)
            .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?,
        system_time_value: optional_string(row, 18)?,
        system_duration_value: optional_string(row, 19)?,
        price_cp: row
            .get(20)
            .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?,
        activation_time: normalized_time(
            "activation_time",
            activation_time_kind,
            row.get(22)
                .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?,
            row.get(23)
                .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?,
            optional_string(row, 24)?,
            optional_string(row, 25)?,
        )?,
        duration: normalized_time(
            "duration",
            duration_kind,
            None,
            row.get(27)
                .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?,
            optional_string(row, 28)?,
            optional_string(row, 29)?,
        )?,
        metrics: Vec::new(),
        actor_data: None,
        item_data: None,
        spell_data: None,
        publication_title: optional_string(row, 30)?,
        publication_remaster: bool_column("records.publication_remaster", row, 31)?,
        description_text: optional_string(row, 32)?,
        blurb_text: optional_string(row, 33)?,
        publication_family: parse_publication_family(&publication_family)?,
        folder_id: optional_string(row, 35)?,
        taxonomy_families: json_string_array(
            "records.taxonomy_families_json",
            &taxonomy_families_json,
        )?,
        variant_group_key: optional_string(row, 37)?,
        variant_base_name: optional_string(row, 38)?,
        variant_label: optional_string(row, 39)?,
        variant_axes: json_string_array("records.variant_axes_json", &variant_axes_json)?,
        variant_confidence: row
            .get(41)
            .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?,
        variant_source: required_string(row, 42)?,
        source_path: required_string(row, 43)?,
        is_default_visible: bool_column("records.is_default_visible", row, 44)?,
        search_text_projection: required_string(row, 45)?,
        raw_json: required_string(row, 46)?,
    })
}

fn read_metrics(
    connection: &Connection,
) -> Result<BTreeMap<String, Vec<MetricRow>>, RecordLoadError> {
    let mut statement = connection
        .prepare(
            "SELECT record_key, metric_domain, metric_key, value_type, number_value, text_value, bool_value
             FROM record_metrics
             ORDER BY record_key, metric_domain, metric_key",
        )
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut metrics: BTreeMap<String, Vec<MetricRow>> = BTreeMap::new();
    let mut rows = statement
        .query([])
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        let record_key = required_string(row, 0)?;
        let domain = parse_metric_domain(&required_string(row, 1)?)?;
        let key = required_string(row, 2)?;
        let value_type = required_string(row, 3)?;
        let value = match value_type.as_str() {
            "number" => MetricValue::Number(
                row.get::<_, f64>(4)
                    .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?,
            ),
            "text" => MetricValue::Text(required_string(row, 5)?),
            "boolean" => MetricValue::Boolean(bool_column("record_metrics.bool_value", row, 6)?),
            _ => return Err(invalid_value("record_metrics.value_type", value_type)),
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
        .prepare(
            "SELECT record_key, size, languages_json, speed_types_json, senses_json, immunities_json,
                    resistances_json, weaknesses_json, disable_text, disable_skills_json, is_complex
             FROM actor_records
             ORDER BY record_key",
        )
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut values = BTreeMap::new();
    let mut rows = statement
        .query([])
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        let record_key = required_string(row, 0)?;
        values.insert(
            record_key,
            ActorSideData {
                size: optional_string(row, 1)?,
                languages: json_string_array(
                    "actor_records.languages_json",
                    &required_string(row, 2)?,
                )?,
                speed_types: json_string_array(
                    "actor_records.speed_types_json",
                    &required_string(row, 3)?,
                )?,
                senses: json_string_array("actor_records.senses_json", &required_string(row, 4)?)?,
                immunities: json_string_array(
                    "actor_records.immunities_json",
                    &required_string(row, 5)?,
                )?,
                resistances: json_string_array(
                    "actor_records.resistances_json",
                    &required_string(row, 6)?,
                )?,
                weaknesses: json_string_array(
                    "actor_records.weaknesses_json",
                    &required_string(row, 7)?,
                )?,
                disable_text: optional_string(row, 8)?,
                disable_skills: json_string_array(
                    "actor_records.disable_skills_json",
                    &required_string(row, 9)?,
                )?,
                is_complex: bool_column("actor_records.is_complex", row, 10)?,
            },
        );
    }
    Ok(values)
}

fn read_item_data(
    connection: &Connection,
) -> Result<BTreeMap<String, ItemSideData>, RecordLoadError> {
    let mut statement = connection
        .prepare(
            "SELECT record_key, system_category, system_base_item, system_group, system_usage,
                    price_cp, bulk_value, hands_requirement, damage_types_json
             FROM item_records
             ORDER BY record_key",
        )
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut values = BTreeMap::new();
    let mut rows = statement
        .query([])
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        let record_key = required_string(row, 0)?;
        values.insert(
            record_key,
            ItemSideData {
                system_category: optional_string(row, 1)?,
                system_base_item: optional_string(row, 2)?,
                system_group: optional_string(row, 3)?,
                system_usage: optional_string(row, 4)?,
                price_cp: row
                    .get(5)
                    .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?,
                bulk_value: row
                    .get(6)
                    .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?,
                hands_requirement: optional_string(row, 7)?,
                damage_types: json_string_array(
                    "item_records.damage_types_json",
                    &required_string(row, 8)?,
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
        .prepare(
            "SELECT record_key, traditions_json, spell_kinds_json, range_text, range_value,
                    target_text, area_type, area_value, save_type, sustained, basic_save, damage_types_json
             FROM spell_records
             ORDER BY record_key",
        )
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut values = BTreeMap::new();
    let mut rows = statement
        .query([])
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        let record_key = required_string(row, 0)?;
        values.insert(
            record_key,
            SpellSideData {
                traditions: json_string_array(
                    "spell_records.traditions_json",
                    &required_string(row, 1)?,
                )?,
                spell_kinds: json_string_array(
                    "spell_records.spell_kinds_json",
                    &required_string(row, 2)?,
                )?,
                range_text: optional_string(row, 3)?,
                range_value: row
                    .get(4)
                    .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?,
                target_text: optional_string(row, 5)?,
                area_type: optional_string(row, 6)?,
                area_value: row
                    .get(7)
                    .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?,
                save_type: optional_string(row, 8)?,
                sustained: bool_column("spell_records.sustained", row, 9)?,
                basic_save: bool_column("spell_records.basic_save", row, 10)?,
                damage_types: json_string_array(
                    "spell_records.damage_types_json",
                    &required_string(row, 11)?,
                )?,
            },
        );
    }
    Ok(values)
}

fn read_reference_edges(connection: &Connection) -> Result<Vec<ReferenceEdge>, RecordLoadError> {
    let mut statement = connection
        .prepare(
            "SELECT from_record_key, to_record_key, display_text, reference_text
             FROM reference_edges
             ORDER BY from_record_key, to_record_key, reference_text",
        )
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
            from_record_key: parse_record_key(&required_string(row, 0)?)?,
            to_record_key: parse_record_key(&required_string(row, 1)?)?,
            display_text: optional_string(row, 2)?,
            reference_text: required_string(row, 3)?,
        });
    }
    Ok(edges)
}

fn read_aliases(connection: &Connection) -> Result<Vec<RecordAlias>, RecordLoadError> {
    let mut statement = connection
        .prepare(
            "SELECT canonical_record_key, alias_text, normalized_alias, source_kind, source_ref
             FROM record_aliases
             ORDER BY canonical_record_key, normalized_alias, source_kind, source_ref",
        )
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
            canonical_record_key: parse_record_key(&required_string(row, 0)?)?,
            alias_text: required_string(row, 1)?,
            normalized_alias: required_string(row, 2)?,
            source: parse_alias_source(&required_string(row, 3)?)?,
            source_ref: required_string(row, 4)?,
        });
    }
    Ok(aliases)
}

fn read_remaster_links(connection: &Connection) -> Result<Vec<RemasterLink>, RecordLoadError> {
    let mut statement = connection
        .prepare(
            "SELECT remaster_record_key, legacy_record_key, source_kind, source_ref
             FROM remaster_links
             ORDER BY remaster_record_key, legacy_record_key, source_kind, source_ref",
        )
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
            remaster_record_key: parse_record_key(&required_string(row, 0)?)?,
            legacy_record_key: parse_record_key(&required_string(row, 1)?)?,
            source: parse_remaster_link_source(&required_string(row, 2)?)?,
            source_ref: required_string(row, 3)?,
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

fn required_string(row: &Row<'_>, index: usize) -> Result<String, RecordLoadError> {
    row.get(index)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))
}

fn optional_string(row: &Row<'_>, index: usize) -> Result<Option<String>, RecordLoadError> {
    row.get(index)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))
}

fn bool_column(name: &'static str, row: &Row<'_>, index: usize) -> Result<bool, RecordLoadError> {
    match row
        .get::<_, i64>(index)
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

fn parse_record_key(value: &str) -> Result<RecordKey, RecordLoadError> {
    RecordKey::parse(value).map_err(invalid_parse("record_key"))
}

fn parse_record_family(value: &str) -> Result<RecordFamily, RecordLoadError> {
    RecordFamily::from_canonical(value)
        .ok_or_else(|| invalid_value("records.record_family", value.to_string()))
}

fn parse_publication_family(value: &str) -> Result<PublicationFamily, RecordLoadError> {
    match value {
        "core" => Ok(PublicationFamily::Core),
        "rules" => Ok(PublicationFamily::Rules),
        "adventure" => Ok(PublicationFamily::Adventure),
        "unknown" => Ok(PublicationFamily::Unknown),
        _ => Err(invalid_value(
            "records.publication_family",
            value.to_string(),
        )),
    }
}

fn parse_metric_domain(value: &str) -> Result<MetricDomain, RecordLoadError> {
    match value {
        "actor" => Ok(MetricDomain::Actor),
        "item" => Ok(MetricDomain::Item),
        _ => Err(invalid_value(
            "record_metrics.metric_domain",
            value.to_string(),
        )),
    }
}

fn parse_alias_source(value: &str) -> Result<AliasSource, RecordLoadError> {
    match value {
        "remaster_journal" => Ok(AliasSource::RemasterJournal),
        "migration" => Ok(AliasSource::Migration),
        "compendium_source" => Ok(AliasSource::CompendiumSource),
        _ => Err(invalid_value(
            "record_aliases.source_kind",
            value.to_string(),
        )),
    }
}

fn parse_remaster_link_source(value: &str) -> Result<RemasterLinkSource, RecordLoadError> {
    match value {
        "remaster_journal" => Ok(RemasterLinkSource::RemasterJournal),
        "migration" => Ok(RemasterLinkSource::Migration),
        _ => Err(invalid_value(
            "remaster_links.source_kind",
            value.to_string(),
        )),
    }
}

fn parse_time_kind(value: &str) -> Result<TimeKind, RecordLoadError> {
    match value {
        "actions" => Ok(TimeKind::Actions),
        "free" => Ok(TimeKind::Free),
        "reaction" => Ok(TimeKind::Reaction),
        "duration" => Ok(TimeKind::Duration),
        "variable" => Ok(TimeKind::Variable),
        "other" => Ok(TimeKind::Other),
        _ => Err(invalid_value("time.kind", value.to_string())),
    }
}

fn parse_time_unit(value: &str) -> Result<TimeUnit, RecordLoadError> {
    match value {
        "round" => Ok(TimeUnit::Round),
        "minute" => Ok(TimeUnit::Minute),
        "hour" => Ok(TimeUnit::Hour),
        "day" => Ok(TimeUnit::Day),
        "week" => Ok(TimeUnit::Week),
        "month" => Ok(TimeUnit::Month),
        "year" => Ok(TimeUnit::Year),
        _ => Err(invalid_value("time.unit", value.to_string())),
    }
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
