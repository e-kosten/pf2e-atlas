use std::fmt;
use std::path::Path;

use atlas_domain::{Rarity, RecordKey};
use atlas_record::{
    FactValue, HazardComplexity, HazardDefenseSourceMetadata, HazardDefenses, HazardDetection,
    HazardDiagnosticCode, HazardEmitsSound, HazardExpectedShape, HazardFact,
    HazardHitPointSourceMetadata, HazardHitPoints, HazardIdentity, HazardIwr, HazardLifecycle,
    HazardProvenance, HazardProvenanceValue, HazardPublication, HazardRecord,
    HazardSaveSourceMetadata, HazardSaves, HazardSize, HazardSourceId, HazardSourceShape,
    HazardSourceValue, HazardTokenSourceMetadata, HazardTrait, HazardUnsupportedFact,
    HazardUnsupportedField, HazardUnsupportedOwner, HazardUnsupportedValue, PublicationLicense,
    RecordBody,
};
use serde_json::Value;

use super::dto::{
    HazardDefensesSource, HazardDetectionSource, HazardEmitsSoundSource, HazardHitPointsSource,
    HazardIwrSource, HazardLifecycleSource, HazardPublicationSource, HazardSaveSource,
    HazardSavesSource, HazardSourceField, HazardSourceValue as DtoValue, SourcePresence,
    ValueSummary, VersionedHazardSource,
};
use super::normalize::{LocalizationResolver, parse_foundry_content_with_localization};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardCoreConversion {
    pub(crate) body: RecordBody,
    pub(crate) embedded_identities: Vec<super::hazard_entities::HazardResolvedItemIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardConversionError {
    source_field: String,
    message: String,
}

impl fmt::Display for HazardConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "canonical hazard conversion failed at {}: {}",
            self.source_field, self.message
        )
    }
}

impl std::error::Error for HazardConversionError {}

