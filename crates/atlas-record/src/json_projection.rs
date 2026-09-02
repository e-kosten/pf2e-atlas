mod creature;

use std::{collections::BTreeMap, ops::Deref};

use atlas_domain::{DetailLevel, RecordKind};
use serde::Serialize;

pub use creature::{
    CreatureAbilitiesJson, CreatureActionCostJson, CreatureActionJson, CreatureArmorClassJson,
    CreatureAvailabilityEvidenceJson, CreatureAvailabilityFieldJson, CreatureAvailabilityJson,
    CreatureAvailabilityStateJson, CreatureContentJson, CreatureContentOwnerJson,
    CreatureContentProvenanceJson, CreatureDamageJson, CreatureDefensesJson, CreatureEquipmentJson,
    CreatureFactProvenanceJson, CreatureFactProvenanceSetJson, CreatureFrequencyJson,
    CreatureHitPointsJson, CreatureInitiativeJson, CreatureIntegerPresenceJson, CreatureIwrJson,
    CreatureLoreJson, CreatureMovementJson, CreatureMovementModeJson,
    CreatureOccurrenceContextJson, CreatureOccurrenceProvenanceJson, CreaturePreparedSpellJson,
    CreatureProvenanceJson, CreatureRelationshipJson, CreatureRelationshipTargetJson,
    CreatureResourceJson, CreatureRitualsJson, CreatureRollJson, CreatureSaveJson,
    CreatureSavesJson, CreatureSenseJson, CreatureShieldJson, CreatureSkillJson,
    CreatureSkillSourceEntryJson, CreatureSkillVariantJson, CreatureSpellAreaJson,
    CreatureSpellDefenseJson, CreatureSpellDurationJson, CreatureSpellJson,
    CreatureSpellRitualJson, CreatureSpellSlotJson, CreatureSpellcastingEntryJson,
    CreatureSpellcastingJson, CreatureStrikeJson, CreatureUnmodeledSkillAvailabilityJson,
    CreatureUnmodeledSkillJson, CreatureUseLimitJson,
};

use crate::{
    AtlasRecord, PresentationBlock, PresentationContent, PresentationFact,
    PresentationRelationship, PresentationRelationshipKind, PresentationSection,
    PresentationSectionKind, RecordBody, RetrievedRecord, build_record_presentation_document,
    render_plain_text,
};

