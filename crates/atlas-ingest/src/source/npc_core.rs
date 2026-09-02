use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use atlas_domain::{Rarity, RecordKey};
use atlas_record::{
    CreatureAdjustment, CreatureAllianceName, CreatureArmorClass, CreatureComponentId,
    CreatureDefenses, CreatureFact, CreatureFamily, CreatureHitPoints, CreatureIdentity,
    CreatureInitiative, CreatureInitiativeStatistic, CreatureIwr, CreatureIwrKind,
    CreatureLanguages, CreatureLegacyAbilities, CreatureMovementMode, CreatureNote, CreatureNumber,
    CreaturePerception, CreaturePredicate, CreatureProvenance, CreaturePublication, CreatureRecord,
    CreatureResource, CreatureResourceAmount, CreatureResourceKind, CreatureSave, CreatureSaveKind,
    CreatureSaves, CreatureSense, CreatureShield, CreatureSize, CreatureSkill, CreatureSkillKind,
    CreatureSkillSourceEntry, CreatureSkillVariant, CreatureSourceAlliance, CreatureSourceField,
    CreatureSourceId, CreatureSpeed, CreatureStatistic, CreatureTrait, CreatureUnmodeledSkill,
    CreatureUnmodeledSkillReason, CreatureUnsupportedSourceFact, CreatureUnsupportedSourceField,
    FactValue, IwrQualifier, IwrType, Language, PredicateTerm, PublicationLicense, RecordBody,
    ResourceCurrentPolicy, SenseAcuity, SenseType, ShieldCurrentPolicy, UnsupportedSourceReason,
    UnsupportedSourceShape, UnsupportedSourceValue,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::dto::{
    ItemType, NpcCoreSource, NpcIwrSource, NpcPredicateSource, NpcResourceAmountSource,
    NpcResourceSource, NpcSaveSource, NpcSavesSource, NpcSkillSource, NpcSkillVariantSource,
    SourceInteger, SourcePresence, VersionedNpcSource,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NpcCoreConversion {
    pub(crate) body: RecordBody,
    pub(crate) diagnostics: Vec<NpcCoreDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct NpcCoreDiagnostic {
    pub(crate) code: &'static str,
    pub(crate) kind: NpcCoreDiagnosticKind,
    pub(crate) record_key: String,
    pub(crate) source_path: String,
    pub(crate) source_field: String,
    pub(crate) observed_issue: &'static str,
    pub(crate) source_value: String,
    pub(crate) disposition: NpcCoreDiagnosticDisposition,
    pub(crate) owner: NpcCoreDiagnosticOwner,
}

impl NpcCoreDiagnostic {
    fn new(
        kind: NpcCoreDiagnosticKind,
        source_field: impl Into<String>,
        source_value: impl Into<String>,
    ) -> Self {
        Self {
            code: kind.code(),
            kind,
            record_key: String::new(),
            source_path: String::new(),
            source_field: source_field.into(),
            observed_issue: kind.observed_issue(),
            source_value: source_value.into(),
            disposition: kind.disposition(),
            owner: NpcCoreDiagnosticOwner::CanonicalNpcCore,
        }
    }

    fn bind(mut self, record_key: &str, source_path: &str) -> Self {
        self.record_key = record_key.to_string();
        self.source_path = source_path.to_string();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NpcCoreDiagnosticKind {
    UnsupportedOpenValue,
    UnsupportedLegacyShape,
    UnstableSourceOrderIdentity,
    ComponentIdSourceFallback,
    ComponentIdScopedOrdinalFallback,
    ResourceMaximumDrift,
}

impl NpcCoreDiagnosticKind {
    const fn code(self) -> &'static str {
        match self {
            Self::UnsupportedOpenValue => "atlas.npc_core.unsupported_open_value.v1",
            Self::UnsupportedLegacyShape => "atlas.npc_core.unsupported_legacy_shape.v1",
            Self::UnstableSourceOrderIdentity => "atlas.npc_core.unstable_source_order_identity.v1",
            Self::ComponentIdSourceFallback => "atlas.npc_core.component_id_source_fallback.v1",
            Self::ComponentIdScopedOrdinalFallback => {
                "atlas.npc_core.component_id_scoped_ordinal_fallback.v1"
            }
            Self::ResourceMaximumDrift => "atlas.npc_core.resource_maxx_source_drift.v1",
        }
    }

    const fn observed_issue(self) -> &'static str {
        match self {
            Self::UnsupportedOpenValue => "source value is outside the supported closed vocabulary",
            Self::UnsupportedLegacyShape => "source value uses an ambiguous legacy shape",
            Self::UnstableSourceOrderIdentity => "nested source component has no stable source id",
            Self::ComponentIdSourceFallback => {
                "source-derived component identity violates canonical id syntax"
            }
            Self::ComponentIdScopedOrdinalFallback => {
                "source component has no usable source identity"
            }
            Self::ResourceMaximumDrift => {
                "source resource uses unsupported maxx field instead of canonical max"
            }
        }
    }

    const fn disposition(self) -> NpcCoreDiagnosticDisposition {
        match self {
            Self::UnsupportedOpenValue
            | Self::UnsupportedLegacyShape
            | Self::ResourceMaximumDrift => NpcCoreDiagnosticDisposition::TypedUnsupported,
            Self::UnstableSourceOrderIdentity | Self::ComponentIdScopedOrdinalFallback => {
                NpcCoreDiagnosticDisposition::DiagnosedScopedOrdinal
            }
            Self::ComponentIdSourceFallback => NpcCoreDiagnosticDisposition::StableSourceFallback,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NpcCoreDiagnosticDisposition {
    #[serde(rename = "preserved_as_typed_unsupported")]
    TypedUnsupported,
    #[serde(rename = "preserved_with_stable_source_fallback")]
    StableSourceFallback,
    #[serde(rename = "preserved_with_diagnosed_scoped_ordinal")]
    DiagnosedScopedOrdinal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NpcCoreDiagnosticOwner {
    CanonicalNpcCore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NpcCoreConversionError {
    pub(crate) source_field: String,
    pub(crate) message: String,
}

impl fmt::Display for NpcCoreConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "canonical NPC conversion failed at {}: {}",
            self.source_field, self.message
        )
    }
}

impl std::error::Error for NpcCoreConversionError {}

pub(crate) fn convert_npc_core(
    record_key: RecordKey,
    source_path: &str,
    source: &VersionedNpcSource,
) -> Result<NpcCoreConversion, NpcCoreConversionError> {
    if Path::new(source_path).is_absolute() {
        return Err(conversion_error(
            "$source.path",
            "canonical NPC diagnostics require a relative source path",
        ));
    }
    let diagnostic_record_key = record_key.to_string();
    let mut diagnostics = Vec::new();
    let core = &source.source.core;
    let creature = CreatureRecord {
        identity: CreatureIdentity {
            record_key,
            source_id: CreatureSourceId::new(source.source.id.clone())
                .map_err(|_| conversion_error("$._id", "invalid stable source id"))?,
            name: source.source.name.clone(),
            family: CreatureFamily::Npc,
        },
        level: CreatureFact::source(level(core), CreatureSourceField::Level),
        rarity: CreatureFact::source(rarity(core)?, CreatureSourceField::Rarity),
        traits: CreatureFact::source(traits(core)?, CreatureSourceField::Traits),
        size: CreatureFact::source(size(core)?, CreatureSourceField::Size),
        publication: CreatureFact::source(publication(core)?, CreatureSourceField::Publication),
        adjustment: CreatureFact::source(
            adjustment(core, &mut diagnostics),
            CreatureSourceField::Adjustment,
        ),
        source_alliance: CreatureFact::source(
            source_alliance(core, &mut diagnostics),
            CreatureSourceField::SourceAlliance,
        ),
        perception: CreatureFact::source(
            perception(core, &mut diagnostics)?,
            CreatureSourceField::Perception,
        ),
        initiative: CreatureFact::source(
            initiative(core, &mut diagnostics),
            CreatureSourceField::Initiative,
        ),
        languages: CreatureFact::source(languages(core)?, CreatureSourceField::Languages),
        skills: CreatureFact::source(
            skills(source, &mut diagnostics)?,
            CreatureSourceField::Skills,
        ),
        legacy_abilities: CreatureFact::source(
            legacy_abilities(core),
            CreatureSourceField::LegacyAbilities,
        ),
        defenses: CreatureFact::source(
            defenses(core, &mut diagnostics)?,
            CreatureSourceField::Defenses,
        ),
        movement: CreatureFact::source(
            movement(core, &diagnostic_record_key, &mut diagnostics)?,
            CreatureSourceField::Movement,
        ),
        resources: CreatureFact::source(
            resources(core, &mut diagnostics)?,
            CreatureSourceField::Resources,
        ),
        embedded_entities: CreatureFact::source(
            FactValue::Missing,
            CreatureSourceField::EmbeddedEntities,
        ),
        content: atlas_record::OwnedRichContent::default(),
        provenance: CreatureProvenance {
            source_path: source_path.to_string(),
            source_contract_version: source.version.contract_version().to_string(),
            source_system_version: source.version.system_version().to_string(),
            source_upstream_commit: source.version.upstream_commit().to_string(),
        },
    };
    let diagnostics = diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.bind(&diagnostic_record_key, source_path))
        .collect();
    Ok(NpcCoreConversion {
        body: RecordBody::Creature(creature),
        diagnostics,
    })
}

fn level(core: &NpcCoreSource) -> FactValue<i64> {
    nested(&core.details, |details| {
        nested(&details.level, |level| presence(&level.value))
    })
}

fn adjustment(
    core: &NpcCoreSource,
    diagnostics: &mut Vec<NpcCoreDiagnostic>,
) -> FactValue<CreatureAdjustment> {
    nested(&core.attributes, |attributes| {
        map_presence(&attributes.adjustment, |value| {
            CreatureAdjustment::from_source(value).unwrap_or_else(|| {
                diagnostics.push(NpcCoreDiagnostic::new(
                    NpcCoreDiagnosticKind::UnsupportedOpenValue,
                    "$.system.attributes.adjustment",
                    value,
                ));
                CreatureAdjustment::Unsupported(unsupported_string(
                    value,
                    UnsupportedSourceReason::OpenVocabulary,
                ))
            })
        })
    })
}

fn source_alliance(
    core: &NpcCoreSource,
    diagnostics: &mut Vec<NpcCoreDiagnostic>,
) -> FactValue<CreatureSourceAlliance> {
    nested(&core.details, |details| {
        map_presence(&details.alliance, |value| {
            CreatureAllianceName::new(value.clone())
                .map(CreatureSourceAlliance::Named)
                .unwrap_or_else(|_| {
                    diagnostics.push(NpcCoreDiagnostic::new(
                        NpcCoreDiagnosticKind::UnsupportedOpenValue,
                        "$.system.details.alliance",
                        value,
                    ));
                    CreatureSourceAlliance::Unsupported(unsupported_string(
                        value,
                        UnsupportedSourceReason::OpenVocabulary,
                    ))
                })
        })
    })
}

fn initiative(
    core: &NpcCoreSource,
    diagnostics: &mut Vec<NpcCoreDiagnostic>,
) -> FactValue<CreatureInitiative> {
    map_presence(&core.initiative, |initiative| CreatureInitiative {
        statistic: map_presence(&initiative.statistic, |value| {
            CreatureStatistic::new(value.clone())
                .map(CreatureInitiativeStatistic::Named)
                .unwrap_or_else(|_| {
                    diagnostics.push(NpcCoreDiagnostic::new(
                        NpcCoreDiagnosticKind::UnsupportedOpenValue,
                        "$.system.initiative.statistic",
                        value,
                    ));
                    CreatureInitiativeStatistic::Unsupported(unsupported_string(
                        value,
                        UnsupportedSourceReason::OpenVocabulary,
                    ))
                })
        }),
    })
}

fn legacy_abilities(core: &NpcCoreSource) -> FactValue<CreatureLegacyAbilities> {
    map_presence(&core.abilities, |abilities| CreatureLegacyAbilities {
        strength: legacy_ability(&abilities.strength),
        dexterity: legacy_ability(&abilities.dexterity),
        constitution: legacy_ability(&abilities.constitution),
        intelligence: legacy_ability(&abilities.intelligence),
        wisdom: legacy_ability(&abilities.wisdom),
        charisma: legacy_ability(&abilities.charisma),
    })
}

fn legacy_ability(source: &SourcePresence<super::dto::NpcLegacyAbilitySource>) -> FactValue<i64> {
    nested(source, |ability| presence(&ability.r#mod))
}

fn rarity(core: &NpcCoreSource) -> Result<FactValue<Rarity>, NpcCoreConversionError> {
    nested_result(&core.traits, |traits| {
        map_presence_result(&traits.rarity, |value| {
            Rarity::from_canonical(value).ok_or_else(|| {
                conversion_error(
                    "$.system.traits.rarity",
                    &format!("unsupported rarity {value:?}"),
                )
            })
        })
    })
}

fn traits(core: &NpcCoreSource) -> Result<FactValue<Vec<CreatureTrait>>, NpcCoreConversionError> {
    nested_result(&core.traits, |traits| {
        map_presence_result(&traits.values, |values| {
            values
                .iter()
                .map(|value| {
                    CreatureTrait::new(value.clone()).map_err(|_| {
                        conversion_error(
                            "$.system.traits.value[]",
                            &format!("invalid trait slug {value:?}"),
                        )
                    })
                })
                .collect()
        })
    })
}

fn size(core: &NpcCoreSource) -> Result<FactValue<CreatureSize>, NpcCoreConversionError> {
    nested_result(&core.traits, |traits| {
        nested_result(&traits.size, |size| {
            map_presence_result(&size.value, |value| {
                CreatureSize::from_source(value).ok_or_else(|| {
                    conversion_error(
                        "$.system.traits.size.value",
                        &format!("unsupported closed creature size {value:?}"),
                    )
                })
            })
        })
    })
}

fn publication(
    core: &NpcCoreSource,
) -> Result<FactValue<CreaturePublication>, NpcCoreConversionError> {
    nested_result(&core.details, |details| {
        map_presence_result(&details.publication, |publication| {
            Ok(CreaturePublication {
                title: presence(&publication.title),
                remaster: presence(&publication.remaster),
                license: map_presence_result(&publication.license, |license| {
                    PublicationLicense::new(license.clone()).map_err(|_| {
                        conversion_error(
                            "$.system.details.publication.license",
                            "invalid publication license",
                        )
                    })
                })?,
            })
        })
    })
}

fn perception(
    core: &NpcCoreSource,
    diagnostics: &mut Vec<NpcCoreDiagnostic>,
) -> Result<FactValue<CreaturePerception>, NpcCoreConversionError> {
    map_presence_result(&core.perception, |perception| {
        let modifier = match (&perception.modifier, &perception.legacy_value) {
            (SourcePresence::Missing, SourcePresence::Value(value)) => FactValue::Value(*value),
            (SourcePresence::Null, SourcePresence::Value(value)) => FactValue::Value(*value),
            (source, SourcePresence::Value(legacy)) => {
                if source.as_value().is_some_and(|value| value != legacy) {
                    diagnostics.push(NpcCoreDiagnostic::new(
                        NpcCoreDiagnosticKind::UnsupportedLegacyShape,
                        "$.system.perception.value",
                        legacy.to_string(),
                    ));
                }
                presence(source)
            }
            (source, SourcePresence::Missing | SourcePresence::Null) => presence(source),
        };
        let senses = map_presence_result(&perception.senses, |senses| {
            let mut counts = BTreeMap::<String, u32>::new();
            senses
                .iter()
                .enumerate()
                .map(|(order, sense)| {
                    let sense_type = required_value(
                        &sense.sense_type,
                        &format!("$.system.perception.senses[{order}].type"),
                    )?;
                    let duplicate = counts.entry(sense_type.clone()).or_default();
                    let id = component_id(&format!("sense:{sense_type}:{duplicate}"))?;
                    *duplicate += 1;
                    Ok(CreatureSense {
                        id,
                        authored_order: order as u32,
                        sense_type: SenseType::new(sense_type.clone()).map_err(|_| {
                            conversion_error(
                                &format!("$.system.perception.senses[{order}].type"),
                                "invalid sense type slug",
                            )
                        })?,
                        acuity: map_presence(&sense.acuity, |acuity| match acuity.as_str() {
                            "precise" => SenseAcuity::Precise,
                            "imprecise" => SenseAcuity::Imprecise,
                            "vague" => SenseAcuity::Vague,
                            value => {
                                diagnostics.push(NpcCoreDiagnostic::new(
                                    NpcCoreDiagnosticKind::UnsupportedOpenValue,
                                    format!("$.system.perception.senses[{order}].acuity"),
                                    value,
                                ));
                                SenseAcuity::Unsupported(unsupported_string(
                                    value,
                                    UnsupportedSourceReason::OpenVocabulary,
                                ))
                            }
                        }),
                        range: presence(&sense.range),
                    })
                })
                .collect::<Result<Vec<_>, NpcCoreConversionError>>()
        })?;
        Ok(CreaturePerception {
            modifier,
            details: note_presence(&perception.details),
            has_vision: presence(&perception.vision),
            senses,
        })
    })
}

fn languages(core: &NpcCoreSource) -> Result<FactValue<CreatureLanguages>, NpcCoreConversionError> {
    nested_result(&core.details, |details| {
        map_presence_result(&details.languages, |languages| {
            Ok(CreatureLanguages {
                values: map_presence_result(&languages.values, |values| {
                    values
                        .iter()
                        .map(|value| {
                            Language::new(value.clone()).map_err(|_| {
                                conversion_error(
                                    "$.system.details.languages.value[]",
                                    &format!("invalid language slug {value:?}"),
                                )
                            })
                        })
                        .collect()
                })?,
                details: note_presence(&languages.details),
            })
        })
    })
}

fn skills(
    source: &VersionedNpcSource,
    diagnostics: &mut Vec<NpcCoreDiagnostic>,
) -> Result<FactValue<Vec<CreatureSkill>>, NpcCoreConversionError> {
    let mut canonical = match &source.source.core.skills {
        SourcePresence::Missing => return Ok(FactValue::Missing),
        SourcePresence::Null => return Ok(FactValue::Null),
        SourcePresence::Value(skills) => {
            let mut modeled = Vec::new();
            for kind in CreatureSkillKind::STANDARD {
                let entries = modeled_skill_entries(kind, skills);
                if !entries.is_empty() {
                    let authored_order = modeled.len() as u32;
                    modeled.push(standard_skill(kind, &entries, authored_order, diagnostics)?);
                }
            }
            modeled
        }
    };

    if let SourcePresence::Value(skills) = &source.source.core.skills {
        for (slug, skill) in skills {
            if modeled_skill_kind(slug).is_none() {
                let authored_order = canonical.len() as u32;
                canonical.push(unmodeled_skill(slug, skill, authored_order)?);
            }
        }
    }

    let mut lore = source
        .source
        .items
        .as_value()
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(|(source_order, item)| {
            (item.item_type() == ItemType::Lore).then_some((source_order, item.source()))
        })
        .map(|(source_order, item)| {
            let lore = item.lore.as_ref().ok_or_else(|| {
                conversion_error(
                    &format!("$.items[{source_order}].system"),
                    "Lore item lacks its typed Lore source DTO",
                )
            })?;
            let modifier = nested(&lore.modifier, |modifier| presence(&modifier.value));
            let source_id = CreatureSourceId::new(item.id.clone()).map_err(|_| {
                conversion_error(
                    &format!("$.items[{source_order}]._id"),
                    "invalid Lore source id",
                )
            })?;
            Ok((
                item.sort.as_value().copied(),
                source_order,
                CreatureSkill {
                    id: component_id(&format!("skill:lore:{}", source_id.as_str()))?,
                    authored_order: 0,
                    source_entries: Vec::new(),
                    kind: CreatureSkillKind::Lore,
                    label: item.name.clone(),
                    modifier,
                    note: FactValue::Missing,
                    variants: FactValue::Missing,
                    source_item_id: FactValue::Value(source_id),
                    unmodeled: FactValue::Missing,
                },
            ))
        })
        .collect::<Result<Vec<_>, NpcCoreConversionError>>()?;
    lore.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.id.as_str().cmp(right.2.id.as_str()))
    });
    let start = canonical.len() as u32;
    for (offset, (_, _, mut skill)) in lore.into_iter().enumerate() {
        skill.authored_order = start + offset as u32;
        canonical.push(skill);
    }
    Ok(FactValue::Value(canonical))
}

fn standard_skill(
    kind: CreatureSkillKind,
    entries: &[(&str, &NpcSkillSource)],
    authored_order: u32,
    diagnostics: &mut Vec<NpcCoreDiagnostic>,
) -> Result<CreatureSkill, NpcCoreConversionError> {
    let slug = kind.source_slug();
    let source = entries[0].1;
    Ok(CreatureSkill {
        id: component_id(&format!("skill:{slug}"))?,
        authored_order,
        source_entries: entries
            .iter()
            .map(|(authored_key, source)| CreatureSkillSourceEntry {
                authored_key: (*authored_key).to_string(),
                modifier: presence(&source.base),
            })
            .collect(),
        kind,
        label: title_case_slug(slug),
        modifier: presence(&source.base),
        note: note_presence(&source.note),
        variants: map_presence_result(&source.special, |variants| {
            variants
                .iter()
                .enumerate()
                .map(|(order, variant)| skill_variant(slug, variant, order, diagnostics))
                .collect()
        })?,
        source_item_id: FactValue::Missing,
        unmodeled: FactValue::Missing,
    })
}

fn modeled_skill_entries<'a>(
    kind: CreatureSkillKind,
    skills: &'a BTreeMap<String, NpcSkillSource>,
) -> Vec<(&'a str, &'a NpcSkillSource)> {
    let mut entries = Vec::new();
    let canonical_key = kind.source_slug();
    if let Some(source) = skills.get(canonical_key) {
        entries.push((canonical_key, source));
    }
    if kind == CreatureSkillKind::Intimidation
        && let Some(source) = skills.get("intimidate")
    {
        entries.push(("intimidate", source));
    }
    entries
}

fn modeled_skill_kind(authored_key: &str) -> Option<CreatureSkillKind> {
    match authored_key {
        "intimidate" => Some(CreatureSkillKind::Intimidation),
        exact => CreatureSkillKind::from_source_slug(exact),
    }
}

fn unmodeled_skill(
    authored_key: &str,
    source: &NpcSkillSource,
    authored_order: u32,
) -> Result<CreatureSkill, NpcCoreConversionError> {
    let key_digest = format!("{:x}", Sha256::digest(authored_key.as_bytes()));
    Ok(CreatureSkill {
        id: component_id(&format!("skill:unmodeled:{key_digest}"))?,
        authored_order,
        source_entries: vec![CreatureSkillSourceEntry {
            authored_key: authored_key.to_string(),
            modifier: presence(&source.base),
        }],
        kind: CreatureSkillKind::Unmodeled,
        label: authored_key.to_string(),
        modifier: presence(&source.base),
        note: note_presence(&source.note),
        variants: FactValue::Missing,
        source_item_id: FactValue::Missing,
        unmodeled: FactValue::Value(CreatureUnmodeledSkill {
            authored_key: authored_key.to_string(),
            base: presence(&source.base),
            reason: CreatureUnmodeledSkillReason::UnknownAuthoredKey,
        }),
    })
}

fn skill_variant(
    skill_slug: &str,
    source: &NpcSkillVariantSource,
    order: usize,
    diagnostics: &mut Vec<NpcCoreDiagnostic>,
) -> Result<CreatureSkillVariant, NpcCoreConversionError> {
    diagnostics.push(NpcCoreDiagnostic::new(
        NpcCoreDiagnosticKind::UnstableSourceOrderIdentity,
        format!("$.system.skills.{skill_slug}.special[{order}]"),
        "typed skill family plus ordinal",
    ));
    Ok(CreatureSkillVariant {
        id: component_id(&format!("skill:{skill_slug}:variant:{order}"))?,
        authored_order: order as u32,
        modifier: presence(&source.base),
        label: presence(&source.label),
        predicate: map_presence_result(&source.predicate, |predicates| {
            predicates
                .iter()
                .map(|predicate| predicate_value(predicate, diagnostics))
                .collect()
        })?,
    })
}

fn predicate_value(
    source: &NpcPredicateSource,
    diagnostics: &mut Vec<NpcCoreDiagnostic>,
) -> Result<CreaturePredicate, NpcCoreConversionError> {
    let term = |value: &str| {
        PredicateTerm::new(value.to_string()).map_err(|_| {
            conversion_error(
                "$.system.skills.*.special[].predicate[]",
                "blank predicate term",
            )
        })
    };
    Ok(match source {
        NpcPredicateSource::Term(value) => CreaturePredicate::Term(term(value)?),
        NpcPredicateSource::Not(value) => CreaturePredicate::Not(term(value)?),
        NpcPredicateSource::Any(values) => CreaturePredicate::Any(
            values
                .iter()
                .map(|value| term(value))
                .collect::<Result<_, _>>()?,
        ),
        NpcPredicateSource::AtLeast {
            term: value,
            minimum,
        } => CreaturePredicate::AtLeast {
            term: term(value)?,
            minimum: *minimum,
        },
        NpcPredicateSource::Unsupported(value) => {
            diagnostics.push(NpcCoreDiagnostic::new(
                NpcCoreDiagnosticKind::UnsupportedOpenValue,
                "$.system.skills.*.special[].predicate[]",
                value,
            ));
            CreaturePredicate::Unsupported(UnsupportedSourceValue {
                shape: UnsupportedSourceShape::Object,
                value: value.clone(),
                reason: UnsupportedSourceReason::InvalidPredicate,
            })
        }
    })
}

fn defenses(
    core: &NpcCoreSource,
    diagnostics: &mut Vec<NpcCoreDiagnostic>,
) -> Result<FactValue<CreatureDefenses>, NpcCoreConversionError> {
    map_presence_result(&core.attributes, |attributes| {
        Ok(CreatureDefenses {
            armor_class: map_presence(&attributes.armor_class, |armor_class| CreatureArmorClass {
                value: presence(&armor_class.value),
                details: note_presence(&armor_class.details),
            }),
            hit_points: map_presence(&attributes.hit_points, |hp| CreatureHitPoints {
                value: map_presence(&hp.value, |value| match value {
                    SourceInteger::Integer(value) => CreatureNumber::Integer(*value),
                    SourceInteger::Text(value) => {
                        diagnostics.push(NpcCoreDiagnostic::new(
                            NpcCoreDiagnosticKind::UnsupportedLegacyShape,
                            "$.system.attributes.hp.value",
                            value,
                        ));
                        CreatureNumber::Unsupported(unsupported_string(
                            value,
                            UnsupportedSourceReason::AmbiguousLegacyShape,
                        ))
                    }
                }),
                maximum: presence(&hp.maximum),
                temporary: presence(&hp.temporary),
                temporary_maximum: presence(&hp.temporary_maximum),
                details: note_presence(&hp.details),
            }),
            hardness: nested(&attributes.hardness, |hardness| presence(&hardness.value)),
            shield: map_presence(&attributes.shield, |shield| CreatureShield {
                armor_class_bonus: presence(&shield.armor_class_bonus),
                broken_threshold: presence(&shield.broken_threshold),
                hardness: presence(&shield.hardness),
                maximum_hit_points: presence(&shield.maximum_hit_points),
                serialized_hit_points: presence(&shield.serialized_hit_points),
                current_policy: ShieldCurrentPolicy::SerializedHitPointsAreProvenanceOnly,
            }),
            saves: saves(&core.saves)?,
            all_saves_note: nested(&attributes.all_saves, |all_saves| {
                note_presence(&all_saves.value)
            }),
            immunities: iwr_collection(&attributes.immunities, CreatureIwrKind::Immunity)?,
            resistances: iwr_collection(&attributes.resistances, CreatureIwrKind::Resistance)?,
            weaknesses: iwr_collection(&attributes.weaknesses, CreatureIwrKind::Weakness)?,
        })
    })
}

fn saves(
    source: &SourcePresence<NpcSavesSource>,
) -> Result<FactValue<CreatureSaves>, NpcCoreConversionError> {
    map_presence_result(source, |saves| {
        Ok(CreatureSaves {
            fortitude: save(&saves.fortitude, CreatureSaveKind::Fortitude)?,
            reflex: save(&saves.reflex, CreatureSaveKind::Reflex)?,
            will: save(&saves.will, CreatureSaveKind::Will)?,
        })
    })
}

fn save(
    source: &SourcePresence<NpcSaveSource>,
    kind: CreatureSaveKind,
) -> Result<FactValue<CreatureSave>, NpcCoreConversionError> {
    map_presence_result(source, |save| {
        let slug = match kind {
            CreatureSaveKind::Fortitude => "fortitude",
            CreatureSaveKind::Reflex => "reflex",
            CreatureSaveKind::Will => "will",
        };
        Ok(CreatureSave {
            id: component_id(&format!("save:{slug}"))?,
            kind,
            value: presence(&save.value),
            details: note_presence(&save.details),
        })
    })
}

fn iwr_collection(
    source: &SourcePresence<Vec<NpcIwrSource>>,
    kind: CreatureIwrKind,
) -> Result<FactValue<Vec<CreatureIwr>>, NpcCoreConversionError> {
    map_presence_result(source, |entries| {
        let mut duplicate_counts = BTreeMap::<String, u32>::new();
        entries
            .iter()
            .enumerate()
            .map(|(order, entry)| {
                let iwr_type = required_value(
                    &entry.iwr_type,
                    &format!("$.system.attributes.{}[{order}].type", iwr_family(kind)),
                )?;
                let duplicate = duplicate_counts.entry(iwr_type.clone()).or_default();
                let id = component_id(&format!(
                    "iwr:{}:{iwr_type}:{duplicate}",
                    iwr_kind_slug(kind)
                ))?;
                *duplicate += 1;
                Ok(CreatureIwr {
                    id,
                    authored_order: order as u32,
                    kind,
                    iwr_type: IwrType::new(iwr_type.clone()).map_err(|_| {
                        conversion_error(
                            &format!("$.system.attributes.{}[{order}].type", iwr_family(kind)),
                            "invalid IWR type slug",
                        )
                    })?,
                    value: presence(&entry.value),
                    exceptions: qualifier_presence(&entry.exceptions)?,
                    double_vs: qualifier_presence(&entry.double_vs)?,
                    apply_once: presence(&entry.apply_once),
                })
            })
            .collect()
    })
}

fn movement(
    core: &NpcCoreSource,
    owner_record_key: &str,
    diagnostics: &mut Vec<NpcCoreDiagnostic>,
) -> Result<FactValue<Vec<CreatureSpeed>>, NpcCoreConversionError> {
    let speed = match &core.attributes {
        SourcePresence::Missing => return Ok(FactValue::Missing),
        SourcePresence::Null => return Ok(FactValue::Null),
        SourcePresence::Value(attributes) => match &attributes.speed {
            SourcePresence::Missing => return Ok(FactValue::Missing),
            SourcePresence::Null => return Ok(FactValue::Null),
            SourcePresence::Value(speed) => speed,
        },
    };
    let mut values = Vec::new();
    if !matches!(speed.value, SourcePresence::Missing) {
        values.push(CreatureSpeed {
            id: component_id("speed:land:0")?,
            authored_order: 0,
            mode: CreatureMovementMode::Land,
            value: presence(&speed.value),
            label: FactValue::Missing,
            details: note_presence(&speed.details),
        });
    }
    if let SourcePresence::Value(other_speeds) = &speed.other_speeds {
        let mut duplicate_counts = BTreeMap::<String, u32>::new();
        for (source_order, other) in other_speeds.iter().enumerate() {
            let source_field =
                format!("$.system.attributes.speed.otherSpeeds[{source_order}].type");
            let identity_key = match &other.speed_type {
                SourcePresence::Missing => "<missing>".to_string(),
                SourcePresence::Null => "<null>".to_string(),
                SourcePresence::Value(value) if value.is_empty() => "<empty>".to_string(),
                SourcePresence::Value(value) => value.clone(),
            };
            let duplicate = duplicate_counts.entry(identity_key).or_default();
            let id = movement_component_id(
                owner_record_key,
                &source_field,
                &other.speed_type,
                source_order,
                *duplicate,
                diagnostics,
            )?;
            *duplicate += 1;
            let mode = movement_mode(&other.speed_type, &source_field, diagnostics);
            values.push(CreatureSpeed {
                id,
                authored_order: values.len() as u32,
                mode,
                value: presence(&other.value),
                label: presence(&other.label),
                details: FactValue::Missing,
            });
        }
    }
    Ok(FactValue::Value(values))
}

fn movement_component_id(
    owner_record_key: &str,
    source_field: &str,
    source_type: &SourcePresence<String>,
    source_order: usize,
    duplicate: u32,
    diagnostics: &mut Vec<NpcCoreDiagnostic>,
) -> Result<CreatureComponentId, NpcCoreConversionError> {
    if let SourcePresence::Value(source_type) = source_type {
        let direct = format!("speed:{source_type}:{duplicate}");
        if !source_type.is_empty() {
            if let Ok(id) = CreatureComponentId::new(direct) {
                return Ok(id);
            }

            let fallback = format!(
                "speed:source:{owner_record_key}:{}:{duplicate}",
                encode_component_id_segment(source_type)
            );
            diagnostics.push(NpcCoreDiagnostic::new(
                NpcCoreDiagnosticKind::ComponentIdSourceFallback,
                source_field,
                source_type,
            ));
            return CreatureComponentId::new(fallback).map_err(|_| {
                conversion_error(
                    "$canonical.id",
                    "source-scoped movement component fallback is invalid",
                )
            });
        }
    }

    let observed = match source_type {
        SourcePresence::Missing => "<missing>",
        SourcePresence::Null => "<null>",
        SourcePresence::Value(_) => "<empty>",
    };
    let fallback =
        format!("speed:scoped:{owner_record_key}:other-speed:{source_order}:{duplicate}");
    diagnostics.push(NpcCoreDiagnostic::new(
        NpcCoreDiagnosticKind::ComponentIdScopedOrdinalFallback,
        source_field,
        observed,
    ));
    CreatureComponentId::new(fallback).map_err(|_| {
        conversion_error(
            "$canonical.id",
            "owner-scoped movement component fallback is invalid",
        )
    })
}

fn movement_mode(
    source_type: &SourcePresence<String>,
    source_field: &str,
    diagnostics: &mut Vec<NpcCoreDiagnostic>,
) -> CreatureMovementMode {
    match source_type {
        SourcePresence::Value(value) => match value.as_str() {
            "land" => CreatureMovementMode::Land,
            "burrow" => CreatureMovementMode::Burrow,
            "climb" => CreatureMovementMode::Climb,
            "fly" => CreatureMovementMode::Fly,
            "swim" => CreatureMovementMode::Swim,
            value => {
                diagnostics.push(NpcCoreDiagnostic::new(
                    NpcCoreDiagnosticKind::UnsupportedOpenValue,
                    source_field,
                    value,
                ));
                CreatureMovementMode::Unsupported(unsupported_string(
                    value,
                    UnsupportedSourceReason::OpenVocabulary,
                ))
            }
        },
        SourcePresence::Missing => CreatureMovementMode::Unsupported(UnsupportedSourceValue {
            shape: UnsupportedSourceShape::Missing,
            value: String::new(),
            reason: UnsupportedSourceReason::AmbiguousLegacyShape,
        }),
        SourcePresence::Null => CreatureMovementMode::Unsupported(UnsupportedSourceValue {
            shape: UnsupportedSourceShape::Null,
            value: String::new(),
            reason: UnsupportedSourceReason::AmbiguousLegacyShape,
        }),
    }
}

fn encode_component_id_segment(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
    }
    encoded
}

fn resources(
    core: &NpcCoreSource,
    diagnostics: &mut Vec<NpcCoreDiagnostic>,
) -> Result<FactValue<Vec<CreatureResource>>, NpcCoreConversionError> {
    map_presence_result(&core.resources, |resources| {
        resources
            .iter()
            .enumerate()
            .map(|(order, (key, resource))| resource_value(key, resource, order, diagnostics))
            .collect()
    })
}

fn resource_value(
    key: &str,
    source: &NpcResourceSource,
    order: usize,
    diagnostics: &mut Vec<NpcCoreDiagnostic>,
) -> Result<CreatureResource, NpcCoreConversionError> {
    let kind = CreatureResourceKind::new(key.to_string()).map_err(|_| {
        conversion_error(
            &format!("$.system.resources.{key}"),
            "invalid resource kind slug",
        )
    })?;
    Ok(CreatureResource {
        id: component_id(&format!("resource:{key}"))?,
        authored_order: order as u32,
        kind,
        label: title_case_slug(key),
        maximum: map_presence(&source.maximum, |amount| {
            resource_amount(amount, key, "max", diagnostics)
        }),
        serialized_value: map_presence(&source.value, |amount| {
            resource_amount(amount, key, "value", diagnostics)
        }),
        source_drift: map_presence(&source.maximum_drift, |value| {
            diagnostics.push(NpcCoreDiagnostic::new(
                NpcCoreDiagnosticKind::ResourceMaximumDrift,
                format!("$.system.resources.{key}.maxx"),
                value.to_string(),
            ));
            vec![CreatureUnsupportedSourceFact {
                field: CreatureUnsupportedSourceField::ResourceMaximumDrift,
                value: UnsupportedSourceValue {
                    shape: UnsupportedSourceShape::Number,
                    value: value.to_string(),
                    reason: UnsupportedSourceReason::SourceFieldDrift,
                },
            }]
        }),
        current_policy: ResourceCurrentPolicy::SerializedValueIsProvenanceOnly,
    })
}

fn resource_amount(
    source: &NpcResourceAmountSource,
    key: &str,
    field: &str,
    diagnostics: &mut Vec<NpcCoreDiagnostic>,
) -> CreatureResourceAmount {
    match source {
        NpcResourceAmountSource::Integer(value) => CreatureResourceAmount::Integer(*value),
        NpcResourceAmountSource::Nested { maximum, value } => maximum
            .as_value()
            .or_else(|| value.as_value())
            .copied()
            .map(CreatureResourceAmount::Integer)
            .unwrap_or_else(|| {
                unsupported_resource(
                    key,
                    field,
                    "{}",
                    UnsupportedSourceShape::Object,
                    diagnostics,
                )
            }),
        NpcResourceAmountSource::Text(value) => unsupported_resource(
            key,
            field,
            value,
            UnsupportedSourceShape::String,
            diagnostics,
        ),
        NpcResourceAmountSource::UnsupportedObject => unsupported_resource(
            key,
            field,
            "{}",
            UnsupportedSourceShape::Object,
            diagnostics,
        ),
    }
}

fn unsupported_resource(
    key: &str,
    field: &str,
    value: &str,
    shape: UnsupportedSourceShape,
    diagnostics: &mut Vec<NpcCoreDiagnostic>,
) -> CreatureResourceAmount {
    diagnostics.push(NpcCoreDiagnostic::new(
        NpcCoreDiagnosticKind::UnsupportedLegacyShape,
        format!("$.system.resources.{key}.{field}"),
        value,
    ));
    CreatureResourceAmount::Unsupported(UnsupportedSourceValue {
        shape,
        value: value.to_string(),
        reason: UnsupportedSourceReason::AmbiguousLegacyShape,
    })
}

fn qualifier_presence(
    source: &SourcePresence<Vec<String>>,
) -> Result<FactValue<Vec<IwrQualifier>>, NpcCoreConversionError> {
    map_presence_result(source, |values| {
        values
            .iter()
            .map(|value| {
                IwrQualifier::new(value.clone()).map_err(|_| {
                    conversion_error(
                        "$.system.attributes.*[].exceptions",
                        "invalid IWR qualifier",
                    )
                })
            })
            .collect()
    })
}

fn note_presence(source: &SourcePresence<String>) -> FactValue<CreatureNote> {
    map_presence(source, |value| CreatureNote::new(value.clone()))
}

fn presence<T: Clone>(source: &SourcePresence<T>) -> FactValue<T> {
    match source {
        SourcePresence::Missing => FactValue::Missing,
        SourcePresence::Null => FactValue::Null,
        SourcePresence::Value(value) => FactValue::Value(value.clone()),
    }
}

fn map_presence<T, U>(source: &SourcePresence<T>, map: impl FnOnce(&T) -> U) -> FactValue<U> {
    match source {
        SourcePresence::Missing => FactValue::Missing,
        SourcePresence::Null => FactValue::Null,
        SourcePresence::Value(value) => FactValue::Value(map(value)),
    }
}

fn map_presence_result<T, U>(
    source: &SourcePresence<T>,
    map: impl FnOnce(&T) -> Result<U, NpcCoreConversionError>,
) -> Result<FactValue<U>, NpcCoreConversionError> {
    match source {
        SourcePresence::Missing => Ok(FactValue::Missing),
        SourcePresence::Null => Ok(FactValue::Null),
        SourcePresence::Value(value) => map(value).map(FactValue::Value),
    }
}

fn nested<T, U>(outer: &SourcePresence<T>, inner: impl FnOnce(&T) -> FactValue<U>) -> FactValue<U> {
    match outer {
        SourcePresence::Missing => FactValue::Missing,
        SourcePresence::Null => FactValue::Null,
        SourcePresence::Value(value) => inner(value),
    }
}

fn nested_result<T, U>(
    outer: &SourcePresence<T>,
    inner: impl FnOnce(&T) -> Result<FactValue<U>, NpcCoreConversionError>,
) -> Result<FactValue<U>, NpcCoreConversionError> {
    match outer {
        SourcePresence::Missing => Ok(FactValue::Missing),
        SourcePresence::Null => Ok(FactValue::Null),
        SourcePresence::Value(value) => inner(value),
    }
}

fn required_value<'a, T>(
    source: &'a SourcePresence<T>,
    path: &str,
) -> Result<&'a T, NpcCoreConversionError> {
    source
        .as_value()
        .ok_or_else(|| conversion_error(path, "required value is missing or null"))
}