pub(crate) fn convert_hazard_core(
    record_key: RecordKey,
    source_path: &str,
    source: &VersionedHazardSource,
    localization: Option<&dyn LocalizationResolver>,
) -> Result<HazardCoreConversion, HazardConversionError> {
    if Path::new(source_path).is_absolute() {
        return Err(conversion_error(
            "$source.path",
            "canonical hazard provenance requires a relative source path",
        ));
    }
    debug_assert_eq!(source.source.actor_type, "hazard");
    let owner = HazardUnsupportedOwner::Record(record_key.clone());
    let mut unsupported_fields = source
        .source
        .unclaimed
        .iter()
        .map(|value| {
            unsupported_fact(
                HazardUnsupportedField::HazardUnexpected(value.source_path.clone()),
                value,
                HazardExpectedShape::Any,
                owner.clone(),
                HazardDiagnosticCode::UnsupportedValue,
            )
        })
        .collect::<Vec<_>>();
    let level = source_fact_clone(
        &source.source.level,
        "/system/details/level/value",
        HazardExpectedShape::Integer,
        owner.clone(),
    );
    let publication = source_fact_map(
        &source.source.publication,
        "/system/details/publication",
        HazardExpectedShape::Object,
        owner.clone(),
        |value| convert_publication(value, &owner),
        |_| "{}".to_string(),
    );
    let complexity = source_fact_map(
        &source.source.complexity,
        "/system/details/isComplex",
        HazardExpectedShape::Boolean,
        owner.clone(),
        |value| {
            Ok(if *value {
                HazardComplexity::Complex
            } else {
                HazardComplexity::Simple
            })
        },
        |value| value.to_string(),
    );
    let detection = source_fact_map(
        &source.source.detection,
        "/system/attributes/stealth",
        HazardExpectedShape::Object,
        owner.clone(),
        |value| Ok(convert_detection(value, &owner, localization)),
        |_| "{}".to_string(),
    );
    let defenses = source_fact_map(
        &source.source.defenses,
        "/system/attributes",
        HazardExpectedShape::Object,
        owner.clone(),
        |value| {
            Ok(convert_defenses(
                value,
                &owner,
                localization,
                &mut unsupported_fields,
            ))
        },
        |_| "{}".to_string(),
    );
    let lifecycle = source_fact_map(
        &source.source.lifecycle,
        "/system/details",
        HazardExpectedShape::Object,
        owner.clone(),
        |value| Ok(convert_lifecycle(value, &owner, localization)),
        |_| "{}".to_string(),
    );
    let emits_sound = source_fact_map(
        &source.source.emits_sound,
        "/system/attributes/emitsSound",
        HazardExpectedShape::StringOrBoolean,
        owner.clone(),
        |value| match value {
            HazardEmitsSoundSource::Boolean(value) => Ok(HazardEmitsSound::Boolean(*value)),
            HazardEmitsSoundSource::Named(value) if value == "encounter" => {
                Ok(HazardEmitsSound::Named(value.clone()))
            }
            HazardEmitsSoundSource::Named(_) => Err(HazardDiagnosticCode::UnsupportedValue),
        },
        |value| match value {
            HazardEmitsSoundSource::Boolean(value) => value.to_string(),
            HazardEmitsSoundSource::Named(value) => json_string(value),
        },
    );
    let rarity = nested_fact(
        &source.source.traits,
        "/system/traits/rarity",
        |traits| {
            source_fact_map(
                &traits.rarity,
                "/system/traits/rarity",
                HazardExpectedShape::ClosedVocabulary,
                owner.clone(),
                |value| {
                    Rarity::from_canonical(value).ok_or(HazardDiagnosticCode::InvalidCanonicalValue)
                },
                |value| json_string(value),
            )
        },
        owner.clone(),
    );
    let hazard_traits = nested_fact(
        &source.source.traits,
        "/system/traits/value",
        |traits| {
            source_fact_map(
                &traits.values,
                "/system/traits/value",
                HazardExpectedShape::Array,
                owner.clone(),
                |values| {
                    values
                        .iter()
                        .map(|value| {
                            HazardTrait::new(value.clone())
                                .map_err(|_| HazardDiagnosticCode::InvalidCanonicalValue)
                        })
                        .collect()
                },
                |values| {
                    Value::Array(values.iter().cloned().map(Value::String).collect()).to_string()
                },
            )
        },
        owner.clone(),
    );
    let size = nested_fact(
        &source.source.traits,
        "/system/traits/size/value",
        |traits| {
            source_fact_map(
                &traits.size,
                "/system/traits/size/value",
                HazardExpectedShape::ClosedVocabulary,
                owner.clone(),
                |value| hazard_size(value).ok_or(HazardDiagnosticCode::InvalidCanonicalValue),
                |value| json_string(value),
            )
        },
        owner.clone(),
    );
    let actor_effects = source_fact_map(
        &source.source.effects,
        "/effects",
        HazardExpectedShape::Array,
        owner.clone(),
        |values| {
            Ok(values
                .iter()
                .enumerate()
                .map(|(order, value)| HazardProvenanceValue {
                    authored_order: order as u32,
                    exact_json: value.value.clone(),
                    source_shape: shape_from_summary(value),
                })
                .collect())
        },
        |_| "[]".to_string(),
    );
    let source_creature_type = source_fact_clone(
        &source.source.creature_type,
        "/system/creatureType",
        HazardExpectedShape::String,
        owner.clone(),
    );
    let source_status_effects = source_fact_clone(
        &source.source.status_effects,
        "/system/statusEffects",
        HazardExpectedShape::Array,
        owner.clone(),
    );
    let source_folder = source_fact_clone(
        &source.source.folder,
        "/folder",
        HazardExpectedShape::String,
        owner.clone(),
    );
    let image = source_fact_clone(
        &source.source.image,
        "/img",
        HazardExpectedShape::String,
        owner,
    );
    let token = source_fact_map(
        &source.source.prototype_token,
        "/prototypeToken",
        HazardExpectedShape::Object,
        HazardUnsupportedOwner::Record(record_key.clone()),
        |value| {
            Ok(HazardTokenSourceMetadata {
                name: source_fact_clone(
                    &value.name,
                    "/prototypeToken/name",
                    HazardExpectedShape::String,
                    HazardUnsupportedOwner::Record(record_key.clone()),
                ),
            })
        },
        |_| "{}".to_string(),
    );

    let embedded = super::hazard_entities::convert_hazard_embedded_entities(
        &record_key,
        source,
        localization,
    )?;
    let hazard = HazardRecord {
        identity: HazardIdentity {
            record_key,
            source_id: HazardSourceId::new(source.source.id.clone())
                .map_err(|_| conversion_error("$._id", "invalid stable source id"))?,
            name: source.source.name.clone(),
        },
        level,
        rarity,
        traits: hazard_traits,
        size,
        publication,
        complexity,
        detection,
        defenses,
        lifecycle,
        emits_sound,
        embedded_entities: embedded.embedded,
        content: atlas_record::OwnedRichContent::default(),
        relationships: embedded.relationships,
        unsupported_fields,
        provenance: HazardProvenance {
            source_path: source_path.to_string(),
            source_contract_version: source.version.contract_version().to_string(),
            source_system_version: source.version.system_version().to_string(),
            source_upstream_commit: source.version.upstream_commit().to_string(),
            source_folder,
            image,
            source_creature_type,
            source_status_effects,
            actor_effects,
            token,
        },
    };
    Ok(HazardCoreConversion {
        body: RecordBody::Hazard(hazard),
        embedded_identities: embedded.resolved_identities,
    })
}