const DESCRIPTION_PREVIEW_WORDS: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordJsonOptions {
    pub detail: DetailLevel,
    pub include_source_json: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordEditionContextJson {
    pub status: RecordEditionStatusJson,
    pub counterpart_lookup: RecordEditionCounterpartLookupJson,
}

impl RecordEditionContextJson {
    const fn lookup_not_performed(status: RecordEditionStatusJson) -> Self {
        Self {
            status,
            counterpart_lookup: RecordEditionCounterpartLookupJson::NotPerformed,
        }
    }

    fn verified(
        status: RecordEditionStatusJson,
        counterparts: Vec<RecordEditionCounterpartJson>,
    ) -> Self {
        Self {
            status,
            counterpart_lookup: RecordEditionCounterpartLookupJson::Verified { counterparts },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum RecordEditionCounterpartLookupJson {
    NotPerformed,
    Verified {
        counterparts: Vec<RecordEditionCounterpartJson>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordEditionCounterpartJson {
    pub role: RecordEditionCounterpartRoleJson,
    pub record_key: String,
    pub title: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordEditionStatusJson {
    Legacy,
    Remaster,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordEditionCounterpartRoleJson {
    LegacyCounterpart,
    RemasteredCounterpart,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordEditionLookup {
    NotPerformed,
    Verified(VerifiedRecordEditionLookup),
}

impl RecordEditionLookup {
    pub fn verified<'a>(
        seed: &RetrievedRecord,
        links: impl IntoIterator<Item = (&'a RetrievedRecord, &'a RetrievedRecord)>,
    ) -> Result<Self, RecordEditionLookupError> {
        VerifiedRecordEditionLookup::new(seed, links).map(Self::Verified)
    }

    fn context_for(
        self,
        record: &AtlasRecord,
    ) -> Result<RecordEditionContextJson, RecordJsonError> {
        let status = RecordEditionStatusJson::from_remaster(record.publication.remaster);
        match self {
            Self::NotPerformed => Ok(RecordEditionContextJson::lookup_not_performed(status)),
            Self::Verified(lookup) => {
                if lookup.seed_record_key != record.identity.key
                    || lookup.seed_remaster != record.publication.remaster
                {
                    return Err(RecordJsonError::EditionLookupSeedMismatch {
                        record_key: record.identity.key.to_string(),
                        record_remaster: record.publication.remaster,
                        lookup_record_key: lookup.seed_record_key.to_string(),
                        lookup_remaster: lookup.seed_remaster,
                    });
                }
                Ok(RecordEditionContextJson::verified(
                    status,
                    lookup.counterparts,
                ))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedRecordEditionLookup {
    seed_record_key: atlas_domain::RecordKey,
    seed_remaster: bool,
    counterparts: Vec<RecordEditionCounterpartJson>,
}

impl VerifiedRecordEditionLookup {
    fn new<'a>(
        seed: &RetrievedRecord,
        links: impl IntoIterator<Item = (&'a RetrievedRecord, &'a RetrievedRecord)>,
    ) -> Result<Self, RecordEditionLookupError> {
        let seed_key = &seed.record.identity.key;
        let seed_remaster = seed.record.publication.remaster;
        let mut counterparts = BTreeMap::new();
        for (remaster, legacy) in links {
            let (seed_side, counterpart, role) = if seed_remaster {
                (
                    remaster,
                    legacy,
                    RecordEditionCounterpartRoleJson::LegacyCounterpart,
                )
            } else {
                (
                    legacy,
                    remaster,
                    RecordEditionCounterpartRoleJson::RemasteredCounterpart,
                )
            };
            if seed_side.record.identity.key != *seed_key
                || seed_side.record.publication.remaster != seed_remaster
                || !remaster.record.publication.remaster
                || legacy.record.publication.remaster
                || counterpart.record.identity.key == *seed_key
            {
                return Err(RecordEditionLookupError::InvalidLink {
                    seed_record_key: seed_key.to_string(),
                    remaster_record_key: remaster.record.identity.key.to_string(),
                    legacy_record_key: legacy.record.identity.key.to_string(),
                });
            }
            let counterpart_key = counterpart.record.identity.key.clone();
            let counterpart_value = (role, counterpart.record.identity.name.clone());
            if let Some(existing) = counterparts.get(&counterpart_key) {
                if existing != &counterpart_value {
                    return Err(RecordEditionLookupError::ConflictingCounterpart {
                        record_key: counterpart_key.to_string(),
                    });
                }
            } else {
                counterparts.insert(counterpart_key, counterpart_value);
            }
        }
        Ok(Self {
            seed_record_key: seed_key.clone(),
            seed_remaster,
            counterparts: counterparts
                .into_iter()
                .map(|(record_key, (role, title))| RecordEditionCounterpartJson {
                    role,
                    record_key: record_key.to_string(),
                    title,
                })
                .collect(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordEditionLookupError {
    InvalidLink {
        seed_record_key: String,
        remaster_record_key: String,
        legacy_record_key: String,
    },
    ConflictingCounterpart {
        record_key: String,
    },
}

impl std::fmt::Display for RecordEditionLookupError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLink {
                seed_record_key,
                remaster_record_key,
                legacy_record_key,
            } => write!(
                formatter,
                "edition link {legacy_record_key} -> {remaster_record_key} is invalid for seed `{seed_record_key}`"
            ),
            Self::ConflictingCounterpart { record_key } => write!(
                formatter,
                "edition lookup returned conflicting identity for counterpart `{record_key}`"
            ),
        }
    }
}

impl std::error::Error for RecordEditionLookupError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordJsonContext {
    edition: RecordEditionLookup,
    relationships: RecordRelationshipLookupJson,
    include_provenance_evidence: bool,
}

impl RecordJsonContext {
    pub fn without_lookups(_record: &AtlasRecord) -> Self {
        Self {
            edition: RecordEditionLookup::NotPerformed,
            relationships: RecordRelationshipLookupJson::NotPerformed,
            include_provenance_evidence: false,
        }
    }

    pub fn with_edition_lookup(mut self, edition: RecordEditionLookup) -> Self {
        self.edition = edition;
        self
    }

    pub fn with_relationships(mut self, relationships: RecordRelationshipLookupJson) -> Self {
        self.relationships = relationships;
        self
    }

    pub fn with_provenance_evidence(mut self) -> Self {
        self.include_provenance_evidence = true;
        self
    }
}

impl RecordRelationshipLookupJson {
    pub fn verified(
        record_key: &atlas_domain::RecordKey,
        outgoing: &[crate::ReferenceEdge],
        backlinks: &[crate::ReferenceEdge],
    ) -> Result<Self, RecordRelationshipContextError> {
        let mut relationships = outgoing
            .iter()
            .map(|edge| {
                RecordCanonicalRelationshipJson::from_edge(
                    record_key,
                    RecordRelationshipDirectionJson::Reference,
                    edge,
                )
            })
            .chain(backlinks.iter().map(|edge| {
                RecordCanonicalRelationshipJson::from_edge(
                    record_key,
                    RecordRelationshipDirectionJson::Backlink,
                    edge,
                )
            }))
            .collect::<Result<Vec<_>, _>>()?;
        relationships.sort_by(|left, right| {
            left.direction
                .cmp(&right.direction)
                .then_with(|| left.target_record_key.cmp(&right.target_record_key))
                .then_with(|| left.label.cmp(&right.label))
                .then_with(|| {
                    left.provenance
                        .source_kind
                        .cmp(&right.provenance.source_kind)
                })
        });
        Ok(Self::Verified { relationships })
    }
}

impl RecordCanonicalRelationshipJson {
    fn from_edge(
        record_key: &atlas_domain::RecordKey,
        direction: RecordRelationshipDirectionJson,
        edge: &crate::ReferenceEdge,
    ) -> Result<Self, RecordRelationshipContextError> {
        let matches_seed = match direction {
            RecordRelationshipDirectionJson::Reference => &edge.from_record_key == record_key,
            RecordRelationshipDirectionJson::Backlink => &edge.to_record_key == record_key,
        };
        if !matches_seed {
            return Err(RecordRelationshipContextError {
                record_key: record_key.to_string(),
                direction,
                from_record_key: edge.from_record_key.to_string(),
                to_record_key: edge.to_record_key.to_string(),
            });
        }
        let target_record_key = match direction {
            RecordRelationshipDirectionJson::Reference => &edge.to_record_key,
            RecordRelationshipDirectionJson::Backlink => &edge.from_record_key,
        };
        Ok(Self {
            direction,
            kind: edge.relation_kind,
            label: edge
                .display_text
                .clone()
                .unwrap_or_else(|| edge.reference_text.clone()),
            target_record_key: target_record_key.to_string(),
            provenance: RecordRelationshipProvenanceJson {
                from_record_key: edge.from_record_key.to_string(),
                to_record_key: edge.to_record_key.to_string(),
                source_kind: edge.source_kind,
                visibility: edge.visibility,
            },
        })
    }
}

impl RecordEditionStatusJson {
    const fn from_remaster(remaster: bool) -> Self {
        if remaster {
            Self::Remaster
        } else {
            Self::Legacy
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum RecordRelationshipLookupJson {
    NotPerformed,
    Verified {
        relationships: Vec<RecordCanonicalRelationshipJson>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordCanonicalRelationshipJson {
    pub direction: RecordRelationshipDirectionJson,
    pub kind: crate::ReferenceRelationKind,
    pub label: String,
    pub target_record_key: String,
    #[serde(skip)]
    pub provenance: RecordRelationshipProvenanceJson,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordRelationshipDirectionJson {
    Reference,
    Backlink,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordRelationshipProvenanceJson {
    pub from_record_key: String,
    pub to_record_key: String,
    pub source_kind: crate::ContentSourceKind,
    pub visibility: crate::ContentVisibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordRelationshipContextError {
    pub record_key: String,
    pub direction: RecordRelationshipDirectionJson,
    pub from_record_key: String,
    pub to_record_key: String,
}

impl std::fmt::Display for RecordRelationshipContextError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} relationship {} -> {} does not belong to record `{}`",
            match self.direction {
                RecordRelationshipDirectionJson::Reference => "outgoing",
                RecordRelationshipDirectionJson::Backlink => "backlink",
            },
            self.from_record_key,
            self.to_record_key,
            self.record_key
        )
    }
}

impl std::error::Error for RecordRelationshipContextError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordJsonError {
    MissingCreatureBody {
        record_key: String,
    },
    UnexpectedCreatureBody {
        record_key: String,
    },
    EditionLookupSeedMismatch {
        record_key: String,
        record_remaster: bool,
        lookup_record_key: String,
        lookup_remaster: bool,
    },
}

impl std::fmt::Display for RecordJsonError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingCreatureBody { record_key } => write!(
                formatter,
                "retrieved creature record `{record_key}` is missing its canonical creature body"
            ),
            Self::UnexpectedCreatureBody { record_key } => write!(
                formatter,
                "retrieved non-creature record `{record_key}` has an unexpected canonical creature body"
            ),
            Self::EditionLookupSeedMismatch {
                record_key,
                record_remaster,
                lookup_record_key,
                lookup_remaster,
            } => write!(
                formatter,
                "verified edition lookup seed `{lookup_record_key}` (remaster={lookup_remaster}) does not match projected record `{record_key}` (remaster={record_remaster})"
            ),
        }
    }
}

impl std::error::Error for RecordJsonError {}

/// The one durable CLI/agent record presentation contract.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RecordJson {
    #[serde(flatten)]
    pub base: RecordJsonBase,
    #[serde(flatten)]
    pub presentation: RecordPresentationJson,
}

impl Deref for RecordJson {
    type Target = RecordJsonBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl RecordJson {
    pub fn generic_sections(&self) -> &[RecordSectionJson] {
        match &self.presentation {
            RecordPresentationJson::Creature { .. } => &[],
            RecordPresentationJson::Unmigrated { sections, .. } => sections,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RecordJsonBase {
    pub key: String,
    pub name: String,
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rarity: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<RecordSourceJson>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub supplementary_sections: Vec<RecordSectionJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "presentation_type", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)] // The serialized variants intentionally stay flat.
pub enum RecordPresentationJson {
    Creature {
        #[serde(skip_serializing_if = "Option::is_none")]
        teaser: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        adjustment: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        initiative: Option<CreatureInitiativeJson>,
        #[serde(skip_serializing_if = "Option::is_none")]
        abilities: Option<CreatureAbilitiesJson>,
        #[serde(skip_serializing_if = "Option::is_none")]
        defenses: Option<CreatureDefensesJson>,
        #[serde(skip_serializing_if = "Option::is_none")]
        perception: Option<CreaturePerceptionJson>,
        #[serde(skip_serializing_if = "Option::is_none")]
        languages: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        skills: Option<Vec<CreatureSkillJson>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        movement: Option<CreatureMovementJson>,
        #[serde(skip_serializing_if = "Option::is_none")]
        resources: Option<Vec<CreatureResourceJson>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        strikes: Option<Vec<CreatureStrikeJson>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        actions: Option<Vec<CreatureActionJson>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        spellcasting: Option<CreatureSpellcastingJson>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rituals: Option<CreatureRitualsJson>,
        #[serde(skip_serializing_if = "Option::is_none")]
        equipment: Option<Vec<CreatureEquipmentJson>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        lore: Option<Vec<CreatureLoreJson>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<Vec<CreatureContentJson>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        relationships: Option<Vec<CreatureRelationshipJson>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        provenance: Option<CreatureProvenanceJson>,
        #[serde(skip_serializing_if = "Option::is_none")]
        edition: Option<RecordEditionContextJson>,
        #[serde(skip_serializing_if = "Option::is_none")]
        record_relationships: Option<RecordRelationshipLookupJson>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        availability: Vec<CreatureAvailabilityJson>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        unmodeled_skill_availability: Vec<CreatureUnmodeledSkillAvailabilityJson>,
        #[serde(skip_serializing_if = "Option::is_none")]
        availability_evidence: Option<Vec<CreatureAvailabilityEvidenceJson>>,
    },
    Unmigrated {
        migration: UnmigratedRegistryJson,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        sections: Vec<RecordSectionJson>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreaturePerceptionJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_vision: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub senses: Option<Vec<CreatureSenseJson>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnmigratedRegistryJson {
    pub family: &'static str,
    pub plan_id: &'static str,
    pub acceptance_checkpoint: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordSourceJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack: Option<RecordPackJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_remaster: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foundry: Option<FoundrySourceJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordPackJson {
    pub name: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FoundrySourceJson {
    pub document_type: String,
    pub record_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordSectionJson {
    pub kind: &'static str,
    pub title: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub blocks: Vec<RecordBlockJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RecordBlockJson {
    FactList {
        facts: Vec<RecordFactJson>,
    },
    Prose {
        text: String,
    },
    Content {
        content: PresentationContent,
    },
    Relationships {
        relationships: Vec<RecordRelationshipJson>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordFactJson {
    pub key: String,
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordRelationshipJson {
    pub kind: &'static str,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_key: Option<String>,
}

pub fn record_json(
    retrieved: &RetrievedRecord,
    options: RecordJsonOptions,
) -> Result<RecordJson, RecordJsonError> {
    record_json_with_context(
        retrieved,
        options,
        RecordJsonContext::without_lookups(&retrieved.record),
    )
}

pub fn record_json_with_context(
    retrieved: &RetrievedRecord,
    options: RecordJsonOptions,
    context: RecordJsonContext,
) -> Result<RecordJson, RecordJsonError> {
    let record = &retrieved.record;
    let RecordJsonContext {
        edition,
        relationships,
        include_provenance_evidence,
    } = context;
    let document = build_record_presentation_document(record);
    let detailed_sections = sections_for_detail(record, &document.sections, options.detail);
    let presentation = match (record.classification.kind, &retrieved.body) {
        (RecordKind::Creature, Some(RecordBody::Creature(creature))) => {
            creature::creature_presentation(
                creature,
                options.detail,
                Some(edition.context_for(record)?),
                Some(relationships),
                include_provenance_evidence,
                matches!(options.detail, DetailLevel::Preview | DetailLevel::Standard)
                    .then(|| record.content.description())
                    .flatten()
                    .and_then(|document| {
                        truncate_words(&render_plain_text(document), DESCRIPTION_PREVIEW_WORDS)
                    }),
            )
        }
        (RecordKind::Creature, None) => {
            return Err(RecordJsonError::MissingCreatureBody {
                record_key: record.identity.key.to_string(),
            });
        }
        (_, Some(RecordBody::Creature(_))) => {
            return Err(RecordJsonError::UnexpectedCreatureBody {
                record_key: record.identity.key.to_string(),
            });
        }
        (_, None) => RecordPresentationJson::Unmigrated {
            migration: unmigrated_registry(record),
            sections: generic_sections(&detailed_sections),
        },
    };

    Ok(RecordJson {
        base: RecordJsonBase {
            key: record.identity.key.to_string(),
            name: record.identity.name.clone(),
            kind: record.classification.kind.as_str(),
            level: (options.detail != DetailLevel::Summary)
                .then_some(record.classification.level)
                .flatten(),
            rarity: (options.detail != DetailLevel::Summary)
                .then(|| {
                    record
                        .classification
                        .rarity
                        .map(|rarity| rarity.as_str().to_string())
                })
                .flatten(),
            traits: if options.detail != DetailLevel::Summary {
                record.classification.traits.clone()
            } else {
                Vec::new()
            },
            source: (options.detail != DetailLevel::Summary)
                .then(|| {
                    source_json(
                        record,
                        options.detail,
                        include_provenance_evidence
                            || record.classification.kind != RecordKind::Creature,
                    )
                })
                .flatten(),
            supplementary_sections: if record.classification.kind == RecordKind::Creature {
                creature_supplementary_sections(&detailed_sections)
            } else {
                supplementary_sections(&detailed_sections)
            },
            source_json: options
                .include_source_json
                .then(|| record.provenance.raw_json.clone())
                .flatten(),
        },
        presentation,
    })
}

fn source_json(
    record: &AtlasRecord,
    detail: DetailLevel,
    include_provenance_evidence: bool,
) -> Option<RecordSourceJson> {
    let full = detail == DetailLevel::Full;
    Some(RecordSourceJson {
        publication_title: record.publication.title.clone(),
        pack: Some(RecordPackJson {
            name: record.identity.pack().to_string(),
            label: record.foundry.pack_label.clone(),
        }),
        category: full.then(|| record.publication.category.as_str()),
        publication_remaster: full.then_some(record.publication.remaster),
        source_path: (full && include_provenance_evidence)
            .then(|| record.provenance.source_path.clone()),
        foundry: (full && include_provenance_evidence).then(|| FoundrySourceJson {
            document_type: record.foundry.document_type.as_str().to_string(),
            record_type: record.foundry.record_type.as_str().to_string(),
        }),
    })
}

fn unmigrated_registry(record: &AtlasRecord) -> UnmigratedRegistryJson {
    use crate::FoundryRecordType;

    let (family, plan_id) = match record.classification.kind {
        // This private registry is called only from the non-creature branch in
        // `record_json`; retain an explicit entry so the match stays total.
        RecordKind::Creature => ("creature", "D2"),
        RecordKind::Hazard => ("hazard", "H1"),
        RecordKind::Spell => ("spell_or_ritual", "H2"),
        RecordKind::Equipment => match record.foundry.record_type {
            FoundryRecordType::Weapon | FoundryRecordType::Ammo => ("weapon_or_ammunition", "H3"),
            FoundryRecordType::Armor | FoundryRecordType::Shield => ("armor_or_shield", "H4"),
            FoundryRecordType::Consumable => ("consumable", "H5"),
            _ => ("remaining_physical_item", "H6"),
        },
        RecordKind::Feat
        | RecordKind::Affliction
        | RecordKind::Rule
        | RecordKind::CharacterOption => ("rules_content", "H7"),
        RecordKind::Lore | RecordKind::CampaignFeature => ("journal_or_table_content", "H8"),
        RecordKind::Character | RecordKind::Companion | RecordKind::Army | RecordKind::Vehicle => {
            ("actor_family", "H9")
        }
        RecordKind::Tooling => ("generated_container_or_embedded_context", "H10"),
    };
    UnmigratedRegistryJson {
        family,
        plan_id,
        acceptance_checkpoint: "H12",
    }
}

fn sections_for_detail(
    record: &AtlasRecord,
    sections: &[PresentationSection],
    detail: DetailLevel,
) -> Vec<RecordSectionJson> {
    let mut projected = Vec::new();
    if let Some(summary) = sections
        .iter()
        .find(|section| section.kind == PresentationSectionKind::Summary)
        .and_then(section_json)
    {
        projected.push(summary);
    }
    match detail {
        DetailLevel::Summary => {}
        DetailLevel::Preview => {
            projected.extend(
                sections
                    .iter()
                    .filter(|section| {
                        !matches!(
                            section.kind,
                            PresentationSectionKind::Summary
                                | PresentationSectionKind::Description
                                | PresentationSectionKind::Details
                        )
                    })
                    .filter_map(section_json),
            );
            if let Some(preview) = description_preview_section(record) {
                projected.push(preview);
            }
        }
        DetailLevel::Description => {
            if let Some(description) = sections
                .iter()
                .find(|section| section.kind == PresentationSectionKind::Description)
                .and_then(section_json)
            {
                projected.push(description);
            }
        }
        DetailLevel::Standard | DetailLevel::Full => {
            projected.extend(
                sections
                    .iter()
                    .filter(|section| section.kind != PresentationSectionKind::Summary)
                    .filter_map(section_json),
            );
        }
    }
    projected
}

fn description_preview_section(record: &AtlasRecord) -> Option<RecordSectionJson> {
    let description = record.content.description()?;
    let preview = truncate_words(&render_plain_text(description), DESCRIPTION_PREVIEW_WORDS)?;
    Some(RecordSectionJson {
        kind: PresentationSectionKind::DescriptionPreview.as_str(),
        title: PresentationSectionKind::DescriptionPreview
            .default_title()
            .to_string(),
        blocks: vec![RecordBlockJson::Prose { text: preview }],
    })
}

fn truncate_words(text: &str, max_words: usize) -> Option<String> {
    let mut words = text.split_whitespace();
    let mut preview = Vec::new();
    for _ in 0..max_words {
        if let Some(word) = words.next() {
            preview.push(word);
        } else {
            break;
        }
    }
    if preview.is_empty() {
        return None;
    }
    let mut output = preview.join(" ");
    if words.next().is_some() {
        output.push_str("...");
    }
    Some(output)
}

fn supplementary_sections(sections: &[RecordSectionJson]) -> Vec<RecordSectionJson> {
    sections
        .iter()
        .filter_map(|section| section_with_blocks(section, false))
        .collect()
}

fn creature_supplementary_sections(sections: &[RecordSectionJson]) -> Vec<RecordSectionJson> {
    sections
        .iter()
        .filter(|section| matches!(section.kind, "references" | "backlinks"))
        .filter_map(|section| section_with_blocks(section, false))
        .collect()
}

fn generic_sections(sections: &[RecordSectionJson]) -> Vec<RecordSectionJson> {
    sections
        .iter()
        .filter_map(|section| section_with_blocks(section, true))
        .collect()
}

fn section_with_blocks(section: &RecordSectionJson, facts: bool) -> Option<RecordSectionJson> {
    let blocks = section
        .blocks
        .iter()
        .filter(|block| matches!(block, RecordBlockJson::FactList { .. }) == facts)
        .cloned()
        .collect::<Vec<_>>();
    (!blocks.is_empty()).then(|| RecordSectionJson {
        kind: section.kind,
        title: section.title.clone(),
        blocks,
    })
}

fn section_json(section: &PresentationSection) -> Option<RecordSectionJson> {
    let blocks = section
        .blocks
        .iter()
        .filter_map(block_json)
        .collect::<Vec<_>>();
    (!blocks.is_empty()).then(|| RecordSectionJson {
        kind: section.kind.as_str(),
        title: section.title.clone(),
        blocks,
    })
}

fn block_json(block: &PresentationBlock) -> Option<RecordBlockJson> {
    match block {
        PresentationBlock::FactList(facts) => {
            let facts = facts.iter().map(fact_json).collect::<Vec<_>>();
            (!facts.is_empty()).then_some(RecordBlockJson::FactList { facts })
        }
        PresentationBlock::Prose(text) => {
            (!text.text.trim().is_empty()).then(|| RecordBlockJson::Prose {
                text: text.text.clone(),
            })
        }
        PresentationBlock::Content(content) => {
            (!content.is_empty()).then_some(RecordBlockJson::Content {
                content: content.clone(),
            })
        }
        PresentationBlock::Relationships(relationships) => {
            let relationships = relationships
                .iter()
                .map(relationship_json)
                .collect::<Vec<_>>();
            (!relationships.is_empty()).then_some(RecordBlockJson::Relationships { relationships })
        }
    }
}

fn fact_json(fact: &PresentationFact) -> RecordFactJson {
    RecordFactJson {
        key: fact.key.clone(),
        label: fact.label.clone(),
        value: fact.value.clone(),
    }
}

fn relationship_json(relationship: &PresentationRelationship) -> RecordRelationshipJson {
    RecordRelationshipJson {
        kind: match relationship.kind {
            PresentationRelationshipKind::Reference => "reference",
            PresentationRelationshipKind::Backlink => "backlink",
        },
        label: relationship.label.clone(),
        record_key: relationship.record_key.as_ref().map(ToString::to_string),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use atlas_domain::{PackName, PublicationCategory, RecordId, RecordKey};

    use super::*;
    use crate::{
        ContentId, ContentIdentityStability, ContentKey, ContentOrigin, ContentOwner,
        ContentProvenance, ContentRole, ContentSourceKind, ContentVisibility,
        DuplicateContentStatus, FactValue, FoundryDocumentType, FoundryRecordInfo,
        FoundryRecordType, OwnedRichContentDocument, RecordClassification, RecordContent,
        RecordContentDocument, RecordIdentity, RecordMechanics, RecordProvenance,
        RecordPublication, RecordRequirements, RecordTaxonomy, RecordTiming, RecordVisibility,
        RichDocument, RichNode,
    };

    #[test]
    fn non_creature_is_registry_bound_and_raw_is_an_independent_opt_in() {
        let record = fixture_record();
        let without_raw = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Preview,
                include_source_json: false,
            },
        )
        .expect("non-creature projection");
        let with_raw = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Preview,
                include_source_json: true,
            },
        )
        .expect("non-creature projection with raw source");

        assert!(without_raw.source_json.is_none());
        assert!(with_raw.source_json.is_some());
        assert!(matches!(
            without_raw.presentation,
            RecordPresentationJson::Unmigrated {
                migration: UnmigratedRegistryJson { plan_id: "H7", .. },
                ..
            }
        ));
    }

    #[test]
    fn record_json_fails_closed_on_carrier_body_mismatches() {
        let mut missing = fixture_creature_record();
        missing.body = None;
        assert!(matches!(
            record_json(
                &missing,
                RecordJsonOptions {
                    detail: DetailLevel::Standard,
                    include_source_json: false,
                }
            ),
            Err(RecordJsonError::MissingCreatureBody { .. })
        ));

        let mut unexpected = fixture_record();
        unexpected.body = fixture_creature_record().body;
        assert!(matches!(
            record_json(
                &unexpected,
                RecordJsonOptions {
                    detail: DetailLevel::Standard,
                    include_source_json: false,
                }
            ),
            Err(RecordJsonError::UnexpectedCreatureBody { .. })
        ));
    }

    #[test]
    fn creature_exposes_direct_typed_scan_fields_without_generic_mechanics() {
        let json = record_json(
            &fixture_creature_record(),
            RecordJsonOptions {
                detail: DetailLevel::Standard,
                include_source_json: false,
            },
        )
        .expect("creature projection");
        let RecordPresentationJson::Creature {
            defenses,
            perception,
            languages,
            movement,
            ..
        } = &json.presentation
        else {
            panic!("creature presentation")
        };

        let defenses = defenses.as_ref().expect("standard defenses");
        assert_eq!(defenses.ac.as_ref().expect("ac").value, Some(25));
        assert_eq!(defenses.hp.as_ref().expect("hp").maximum, Some(80));
        assert_eq!(
            defenses
                .saves
                .as_ref()
                .expect("saves")
                .fortitude
                .as_ref()
                .expect("fortitude")
                .value,
            Some(14)
        );
        assert_eq!(perception.as_ref().expect("perception").modifier, Some(12));
        assert_eq!(
            languages.as_deref(),
            Some(["common".to_string()].as_slice())
        );
        assert_eq!(
            movement.as_ref().expect("standard movement").modes[0].value_feet,
            Some(25)
        );
        assert!(json.generic_sections().is_empty());
        assert!(json.supplementary_sections.iter().all(|section| {
            section
                .blocks
                .iter()
                .all(|block| !matches!(block, RecordBlockJson::FactList { .. }))
        }));
    }

    #[test]
    fn creature_detail_levels_keep_the_tagged_schema_with_purposeful_hydration() {
        let record = fixture_creature_record();
        let summary = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Summary,
                include_source_json: false,
            },
        )
        .expect("summary projection");
        let preview = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Preview,
                include_source_json: false,
            },
        )
        .expect("preview projection");
        let description = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Description,
                include_source_json: false,
            },
        )
        .expect("description projection");
        let standard = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Standard,
                include_source_json: false,
            },
        )
        .expect("standard projection");
        let full = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Full,
                include_source_json: false,
            },
        )
        .expect("full projection");

        let summary_value = serde_json::to_value(&summary).expect("summary json");
        let description_value = serde_json::to_value(&description).expect("description json");
        for field in [
            "defenses",
            "perception",
            "languages",
            "skills",
            "movement",
            "resources",
            "strikes",
            "actions",
            "spellcasting",
        ] {
            assert!(
                summary_value.get(field).is_none(),
                "summary omitted {field}"
            );
            assert!(
                description_value.get(field).is_none(),
                "description omitted {field}"
            );
        }
        assert!(summary_value.get("level").is_none());
        assert!(summary_value.get("source").is_none());
        assert!(summary_value.get("edition").is_none());
        let RecordPresentationJson::Creature { defenses, .. } = preview.presentation else {
            panic!("preview creature")
        };
        assert_eq!(
            defenses
                .expect("preview defenses")
                .ac
                .expect("preview ac")
                .value,
            Some(25)
        );
        let preview_value = serde_json::to_value(
            record_json(
                &record,
                RecordJsonOptions {
                    detail: DetailLevel::Preview,
                    include_source_json: false,
                },
            )
            .expect("preview projection"),
        )
        .expect("preview json");
        let preview_strike = &preview_value["strikes"][0];
        assert!(preview_strike.get("rolls").is_none());
        assert!(preview_strike.get("damage").is_none());
        assert!(preview_strike.get("modes").is_none());
        let standard_value = serde_json::to_value(standard).expect("standard json");
        let standard_strike = &standard_value["strikes"][0];
        assert!(standard_strike["rolls"].is_array());
        assert!(standard_strike["damage"].is_array());
        assert!(standard_value["actions"][0]["rolls"].is_array());
        assert_eq!(standard_value["skills"][0]["slug"], "lore");
        assert_eq!(standard_value["skills"][0]["note"], "stage performances");
        assert_eq!(standard_value["resources"][0]["maximum"], 1);
        assert_eq!(standard_value["resources"][0]["serialized_value"], 1);
        assert_eq!(standard_value["defenses"]["resistances"][0]["value"], 10);
        assert_eq!(standard_value["defenses"]["weaknesses"][0]["value"], 10);
        assert_eq!(
            standard_value["actions"][0]["action_cost"]["kind"],
            "actions"
        );
        assert_eq!(standard_value["actions"][0]["action_cost"]["actions"], 1);
        assert_eq!(standard_value["actions"][0]["frequency"]["maximum"], 1);
        assert_eq!(standard_value["actions"][0]["frequency"]["period"], "PT1M");
        assert_eq!(
            standard_value["actions"][0]["frequency"]["display"],
            "1 per minute"
        );
        assert_eq!(
            standard_value["actions"][0]["frequency"]["serialized_value"],
            1
        );
        assert_eq!(standard_value["spellcasting"]["entries"][0]["order"], 0);
        assert_eq!(
            standard_value["spellcasting"]["entries"][0]["slots"][0]["maximum"],
            1
        );
        assert_eq!(
            standard_value["spellcasting"]["entries"][0]["spells"][0]["order"],
            2
        );
        assert_eq!(
            standard_value["spellcasting"]["entries"][0]["spells"][0]["context"]["rank"],
            1
        );
        assert_eq!(
            standard_value["spellcasting"]["entries"][0]["spells"][0]["context"]["uses"]["maximum"],
            1
        );
        assert!(
            standard_value["spellcasting"]["entries"][0]
                .get("id")
                .is_none()
        );
        assert!(
            standard_value["spellcasting"]["entries"][0]
                .get("target_entity_id")
                .is_none()
        );
        assert!(
            standard_value["spellcasting"]["entries"][0]["spells"][0]
                .get("id")
                .is_none()
        );
        assert!(
            standard_value["spellcasting"]["entries"][0]["spells"][0]["context"]
                .get("slot")
                .is_none()
        );
        assert!(
            standard_value["spellcasting"]["entries"][0]["spells"][0]
                .get("parent_entry_id")
                .is_none()
        );
        let prepared = &standard_value["spellcasting"]["entries"][0]["slots"][0]["prepared"][0];
        assert!(prepared.get("id").is_none());
        assert_eq!(prepared["name"], "Magic Missile");
        let full_value = serde_json::to_value(full).expect("full json");
        assert!(full_value.get("provenance").is_none());
        assert!(full_value.get("relationships").is_none());
        assert!(full_value["source"].get("source_path").is_none());
        assert!(full_value["source"].get("foundry").is_none());
        assert!(full_value["strikes"][0].get("provenance").is_none());
        assert_eq!(full_value["edition"]["status"], "remaster");
        assert_eq!(
            full_value["edition"]["counterpart_lookup"]["state"],
            "not_performed"
        );
        assert!(
            full_value["edition"]["counterpart_lookup"]
                .get("counterparts")
                .is_none()
        );
        assert!(description.supplementary_sections.is_empty());
        assert!(description_value.get("content").is_none());

        let provenance_value = serde_json::to_value(
            record_json_with_context(
                &record,
                RecordJsonOptions {
                    detail: DetailLevel::Full,
                    include_source_json: false,
                },
                RecordJsonContext::without_lookups(&record.record).with_provenance_evidence(),
            )
            .expect("provenance projection"),
        )
        .expect("provenance json");
        assert_eq!(
            provenance_value["provenance"]["source_contract_version"],
            "test"
        );
        assert_eq!(
            provenance_value["provenance"]["facts"]["defenses"]["field"],
            "defenses"
        );
        assert_eq!(
            provenance_value["source"]["source_path"],
            "packs/actions/treat-wounds.json"
        );
        assert!(provenance_value["source"]["foundry"].is_object());
        assert!(provenance_value["strikes"][0]["provenance"].is_object());
    }

    #[test]
    fn verified_edition_lookup_derives_status_roles_order_and_deduplicates() {
        let legacy = edition_creature_record("legacy-pack:seed", "Legacy Seed", false);
        let remaster_z = edition_creature_record("remaster-pack:zeta", "Zeta Remaster", true);
        let remaster_a = edition_creature_record("remaster-pack:alpha", "Alpha Remaster", true);
        let lookup = RecordEditionLookup::verified(
            &legacy,
            [
                (&remaster_z, &legacy),
                (&remaster_a, &legacy),
                (&remaster_a, &legacy),
            ],
        )
        .expect("verified B4 lookup");
        let value = serde_json::to_value(
            record_json_with_context(
                &legacy,
                RecordJsonOptions {
                    detail: DetailLevel::Full,
                    include_source_json: false,
                },
                RecordJsonContext::without_lookups(&legacy.record).with_edition_lookup(lookup),
            )
            .expect("edition projection"),
        )
        .expect("json");
        assert_eq!(value["edition"]["status"], "legacy");
        let counterparts = value["edition"]["counterpart_lookup"]["counterparts"]
            .as_array()
            .expect("counterparts");
        assert_eq!(counterparts.len(), 2);
        assert_eq!(counterparts[0]["record_key"], "remaster-pack:alpha");
        assert_eq!(counterparts[1]["record_key"], "remaster-pack:zeta");
        assert!(
            counterparts
                .iter()
                .all(|counterpart| counterpart["role"] == "remastered_counterpart")
        );

        let remaster_lookup =
            RecordEditionLookup::verified(&remaster_a, std::iter::once((&remaster_a, &legacy)))
                .expect("remaster seed lookup");
        let remaster_value = serde_json::to_value(
            record_json_with_context(
                &remaster_a,
                RecordJsonOptions {
                    detail: DetailLevel::Standard,
                    include_source_json: false,
                },
                RecordJsonContext::without_lookups(&remaster_a.record)
                    .with_edition_lookup(remaster_lookup),
            )
            .expect("remaster projection"),
        )
        .expect("json");
        assert_eq!(remaster_value["edition"]["status"], "remaster");
        assert_eq!(
            remaster_value["edition"]["counterpart_lookup"]["counterparts"][0]["role"],
            "legacy_counterpart"
        );
    }

    #[test]
    fn verified_zero_counterparts_is_distinct_from_lookup_not_performed() {
        let record = fixture_creature_record();
        let lookup = RecordEditionLookup::verified(
            &record,
            std::iter::empty::<(&RetrievedRecord, &RetrievedRecord)>(),
        )
        .expect("verified empty B4 lookup");
        let value = serde_json::to_value(
            record_json_with_context(
                &record,
                RecordJsonOptions {
                    detail: DetailLevel::Standard,
                    include_source_json: false,
                },
                RecordJsonContext::without_lookups(&record.record).with_edition_lookup(lookup),
            )
            .expect("verified empty edition projection"),
        )
        .expect("json");
        assert_eq!(value["edition"]["counterpart_lookup"]["state"], "verified");
        assert_eq!(
            value["edition"]["counterpart_lookup"]["counterparts"]
                .as_array()
                .expect("verified counterparts")
                .len(),
            0
        );
    }

    #[test]
    fn verified_edition_lookup_rejects_seed_status_contradiction_and_invalid_pairs() {
        let remaster = edition_creature_record("shared-pack:seed", "Remaster Seed", true);
        let legacy = edition_creature_record("legacy-pack:counterpart", "Legacy", false);
        let lookup =
            RecordEditionLookup::verified(&remaster, std::iter::once((&remaster, &legacy)))
                .expect("verified lookup");
        let contradictory_seed = edition_creature_record("shared-pack:seed", "Legacy Seed", false);
        assert!(matches!(
            record_json_with_context(
                &contradictory_seed,
                RecordJsonOptions {
                    detail: DetailLevel::Standard,
                    include_source_json: false,
                },
                RecordJsonContext::without_lookups(&contradictory_seed.record)
                    .with_edition_lookup(lookup),
            ),
            Err(RecordJsonError::EditionLookupSeedMismatch { .. })
        ));

        let invalid_remaster = edition_creature_record("remaster-pack:invalid", "Invalid", false);
        assert!(matches!(
            RecordEditionLookup::verified(&legacy, std::iter::once((&invalid_remaster, &legacy)),),
            Err(RecordEditionLookupError::InvalidLink { .. })
        ));

        let remaster_one = edition_creature_record("remaster-pack:same", "First Title", true);
        let remaster_conflict =
            edition_creature_record("remaster-pack:same", "Conflicting Title", true);
        assert!(matches!(
            RecordEditionLookup::verified(
                &legacy,
                [(&remaster_one, &legacy), (&remaster_conflict, &legacy)],
            ),
            Err(RecordEditionLookupError::ConflictingCounterpart { .. })
        ));
    }

    #[test]
    fn canonical_reference_and_backlink_context_preserves_identity_without_ordinary_debug_data() {
        let record = fixture_creature_record();
        let record_key = record.record.identity.key.clone();
        let outgoing = crate::ReferenceEdge {
            from_record_key: record_key.clone(),
            to_record_key: RecordKey::parse("rules:target").expect("target key"),
            display_text: Some("Exact Target".to_string()),
            reference_text: "@UUID[target]".to_string(),
            relation_kind: crate::ReferenceRelationKind::Reference,
            source_kind: ContentSourceKind::Description,
            visibility: ContentVisibility::Public,
        };
        let backlink = crate::ReferenceEdge {
            from_record_key: RecordKey::parse("creatures:caller").expect("caller key"),
            to_record_key: record_key.clone(),
            display_text: Some("Exact Caller".to_string()),
            reference_text: "@UUID[caller]".to_string(),
            relation_kind: crate::ReferenceRelationKind::Embed,
            source_kind: ContentSourceKind::PublicNotes,
            visibility: ContentVisibility::GmOnly,
        };
        let verified = RecordRelationshipLookupJson::verified(
            &record_key,
            std::slice::from_ref(&outgoing),
            std::slice::from_ref(&backlink),
        )
        .expect("verified relationships");
        let RecordRelationshipLookupJson::Verified { relationships } = &verified else {
            unreachable!()
        };
        assert_eq!(
            relationships[0].provenance.source_kind,
            ContentSourceKind::Description
        );
        assert_eq!(
            relationships[1].provenance.visibility,
            ContentVisibility::GmOnly
        );

        for detail in [
            DetailLevel::Summary,
            DetailLevel::Preview,
            DetailLevel::Description,
            DetailLevel::Standard,
            DetailLevel::Full,
        ] {
            let value = serde_json::to_value(
                record_json_with_context(
                    &record,
                    RecordJsonOptions {
                        detail,
                        include_source_json: false,
                    },
                    RecordJsonContext::without_lookups(&record.record)
                        .with_relationships(verified.clone()),
                )
                .expect("relationship projection"),
            )
            .expect("json");
            if detail == DetailLevel::Summary {
                assert!(value.get("record_relationships").is_none());
                continue;
            }
            assert_eq!(value["record_relationships"]["state"], "verified");
            let relationships = value["record_relationships"]["relationships"]
                .as_array()
                .expect("relationships");
            assert_eq!(relationships.len(), 2);
            let reference = relationships
                .iter()
                .find(|relationship| relationship["direction"] == "reference")
                .expect("reference");
            assert_eq!(reference["label"], "Exact Target");
            assert_eq!(reference["target_record_key"], "rules:target");
            assert_eq!(reference["kind"], "reference");
            assert!(reference.get("provenance").is_none());
            let backlink = relationships
                .iter()
                .find(|relationship| relationship["direction"] == "backlink")
                .expect("backlink");
            assert_eq!(backlink["label"], "Exact Caller");
            assert_eq!(backlink["target_record_key"], "creatures:caller");
            assert_eq!(backlink["kind"], "embed");
            assert!(backlink.get("provenance").is_none());
        }
    }

    #[test]
    fn canonical_relationship_context_rejects_edges_for_another_seed() {
        let record_key = RecordKey::parse("creatures:seed").expect("seed key");
        let unrelated = crate::ReferenceEdge {
            from_record_key: RecordKey::parse("creatures:other").expect("other key"),
            to_record_key: RecordKey::parse("rules:target").expect("target key"),
            display_text: None,
            reference_text: "target".to_string(),
            relation_kind: crate::ReferenceRelationKind::Reference,
            source_kind: ContentSourceKind::Description,
            visibility: ContentVisibility::Public,
        };
        let error = RecordRelationshipLookupJson::verified(
            &record_key,
            std::slice::from_ref(&unrelated),
            &[],
        )
        .expect_err("unrelated edge must fail");
        assert_eq!(error.record_key, "creatures:seed");
        assert_eq!(error.from_record_key, "creatures:other");
        assert_eq!(error.to_record_key, "rules:target");
    }

    #[test]
    fn creature_content_placement_claims_once_and_preserves_owner_identity() {
        let mut retrieved = fixture_creature_record();
        let RecordBody::Creature(creature) = retrieved.body.as_mut().expect("creature body");
        let owner = creature.identity.record_key.clone();
        creature.content.documents = vec![
            owned_document(&owner, "general", ContentOwner::Record(owner.clone()), 0),
            owned_document(
                &owner,
                "jaws-content",
                ContentOwner::CreatureOccurrence(
                    crate::CreatureOccurrenceId::new("jaws").expect("occurrence"),
                ),
                1,
            ),
            owned_document(
                &owner,
                "jaws-entity-content",
                ContentOwner::CreatureEntity(crate::CreatureEntityId::new("jaws").expect("entity")),
                2,
            ),
            owned_document(
                &owner,
                "unclaimed-content",
                ContentOwner::CreatureEntity(
                    crate::CreatureEntityId::new("absent-entity").expect("entity"),
                ),
                3,
            ),
            owned_document(
                &owner,
                "entry-content",
                ContentOwner::CreatureOccurrence(
                    crate::CreatureOccurrenceId::new("occult-innate").expect("occurrence"),
                ),
                4,
            ),
        ];
        let placement = crate::place_creature_content(creature);
        assert_eq!(placement.record_owned_documents(creature).count(), 1);
        assert_eq!(placement.unclaimed_documents(creature).count(), 1);
        assert_eq!(
            placement
                .entity_documents(
                    creature,
                    &crate::CreatureEntityId::new("jaws").expect("entity"),
                )
                .count(),
            1
        );
        assert_eq!(
            placement
                .occurrence_documents(
                    creature,
                    &crate::CreatureOccurrenceId::new("jaws").expect("occurrence"),
                )
                .count(),
            1
        );
        assert_eq!(
            placement
                .standalone_documents(
                    creature,
                    &crate::CreatureOccurrenceId::new("jaws").expect("occurrence"),
                )
                .count(),
            2
        );
        assert!(placement.is_claimed(1));
        assert!(placement.is_claimed(2));
        assert!(!placement.is_claimed(3));
        assert!(placement.is_claimed(4));
        let full = serde_json::to_value(
            record_json(
                &retrieved,
                RecordJsonOptions {
                    detail: DetailLevel::Full,
                    include_source_json: false,
                },
            )
            .expect("full projection"),
        )
        .expect("json");
        assert_eq!(
            full["content"].as_array().expect("general content").len(),
            2
        );
        assert_eq!(full["content"][0]["content_key"], "general");
        assert_eq!(full["content"][1]["content_key"], "unclaimed-content");
        assert_eq!(
            full["strikes"][0]["content"][0]["content_key"],
            "jaws-content"
        );
        assert_eq!(
            full["strikes"][0]["content"][1]["content_key"],
            "jaws-entity-content"
        );
        assert_eq!(
            full["spellcasting"]["entries"][0]["content"][0]["content_key"],
            "entry-content"
        );
        assert!(full["strikes"][0]["content"][0].get("owner").is_none());
        let description = serde_json::to_value(
            record_json(
                &retrieved,
                RecordJsonOptions {
                    detail: DetailLevel::Description,
                    include_source_json: false,
                },
            )
            .expect("description projection"),
        )
        .expect("json");
        assert_eq!(description["content"].as_array().expect("content").len(), 5);
        assert!(description.get("strikes").is_none());
    }

    #[test]
    fn missing_or_null_embedded_entities_keep_unclaimed_content_reachable_in_global_order() {
        for embedded_state in ["missing", "null"] {
            let mut retrieved = fixture_creature_record();
            let RecordBody::Creature(creature) = retrieved.body.as_mut().expect("creature body");
            creature.embedded_entities.value = if embedded_state == "missing" {
                FactValue::Missing
            } else {
                FactValue::Null
            };
            let owner = creature.identity.record_key.clone();
            creature.content.documents = vec![
                owned_document(
                    &owner,
                    "record-third",
                    ContentOwner::Record(owner.clone()),
                    3,
                ),
                owned_document(
                    &owner,
                    "occurrence-first",
                    ContentOwner::CreatureOccurrence(
                        crate::CreatureOccurrenceId::new("absent-occurrence").expect("occurrence"),
                    ),
                    1,
                ),
                owned_document(
                    &owner,
                    "entity-second",
                    ContentOwner::CreatureEntity(
                        crate::CreatureEntityId::new("absent-entity").expect("entity"),
                    ),
                    2,
                ),
            ];
            let placement = crate::place_creature_content(creature);
            assert_eq!(
                placement
                    .all_documents(creature)
                    .map(|document| document.id.content_key.as_str())
                    .collect::<Vec<_>>(),
                ["occurrence-first", "entity-second", "record-third"]
            );
            assert_eq!(
                placement
                    .unclaimed_documents(creature)
                    .map(|document| document.id.content_key.as_str())
                    .collect::<Vec<_>>(),
                ["occurrence-first", "entity-second"]
            );
            assert_eq!(
                placement
                    .general_documents(creature)
                    .map(|document| document.id.content_key.as_str())
                    .collect::<Vec<_>>(),
                ["occurrence-first", "entity-second", "record-third"]
            );
            let full = serde_json::to_value(
                record_json(
                    &retrieved,
                    RecordJsonOptions {
                        detail: DetailLevel::Full,
                        include_source_json: false,
                    },
                )
                .expect("full projection"),
            )
            .expect("json");
            assert_eq!(
                full["content"]
                    .as_array()
                    .expect("unclaimed full content")
                    .iter()
                    .map(|document| document["content_key"].as_str().expect("content key"))
                    .collect::<Vec<_>>(),
                ["occurrence-first", "entity-second", "record-third"]
            );
            assert!(full["availability"].as_array().is_some_and(|causes| {
                causes.iter().any(|cause| {
                    cause["field"] == "embedded_entities" && cause["state"] == embedded_state
                })
            }));
        }
    }

    #[test]
    fn every_content_bucket_uses_one_global_deterministic_order() {
        let mut retrieved = fixture_creature_record();
        let RecordBody::Creature(creature) = retrieved.body.as_mut().expect("creature body");
        let owner = creature.identity.record_key.clone();
        let jaws_entity = crate::CreatureEntityId::new("jaws").expect("entity");
        let jaws_occurrence = crate::CreatureOccurrenceId::new("jaws").expect("occurrence");
        creature.content.documents = vec![
            owned_document(
                &owner,
                "entity-third",
                ContentOwner::CreatureEntity(jaws_entity.clone()),
                3,
            ),
            owned_document(
                &owner,
                "occurrence-second",
                ContentOwner::CreatureOccurrence(jaws_occurrence.clone()),
                2,
            ),
            owned_document(
                &owner,
                "record-fourth",
                ContentOwner::Record(owner.clone()),
                4,
            ),
            owned_document(
                &owner,
                "occurrence-first",
                ContentOwner::CreatureOccurrence(jaws_occurrence.clone()),
                0,
            ),
            owned_document(
                &owner,
                "unclaimed-fifth",
                ContentOwner::CreatureEntity(
                    crate::CreatureEntityId::new("absent").expect("entity"),
                ),
                5,
            ),
            owned_document(
                &owner,
                "entity-first",
                ContentOwner::CreatureEntity(jaws_entity.clone()),
                1,
            ),
        ];
        let placement = crate::place_creature_content(creature);
        assert_eq!(
            placement
                .all_documents(creature)
                .map(|document| document.id.content_key.as_str())
                .collect::<Vec<_>>(),
            [
                "occurrence-first",
                "entity-first",
                "occurrence-second",
                "entity-third",
                "record-fourth",
                "unclaimed-fifth",
            ]
        );
        assert_eq!(
            placement
                .entity_documents(creature, &jaws_entity)
                .map(|document| document.id.content_key.as_str())
                .collect::<Vec<_>>(),
            ["entity-first", "entity-third"]
        );
        assert_eq!(
            placement
                .occurrence_documents(creature, &jaws_occurrence)
                .map(|document| document.id.content_key.as_str())
                .collect::<Vec<_>>(),
            ["occurrence-first", "occurrence-second"]
        );
        assert_eq!(
            placement
                .documents_for_occurrence(creature, &jaws_occurrence)
                .map(|document| document.id.content_key.as_str())
                .collect::<Vec<_>>(),
            [
                "occurrence-first",
                "entity-first",
                "occurrence-second",
                "entity-third",
            ]
        );
        assert_eq!(
            placement
                .general_documents(creature)
                .map(|document| document.id.content_key.as_str())
                .collect::<Vec<_>>(),
            ["record-fourth", "unclaimed-fifth"]
        );
    }

    #[test]
    fn repeated_same_target_occurrences_keep_identity_multiplicity_and_content() {
        let mut retrieved = fixture_creature_record();
        let RecordBody::Creature(creature) = retrieved.body.as_mut().expect("creature body");
        let owner = creature.identity.record_key.clone();
        let FactValue::Value(embedded) = &mut creature.embedded_entities.value else {
            panic!("embedded entities")
        };
        let mut repeated = embedded.occurrences[1].clone();
        repeated.id = crate::CreatureOccurrenceId::new("second-jaws").expect("occurrence");
        repeated.authored_order = 4;
        embedded.occurrences.push(repeated);
        creature.content.documents = vec![
            owned_document(
                &owner,
                "first-jaws",
                ContentOwner::CreatureOccurrence(
                    crate::CreatureOccurrenceId::new("jaws").expect("occurrence"),
                ),
                1,
            ),
            owned_document(
                &owner,
                "second-jaws",
                ContentOwner::CreatureOccurrence(
                    crate::CreatureOccurrenceId::new("second-jaws").expect("occurrence"),
                ),
                2,
            ),
        ];
        let full = record_json(
            &retrieved,
            RecordJsonOptions {
                detail: DetailLevel::Full,
                include_source_json: false,
            },
        )
        .expect("full projection");
        let RecordPresentationJson::Creature { strikes, .. } = &full.presentation else {
            panic!("creature presentation")
        };
        let typed_strikes = strikes.as_ref().expect("typed strikes");
        assert_eq!(typed_strikes[0].id, "jaws");
        assert_eq!(typed_strikes[1].id, "second-jaws");
        assert_eq!(typed_strikes[0].target_entity_id.as_deref(), Some("jaws"));
        assert_eq!(typed_strikes[1].target_entity_id.as_deref(), Some("jaws"));
        let full = serde_json::to_value(full).expect("json");
        let strikes = full["strikes"].as_array().expect("strikes");
        assert_eq!(strikes.len(), 2);
        assert!(strikes.iter().all(|strike| strike.get("id").is_none()));
        assert!(
            strikes
                .iter()
                .all(|strike| strike.get("target_entity_id").is_none())
        );
        assert_eq!(strikes[0]["content"][0]["content_key"], "first-jaws");
        assert_eq!(strikes[1]["content"][0]["content_key"], "second-jaws");
    }

    #[test]
    fn association_mutation_fails_only_the_ambiguous_row() {
        let mut retrieved = fixture_creature_record();
        let RecordBody::Creature(creature) = retrieved.body.as_mut().expect("creature body");
        let owner = creature.identity.record_key.clone();
        let duplicate = owned_document(
            &owner,
            "jaws-content",
            ContentOwner::CreatureOccurrence(
                crate::CreatureOccurrenceId::new("jaws").expect("occurrence"),
            ),
            1,
        );
        creature.content.documents = vec![duplicate.clone(), duplicate];
        let placement = crate::place_creature_content(creature);
        let placement_state = (
            placement.unclaimed_documents(creature).count(),
            placement.association_failed_documents(creature).count(),
            placement.is_claimed(0),
            placement.is_claimed(1),
        );
        let standard = serde_json::to_value(
            record_json(
                &retrieved,
                RecordJsonOptions {
                    detail: DetailLevel::Standard,
                    include_source_json: false,
                },
            )
            .expect("standard projection"),
        )
        .expect("json");
        assert_eq!(standard["strikes"].as_array().expect("strikes").len(), 1);
        assert_eq!(standard["actions"].as_array().expect("actions").len(), 1);
        assert!(
            standard["availability"]
                .as_array()
                .expect("availability")
                .iter()
                .any(|cause| cause["field"] == "content_association")
        );
        assert!(standard["availability"].as_array().is_some_and(|causes| {
            causes
                .iter()
                .all(|cause| cause.get("component_id").is_none())
        }));
        assert_eq!(placement_state, (0, 2, false, false));
        let full = serde_json::to_value(
            record_json(
                &retrieved,
                RecordJsonOptions {
                    detail: DetailLevel::Full,
                    include_source_json: false,
                },
            )
            .expect("full projection"),
        )
        .expect("json");
        assert_eq!(
            full["content"]
                .as_array()
                .expect("failed association remains reachable")
                .iter()
                .map(|document| document["content_key"].as_str().expect("content key"))
                .collect::<Vec<_>>(),
            ["jaws-content", "jaws-content"]
        );
        assert!(full["strikes"][0].get("content").is_none());
    }

    #[test]
    fn prefailed_occurrence_content_remains_reachable_once_in_full() {
        for case in ["missing-entity", "duplicate-entity", "duplicate-occurrence"] {
            let mut retrieved = fixture_creature_record();
            let RecordBody::Creature(creature) = retrieved.body.as_mut().expect("creature body");
            let owner = creature.identity.record_key.clone();
            let jaws = crate::CreatureEntityId::new("jaws").expect("entity");
            creature.content.documents = vec![owned_document(
                &owner,
                "prefailed-content",
                ContentOwner::CreatureEntity(jaws.clone()),
                1,
            )];
            let FactValue::Value(embedded) = &mut creature.embedded_entities.value else {
                panic!("embedded entities")
            };
            match case {
                "missing-entity" => embedded.entities.retain(|entity| entity.id != jaws),
                "duplicate-entity" => {
                    let duplicate = embedded
                        .entities
                        .iter()
                        .find(|entity| entity.id == jaws)
                        .expect("jaws entity")
                        .clone();
                    embedded.entities.push(duplicate);
                }
                "duplicate-occurrence" => {
                    let other_id = crate::CreatureEntityId::new("other-jaws").expect("entity");
                    let mut other_entity = embedded
                        .entities
                        .iter()
                        .find(|entity| entity.id == jaws)
                        .expect("jaws entity")
                        .clone();
                    other_entity.id = other_id.clone();
                    embedded.entities.push(other_entity);
                    let mut duplicate = embedded
                        .occurrences
                        .iter()
                        .find(|occurrence| occurrence.id.as_str() == "jaws")
                        .expect("jaws occurrence")
                        .clone();
                    duplicate.target = crate::CreatureEntityTarget::ActorOwned(other_id);
                    duplicate.authored_order += 20;
                    embedded.occurrences.push(duplicate);
                }
                _ => unreachable!(),
            }
            let placement = crate::place_creature_content(creature);
            assert_eq!(placement.all_documents(creature).count(), 1, "{case}");
            assert_eq!(
                placement.association_failed_documents(creature).count(),
                1,
                "{case}"
            );
            assert_eq!(placement.general_documents(creature).count(), 1, "{case}");
            assert_eq!(
                placement
                    .association_safe_general_documents(creature)
                    .count(),
                0,
                "{case}"
            );
            assert!(!placement.is_claimed(0), "{case}");

            let full = serde_json::to_value(
                record_json(
                    &retrieved,
                    RecordJsonOptions {
                        detail: DetailLevel::Full,
                        include_source_json: false,
                    },
                )
                .expect("full projection"),
            )
            .expect("json");
            assert_eq!(
                full["content"]
                    .as_array()
                    .expect("prefailed content remains general")
                    .iter()
                    .filter(|document| document["content_key"] == "prefailed-content")
                    .count(),
                1,
                "{case}"
            );
            assert!(
                full["strikes"]
                    .as_array()
                    .expect("strikes")
                    .iter()
                    .filter(|strike| strike["label"] == "Jaws")
                    .all(|strike| strike.get("content").is_none()),
                "{case}"
            );
            assert!(
                full["availability"]
                    .as_array()
                    .expect("availability")
                    .iter()
                    .any(|cause| cause["field"] == "content_association"),
                "{case}"
            );
        }
    }

    #[test]
    fn availability_distinguishes_required_absence_and_malformed_values() {
        let mut retrieved = fixture_creature_record();
        let RecordBody::Creature(creature) = retrieved.body.as_mut().expect("creature body");
        creature.perception.value = FactValue::Null;
        creature.adjustment.value = FactValue::Null;
        creature.languages.value = FactValue::Value(crate::CreatureLanguages {
            values: FactValue::Value(Vec::new()),
            details: FactValue::Missing,
        });
        creature.skills.value = FactValue::Value(vec![crate::CreatureSkill {
            id: crate::CreatureComponentId::new("malformed-skill").expect("skill id"),
            authored_order: 0,
            source_entries: vec![crate::CreatureSkillSourceEntry {
                authored_key: "acrobatics+13".to_string(),
                modifier: FactValue::Null,
            }],
            kind: crate::CreatureSkillKind::Unmodeled,
            label: "acrobatics+13".to_string(),
            modifier: FactValue::Null,
            note: FactValue::Missing,
            variants: FactValue::Value(Vec::new()),
            source_item_id: FactValue::Missing,
            unmodeled: FactValue::Value(crate::CreatureUnmodeledSkill {
                authored_key: "acrobatics+13".to_string(),
                base: FactValue::Null,
                reason: crate::CreatureUnmodeledSkillReason::UnknownAuthoredKey,
            }),
        }]);
        let FactValue::Value(embedded) = &mut creature.embedded_entities.value else {
            panic!("embedded entities")
        };
        embedded.occurrences[3].capability =
            crate::CreatureCapability::Unsupported(crate::CreatureUnsupportedCapability {
                source_item_type: "effect".to_string(),
                source_slug: FactValue::Null,
                traits: FactValue::Value(Vec::new()),
                unsupported_notes: Vec::new(),
            });
        let standard = serde_json::to_value(
            record_json(
                &retrieved,
                RecordJsonOptions {
                    detail: DetailLevel::Standard,
                    include_source_json: false,
                },
            )
            .expect("standard projection"),
        )
        .expect("json");
        let availability = standard["availability"].as_array().expect("availability");
        assert!(
            availability
                .iter()
                .any(|cause| { cause["field"] == "size" && cause["state"] == "missing" })
        );
        assert!(
            availability
                .iter()
                .any(|cause| { cause["field"] == "perception" && cause["state"] == "null" })
        );
        let unmodeled = standard["unmodeled_skill_availability"]
            .as_array()
            .and_then(|rows| rows.first())
            .expect("unmodeled availability");
        assert_eq!(unmodeled["skill_id"], "malformed-skill");
        assert_eq!(unmodeled["authored_order"], 0);
        assert_eq!(unmodeled["authored_key"], "acrobatics+13");
        assert_eq!(unmodeled["modifier"]["state"], "null");
        assert_eq!(
            unmodeled["message"],
            "The source supplied an unrecognized skill key."
        );
        let unsupported = availability
            .iter()
            .find(|cause| cause["field"] == "unsupported_capability")
            .expect("unsupported capability availability");
        assert_eq!(unsupported["authored_key"], "effect");
        assert!(
            !availability.iter().any(|cause| {
                matches!(cause["field"].as_str(), Some("adjustment" | "languages"))
            })
        );
        assert_eq!(
            standard["languages"].as_array().expect("languages").len(),
            0
        );
        assert!(standard.get("relationships").is_none());
    }

    #[test]
    fn unmodeled_skill_availability_preserves_occurrence_order_multiplicity_and_presence() {
        let mut retrieved = fixture_creature_record();
        let RecordBody::Creature(creature) = retrieved.body.as_mut().expect("creature body");
        creature.skills.value = FactValue::Value(
            [
                ("skill-value", 8, FactValue::Value(17)),
                ("skill-missing", 3, FactValue::Missing),
                ("skill-null", 5, FactValue::Null),
            ]
            .into_iter()
            .map(|(id, authored_order, base)| crate::CreatureSkill {
                id: crate::CreatureComponentId::new(id).expect("skill id"),
                authored_order,
                source_entries: vec![crate::CreatureSkillSourceEntry {
                    authored_key: "same-authored-key".to_string(),
                    modifier: base.clone(),
                }],
                kind: crate::CreatureSkillKind::Unmodeled,
                label: "same-authored-key".to_string(),
                modifier: base.clone(),
                note: FactValue::Missing,
                variants: FactValue::Value(Vec::new()),
                source_item_id: FactValue::Missing,
                unmodeled: FactValue::Value(crate::CreatureUnmodeledSkill {
                    authored_key: "same-authored-key".to_string(),
                    base,
                    reason: crate::CreatureUnmodeledSkillReason::UnknownAuthoredKey,
                }),
            })
            .collect(),
        );

        let project = |retrieved: &RetrievedRecord| {
            let value = serde_json::to_value(
                record_json(
                    retrieved,
                    RecordJsonOptions {
                        detail: DetailLevel::Standard,
                        include_source_json: false,
                    },
                )
                .expect("standard projection"),
            )
            .expect("json");
            value["unmodeled_skill_availability"]
                .as_array()
                .expect("unmodeled rows")
                .clone()
        };
        let forward = project(&retrieved);
        assert_eq!(forward.len(), 3);
        assert_eq!(forward[0]["skill_id"], "skill-value");
        assert_eq!(forward[1]["skill_id"], "skill-missing");
        assert_eq!(forward[2]["skill_id"], "skill-null");
        assert_eq!(forward[0]["modifier"]["value"], 17);
        assert_eq!(forward[1]["modifier"]["state"], "missing");
        assert_eq!(forward[2]["modifier"]["state"], "null");

        let RecordBody::Creature(creature) = retrieved.body.as_mut().expect("creature body");
        let FactValue::Value(skills) = &mut creature.skills.value else {
            panic!("skills")
        };
        skills.reverse();
        let reversed = project(&retrieved);
        assert_eq!(reversed[0]["skill_id"], "skill-null");
        assert_eq!(reversed[1]["skill_id"], "skill-missing");
        assert_eq!(reversed[2]["skill_id"], "skill-value");
        assert_ne!(
            forward, reversed,
            "authored occurrence order must be observable"
        );
    }

    #[test]
    fn populated_unsupported_values_survive_as_complete_typed_causes() {
        let mut retrieved = fixture_creature_record();
        let RecordBody::Creature(creature) = retrieved.body.as_mut().expect("creature body");
        creature.adjustment.value = FactValue::Value(crate::CreatureAdjustment::Unsupported(
            unsupported_value("mythic"),
        ));
        creature.initiative.value = FactValue::Value(crate::CreatureInitiative {
            statistic: FactValue::Value(crate::CreatureInitiativeStatistic::Unsupported(
                unsupported_value("initiative-source"),
            )),
        });
        creature.source_alliance.value = FactValue::Value(
            crate::CreatureSourceAlliance::Unsupported(unsupported_value("alliance-source")),
        );
        let defenses = creature.defenses.value.as_value().expect("defenses");
        let mut defenses = defenses.clone();
        let hit_points = defenses.hit_points.as_value().expect("hit points");
        let mut hit_points = hit_points.clone();
        hit_points.value = FactValue::Value(crate::CreatureNumber::Unsupported(unsupported_value(
            "hp-source",
        )));
        defenses.hit_points = FactValue::Value(hit_points);
        creature.defenses.value = FactValue::Value(defenses);
        let perception = creature.perception.value.as_value().expect("perception");
        let mut perception = perception.clone();
        perception.senses = FactValue::Value(vec![crate::CreatureSense {
            id: crate::CreatureComponentId::new("odd-sense").expect("sense id"),
            authored_order: 0,
            sense_type: crate::SenseType::new("odd-sense").expect("sense type"),
            acuity: FactValue::Value(crate::SenseAcuity::Unsupported(unsupported_value(
                "acuity-source",
            ))),
            range: FactValue::Value(30),
        }]);
        creature.perception.value = FactValue::Value(perception);
        let skills = creature.skills.value.as_value().expect("skills");
        let mut skills = skills.clone();
        skills[0].variants = FactValue::Value(vec![crate::CreatureSkillVariant {
            id: crate::CreatureComponentId::new("odd-variant").expect("variant id"),
            authored_order: 0,
            modifier: FactValue::Missing,
            label: FactValue::Missing,
            predicate: FactValue::Value(vec![crate::CreaturePredicate::Unsupported(
                unsupported_value("predicate-source"),
            )]),
        }]);
        creature.skills.value = FactValue::Value(skills);
        let movement = creature.movement.value.as_value().expect("movement");
        let mut movement = movement.clone();
        movement[0].mode =
            crate::CreatureMovementMode::Unsupported(unsupported_value("movement-source"));
        creature.movement.value = FactValue::Value(movement);
        let resources = creature.resources.value.as_value().expect("resources");
        let mut resources = resources.clone();
        resources[0].maximum = FactValue::Value(crate::CreatureResourceAmount::Unsupported(
            unsupported_value("resource-maximum-source"),
        ));
        resources[0].serialized_value =
            FactValue::Value(crate::CreatureResourceAmount::Unsupported(
                unsupported_value("resource-serialized-source"),
            ));
        resources[0].source_drift = FactValue::Value(vec![crate::CreatureUnsupportedSourceFact {
            field: crate::CreatureUnsupportedSourceField::ResourceMaximumDrift,
            value: unsupported_value("resource-drift-source"),
        }]);
        creature.resources.value = FactValue::Value(resources);

        let FactValue::Value(embedded) = &mut creature.embedded_entities.value else {
            panic!("embedded entities")
        };
        embedded.actor_spellcasting = FactValue::Value(crate::CreatureActorSpellcastingContext {
            rituals_dc: FactValue::Value(crate::CreatureSourceScalar::Unsupported(
                unsupported_value("ritual-dc-source"),
            )),
            unsupported_notes: vec![unsupported_note("actor.spellcasting", "actor-note-source")],
        });
        let crate::CreatureCapability::SpellcastingEntry(entry) =
            &mut embedded.occurrences[0].capability
        else {
            panic!("spellcasting entry")
        };
        entry.preparation = FactValue::Value(crate::CreatureSpellPreparation::Unsupported(
            unsupported_value("preparation-source"),
        ));
        entry.slots = FactValue::Value(vec![crate::CreatureSpellSlot {
            rank: 1,
            maximum: FactValue::Value(crate::CreatureSourceScalar::Unsupported(unsupported_value(
                "slot-maximum-source",
            ))),
            serialized_value: FactValue::Value(crate::CreatureSourceScalar::Unsupported(
                unsupported_value("slot-serialized-source"),
            )),
            prepared: FactValue::Value(vec![crate::CreaturePreparedSpellSlot::Unsupported(
                unsupported_value("prepared-slot-source"),
            )]),
        }]);
        entry.unsupported_notes = vec![unsupported_note("entry.path", "entry-note-source")];

        let crate::CreatureCapability::Strike(strike) = &mut embedded.occurrences[1].capability
        else {
            panic!("strike")
        };
        strike.action_cost =
            crate::CreatureActionCost::Unsupported(unsupported_value("action-cost-source"));
        let damage = strike.damage.as_value().expect("damage");
        let mut damage = damage.clone();
        damage[0].kinds = FactValue::Value(vec![crate::CreatureDamageKind::Unsupported(
            unsupported_value("damage-kind-source"),
        )]);
        damage[0].apply_modifier = FactValue::Value(crate::CreatureSourceScalar::Unsupported(
            unsupported_value("damage-modifier-source"),
        ));
        strike.damage = FactValue::Value(damage);
        strike.unsupported_notes = vec![unsupported_note("strike.path", "strike-note-source")];

        let crate::CreatureCapability::Spell(spell) = &mut embedded.occurrences[2].capability
        else {
            panic!("spell")
        };
        spell.ritual = FactValue::Value(crate::CreatureRitualContext {
            primary_check: FactValue::Missing,
            secondary_casters: FactValue::Value(crate::CreatureSourceScalar::Unsupported(
                unsupported_value("ritual-secondary-source"),
            )),
            secondary_checks: FactValue::Missing,
        });
        spell.defense = FactValue::Value(crate::CreatureSpellDefense {
            save: FactValue::Value(crate::CreatureSpellSave::Unsupported(unsupported_value(
                "spell-save-source",
            ))),
            basic: FactValue::Missing,
        });
        spell.unsupported_notes = vec![
            unsupported_note("spell.path", "spell-note-source"),
            unsupported_note_with_shape("spell.empty", crate::UnsupportedSourceShape::String, ""),
            unsupported_note_with_shape(
                "spell.false",
                crate::UnsupportedSourceShape::Boolean,
                "false",
            ),
            unsupported_note_with_shape("spell.zero", crate::UnsupportedSourceShape::Number, "0"),
            unsupported_note_with_shape("spell.null", crate::UnsupportedSourceShape::Null, "null"),
            unsupported_note_with_shape(
                "spell.object",
                crate::UnsupportedSourceShape::Object,
                "{\"value\":0}",
            ),
        ];

        let ordinary = serde_json::to_value(
            record_json(
                &retrieved,
                RecordJsonOptions {
                    detail: DetailLevel::Full,
                    include_source_json: false,
                },
            )
            .expect("ordinary full projection"),
        )
        .expect("ordinary JSON");
        assert!(ordinary.get("availability_evidence").is_none());
        assert!(ordinary["availability"].as_array().is_some_and(|causes| {
            causes.iter().all(|cause| {
                cause.get("component_id").is_none()
                    && cause.get("source_shape").is_none()
                    && cause.get("source_reason").is_none()
                    && cause.get("source_path").is_none()
                    && !matches!(
                        cause["field"].as_str(),
                        Some(
                            "resource_serialized_value"
                                | "resource_source_drift"
                                | "spell_slot_serialized_value"
                                | "unsupported_mechanic"
                        )
                    )
            })
        }));

        let project_availability = |record: &RetrievedRecord| {
            let value = serde_json::to_value(
                record_json_with_context(
                    record,
                    RecordJsonOptions {
                        detail: DetailLevel::Full,
                        include_source_json: false,
                    },
                    RecordJsonContext::without_lookups(&record.record).with_provenance_evidence(),
                )
                .expect("full projection"),
            )
            .expect("json");
            let product = value["availability"]
                .as_array()
                .expect("product availability");
            assert!(product.iter().all(|cause| {
                cause.get("component_id").is_none()
                    && cause.get("source_shape").is_none()
                    && cause.get("source_reason").is_none()
                    && cause.get("source_path").is_none()
                    && cause["field"] != "unsupported_mechanic"
            }));
            assert!(product.iter().all(|cause| {
                cause["source_value"] != ""
                    && cause["source_value"] != "false"
                    && cause["source_value"] != "0"
                    && cause["source_value"] != "null"
                    && !cause["source_value"].is_object()
            }));
            value["availability_evidence"]
                .as_array()
                .expect("availability evidence")
                .clone()
        };
        let forward = project_availability(&retrieved);
        let fields = forward
            .iter()
            .filter_map(|cause| cause["field"].as_str())
            .collect::<std::collections::BTreeSet<_>>();
        for expected in [
            "adjustment",
            "initiative_statistic",
            "source_alliance",
            "hit_points_value",
            "sense_acuity",
            "skill_predicate",
            "movement_mode",
            "resource_maximum",
            "resource_serialized_value",
            "resource_source_drift",
            "ritual_difficulty_class",
            "action_cost",
            "spell_preparation",
            "spell_slot_maximum",
            "spell_slot_serialized_value",
            "prepared_spell_slot",
            "spell_ritual_secondary_casters",
            "spell_defense_save",
            "damage_kind",
            "damage_apply_modifier",
            "unsupported_mechanic",
        ] {
            assert!(fields.contains(expected), "missing {expected}");
        }
        for source in [
            "mythic",
            "initiative-source",
            "resource-maximum-source",
            "slot-maximum-source",
            "prepared-slot-source",
            "damage-kind-source",
            "damage-modifier-source",
            "spell-save-source",
        ] {
            assert!(
                forward.iter().any(|cause| cause["source_value"] == source),
                "missing exact source value {source}"
            );
        }
        assert!(forward.iter().any(|cause| {
            cause["field"] == "unsupported_mechanic"
                && cause["source_path"] == "spell.path"
                && cause["source_value"] == "spell-note-source"
        }));
        for (path, shape, value) in [
            ("spell.empty", "string", ""),
            ("spell.false", "boolean", "false"),
            ("spell.zero", "number", "0"),
            ("spell.null", "null", "null"),
            ("spell.object", "object", "{\"value\":0}"),
        ] {
            assert!(forward.iter().any(|cause| {
                cause["source_path"] == path
                    && cause["source_shape"] == shape
                    && cause["source_value"] == value
            }));
        }

        let RecordBody::Creature(creature) = retrieved.body.as_mut().expect("creature body");
        if let FactValue::Value(embedded) = &mut creature.embedded_entities.value {
            embedded.occurrences.reverse();
        }
        let reverse = project_availability(&retrieved);
        assert_eq!(
            forward, reverse,
            "availability evidence order must be mutation-stable"
        );
    }

    fn unsupported_value(value: &str) -> crate::UnsupportedSourceValue {
        crate::UnsupportedSourceValue {
            shape: crate::UnsupportedSourceShape::String,
            value: value.to_string(),
            reason: crate::UnsupportedSourceReason::OpenVocabulary,
        }
    }

    fn unsupported_note(source_path: &str, value: &str) -> crate::UnsupportedMechanicNote {
        crate::UnsupportedMechanicNote {
            source_path: source_path.to_string(),
            value: unsupported_value(value),
        }
    }

    fn unsupported_note_with_shape(
        source_path: &str,
        shape: crate::UnsupportedSourceShape,
        value: &str,
    ) -> crate::UnsupportedMechanicNote {
        crate::UnsupportedMechanicNote {
            source_path: source_path.to_string(),
            value: crate::UnsupportedSourceValue {
                shape,
                value: value.to_string(),
                reason: crate::UnsupportedSourceReason::SourceFieldDrift,
            },
        }
    }

    fn owned_document(
        owner_key: &RecordKey,
        content_key: &str,
        owner: ContentOwner,
        order: u32,
    ) -> OwnedRichContentDocument {
        OwnedRichContentDocument::new(
            ContentId::new(
                owner_key.clone(),
                ContentKey::new(content_key).expect("content key"),
            ),
            ContentIdentityStability::StableSourceIdentity,
            owner,
            ContentRole::EmbeddedCapability,
            ContentOrigin::RecordField {
                source_kind: ContentSourceKind::Description,
                relative_source_path: format!("content.{content_key}"),
            },
            ContentVisibility::Public,
            ContentProvenance {
                source_record_key: owner_key.clone(),
                relative_source_path: format!("content.{content_key}"),
                field_or_pointer_family: "content".to_string(),
                nested_source_id: None,
                authored_ordinal_or_range: Some(order.to_string()),
                authored_label: None,
            },
            ContentSourceKind::Description,
            order,
            Some(content_key.to_string()),
            RichDocument::new(vec![RichNode::Text {
                text: content_key.to_string(),
            }]),
            DuplicateContentStatus::Unique,
            Vec::new(),
        )
    }

    fn fixture_record() -> RetrievedRecord {
        RetrievedRecord {
            record: fixture_base_record(),
            body: None,
        }
    }

    fn fixture_base_record() -> AtlasRecord {
        AtlasRecord {
            identity: RecordIdentity {
                key: RecordKey::new(
                    PackName::new("actions".to_string()).expect("pack"),
                    RecordId::new("treat-wounds".to_string()).expect("id"),
                ),
                name: "Treat Wounds".to_string(),
            },
            classification: RecordClassification {
                kind: RecordKind::Rule,
                level: None,
                rarity: None,
                traits: vec!["healing".to_string()],
                taxonomy: RecordTaxonomy::default(),
            },
            foundry: FoundryRecordInfo {
                pack_label: "Actions".to_string(),
                document_type: FoundryDocumentType::Item,
                record_type: FoundryRecordType::Action,
                folder_id: None,
            },
            provenance: RecordProvenance {
                source_path: "packs/actions/treat-wounds.json".to_string(),
                raw_json: Some("{\"name\":\"Treat Wounds\"}".to_string()),
            },
            publication: RecordPublication {
                title: Some("Player Core".to_string()),
                remaster: true,
                category: PublicationCategory::Core,
            },
            requirements: RecordRequirements::default(),
            timing: RecordTiming::default(),
            mechanics: RecordMechanics::default(),
            content: RecordContent {
                documents: vec![RecordContentDocument {
                    source_kind: ContentSourceKind::Description,
                    label: None,
                    document: RichDocument::new(vec![RichNode::HtmlElement {
                        tag: "p".to_string(),
                        attributes: BTreeMap::new(),
                        children: vec![RichNode::Text {
                            text: "You spend 10 minutes treating one injured creature.".to_string(),
                        }],
                    }]),
                }],
            },
            variant: None,
            visibility: RecordVisibility::default(),
        }
    }

    fn edition_creature_record(record_key: &str, title: &str, remaster: bool) -> RetrievedRecord {
        let mut retrieved = fixture_creature_record();
        let record_key = RecordKey::parse(record_key).expect("edition fixture key");
        retrieved.record.identity.key = record_key.clone();
        retrieved.record.identity.name = title.to_string();
        retrieved.record.publication.remaster = remaster;
        if let Some(RecordBody::Creature(creature)) = &mut retrieved.body {
            creature.identity.record_key = record_key;
            creature.identity.name = title.to_string();
            creature.publication.value = FactValue::Value(crate::CreaturePublication {
                title: FactValue::Value("Fixture Publication".to_string()),
                remaster: FactValue::Value(remaster),
                license: FactValue::Missing,
            });
        }
        retrieved
    }

    fn fixture_creature_record() -> RetrievedRecord {
        let mut record = fixture_base_record();
        record.identity.key = RecordKey::new(
            PackName::new("creatures".to_string()).expect("pack"),
            RecordId::new("test-guardian".to_string()).expect("id"),
        );
        record.identity.name = "Test Guardian".to_string();
        record.classification.kind = RecordKind::Creature;
        record.classification.level = Some(5);
        record.foundry.document_type = FoundryDocumentType::Actor;
        record.foundry.record_type = FoundryRecordType::Npc;
        let owner = record.identity.key.clone();
        let entry_id = crate::CreatureOccurrenceId::new("occult-innate").expect("entry id");
        let occurrence = |id: &str,
                          order: u32,
                          family: crate::CreatureEntityFamily,
                          label: &str,
                          parent: crate::CreatureOccurrenceParent,
                          context: crate::CreatureOccurrenceContext,
                          capability: crate::CreatureCapability| {
            crate::CreatureEntityOccurrence {
                id: crate::CreatureOccurrenceId::new(id).expect("occurrence id"),
                identity_stability: crate::OccurrenceIdentityStability::StableNestedSourceId,
                owner: owner.clone(),
                target: crate::CreatureEntityTarget::ActorOwned(
                    crate::CreatureEntityId::new(id).expect("entity id"),
                ),
                family,
                authored_order: order,
                source_sort: FactValue::Value(order.into()),
                source_folder: FactValue::Missing,
                source_identity: crate::CreatureEntitySourceIdentity {
                    nested_source_id: FactValue::Missing,
                    stable_source_locator: FactValue::Missing,
                    source_locators: Vec::new(),
                },
                parent,
                context: crate::CreatureOccurrenceContext {
                    contextual_label: FactValue::Value(label.to_string()),
                    ..context
                },
                capability,
                deltas: Vec::new(),
            }
        };
        let occurrences = vec![
            occurrence(
                "occult-innate",
                0,
                crate::CreatureEntityFamily::SpellcastingEntry,
                "Occult Innate Spells",
                crate::CreatureOccurrenceParent::Creature,
                crate::CreatureOccurrenceContext::default(),
                crate::CreatureCapability::SpellcastingEntry(
                    crate::CreatureSpellcastingEntryCapability {
                        preparation: FactValue::Value(crate::CreatureSpellPreparation::Innate),
                        tradition: FactValue::Value("occult".to_string()),
                        attack: FactValue::Value(16),
                        dc: FactValue::Value(26),
                        slots: FactValue::Value(vec![crate::CreatureSpellSlot {
                            rank: 1,
                            maximum: FactValue::Value(crate::CreatureSourceScalar::Value(1)),
                            serialized_value: FactValue::Value(crate::CreatureSourceScalar::Value(
                                1,
                            )),
                            prepared: FactValue::Value(vec![
                                crate::CreaturePreparedSpellSlot::Spell {
                                    id: FactValue::Value(
                                        crate::CreatureSourceId::new("prepared-magic-missile")
                                            .expect("prepared spell id"),
                                    ),
                                    name: FactValue::Value("Magic Missile".to_string()),
                                    expended: FactValue::Value(false),
                                    prepared: FactValue::Value(true),
                                    authored_order: 0,
                                },
                            ]),
                        }]),
                        unsupported_notes: Vec::new(),
                    },
                ),
            ),
            occurrence(
                "jaws",
                1,
                crate::CreatureEntityFamily::Strike,
                "Jaws",
                crate::CreatureOccurrenceParent::Creature,
                crate::CreatureOccurrenceContext::default(),
                crate::CreatureCapability::Strike(crate::CreatureStrikeCapability {
                    traits: FactValue::Value(vec!["magical".to_string()]),
                    attack_effects: FactValue::Value(Vec::new()),
                    rolls: vec![crate::CreatureRoll {
                        id: "attack".to_string(),
                        label: "Attack".to_string(),
                        kind: crate::CreatureRollKind::Attack,
                        value: FactValue::Value(17),
                        ability: FactValue::Missing,
                    }],
                    damage: FactValue::Value(vec![crate::CreatureDamage {
                        id: "piercing".to_string(),
                        formula: FactValue::Value("2d8+8".to_string()),
                        damage_type: FactValue::Value("piercing".to_string()),
                        category: FactValue::Missing,
                        kinds: FactValue::Value(vec![crate::CreatureDamageKind::Damage]),
                        apply_modifier: FactValue::Missing,
                    }]),
                    action_cost: crate::CreatureActionCost::Actions(1),
                    unsupported_notes: Vec::new(),
                }),
            ),
            occurrence(
                "magic-missile",
                2,
                crate::CreatureEntityFamily::Spell,
                "Magic Missile",
                crate::CreatureOccurrenceParent::SpellcastingEntry(entry_id),
                crate::CreatureOccurrenceContext {
                    rank: FactValue::Value(1),
                    slot: FactValue::Value("slot1:0".to_string()),
                    uses: FactValue::Value(crate::CreatureUseLimit {
                        maximum: FactValue::Value(1),
                        serialized_value: FactValue::Value(1),
                    }),
                    ..crate::CreatureOccurrenceContext::default()
                },
                crate::CreatureCapability::Spell(crate::CreatureSpellCapability {
                    traits: FactValue::Value(vec!["force".to_string()]),
                    base_rank: FactValue::Value(1),
                    signature: FactValue::Value(false),
                    traditions: FactValue::Value(vec!["occult".to_string()]),
                    requirements: FactValue::Missing,
                    cost: FactValue::Missing,
                    counteraction: FactValue::Missing,
                    ritual: FactValue::Missing,
                    target: FactValue::Value("one creature".to_string()),
                    area: FactValue::Missing,
                    range: FactValue::Value("120 feet".to_string()),
                    time: FactValue::Missing,
                    duration: FactValue::Missing,
                    defense: FactValue::Missing,
                    damage: FactValue::Value(Vec::new()),
                    action_cost: crate::CreatureActionCost::Actions(1),
                    unsupported_notes: Vec::new(),
                }),
            ),
            occurrence(
                "change-shape",
                3,
                crate::CreatureEntityFamily::Action,
                "Change Shape",
                crate::CreatureOccurrenceParent::Creature,
                crate::CreatureOccurrenceContext::default(),
                crate::CreatureCapability::Action(crate::CreatureActionCapability {
                    category: FactValue::Value("offensive".to_string()),
                    traits: FactValue::Value(vec!["polymorph".to_string()]),
                    action_cost: crate::CreatureActionCost::Actions(1),
                    frequency: FactValue::Value(crate::CreatureFrequency {
                        maximum: FactValue::Value(1),
                        period: FactValue::Value("PT1M".to_string()),
                        serialized_value: FactValue::Value(1),
                    }),
                    self_effect: FactValue::Missing,
                    self_effect_label: FactValue::Missing,
                    requirements: FactValue::Missing,
                    cost: FactValue::Missing,
                    rolls: Vec::new(),
                    damage: FactValue::Value(Vec::new()),
                    unsupported_notes: Vec::new(),
                }),
            ),
        ];
        let entities = ["occult-innate", "jaws", "magic-missile", "change-shape"]
            .into_iter()
            .map(|id| crate::CreatureEntity {
                id: crate::CreatureEntityId::new(id).expect("entity id"),
                family: match id {
                    "occult-innate" => crate::CreatureEntityFamily::SpellcastingEntry,
                    "jaws" => crate::CreatureEntityFamily::Strike,
                    "magic-missile" => crate::CreatureEntityFamily::Spell,
                    _ => crate::CreatureEntityFamily::Action,
                },
                label: id.to_string(),
                source_identity: crate::CreatureEntitySourceIdentity {
                    nested_source_id: FactValue::Missing,
                    stable_source_locator: FactValue::Missing,
                    source_locators: Vec::new(),
                },
            })
            .collect();
        let body = crate::CreatureRecord {
            identity: crate::CreatureIdentity {
                record_key: owner,
                source_id: crate::CreatureSourceId::new("test-guardian").expect("source id"),
                name: "Test Guardian".to_string(),
                family: crate::CreatureFamily::Npc,
            },
            level: missing(crate::CreatureSourceField::Level),
            rarity: missing(crate::CreatureSourceField::Rarity),
            traits: missing(crate::CreatureSourceField::Traits),
            size: missing(crate::CreatureSourceField::Size),
            publication: missing(crate::CreatureSourceField::Publication),
            adjustment: missing(crate::CreatureSourceField::Adjustment),
            source_alliance: missing(crate::CreatureSourceField::SourceAlliance),
            perception: crate::CreatureFact::source(
                FactValue::Value(crate::CreaturePerception {
                    modifier: FactValue::Value(12),
                    details: FactValue::Missing,
                    has_vision: FactValue::Value(true),
                    senses: FactValue::Value(Vec::new()),
                }),
                crate::CreatureSourceField::Perception,
            ),
            initiative: missing(crate::CreatureSourceField::Initiative),
            languages: crate::CreatureFact::source(
                FactValue::Value(crate::CreatureLanguages {
                    values: FactValue::Value(vec![crate::Language::new("common").expect("lang")]),
                    details: FactValue::Missing,
                }),
                crate::CreatureSourceField::Languages,
            ),
            skills: crate::CreatureFact::source(
                FactValue::Value(vec![crate::CreatureSkill {
                    id: crate::CreatureComponentId::new("theater-lore").expect("skill id"),
                    authored_order: 0,
                    source_entries: Vec::new(),
                    kind: crate::CreatureSkillKind::Lore,
                    label: "Theater Lore".to_string(),
                    modifier: FactValue::Value(15),
                    note: FactValue::Value(crate::CreatureNote::new("stage performances")),
                    variants: FactValue::Value(Vec::new()),
                    source_item_id: FactValue::Missing,
                    unmodeled: FactValue::Missing,
                }]),
                crate::CreatureSourceField::Skills,
            ),
            legacy_abilities: missing(crate::CreatureSourceField::LegacyAbilities),
            defenses: crate::CreatureFact::source(
                FactValue::Value(fixture_defenses()),
                crate::CreatureSourceField::Defenses,
            ),
            movement: crate::CreatureFact::source(
                FactValue::Value(vec![crate::CreatureSpeed {
                    id: crate::CreatureComponentId::new("land").expect("speed id"),
                    authored_order: 0,
                    mode: crate::CreatureMovementMode::Land,
                    value: FactValue::Value(25),
                    label: FactValue::Value("Land Speed".to_string()),
                    details: FactValue::Missing,
                }]),
                crate::CreatureSourceField::Movement,
            ),
            resources: crate::CreatureFact::source(
                FactValue::Value(vec![crate::CreatureResource {
                    id: crate::CreatureComponentId::new("focus").expect("resource id"),
                    authored_order: 0,
                    kind: crate::CreatureResourceKind::new("focus").expect("resource kind"),
                    label: "Focus Points".to_string(),
                    maximum: FactValue::Value(crate::CreatureResourceAmount::Integer(1)),
                    serialized_value: FactValue::Value(crate::CreatureResourceAmount::Integer(1)),
                    source_drift: FactValue::Value(Vec::new()),
                    current_policy: crate::ResourceCurrentPolicy::SerializedValueIsProvenanceOnly,
                }]),
                crate::CreatureSourceField::Resources,
            ),
            embedded_entities: crate::CreatureFact::source(
                FactValue::Value(crate::CreatureEmbeddedEntities {
                    entities,
                    occurrences,
                    relationships: Vec::new(),
                    actor_spellcasting: FactValue::Missing,
                }),
                crate::CreatureSourceField::EmbeddedEntities,
            ),
            content: crate::OwnedRichContent::default(),
            provenance: crate::CreatureProvenance {
                source_path: "packs/creatures/test-guardian.json".to_string(),
                source_contract_version: "test".to_string(),
                source_system_version: "test".to_string(),
                source_upstream_commit: "test".to_string(),
            },
        };
        RetrievedRecord {
            record,
            body: Some(RecordBody::Creature(body)),
        }
    }

    fn fixture_defenses() -> crate::CreatureDefenses {
        let save = |id: &str, kind, value| crate::CreatureSave {
            id: crate::CreatureComponentId::new(id).expect("save id"),
            kind,
            value: FactValue::Value(value),
            details: FactValue::Missing,
        };
        let iwr = |id: &str, kind, iwr_type: &str, value| crate::CreatureIwr {
            id: crate::CreatureComponentId::new(id).expect("iwr id"),
            authored_order: 0,
            kind,
            iwr_type: crate::IwrType::new(iwr_type).expect("iwr type"),
            value: FactValue::Value(value),
            exceptions: FactValue::Value(Vec::new()),
            double_vs: FactValue::Value(Vec::new()),
            apply_once: FactValue::Missing,
        };
        crate::CreatureDefenses {
            armor_class: FactValue::Value(crate::CreatureArmorClass {
                value: FactValue::Value(25),
                details: FactValue::Missing,
            }),
            hit_points: FactValue::Value(crate::CreatureHitPoints {
                value: FactValue::Value(crate::CreatureNumber::Integer(80)),
                maximum: FactValue::Value(80),
                temporary: FactValue::Missing,
                temporary_maximum: FactValue::Missing,
                details: FactValue::Missing,
            }),
            hardness: FactValue::Missing,
            shield: FactValue::Missing,
            saves: FactValue::Value(crate::CreatureSaves {
                fortitude: FactValue::Value(save(
                    "fortitude",
                    crate::CreatureSaveKind::Fortitude,
                    14,
                )),
                reflex: FactValue::Value(save("reflex", crate::CreatureSaveKind::Reflex, 11)),
                will: FactValue::Value(save("will", crate::CreatureSaveKind::Will, 12)),
            }),
            all_saves_note: FactValue::Missing,
            immunities: FactValue::Value(Vec::new()),
            resistances: FactValue::Value(vec![iwr(
                "mental",
                crate::CreatureIwrKind::Resistance,
                "mental",
                10,
            )]),
            weaknesses: FactValue::Value(vec![iwr(
                "cold-iron",
                crate::CreatureIwrKind::Weakness,
                "cold-iron",
                10,
            )]),
        }
    }

    fn missing<T>(field: crate::CreatureSourceField) -> crate::CreatureFact<T> {
        crate::CreatureFact::source(FactValue::Missing, field)
    }
}
