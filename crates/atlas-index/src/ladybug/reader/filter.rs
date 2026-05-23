use crate::FilteredRecordSort;
use atlas_domain::{
    MetadataBooleanField, MetadataBooleanMatch, MetadataEnumStringField, MetadataNumberField,
    MetadataNumberMatch, MetadataPredicate, MetadataSetField, MetadataSetMatch,
    MetadataStringMatch, MetadataTextMatch, MetadataTextStringField, ScalarValue, SearchFilterNode,
};

use super::{LadybugIndexReaderError, string_literal, unsupported};

pub(crate) struct CompiledScope {
    matches: Vec<String>,
    predicates: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct LadybugValueProjection {
    pub(crate) match_clause: String,
    pub(crate) value_expr: String,
    pub(crate) value_kind: ProjectionValueKind,
    pub(crate) extra_predicate: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProjectionValueKind {
    String,
    Number,
    Boolean,
}

impl LadybugValueProjection {
    fn scalar(property: &'static str, value_kind: ProjectionValueKind) -> Self {
        Self {
            match_clause: String::new(),
            value_expr: format!("record.{property}"),
            value_kind,
            extra_predicate: None,
        }
    }

    fn relationship(
        match_clause: &'static str,
        value_expr: &'static str,
        value_kind: ProjectionValueKind,
    ) -> Self {
        Self {
            match_clause: match_clause.to_string(),
            value_expr: value_expr.to_string(),
            value_kind,
            extra_predicate: None,
        }
    }

    pub(crate) fn relationship_with_predicate(
        match_clause: String,
        value_expr: &'static str,
        value_kind: ProjectionValueKind,
        extra_predicate: String,
    ) -> Self {
        Self {
            match_clause,
            value_expr: value_expr.to_string(),
            value_kind,
            extra_predicate: Some(extra_predicate),
        }
    }

    pub(crate) fn non_empty_string_predicate(&self, _alias: &str) -> &'static str {
        if self.value_kind == ProjectionValueKind::String {
            " AND value <> ''"
        } else {
            ""
        }
    }

    pub(crate) fn extra_where_clause(&self) -> String {
        self.extra_predicate
            .as_ref()
            .map(|predicate| format!("WHERE {predicate}"))
            .unwrap_or_default()
    }
}

impl CompiledScope {
    pub(crate) fn match_with_where(&self, record_alias: &str) -> String {
        format!(
            "MATCH ({record_alias}:Record) {} {}",
            self.optional_match_suffix(record_alias),
            self.where_clause(record_alias)
        )
    }