fn convert_publication(
    value: &HazardPublicationSource,
    owner: &HazardUnsupportedOwner,
) -> Result<HazardPublication, HazardDiagnosticCode> {
    Ok(HazardPublication {
        title: source_fact_clone(
            &value.title,
            "/system/details/publication/title",
            HazardExpectedShape::String,
            owner.clone(),
        ),
        remaster: source_fact_clone(
            &value.remaster,
            "/system/details/publication/remaster",
            HazardExpectedShape::Boolean,
            owner.clone(),
        ),
        license: source_fact_map(
            &value.license,
            "/system/details/publication/license",
            HazardExpectedShape::String,
            owner.clone(),
            |value| {
                PublicationLicense::new(value.clone())
                    .map_err(|_| HazardDiagnosticCode::InvalidCanonicalValue)
            },
            |value| json_string(value),
        ),
    })
}

fn convert_detection(
    value: &HazardDetectionSource,
    owner: &HazardUnsupportedOwner,
    localization: Option<&dyn LocalizationResolver>,
) -> HazardDetection {
    HazardDetection {
        stealth_modifier: source_fact_clone(
            &value.value,
            "/system/attributes/stealth/value",
            HazardExpectedShape::Integer,
            owner.clone(),
        ),
        details: rich_document_fact(
            &value.details,
            "/system/attributes/stealth/details",
            owner.clone(),
            localization,
        ),
    }
}

fn convert_defenses(
    value: &HazardDefensesSource,
    owner: &HazardUnsupportedOwner,
    localization: Option<&dyn LocalizationResolver>,
    diagnostics: &mut Vec<HazardUnsupportedFact>,
) -> HazardDefenses {
    HazardDefenses {
        armor_class: source_fact_clone(
            &value.armor_class,
            "/system/attributes/ac/value",
            HazardExpectedShape::Integer,
            owner.clone(),
        ),
        hardness: source_fact_clone(
            &value.hardness,
            "/system/attributes/hardness",
            HazardExpectedShape::Integer,
            owner.clone(),
        ),
        hit_points: source_fact_map(
            &value.hit_points,
            "/system/attributes/hp",
            HazardExpectedShape::Object,
            owner.clone(),
            |value| Ok(convert_hit_points(value, owner, localization)),
            |_| "{}".to_string(),
        ),
        saves: source_fact_map(
            &value.saves,
            "/system/saves",
            HazardExpectedShape::Object,
            owner.clone(),
            |value| Ok(convert_saves(value, owner)),
            |_| "{}".to_string(),
        ),
        immunities: convert_iwr(
            &value.immunities,
            "/system/attributes/immunities",
            owner,
            diagnostics,
        ),
        weaknesses: convert_iwr(
            &value.weaknesses,
            "/system/attributes/weaknesses",
            owner,
            diagnostics,
        ),
        resistances: convert_iwr(
            &value.resistances,
            "/system/attributes/resistances",
            owner,
            diagnostics,
        ),
        source_metadata: HazardDefenseSourceMetadata {
            has_health: source_fact_clone(
                &value.has_health,
                "/system/attributes/hasHealth",
                HazardExpectedShape::Boolean,
                owner.clone(),
            ),
        },
    }
}

