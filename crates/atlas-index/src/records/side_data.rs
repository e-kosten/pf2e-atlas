use std::collections::BTreeMap;

use atlas_artifact::schema::{
    ACTOR_RECORD_COLUMNS, ITEM_RECORD_COLUMNS, SPELL_RECORD_COLUMNS, actor_record_select_sql,
    actor_records, item_record_select_sql, item_records, spell_record_select_sql, spell_records,
};
use atlas_domain::RecordKey;
use atlas_record::{ActorSideData, ItemSideData, SpellSideData};
use rusqlite::{Connection, params_from_iter, types::Value};

use super::RecordLoadError;
use super::parse::{
    bool_column, json_string_array, optional_f64, optional_i64, optional_string, required_string,
};
use super::scoped::{key_parameters, select_by_keys_sql};

pub(super) fn read_actor_data(
    connection: &Connection,
) -> Result<BTreeMap<String, ActorSideData>, RecordLoadError> {
    read_actor_data_from_sql(connection, &actor_record_select_sql(), Vec::new())
}

pub(super) fn read_actor_data_by_keys(
    connection: &Connection,
    keys: &[RecordKey],
) -> Result<BTreeMap<String, ActorSideData>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    let parameters = key_parameters(keys);
    let sql = select_by_keys_sql(
        actor_records::TABLE.name(),
        ACTOR_RECORD_COLUMNS,
        actor_records::columns::RECORD_KEY.name(),
        &[actor_records::columns::RECORD_KEY.name()],
        parameters.len(),
    );
    read_actor_data_from_sql(connection, &sql, parameters)
}

fn read_actor_data_from_sql(
    connection: &Connection,
    sql: &str,
    parameters: Vec<Value>,
) -> Result<BTreeMap<String, ActorSideData>, RecordLoadError> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut values = BTreeMap::new();
    let mut rows = statement
        .query(params_from_iter(parameters.iter()))
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

pub(super) fn read_item_data(
    connection: &Connection,
) -> Result<BTreeMap<String, ItemSideData>, RecordLoadError> {
    read_item_data_from_sql(connection, &item_record_select_sql(), Vec::new())
}

pub(super) fn read_item_data_by_keys(
    connection: &Connection,
    keys: &[RecordKey],
) -> Result<BTreeMap<String, ItemSideData>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    let parameters = key_parameters(keys);
    let sql = select_by_keys_sql(
        item_records::TABLE.name(),
        ITEM_RECORD_COLUMNS,
        item_records::columns::RECORD_KEY.name(),
        &[item_records::columns::RECORD_KEY.name()],
        parameters.len(),
    );
    read_item_data_from_sql(connection, &sql, parameters)
}

fn read_item_data_from_sql(
    connection: &Connection,
    sql: &str,
    parameters: Vec<Value>,
) -> Result<BTreeMap<String, ItemSideData>, RecordLoadError> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut values = BTreeMap::new();
    let mut rows = statement
        .query(params_from_iter(parameters.iter()))
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

pub(super) fn read_spell_data(
    connection: &Connection,
) -> Result<BTreeMap<String, SpellSideData>, RecordLoadError> {
    read_spell_data_from_sql(connection, &spell_record_select_sql(), Vec::new())
}

pub(super) fn read_spell_data_by_keys(
    connection: &Connection,
    keys: &[RecordKey],
) -> Result<BTreeMap<String, SpellSideData>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    let parameters = key_parameters(keys);
    let sql = select_by_keys_sql(
        spell_records::TABLE.name(),
        SPELL_RECORD_COLUMNS,
        spell_records::columns::RECORD_KEY.name(),
        &[spell_records::columns::RECORD_KEY.name()],
        parameters.len(),
    );
    read_spell_data_from_sql(connection, &sql, parameters)
}

fn read_spell_data_from_sql(
    connection: &Connection,
    sql: &str,
    parameters: Vec<Value>,
) -> Result<BTreeMap<String, SpellSideData>, RecordLoadError> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut values = BTreeMap::new();
    let mut rows = statement
        .query(params_from_iter(parameters.iter()))
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
