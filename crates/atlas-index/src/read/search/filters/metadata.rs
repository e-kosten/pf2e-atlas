use crate::artifact::inventory::{
    Column, Table, actor_records, item_records, record_traits, records, spell_records,
};
use atlas_domain::metadata::{
    MetadataBooleanField, MetadataBooleanMatch, MetadataEnumStringField, MetadataNumberField,
    MetadataNumberMatch, MetadataPredicate, MetadataSetField, MetadataSetMatch,
    MetadataStringMatch, MetadataTextMatch, MetadataTextStringField,
};

use super::FilterCompiler;
use super::error::FilterCompileError;
use super::sql_render::{
    aliased_column, json_array_contains_sql, json_array_empty_sql, record_column,
    record_key_column, side_table_for_column,
};

impl FilterCompiler {
    pub(super) fn metadata_predicate(
        &mut self,
        predicate: &MetadataPredicate,
    ) -> Result<String, FilterCompileError> {
        match predicate {
            MetadataPredicate::Set { field, r#match } => {
                self.metadata_set_predicate(*field, r#match)
            }
            MetadataPredicate::EnumString { field, r#match } => {
                self.metadata_enum_predicate(*field, r#match)
            }
            MetadataPredicate::Text { field, r#match } => {
                self.metadata_text_predicate(*field, r#match)
            }
            MetadataPredicate::Number { field, r#match } => {
                self.metadata_number_predicate(*field, *r#match)
            }
            MetadataPredicate::Boolean { field, r#match } => {
                self.metadata_boolean_predicate(*field, *r#match)
            }
        }
    }

    fn metadata_set_predicate(
        &mut self,
        field: MetadataSetField,
        r#match: &MetadataSetMatch,
    ) -> Result<String, FilterCompileError> {
        let set = match field {
            MetadataSetField::Traits => SetStorage::Rows {
                table: record_traits::TABLE,
                key_column: record_traits::columns::RECORD_KEY,
                value_column: record_traits::columns::TRAIT,
            },
            MetadataSetField::TaxonomyFamilies => {
                SetStorage::JsonColumn(records::columns::TAXONOMY_FAMILIES_JSON)
            }
            MetadataSetField::Traditions => SetStorage::JsonSideTable {
                table: spell_records::TABLE,
                column: spell_records::columns::TRADITIONS_JSON,
            },
            MetadataSetField::SpellKinds => SetStorage::JsonSideTable {
                table: spell_records::TABLE,
                column: spell_records::columns::SPELL_KINDS_JSON,
            },
            MetadataSetField::DamageTypes => {
                return self.multi_json_side_table_set(
                    r#match,
                    &[
                        (
                            item_records::TABLE,
                            item_records::columns::DAMAGE_TYPES_JSON,
                        ),
                        (
                            spell_records::TABLE,
                            spell_records::columns::DAMAGE_TYPES_JSON,
                        ),
                    ],
                );
            }
            MetadataSetField::Languages => SetStorage::JsonSideTable {
                table: actor_records::TABLE,
                column: actor_records::columns::LANGUAGES_JSON,
            },
            MetadataSetField::SpeedTypes => SetStorage::JsonSideTable {
                table: actor_records::TABLE,
                column: actor_records::columns::SPEED_TYPES_JSON,
            },
            MetadataSetField::Senses => SetStorage::JsonSideTable {
                table: actor_records::TABLE,
                column: actor_records::columns::SENSES_JSON,
            },
            MetadataSetField::Immunities => SetStorage::JsonSideTable {
                table: actor_records::TABLE,
                column: actor_records::columns::IMMUNITIES_JSON,
            },
            MetadataSetField::Resistances => SetStorage::JsonSideTable {
                table: actor_records::TABLE,
                column: actor_records::columns::RESISTANCES_JSON,
            },
            MetadataSetField::Weaknesses => SetStorage::JsonSideTable {
                table: actor_records::TABLE,
                column: actor_records::columns::WEAKNESSES_JSON,
            },
            MetadataSetField::DisableSkills => SetStorage::JsonSideTable {
                table: actor_records::TABLE,
                column: actor_records::columns::DISABLE_SKILLS_JSON,
            },
            MetadataSetField::VariantAxes => {
                SetStorage::JsonColumn(records::columns::VARIANT_AXES_JSON)
            }
            MetadataSetField::DerivedTags => {
                return Err(FilterCompileError::Unsupported {
                    filter: "metadata.set.derived_tags".to_string(),
                });
            }
        };
        self.set_predicate(set, r#match)
    }

    fn set_predicate(
        &mut self,
        set: SetStorage,
        r#match: &MetadataSetMatch,
    ) -> Result<String, FilterCompileError> {
        match r#match {
            MetadataSetMatch::Includes { value } => Ok(match set {
                SetStorage::Rows {
                    table,
                    key_column,
                    value_column,
                } => format!(
                    "EXISTS (SELECT 1 FROM {table} s WHERE {key_column} = {record_key} AND {value_column} = {})",
                    self.text(value),
                    table = table.name(),
                    key_column = aliased_column("s", key_column),
                    record_key = record_column(records::columns::RECORD_KEY),
                    value_column = aliased_column("s", value_column),
                ),
                SetStorage::JsonColumn(column) => {
                    json_array_contains_sql(&record_column(column), &self.text(value))
                }
                SetStorage::JsonSideTable { table, column } => format!(
                    "EXISTS (SELECT 1 FROM {table} s WHERE {side_record_key} = {record_key} AND {})",
                    json_array_contains_sql(&aliased_column("s", column), &self.text(value)),
                    table = table.name(),
                    side_record_key = aliased_column("s", record_key_column(table)),
                    record_key = record_column(records::columns::RECORD_KEY),
                ),
            }),
            MetadataSetMatch::IsNull => Ok(match set {
                SetStorage::Rows {
                    table, key_column, ..
                } => format!(
                    "NOT EXISTS (SELECT 1 FROM {table} s WHERE {key_column} = {record_key})",
                    table = table.name(),
                    key_column = aliased_column("s", key_column),
                    record_key = record_column(records::columns::RECORD_KEY),
                ),
                SetStorage::JsonColumn(column) => json_array_empty_sql(&record_column(column)),
                SetStorage::JsonSideTable { table, column } => format!(
                    "NOT EXISTS (SELECT 1 FROM {table} s WHERE {side_record_key} = {record_key} AND NOT {})",
                    json_array_empty_sql(&aliased_column("s", column)),
                    table = table.name(),
                    side_record_key = aliased_column("s", record_key_column(table)),
                    record_key = record_column(records::columns::RECORD_KEY),
                ),
            }),
            MetadataSetMatch::IsNotNull => Ok(match set {
                SetStorage::Rows {
                    table, key_column, ..
                } => {
                    format!(
                        "EXISTS (SELECT 1 FROM {table} s WHERE {key_column} = {record_key})",
                        table = table.name(),
                        key_column = aliased_column("s", key_column),
                        record_key = record_column(records::columns::RECORD_KEY),
                    )
                }
                SetStorage::JsonColumn(column) => {
                    format!("NOT ({})", json_array_empty_sql(&record_column(column)))
                }
                SetStorage::JsonSideTable { table, column } => format!(
                    "EXISTS (SELECT 1 FROM {table} s WHERE {side_record_key} = {record_key} AND NOT {})",
                    json_array_empty_sql(&aliased_column("s", column)),
                    table = table.name(),
                    side_record_key = aliased_column("s", record_key_column(table)),
                    record_key = record_column(records::columns::RECORD_KEY),
                ),
            }),
        }
    }

    fn multi_json_side_table_set(
        &mut self,
        r#match: &MetadataSetMatch,
        tables: &[(Table, Column)],
    ) -> Result<String, FilterCompileError> {
        let predicates = tables
            .iter()
            .map(|(table, column)| {
                self.set_predicate(
                    SetStorage::JsonSideTable {
                        table: *table,
                        column: *column,
                    },
                    r#match,
                )
                .map(|sql| format!("({sql})"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let joiner = match r#match {
            MetadataSetMatch::IsNull => " AND ",
            MetadataSetMatch::Includes { .. } | MetadataSetMatch::IsNotNull => " OR ",
        };
        Ok(predicates.join(joiner))
    }

    fn metadata_enum_predicate(
        &mut self,
        field: MetadataEnumStringField,
        r#match: &MetadataStringMatch,
    ) -> Result<String, FilterCompileError> {
        let column = match field {
            MetadataEnumStringField::PackName => records::columns::PACK_NAME,
            MetadataEnumStringField::PackLabel => records::columns::PACK_LABEL,
            MetadataEnumStringField::PublicationCategory => records::columns::PUBLICATION_FAMILY,
            MetadataEnumStringField::Size => actor_records::columns::SIZE,
            MetadataEnumStringField::Usage => records::columns::SYSTEM_USAGE,
            MetadataEnumStringField::SystemCategory => records::columns::SYSTEM_CATEGORY,
            MetadataEnumStringField::SystemGroup => records::columns::SYSTEM_GROUP,
            MetadataEnumStringField::FoundryRecordType => records::columns::FOUNDRY_RECORD_TYPE,
            MetadataEnumStringField::BaseItem => records::columns::SYSTEM_BASE_ITEM,
            MetadataEnumStringField::Hands => item_records::columns::HANDS_REQUIREMENT,
            MetadataEnumStringField::SaveType => spell_records::columns::SAVE_TYPE,
            MetadataEnumStringField::AreaType => spell_records::columns::AREA_TYPE,
            MetadataEnumStringField::DurationUnit => records::columns::DURATION_UNIT,
            MetadataEnumStringField::Rarity => records::columns::RARITY,
            MetadataEnumStringField::VariantGroupKey => records::columns::VARIANT_GROUP_KEY,
        };
        self.with_field_source(column, |compiler, qualified_column| {
            compiler.string_operator(qualified_column, r#match)
        })
    }

    fn metadata_text_predicate(
        &mut self,
        field: MetadataTextStringField,
        r#match: &MetadataTextMatch,
    ) -> Result<String, FilterCompileError> {
        let column = match field {
            MetadataTextStringField::PublicationTitle => records::columns::PUBLICATION_TITLE,
            MetadataTextStringField::RangeText => spell_records::columns::RANGE_TEXT,
            MetadataTextStringField::DurationText => records::columns::DURATION_TEXT,
            MetadataTextStringField::TargetText => spell_records::columns::TARGET_TEXT,
            MetadataTextStringField::DisableText => actor_records::columns::DISABLE_TEXT,
            MetadataTextStringField::VariantBaseName => records::columns::VARIANT_BASE_NAME,
            MetadataTextStringField::VariantLabel => records::columns::VARIANT_LABEL,
        };
        self.with_field_source(column, |compiler, qualified_column| {
            compiler.text_operator(qualified_column, r#match)
        })
    }

    fn metadata_number_predicate(
        &mut self,
        field: MetadataNumberField,
        r#match: MetadataNumberMatch,
    ) -> Result<String, FilterCompileError> {
        let column = match field {
            MetadataNumberField::Level => records::columns::LEVEL,
            MetadataNumberField::PriceCp => records::columns::PRICE_CP,
            MetadataNumberField::BulkValue => item_records::columns::BULK_VALUE,
            MetadataNumberField::ActionCost => records::columns::ACTIVATION_TIME_ACTIONS,
            MetadataNumberField::RangeValue => spell_records::columns::RANGE_VALUE,
            MetadataNumberField::AreaValue => spell_records::columns::AREA_VALUE,
            MetadataNumberField::Hands => {
                return Err(FilterCompileError::Unsupported {
                    filter: "metadata.number.hands".to_string(),
                });
            }
        };
        self.with_field_source(column, |compiler, qualified_column| {
            compiler.number_operator(qualified_column, r#match)
        })
    }

    fn metadata_boolean_predicate(
        &mut self,
        field: MetadataBooleanField,
        r#match: MetadataBooleanMatch,
    ) -> Result<String, FilterCompileError> {
        let column = match field {
            MetadataBooleanField::PublicationRemaster => records::columns::PUBLICATION_REMASTER,
            MetadataBooleanField::Sustained => spell_records::columns::SUSTAINED,
            MetadataBooleanField::BasicSave => spell_records::columns::BASIC_SAVE,
            MetadataBooleanField::IsComplex => actor_records::columns::IS_COMPLEX,
        };
        self.with_field_source(column, |compiler, qualified_column| {
            compiler.boolean_operator(qualified_column, r#match)
        })
    }

    fn with_field_source(
        &mut self,
        column: Column,
        compile: impl FnOnce(&mut Self, &str) -> Result<String, FilterCompileError>,
    ) -> Result<String, FilterCompileError> {
        if let Some((alias, table)) = side_table_for_column(column) {
            let qualified_column = aliased_column(alias, column);
            let predicate = compile(self, &qualified_column)?;
            Ok(format!(
                "EXISTS (SELECT 1 FROM {table} {alias} WHERE {side_record_key} = {record_key} AND {predicate})",
                table = table.name(),
                side_record_key = aliased_column(alias, record_key_column(table)),
                record_key = record_column(records::columns::RECORD_KEY),
            ))
        } else {
            compile(self, &record_column(column))
        }
    }
}

#[derive(Clone, Copy)]
enum SetStorage {
    Rows {
        table: Table,
        key_column: Column,
        value_column: Column,
    },
    JsonColumn(Column),
    JsonSideTable {
        table: Table,
        column: Column,
    },
}