fn convert_hit_points(
    value: &HazardHitPointsSource,
    owner: &HazardUnsupportedOwner,
    localization: Option<&dyn LocalizationResolver>,
) -> HazardHitPoints {
    HazardHitPoints {
        current: source_fact_clone(
            &value.current,
            "/system/attributes/hp/value",
            HazardExpectedShape::Integer,
            owner.clone(),
        ),
        maximum: source_fact_clone(
            &value.maximum,
            "/system/attributes/hp/max",
            HazardExpectedShape::Integer,
            owner.clone(),
        ),
        temporary: source_fact_clone(
            &value.temporary,
            "/system/attributes/hp/temp",
            HazardExpectedShape::Integer,
            owner.clone(),
        ),
        details: rich_document_fact(
            &value.details,
            "/system/attributes/hp/details",
            owner.clone(),
            localization,
        ),
        source_metadata: HazardHitPointSourceMetadata {
            temporary_maximum: source_fact_clone(
                &value.temporary_maximum,
                "/system/attributes/hp/tempmax",
                HazardExpectedShape::Integer,
                owner.clone(),
            ),
        },
    }
}

fn convert_saves(value: &HazardSavesSource, owner: &HazardUnsupportedOwner) -> HazardSaves {
    HazardSaves {
        fortitude: save_value(&value.fortitude, "fortitude", owner),
        reflex: save_value(&value.reflex, "reflex", owner),
        will: save_value(&value.will, "will", owner),
        source_metadata: HazardSaveSourceMetadata {
            fortitude_detail: save_detail(&value.fortitude, "fortitude", owner),
            reflex_detail: save_detail(&value.reflex, "reflex", owner),
            will_detail: save_detail(&value.will, "will", owner),
        },
    }
}

fn save_detail(
    value: &HazardSaveSource,
    slug: &str,
    owner: &HazardUnsupportedOwner,
) -> HazardFact<String> {
    source_fact_clone(
        &value.detail,
        &format!("/system/saves/{slug}/saveDetail"),
        HazardExpectedShape::String,
        owner.clone(),
    )
}

fn save_value(
    value: &HazardSaveSource,
    slug: &str,
    owner: &HazardUnsupportedOwner,
) -> HazardFact<i64> {
    source_fact_clone(
        &value.value,
        &format!("/system/saves/{slug}/value"),
        HazardExpectedShape::Integer,
        owner.clone(),
    )
}

fn convert_iwr(
    value: &HazardSourceField<Vec<HazardIwrSource>>,
    path: &str,
    owner: &HazardUnsupportedOwner,
    diagnostics: &mut Vec<HazardUnsupportedFact>,
) -> HazardFact<Vec<HazardIwr>> {
    source_fact_map(
        value,
        path,
        HazardExpectedShape::Array,
        owner.clone(),
        |values| {
            values
                .iter()
                .map(|value| convert_iwr_entry(value, path, owner, diagnostics))
                .collect()
        },
        |_| "[]".to_string(),
    )
}