fn component_id(value: &str) -> Result<CreatureComponentId, NpcCoreConversionError> {
    CreatureComponentId::new(value.to_string())
        .map_err(|_| conversion_error("$canonical.id", &format!("invalid component id {value:?}")))
}

fn unsupported_string(value: &str, reason: UnsupportedSourceReason) -> UnsupportedSourceValue {
    UnsupportedSourceValue {
        shape: UnsupportedSourceShape::String,
        value: value.to_string(),
        reason,
    }
}

fn iwr_family(kind: CreatureIwrKind) -> &'static str {
    match kind {
        CreatureIwrKind::Immunity => "immunities",
        CreatureIwrKind::Resistance => "resistances",
        CreatureIwrKind::Weakness => "weaknesses",
    }
}

fn iwr_kind_slug(kind: CreatureIwrKind) -> &'static str {
    match kind {
        CreatureIwrKind::Immunity => "immunity",
        CreatureIwrKind::Resistance => "resistance",
        CreatureIwrKind::Weakness => "weakness",
    }
}

fn title_case_slug(value: &str) -> String {
    let mut label = String::new();
    for (index, part) in value.split('-').enumerate() {
        if index > 0 {
            label.push(' ');
        }
        let mut characters = part.chars();
        if let Some(first) = characters.next() {
            label.extend(first.to_uppercase());
            label.extend(characters);
        }
    }
    label
}

fn conversion_error(source_field: &str, message: &str) -> NpcCoreConversionError {
    NpcCoreConversionError {
        source_field: source_field.to_string(),
        message: message.to_string(),
    }
}
