use std::collections::BTreeMap;

use atlas_domain::RecordKey;
use atlas_record::{ActorSideData, ItemSideData, SpellSideData};
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::{Queryable, Selectable, SelectableHelper, SqliteConnection};

use crate::schema::{actor_records, item_records, spell_records};

use super::RecordLoadError;
use super::parse::json_string_array;

pub(super) fn read_actor_data(
    connection: &mut SqliteConnection,
) -> Result<BTreeMap<String, ActorSideData>, RecordLoadError> {
    let rows = actor_records::table
        .select(ActorRecordRow::as_select())
        .order(actor_records::record_key.asc())
        .load::<ActorRecordRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    actor_data_from_rows(rows)
}

pub(super) fn read_actor_data_by_keys(
    connection: &mut SqliteConnection,
    keys: &[RecordKey],
) -> Result<BTreeMap<String, ActorSideData>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    let key_strings = keys.iter().map(ToString::to_string).collect::<Vec<_>>();
    let rows = actor_records::table
        .filter(actor_records::record_key.eq_any(key_strings))
        .select(ActorRecordRow::as_select())
        .order(actor_records::record_key.asc())
        .load::<ActorRecordRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    actor_data_from_rows(rows)
}

pub(super) fn read_item_data(
    connection: &mut SqliteConnection,
) -> Result<BTreeMap<String, ItemSideData>, RecordLoadError> {
    let rows = item_records::table
        .select(ItemRecordRow::as_select())
        .order(item_records::record_key.asc())
        .load::<ItemRecordRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    item_data_from_rows(rows)
}

pub(super) fn read_item_data_by_keys(
    connection: &mut SqliteConnection,
    keys: &[RecordKey],
) -> Result<BTreeMap<String, ItemSideData>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    let key_strings = keys.iter().map(ToString::to_string).collect::<Vec<_>>();
    let rows = item_records::table
        .filter(item_records::record_key.eq_any(key_strings))
        .select(ItemRecordRow::as_select())
        .order(item_records::record_key.asc())
        .load::<ItemRecordRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    item_data_from_rows(rows)
}

pub(super) fn read_spell_data(
    connection: &mut SqliteConnection,
) -> Result<BTreeMap<String, SpellSideData>, RecordLoadError> {
    let rows = spell_records::table
        .select(SpellRecordRow::as_select())
        .order(spell_records::record_key.asc())
        .load::<SpellRecordRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    spell_data_from_rows(rows)
}

pub(super) fn read_spell_data_by_keys(
    connection: &mut SqliteConnection,
    keys: &[RecordKey],
) -> Result<BTreeMap<String, SpellSideData>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    let key_strings = keys.iter().map(ToString::to_string).collect::<Vec<_>>();
    let rows = spell_records::table
        .filter(spell_records::record_key.eq_any(key_strings))
        .select(SpellRecordRow::as_select())
        .order(spell_records::record_key.asc())
        .load::<SpellRecordRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    spell_data_from_rows(rows)
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = actor_records)]
#[diesel(check_for_backend(Sqlite))]
struct ActorRecordRow {
    record_key: String,
    size: Option<String>,
    languages_json: String,
    speed_types_json: String,
    senses_json: String,
    immunities_json: String,
    resistances_json: String,
    weaknesses_json: String,
    disable_text: Option<String>,
    disable_skills_json: String,
    is_complex: bool,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = item_records)]
#[diesel(check_for_backend(Sqlite))]
struct ItemRecordRow {
    record_key: String,
    system_category: Option<String>,
    system_base_item: Option<String>,
    system_group: Option<String>,
    system_usage: Option<String>,
    price_cp: Option<i64>,
    bulk_value: Option<f64>,
    hands_requirement: Option<String>,
    damage_types_json: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = spell_records)]
#[diesel(check_for_backend(Sqlite))]
struct SpellRecordRow {
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
    basic_save: bool,
    damage_types_json: String,
}

fn actor_data_from_rows(
    rows: Vec<ActorRecordRow>,
) -> Result<BTreeMap<String, ActorSideData>, RecordLoadError> {
    let mut values = BTreeMap::new();
    for row in rows {
        values.insert(
            row.record_key,
            ActorSideData {
                size: row.size,
                languages: json_string_array("actor_records.languages_json", &row.languages_json)?,
                speed_types: json_string_array(
                    "actor_records.speed_types_json",
                    &row.speed_types_json,
                )?,
                senses: json_string_array("actor_records.senses_json", &row.senses_json)?,
                immunities: json_string_array(
                    "actor_records.immunities_json",
                    &row.immunities_json,
                )?,
                resistances: json_string_array(
                    "actor_records.resistances_json",
                    &row.resistances_json,
                )?,
                weaknesses: json_string_array(
                    "actor_records.weaknesses_json",
                    &row.weaknesses_json,
                )?,
                disable_text: row.disable_text,
                disable_skills: json_string_array(
                    "actor_records.disable_skills_json",
                    &row.disable_skills_json,
                )?,
                is_complex: row.is_complex,
            },
        );
    }
    Ok(values)
}

fn item_data_from_rows(
    rows: Vec<ItemRecordRow>,
) -> Result<BTreeMap<String, ItemSideData>, RecordLoadError> {
    let mut values = BTreeMap::new();
    for row in rows {
        values.insert(
            row.record_key,
            ItemSideData {
                system_category: row.system_category,
                system_base_item: row.system_base_item,
                system_group: row.system_group,
                system_usage: row.system_usage,
                price_cp: row.price_cp,
                bulk_value: row.bulk_value,
                hands_requirement: row.hands_requirement,
                damage_types: json_string_array(
                    "item_records.damage_types_json",
                    &row.damage_types_json,
                )?,
            },
        );
    }
    Ok(values)
}

fn spell_data_from_rows(
    rows: Vec<SpellRecordRow>,
) -> Result<BTreeMap<String, SpellSideData>, RecordLoadError> {
    let mut values = BTreeMap::new();
    for row in rows {
        values.insert(
            row.record_key,
            SpellSideData {
                traditions: json_string_array(
                    "spell_records.traditions_json",
                    &row.traditions_json,
                )?,
                spell_kinds: json_string_array(
                    "spell_records.spell_kinds_json",
                    &row.spell_kinds_json,
                )?,
                range_text: row.range_text,
                range_value: row.range_value,
                target_text: row.target_text,
                area_type: row.area_type,
                area_value: row.area_value,
                save_type: row.save_type,
                sustained: row.sustained,
                basic_save: row.basic_save,
                damage_types: json_string_array(
                    "spell_records.damage_types_json",
                    &row.damage_types_json,
                )?,
            },
        );
    }
    Ok(values)
}