fn convert_iwr_entry(
    value: &HazardIwrSource,
    path: &str,
    owner: &HazardUnsupportedOwner,
    diagnostics: &mut Vec<HazardUnsupportedFact>,
) -> Result<HazardIwr, HazardDiagnosticCode> {
    for unclaimed in &value.unclaimed {
        diagnostics.push(unsupported_fact(
            HazardUnsupportedField::HazardUnexpected(unclaimed.source_path.clone()),
            unclaimed,
            HazardExpectedShape::Object,
            owner.clone(),
            HazardDiagnosticCode::UnsupportedValue,
        ));
    }
    let ordinal = value.source_ordinal;
    Ok(HazardIwr {
        id: atlas_record::HazardComponentId::new(format!("iwr-{ordinal}"))
            .map_err(|_| HazardDiagnosticCode::UnstableIdentity)?,
        authored_order: ordinal,
        iwr_type: source_fact_clone(
            &value.iwr_type,
            &format!("{path}/{ordinal}/type"),
            HazardExpectedShape::String,
            owner.clone(),
        ),
        value: source_fact_clone(
            &value.value,
            &format!("{path}/{ordinal}/value"),
            HazardExpectedShape::Integer,
            owner.clone(),
        ),
        exceptions: source_fact_clone(
            &value.exceptions,
            &format!("{path}/{ordinal}/exceptions"),
            HazardExpectedShape::Array,
            owner.clone(),
        ),
        double_vs: source_fact_clone(
            &value.double_vs,
            &format!("{path}/{ordinal}/doubleVs"),
            HazardExpectedShape::Array,
            owner.clone(),
        ),
    })
}

fn convert_lifecycle(
    value: &HazardLifecycleSource,
    owner: &HazardUnsupportedOwner,
    localization: Option<&dyn LocalizationResolver>,
) -> HazardLifecycle {
    HazardLifecycle {
        description: rich_document_fact(
            &value.description,
            "/system/details/description",
            owner.clone(),
            localization,
        ),
        disable: rich_document_fact(
            &value.disable,
            "/system/details/disable",
            owner.clone(),
            localization,
        ),
        routine: rich_document_fact(
            &value.routine,
            "/system/details/routine",
            owner.clone(),
            localization,
        ),
        reset: rich_document_fact(
            &value.reset,
            "/system/details/reset",
            owner.clone(),
            localization,
        ),
    }
}

fn rich_document_fact(
    value: &HazardSourceField<String>,
    path: &str,
    owner: HazardUnsupportedOwner,
    localization: Option<&dyn LocalizationResolver>,
) -> HazardFact<atlas_record::RichDocument> {
    source_fact_map(
        value,
        path,
        HazardExpectedShape::RichDocument,
        owner,
        |value| Ok(parse_foundry_content_with_localization(value, localization).document),
        |value| json_string(value),
    )
}

fn nested_fact<T, U>(
    outer: &HazardSourceField<T>,
    path: &str,
    convert: impl FnOnce(&T) -> HazardFact<U>,
    owner: HazardUnsupportedOwner,
) -> HazardFact<U> {
    match outer {
        SourcePresence::Missing => HazardFact::source(FactValue::Missing, path),
        SourcePresence::Null => HazardFact::source(FactValue::Null, path),
        SourcePresence::Value(DtoValue::Typed(value)) => convert(value),
        SourcePresence::Value(DtoValue::Unsupported(value)) => HazardFact::source(
            FactValue::Value(HazardSourceValue::Unsupported(unsupported_value(
                value,
                HazardExpectedShape::Object,
                owner,
                HazardDiagnosticCode::UnexpectedShape,
            ))),
            path,
        ),
    }
}

fn source_fact_clone<T: Clone>(
    value: &HazardSourceField<T>,
    path: &str,
    expected: HazardExpectedShape,
    owner: HazardUnsupportedOwner,
) -> HazardFact<T> {
    source_fact_map(
        value,
        path,
        expected,
        owner,
        |value| Ok(value.clone()),
        |_| "null".to_string(),
    )
}