    pub(crate) fn optional_match_prefix(&self) -> String {
        self.matches
            .iter()
            .map(|pattern| format!("MATCH {pattern}"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub(crate) fn optional_match_suffix(&self, record_alias: &str) -> String {
        let matches = self
            .matches
            .iter()
            .map(|pattern| pattern.replace("(record)", &format!("({record_alias})")))
            .collect::<Vec<_>>()
            .join(" ");
        if matches.is_empty() {
            String::new()
        } else {
            format!("MATCH {matches}")
        }
    }

    pub(crate) fn where_clause(&self, record_alias: &str) -> String {
        let mut predicates = vec![format!("{record_alias}.is_default_visible")];
        predicates.extend(
            self.predicates
                .iter()
                .map(|predicate| predicate.replace("record.", &format!("{record_alias}."))),
        );
        format!("WHERE {}", predicates.join(" AND "))
    }
}

pub(crate) fn ladybug_projection(field: &str) -> Option<LadybugValueProjection> {
    use ProjectionValueKind::{Boolean, Number, String};
    Some(match field {
        "record_family" => LadybugValueProjection::scalar("record_family", String),
        "pack_name" => LadybugValueProjection::relationship(
            "MATCH (record)-[:FROM_PACK]->(value_pack:Pack)",
            "value_pack.pack_name",
            String,
        ),
        "pack_label" => LadybugValueProjection::relationship(
            "MATCH (record)-[:FROM_PACK]->(value_pack:Pack)",
            "value_pack.pack_label",
            String,
        ),
        "foundry_record_type" => LadybugValueProjection::scalar("foundry_record_type", String),
        "publication_title" => LadybugValueProjection::scalar("publication_title", String),
        "publication_family" => LadybugValueProjection::scalar("publication_family", String),
        "publication_remaster" => LadybugValueProjection::scalar("publication_remaster", Boolean),
        "rarity" => LadybugValueProjection::scalar("rarity", String),
        "level" => LadybugValueProjection::scalar("level", Number),
        "action_cost" => LadybugValueProjection::scalar("activation_time_actions", Number),
        "traits" => LadybugValueProjection::relationship(
            "MATCH (record)-[:HAS_TRAIT]->(value_trait:Trait)",
            "value_trait.name",
            String,
        ),
        "taxonomy_families" => filter_value_projection("taxonomy_families"),
        "traditions" => filter_value_projection("traditions"),
        "spell_kinds" => filter_value_projection("spell_kinds"),
        "damage_types" => filter_value_projection("damage_types"),
        "languages" => filter_value_projection("languages"),
        "speed_types" => filter_value_projection("speed_types"),
        "senses" => filter_value_projection("senses"),
        "immunities" => filter_value_projection("immunities"),
        "resistances" => filter_value_projection("resistances"),
        "weaknesses" => filter_value_projection("weaknesses"),
        "disable_skills" => filter_value_projection("disable_skills"),
        "variant_axes" => filter_value_projection("variant_axes"),
        "size" => LadybugValueProjection::scalar("actor_size", String),
        "usage" => LadybugValueProjection::scalar("system_usage", String),
        "item_group" => LadybugValueProjection::scalar("system_group", String),
        "item_category" => LadybugValueProjection::scalar("system_category", String),
        "base_item" => LadybugValueProjection::scalar("system_base_item", String),
        "hands" => LadybugValueProjection::scalar("item_hands_requirement", String),
        "save_type" => LadybugValueProjection::scalar("spell_save_type", String),
        "area_type" => LadybugValueProjection::scalar("spell_area_type", String),
        "duration_unit" => LadybugValueProjection::scalar("duration_unit", String),
        "sustained" => LadybugValueProjection::scalar("spell_sustained", Boolean),
        "basic_save" => LadybugValueProjection::scalar("spell_basic_save", Boolean),
        "is_complex" => LadybugValueProjection::scalar("actor_is_complex", Boolean),
        "price_cp" => LadybugValueProjection::scalar("price_cp", Number),
        "bulk_value" => LadybugValueProjection::scalar("item_bulk_value", Number),
        "range_value" => LadybugValueProjection::scalar("spell_range_value", Number),
        "area_value" => LadybugValueProjection::scalar("spell_area_value", Number),
        "variant_group_key" => LadybugValueProjection::scalar("variant_group_key", String),
        "range_text" => LadybugValueProjection::scalar("spell_range_text", String),
        "duration_text" => LadybugValueProjection::scalar("duration_text", String),
        "target_text" => LadybugValueProjection::scalar("spell_target_text", String),
        "disable_text" => LadybugValueProjection::scalar("actor_disable_text", String),
        "variant_base_name" => LadybugValueProjection::scalar("variant_base_name", String),
        "variant_label" => LadybugValueProjection::scalar("variant_label", String),
        _ => return None,
    })
}

fn filter_value_projection(field: &'static str) -> LadybugValueProjection {
    LadybugValueProjection::relationship_with_predicate(
        "MATCH (record)-[:HAS_FILTER_VALUE]->(value_filter:FilterValue)".to_string(),
        "value_filter.value",
        ProjectionValueKind::String,
        format!("value_filter.field = {}", string_literal(field)),
    )
}

pub(crate) fn compile_scope(
    filter: Option<&SearchFilterNode>,
) -> Result<CompiledScope, LadybugIndexReaderError> {
    let mut matches = Vec::new();
    let mut predicates = Vec::new();
    if let Some(filter) = filter {
        predicates.push(compile_filter(filter, &mut matches)?);
    }
    Ok(CompiledScope {
        matches,
        predicates,
    })
}

fn compile_filter(
    filter: &SearchFilterNode,
    matches: &mut Vec<String>,
) -> Result<String, LadybugIndexReaderError> {
    match filter {
        SearchFilterNode::RecordFamily { value } => Ok(format!(
            "record.record_family = {}",
            string_literal(value.as_str())
        )),
        SearchFilterNode::LinksTo { target } => {
            matches.push(format!(
                "(record)-[:REFERENCES]->(:Record {{record_key: {}}})",
                string_literal(&target.to_string())
            ));
            Ok("true".to_string())
        }
        SearchFilterNode::LinkedFrom { source } => {
            matches.push(format!(
                "(:Record {{record_key: {}}})-[:REFERENCES]->(record)",
                string_literal(&source.to_string())
            ));
            Ok("true".to_string())
        }
        SearchFilterNode::MetadataPredicate { predicate } => compile_metadata(predicate, matches),
        SearchFilterNode::Metric { metric, r#match } => {
            let metric_alias = format!("metric_{}", matches.len());
            matches.push(format!(
                "(record)-[{metric_alias}_rel:HAS_METRIC]->({metric_alias}:Metric {{metric_key: {}}})",
                string_literal(metric)
            ));
            Ok(match r#match {
                atlas_domain::MetricMatch::Eq {
                    value: ScalarValue::Number(value),
                } => {
                    format!("{metric_alias}_rel.number_value = {value}")
                }
                atlas_domain::MetricMatch::NotEq {
                    value: ScalarValue::Number(value),
                } => {
                    format!("{metric_alias}_rel.number_value <> {value}")
                }
                atlas_domain::MetricMatch::Eq { .. } | atlas_domain::MetricMatch::NotEq { .. } => {
                    return unsupported("non-numeric metric equality");
                }
                atlas_domain::MetricMatch::Gt { value } => {
                    format!("{metric_alias}_rel.number_value > {value}")
                }
                atlas_domain::MetricMatch::Gte { value } => {
                    format!("{metric_alias}_rel.number_value >= {value}")
                }
                atlas_domain::MetricMatch::Lt { value } => {
                    format!("{metric_alias}_rel.number_value < {value}")
                }
                atlas_domain::MetricMatch::Lte { value } => {
                    format!("{metric_alias}_rel.number_value <= {value}")
                }
            })
        }
        SearchFilterNode::MetricCompare { .. } => unsupported("metric comparison filters"),
        SearchFilterNode::AnyOf { children } => match compile_set_include_any(children, matches) {
            Some(filter) => filter,
            None => boolean_filter(children, " OR ", matches),
        },
        SearchFilterNode::AllOf { children } => boolean_filter(children, " AND ", matches),
        SearchFilterNode::Not { child } => Ok(format!("NOT ({})", compile_filter(child, matches)?)),
    }
}

fn boolean_filter(
    children: &[SearchFilterNode],
    joiner: &str,
    matches: &mut Vec<String>,
) -> Result<String, LadybugIndexReaderError> {
    if children.is_empty() {
        return Ok(if joiner.contains("AND") {
            "true"
        } else {
            "false"
        }
        .to_string());
    }
    let parts = children
        .iter()
        .map(|child| compile_filter(child, matches).map(|value| format!("({value})")))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(parts.join(joiner))
}

fn compile_set_include_any(
    children: &[SearchFilterNode],
    matches: &mut Vec<String>,
) -> Option<Result<String, LadybugIndexReaderError>> {
    let mut values = Vec::new();
    let mut field = None;
    for child in children {
        let SearchFilterNode::MetadataPredicate {
            predicate:
                MetadataPredicate::Set {
                    field: child_field,
                    r#match: MetadataSetMatch::Includes { value },
                },
        } = child
        else {
            return None;
        };
        if field.is_some_and(|field| field != *child_field) {
            return None;
        }
        field = Some(*child_field);
        values.push(value.clone());
    }
    let field = field?;
    let alias = format!("set_any_{}", matches.len());
    let value_list = values
        .iter()
        .map(|value| string_literal(value))
        .collect::<Vec<_>>()
        .join(", ");
    match field {
        MetadataSetField::Traits => {
            matches.push(format!("(record)-[:HAS_TRAIT]->({alias}:Trait)"));
            Some(Ok(format!("{alias}.name IN [{value_list}]")))
        }
        field => match filter_value_field_name(field) {
            Some(field_name) => {
                matches.push(format!(
                    "(record)-[:HAS_FILTER_VALUE]->({alias}:FilterValue)"
                ));
                Some(Ok(format!(
                    "{alias}.field = {} AND {alias}.value IN [{value_list}]",
                    string_literal(field_name)
                )))
            }
            None => Some(unsupported("this metadata set filter")),
        },
    }
}

fn compile_metadata(
    predicate: &MetadataPredicate,
    matches: &mut Vec<String>,
) -> Result<String, LadybugIndexReaderError> {
    match predicate {
        MetadataPredicate::Set { field, r#match } => compile_set_metadata(*field, r#match, matches),
        MetadataPredicate::EnumString { field, r#match } => {
            compile_enum_string_metadata(*field, r#match, matches)
        }
        MetadataPredicate::Text { field, r#match } => compile_text_metadata(*field, r#match),
        MetadataPredicate::Number { field, r#match } => {
            let Some(column) = number_field_column(*field) else {
                return unsupported("this metadata number filter");
            };
            number_match(column, *r#match)
        }
        MetadataPredicate::Boolean { field, r#match } => {
            let Some(column) = boolean_field_column(*field) else {
                return unsupported("this metadata boolean filter");
            };
            boolean_match(column, *r#match)
        }
    }
}

fn compile_set_metadata(
    field: MetadataSetField,
    r#match: &MetadataSetMatch,
    matches: &mut Vec<String>,
) -> Result<String, LadybugIndexReaderError> {
    match (field, r#match) {
        (MetadataSetField::Traits, MetadataSetMatch::Includes { value }) => {
            matches.push(format!(
                "(record)-[:HAS_TRAIT]->(:Trait {{name: {}}})",
                string_literal(value)
            ));
            Ok("true".to_string())
        }
        (MetadataSetField::Traits, MetadataSetMatch::IsNotNull) => {
            matches.push("(record)-[:HAS_TRAIT]->(:Trait)".to_string());
            Ok("true".to_string())
        }
        (MetadataSetField::Traits, MetadataSetMatch::IsNull) => unsupported("null trait filters"),
        (field, MetadataSetMatch::Includes { value }) => {
            let Some(field_name) = filter_value_field_name(field) else {
                return unsupported("this metadata set filter");
            };
            matches.push(format!(
                "(record)-[:HAS_FILTER_VALUE]->(:FilterValue {{filter_value_key: {}}})",
                string_literal(&filter_value_key(field_name, value))
            ));
            Ok("true".to_string())
        }
        (field, MetadataSetMatch::IsNotNull) => {
            let Some(field_name) = filter_value_field_name(field) else {
                return unsupported("this metadata set filter");
            };
            let alias = format!("set_exists_{}", matches.len());
            matches.push(format!(
                "(record)-[:HAS_FILTER_VALUE]->({alias}:FilterValue)"
            ));
            Ok(format!("{alias}.field = {}", string_literal(field_name)))
        }
        (_, MetadataSetMatch::IsNull) => unsupported("null set filters"),
    }
}

fn compile_enum_string_metadata(
    field: MetadataEnumStringField,
    r#match: &MetadataStringMatch,
    matches: &mut Vec<String>,
) -> Result<String, LadybugIndexReaderError> {
    match field {
        MetadataEnumStringField::PackName => compile_pack_metadata("pack_name", r#match, matches),
        MetadataEnumStringField::PackLabel => compile_pack_metadata("pack_label", r#match, matches),
        field => {
            let Some(column) = enum_string_field_column(field) else {
                return unsupported("this metadata enum filter");
            };
            string_match(column, r#match)
        }
    }
}

fn compile_pack_metadata(
    column: &'static str,
    r#match: &MetadataStringMatch,
    matches: &mut Vec<String>,
) -> Result<String, LadybugIndexReaderError> {
    let alias = format!("pack_{}", matches.len());
    matches.push(format!("(record)-[:FROM_PACK]->({alias}:Pack)"));
    string_match(&format!("{alias}.{column}"), r#match)
}

fn compile_text_metadata(
    field: MetadataTextStringField,
    r#match: &MetadataTextMatch,
) -> Result<String, LadybugIndexReaderError> {
    let Some(column) = text_field_column(field) else {
        return unsupported("this metadata text filter");
    };
    text_match(column, r#match)
}

fn string_match(
    field: &str,
    r#match: &MetadataStringMatch,
) -> Result<String, LadybugIndexReaderError> {
    match r#match {
        MetadataStringMatch::Eq { value } => Ok(format!("{field} = {}", string_literal(value))),
        MetadataStringMatch::NotEq { value } => Ok(format!("{field} <> {}", string_literal(value))),
        MetadataStringMatch::IsNull => Ok(format!("{field} IS NULL")),
        MetadataStringMatch::IsNotNull => Ok(format!("{field} IS NOT NULL")),
    }
}

fn text_match(field: &str, r#match: &MetadataTextMatch) -> Result<String, LadybugIndexReaderError> {
    match r#match {
        MetadataTextMatch::Eq { value } => Ok(format!("{field} = {}", string_literal(value))),
        MetadataTextMatch::NotEq { value } => Ok(format!("{field} <> {}", string_literal(value))),
        MetadataTextMatch::Contains { value } => {
            Ok(format!("{field} CONTAINS {}", string_literal(value)))
        }
        MetadataTextMatch::NotContains { value } => {
            Ok(format!("NOT ({field} CONTAINS {})", string_literal(value)))
        }
        MetadataTextMatch::IsNull => Ok(format!("{field} IS NULL")),
        MetadataTextMatch::IsNotNull => Ok(format!("{field} IS NOT NULL")),
    }
}

fn filter_value_field_name(field: MetadataSetField) -> Option<&'static str> {
    Some(match field {
        MetadataSetField::Traits | MetadataSetField::DerivedTags => return None,
        MetadataSetField::TaxonomyFamilies => "taxonomy_families",
        MetadataSetField::Traditions => "traditions",
        MetadataSetField::SpellKinds => "spell_kinds",
        MetadataSetField::DamageTypes => "damage_types",
        MetadataSetField::Languages => "languages",
        MetadataSetField::SpeedTypes => "speed_types",
        MetadataSetField::Senses => "senses",
        MetadataSetField::Immunities => "immunities",
        MetadataSetField::Resistances => "resistances",
        MetadataSetField::Weaknesses => "weaknesses",
        MetadataSetField::DisableSkills => "disable_skills",
        MetadataSetField::VariantAxes => "variant_axes",
    })
}

fn filter_value_key(field: &str, value: &str) -> String {
    format!("{field}:{value}")
}

fn enum_string_field_column(field: MetadataEnumStringField) -> Option<&'static str> {
    Some(match field {
        MetadataEnumStringField::PackName | MetadataEnumStringField::PackLabel => return None,
        MetadataEnumStringField::PublicationFamily => "record.publication_family",
        MetadataEnumStringField::Size => "record.actor_size",
        MetadataEnumStringField::Usage => "record.system_usage",
        MetadataEnumStringField::SystemGroup => "record.system_group",
        MetadataEnumStringField::FoundryRecordType => "record.foundry_record_type",
        MetadataEnumStringField::BaseItem => "record.system_base_item",
        MetadataEnumStringField::Hands => "record.item_hands_requirement",
        MetadataEnumStringField::SaveType => "record.spell_save_type",
        MetadataEnumStringField::AreaType => "record.spell_area_type",
        MetadataEnumStringField::DurationUnit => "record.duration_unit",
        MetadataEnumStringField::Rarity => "record.rarity",
        MetadataEnumStringField::VariantGroupKey => "record.variant_group_key",
    })
}

fn text_field_column(field: MetadataTextStringField) -> Option<&'static str> {
    Some(match field {
        MetadataTextStringField::PublicationTitle => "record.publication_title",
        MetadataTextStringField::RangeText => "record.spell_range_text",
        MetadataTextStringField::DurationText => "record.duration_text",
        MetadataTextStringField::TargetText => "record.spell_target_text",
        MetadataTextStringField::DisableText => "record.actor_disable_text",
        MetadataTextStringField::VariantBaseName => "record.variant_base_name",
        MetadataTextStringField::VariantLabel => "record.variant_label",
    })
}

fn number_field_column(field: MetadataNumberField) -> Option<&'static str> {
    Some(match field {
        MetadataNumberField::Level => "record.level",
        MetadataNumberField::PriceCp => "record.price_cp",
        MetadataNumberField::BulkValue => "record.item_bulk_value",
        MetadataNumberField::ActionCost => "record.activation_time_actions",
        MetadataNumberField::Hands => return None,
        MetadataNumberField::RangeValue => "record.spell_range_value",
        MetadataNumberField::AreaValue => "record.spell_area_value",
    })
}

fn boolean_field_column(field: MetadataBooleanField) -> Option<&'static str> {
    Some(match field {
        MetadataBooleanField::PublicationRemaster => "record.publication_remaster",
        MetadataBooleanField::Sustained => "record.spell_sustained",
        MetadataBooleanField::BasicSave => "record.spell_basic_save",
        MetadataBooleanField::IsComplex => "record.actor_is_complex",
    })
}

fn number_match(
    field: &str,
    r#match: MetadataNumberMatch,
) -> Result<String, LadybugIndexReaderError> {
    Ok(match r#match {
        MetadataNumberMatch::Eq { value } => format!("{field} = {value}"),
        MetadataNumberMatch::Gt { value } => format!("{field} > {value}"),
        MetadataNumberMatch::Gte { value } => format!("{field} >= {value}"),
        MetadataNumberMatch::Lt { value } => format!("{field} < {value}"),
        MetadataNumberMatch::Lte { value } => format!("{field} <= {value}"),
        MetadataNumberMatch::Between { min, max } => {
            format!("{field} >= {min} AND {field} <= {max}")
        }
        MetadataNumberMatch::IsNull => format!("{field} IS NULL"),
        MetadataNumberMatch::IsNotNull => format!("{field} IS NOT NULL"),
    })
}

fn boolean_match(
    field: &str,
    r#match: MetadataBooleanMatch,
) -> Result<String, LadybugIndexReaderError> {
    Ok(match r#match {
        MetadataBooleanMatch::Eq { value } => format!("{field} = {value}"),
        MetadataBooleanMatch::IsNull => format!("{field} IS NULL"),
        MetadataBooleanMatch::IsNotNull => format!("{field} IS NOT NULL"),
    })
}

pub(crate) fn sort_clause(
    sort: FilteredRecordSort,
) -> Result<&'static str, LadybugIndexReaderError> {
    match sort {
        FilteredRecordSort::RecordKey => Ok("ORDER BY record.record_key"),
        FilteredRecordSort::Alphabetical => Ok("ORDER BY record.name, record.record_key"),
        FilteredRecordSort::LevelAsc => Ok("ORDER BY record.level, record.name, record.record_key"),
        FilteredRecordSort::LevelDesc => {
            Ok("ORDER BY record.level DESC, record.name, record.record_key")
        }
        FilteredRecordSort::Random { .. } => unsupported("random sorting"),
        FilteredRecordSort::PriceAsc | FilteredRecordSort::PriceDesc => {
            unsupported("price sorting")
        }
    }
}