fn source_fact_map<T, U>(
    value: &HazardSourceField<T>,
    path: &str,
    expected: HazardExpectedShape,
    owner: HazardUnsupportedOwner,
    convert: impl FnOnce(&T) -> Result<U, HazardDiagnosticCode>,
    typed_json: impl FnOnce(&T) -> String,
) -> HazardFact<U> {
    let value = match value {
        SourcePresence::Missing => FactValue::Missing,
        SourcePresence::Null => FactValue::Null,
        SourcePresence::Value(DtoValue::Unsupported(value)) => {
            FactValue::Value(HazardSourceValue::Unsupported(unsupported_value(
                value,
                expected,
                owner,
                HazardDiagnosticCode::UnexpectedShape,
            )))
        }
        SourcePresence::Value(DtoValue::Typed(value)) => match convert(value) {
            Ok(value) => FactValue::Value(HazardSourceValue::Typed(value)),
            Err(code) => FactValue::Value(HazardSourceValue::Unsupported(HazardUnsupportedValue {
                exact_json: typed_json(value),
                expected_shape: expected,
                actual_shape: shape_for_expected(expected),
                relative_source_path: path.to_string(),
                owner,
                diagnostic_code: code,
            })),
        },
    };
    HazardFact::source(value, path)
}

fn unsupported_fact(
    field: HazardUnsupportedField,
    value: &ValueSummary,
    expected: HazardExpectedShape,
    owner: HazardUnsupportedOwner,
    code: HazardDiagnosticCode,
) -> HazardUnsupportedFact {
    HazardUnsupportedFact {
        field,
        value: unsupported_value(value, expected, owner, code),
    }
}

fn unsupported_value(
    value: &ValueSummary,
    expected: HazardExpectedShape,
    owner: HazardUnsupportedOwner,
    code: HazardDiagnosticCode,
) -> HazardUnsupportedValue {
    HazardUnsupportedValue {
        exact_json: value.value.clone(),
        expected_shape: expected,
        actual_shape: shape_from_summary(value),
        relative_source_path: value.source_path.clone(),
        owner,
        diagnostic_code: code,
    }
}

fn shape_from_summary(value: &ValueSummary) -> HazardSourceShape {
    let shape = value
        .shape
        .split_whitespace()
        .last()
        .unwrap_or(&value.shape);
    match shape {
        "missing" => HazardSourceShape::Missing,
        "null" => HazardSourceShape::Null,
        "boolean" => HazardSourceShape::Boolean,
        "integer" | "number" => HazardSourceShape::Number,
        "string" => HazardSourceShape::String,
        "array" | "strings" => HazardSourceShape::Array,
        "object" => HazardSourceShape::Object,
        _ => HazardSourceShape::Object,
    }
}

fn shape_for_expected(value: HazardExpectedShape) -> HazardSourceShape {
    match value {
        HazardExpectedShape::Any => HazardSourceShape::Object,
        HazardExpectedShape::Boolean => HazardSourceShape::Boolean,
        HazardExpectedShape::Integer => HazardSourceShape::Number,
        HazardExpectedShape::String
        | HazardExpectedShape::StringOrBoolean
        | HazardExpectedShape::StringOrArray
        | HazardExpectedShape::ClosedVocabulary
        | HazardExpectedShape::RichDocument => HazardSourceShape::String,
        HazardExpectedShape::Array => HazardSourceShape::Array,
        HazardExpectedShape::Object => HazardSourceShape::Object,
    }
}

fn hazard_size(value: &str) -> Option<HazardSize> {
    match value {
        "tiny" => Some(HazardSize::Tiny),
        "sm" => Some(HazardSize::Small),
        "med" => Some(HazardSize::Medium),
        "lg" => Some(HazardSize::Large),
        "huge" => Some(HazardSize::Huge),
        "grg" => Some(HazardSize::Gargantuan),
        _ => None,
    }
}

fn json_string(value: &str) -> String {
    Value::String(value.to_string()).to_string()
}

pub(crate) fn conversion_error(
    path: impl Into<String>,
    message: impl Into<String>,
) -> HazardConversionError {
    HazardConversionError {
        source_field: path.into(),
        message: message.into(),
    }
}
