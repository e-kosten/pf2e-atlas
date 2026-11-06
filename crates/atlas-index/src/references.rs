//! Canonical vocabulary content and exact source evidence, without record copies.
use crate::reference_evidence::prepared_record_references;
use crate::{IndexError, SqliteIndexReader};
use atlas_domain::{DefinitionKey, DefinitionNamespace, OwnedContentLocator, RecordKey};
use atlas_record::source_content::PreparedReferenceDefinition;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceCoverage {
    TraitsOnly,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferencePreparation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_package: Option<String>,
    /// Only explicitly declared pack scopes; absent keys inherit the root package.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub pack_systems: std::collections::BTreeMap<String, Option<String>>,
    pub recipe: String,
    pub input_hash: String,
    pub coverage: ReferenceCoverage,
    pub expected_entry_count: usize,
    pub entry_keys_sha256: String,
}

/// Check captured package/pack applicability without consulting source files.
/// Explicit unsupported declarations never inherit the root package.
pub fn reference_pack_applies(
    preparation: Option<&ReferencePreparation>,
    expected_package: &str,
    pack: &str,
) -> bool {
    let Some(preparation) = preparation else {
        return false;
    };
    !expected_package.is_empty()
        && preparation.source_package.as_deref() == Some(expected_package)
        && preparation
            .pack_systems
            .get(pack)
            .is_none_or(|declared| declared.as_deref() == Some(expected_package))
}

pub(crate) fn validate_template_scope(
    record: &atlas_record::source_record::SourceBackedRecord,
    prepared: &atlas_record::source_content::PreparedSourceContent,
    preparation: Option<&ReferencePreparation>,
) -> Result<(), IndexError> {
    let package = preparation
        .and_then(|p| p.source_package.as_deref())
        .unwrap_or("");
    let applicable = reference_pack_applies(preparation, "pf2e", record.key().pack().as_str());
    atlas_record::source_record::validate_general_definition_reading(
        record, package, applicable, prepared,
    )
    .map_err(IndexError::Invalid)
}

/// Compact inventory of selected canonical keys, independent of definition bodies.
pub fn reference_inventory_identity(
    keys: impl IntoIterator<Item = DefinitionKey>,
) -> Result<(usize, String), IndexError> {
    inventory_identity_strings(keys.into_iter().map(|key| key.to_string()))
}

fn inventory_identity_strings(
    keys: impl IntoIterator<Item = String>,
) -> Result<(usize, String), IndexError> {
    let mut selected = std::collections::BTreeSet::new();
    for key in keys {
        if !selected.insert(key) {
            return Err(IndexError::Invalid(
                "duplicate selected reference key".into(),
            ));
        }
    }
    Ok((
        selected.len(),
        crate::codec::sha256(&serde_json::to_vec(&selected)?),
    ))
}

pub(crate) fn validate_inventory(
    preparation: Option<&ReferencePreparation>,
    keys: impl IntoIterator<Item = String>,
) -> Result<(), IndexError> {
    if let Some(preparation) = preparation {
        let (count, hash) = inventory_identity_strings(keys)?;
        if count != preparation.expected_entry_count || hash != preparation.entry_keys_sha256 {
            return Err(IndexError::Invalid(
                "reference inventory differs from captured preparation".into(),
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceSourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferencePublication {
    pub title: Option<String>,
    pub page: Option<u32>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceRecordMapping {
    pub target: RecordKey,
    pub package: String,
    pub expected_document_kind: String,
    pub expected_item_type: Option<String>,
    pub required_localization_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceProvenance {
    pub source_package: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_repository: Option<String>,
    pub source_revision: Option<String>,
    pub source_file: Option<String>,
    pub source_sha256: Option<String>,
    pub label_key: Option<String>,
    pub description_key: Option<String>,
    pub catalogs: Vec<String>,
    pub source_locations: Vec<ReferenceSourceLocation>,
    pub indexing_locale: String,
    /// Unknown in migrated data that did not retain per-entry fallback evidence.
    pub content_locale: Option<String>,
    pub publications: Vec<ReferencePublication>,
    pub record_mapping: Option<ReferenceRecordMapping>,
    pub recipe: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferencePermissionDisposition {
    ExistingTraitPolicy,
    ExistingRecordPolicy,
    ApprovedContent,
    MetadataOnlyUnresolved,
    ExcludedOrWithdrawn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferencePermission {
    pub disposition: ReferencePermissionDisposition,
    pub body_grants: Vec<ReferenceGrantUse>,
    pub label_grants: Vec<ReferenceGrantUse>,
    pub notices: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceGrantUse {
    pub license: String,
    pub scope: ReferenceGrantScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceGrantScope {
    FunctionalContent,
    DescriptiveReferences,
}

/// Discovery metadata. Query construction and execution remain query-owned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceFilterBinding {
    pub id: String,
    pub relation: String,
    pub field: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReferenceBacking {
    Prepared {
        authored_markup: String,
        content: PreparedReferenceDefinition,
    },
    Record {
        key: RecordKey,
    },
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceEntry {
    pub key: DefinitionKey,
    pub label: String,
    pub availability: ReferenceAvailability,
    pub backing: ReferenceBacking,
    pub provenance: ReferenceProvenance,
    pub permission: ReferencePermission,
    pub filter_bindings: Vec<ReferenceFilterBinding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceAvailability {
    Available,
    LabelOnly,
    MissingLocalizedBody,
    MissingTarget,
    WrongFamily,
    ConflictingEvidence,
    AmbiguousMapping,
    SourceInapplicable,
    PermissionUnresolved,
    Withdrawn,
    UnavailableUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceNotice {
    pub id: String,
    pub sha256: String,
    pub text: String,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DefinitionReferenceSource {
    Record {
        key: RecordKey,
        owners: Vec<OwnedContentLocator>,
        field: Option<String>,
    },
    Definition {
        key: DefinitionKey,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DefinitionReferenceTarget {
    Record {
        key: RecordKey,
        owners: Vec<OwnedContentLocator>,
    },
    Definition {
        key: DefinitionKey,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinitionReferenceInput {
    pub source: DefinitionReferenceSource,
    pub target: DefinitionReferenceTarget,
    pub occurrence_path: String,
    pub evidence: String,
    pub label: Option<String>,
    pub visible: bool,
}

impl ReferenceEntry {
    pub(crate) fn from_trait(definition: &crate::TraitDefinition) -> Result<Self, IndexError> {
        let key = DefinitionKey::from(definition.key.clone());
        let backing = match (
            &definition.authored_markup,
            &definition.prepared,
            &definition.description_html,
        ) {
            (Some(markup), Some(content), Some(html)) if content.html == *html => {
                ReferenceBacking::Prepared {
                    authored_markup: markup.clone(),
                    content: content.clone(),
                }
            }
            (None, None, None) => ReferenceBacking::None,
            _ => {
                return Err(IndexError::Invalid(
                    "incomplete trait source/preparation snapshot".into(),
                ));
            }
        };
        Ok(Self {
            key,
            label: definition.label.clone(),
            availability: if matches!(&backing, ReferenceBacking::None) {
                if definition.description_key.is_some() {
                    ReferenceAvailability::MissingLocalizedBody
                } else {
                    ReferenceAvailability::LabelOnly
                }
            } else {
                ReferenceAvailability::Available
            },
            backing,
            provenance: ReferenceProvenance {
                source_package: "pf2e".into(),
                source_repository: None,
                source_revision: definition.source_revision.clone(),
                source_file: None,
                source_sha256: None,
                label_key: definition.label_key.clone(),
                description_key: definition.description_key.clone(),
                catalogs: definition.catalogs.clone(),
                source_locations: definition.source_locations.clone(),
                indexing_locale: definition.indexing_locale.clone(),
                content_locale: definition.content_locale.clone(),
                publications: vec![],
                record_mapping: None,
                recipe: "existing-trait-definition/v1".into(),
            },
            permission: ReferencePermission {
                disposition: ReferencePermissionDisposition::ExistingTraitPolicy,
                body_grants: vec![],
                label_grants: vec![],
                notices: vec![],
            },
            filter_bindings: vec![
                ReferenceFilterBinding {
                    id: "membership".into(),
                    relation: "trait_membership".into(),
                    field: Some("traits".into()),
                    value: Some(definition.key.identifier().into()),
                },
                ReferenceFilterBinding {
                    id: "references".into(),
                    relation: "definition_reference".into(),
                    field: None,
                    value: Some(definition.key.to_string()),
                },
            ],
        })
    }

    pub(crate) fn trait_view(&self) -> Result<crate::TraitDefinition, IndexError> {
        let key = self
            .key
            .trait_key()
            .ok_or_else(|| IndexError::Invalid("reference is not a trait".into()))?;
        let (authored_markup, prepared, description_html) = match &self.backing {
            ReferenceBacking::None => (None, None, None),
            ReferenceBacking::Prepared {
                authored_markup,
                content,
            } => (
                Some(authored_markup.clone()),
                Some(content.clone()),
                Some(content.html.clone()),
            ),
            ReferenceBacking::Record { .. } => {
                return Err(IndexError::Invalid(
                    "trait cannot replace its body with a record pointer".into(),
                ));
            }
        };
        Ok(crate::TraitDefinition {
            key,
            label: self.label.clone(),
            label_key: self.provenance.label_key.clone(),
            description_key: self.provenance.description_key.clone(),
            description_html,
            authored_markup,
            prepared,
            catalogs: self.provenance.catalogs.clone(),
            source_locations: self.provenance.source_locations.clone(),
            source_revision: self.provenance.source_revision.clone(),
            indexing_locale: self.provenance.indexing_locale.clone(),
            content_locale: self.provenance.content_locale.clone(),
        })
    }
}

impl From<crate::TraitReferenceInput> for DefinitionReferenceInput {
    fn from(reference: crate::TraitReferenceInput) -> Self {
        let key = DefinitionKey::from(reference.key);
        let (source, target) = if reference.direction == "outgoing" {
            (
                DefinitionReferenceSource::Definition { key },
                DefinitionReferenceTarget::Record {
                    key: reference.record,
                    owners: reference.owners,
                },
            )
        } else {
            (
                DefinitionReferenceSource::Record {
                    key: reference.record,
                    owners: reference.owners,
                    field: reference.field,
                },
                DefinitionReferenceTarget::Definition { key },
            )
        };
        Self {
            source,
            target,
            occurrence_path: reference.occurrence_path,
            evidence: reference.evidence,
            label: reference.source_label,
            visible: true,
        }
    }
}

impl DefinitionReferenceInput {
    pub(crate) fn trait_view(
        &self,
        key: &atlas_domain::TraitKey,
    ) -> Option<crate::TraitReferenceInput> {
        if !self.visible {
            return None;
        }
        let definition_key = DefinitionKey::from(key.clone());
        let (record, owners, field, direction) = match (&self.source, &self.target) {
            (
                DefinitionReferenceSource::Record { key, owners, field },
                DefinitionReferenceTarget::Definition { key: target },
            ) if target == &definition_key => {
                (key.clone(), owners.clone(), field.clone(), "incoming")
            }
            (
                DefinitionReferenceSource::Definition { key: source },
                DefinitionReferenceTarget::Record { key, owners },
            ) if source == &definition_key => (key.clone(), owners.clone(), None, "outgoing"),
            _ => return None,
        };
        Some(crate::TraitReferenceInput {
            source_label: self.label.clone(),
            owners,
            field,
            direction: direction.into(),
            occurrence_path: self.occurrence_path.clone(),
            key: key.clone(),
            record,
            evidence: self.evidence.clone(),
        })
    }
}

impl SqliteIndexReader {
    pub fn reference_coverage(&self) -> ReferenceCoverage {
        self.context()
            .reference_preparation
            .as_ref()
            .map_or(ReferenceCoverage::TraitsOnly, |p| p.coverage)
    }

    pub fn reference_preparation(&self) -> Option<&ReferencePreparation> {
        self.context().reference_preparation.as_ref()
    }

    pub fn reference_entry(
        &self,
        key: &DefinitionKey,
    ) -> Result<Option<ReferenceEntry>, IndexError> {
        if key.namespace() != DefinitionNamespace::Trait
            && self.reference_coverage() != ReferenceCoverage::Complete
        {
            return Err(IndexError::Unavailable(
                "glossary coverage requires explicit atlas setup".into(),
            ));
        }
        read_entry(self.native()?, key)
    }

    pub fn reference_entries(
        &self,
        namespace: DefinitionNamespace,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<ReferenceEntry>, u64), IndexError> {
        if !(1..=100).contains(&limit) {
            return Err(IndexError::Invalid(
                "reference page size must be 1..100".into(),
            ));
        }
        if namespace != DefinitionNamespace::Trait
            && self.reference_coverage() != ReferenceCoverage::Complete
        {
            return Err(IndexError::Unavailable(
                "glossary coverage requires explicit atlas setup".into(),
            ));
        }
        let prefix = format!("{}:", namespace.as_str());
        let total = self.native()?.query_row(
            "SELECT count(*) FROM reference_entries WHERE substr(key,1,length(?))=?",
            params![prefix, prefix],
            |r| r.get(0),
        )?;
        let keys = self.native()?.prepare("SELECT key FROM reference_entries WHERE substr(key,1,length(?1))=?1 ORDER BY key LIMIT ?2 OFFSET ?3")?.query_map(params![prefix,limit,offset], |r| r.get::<_,String>(0))?.collect::<Result<Vec<_>,_>>()?;
        let entries = keys
            .into_iter()
            .map(|k| {
                let key = k.parse().map_err(|e| {
                    IndexError::Invalid(format!("invalid stored definition key: {e}"))
                })?;
                read_entry(self.native()?, &key)?.ok_or_else(|| {
                    IndexError::Invalid("missing reference entry in selected snapshot".into())
                })
            })
            .collect::<Result<_, _>>()?;
        Ok((entries, total))
    }

    pub fn reference_notice(&self, id: &str) -> Result<Option<ReferenceNotice>, IndexError> {
        let notice = self
            .native()?
            .query_row(
                "SELECT id,sha256,text,source_url FROM reference_notices WHERE id=?",
                [id],
                |r| {
                    Ok(ReferenceNotice {
                        id: r.get(0)?,
                        sha256: r.get(1)?,
                        text: r.get(2)?,
                        source_url: r.get(3)?,
                    })
                },
            )
            .optional()?;
        if let Some(notice) = &notice {
            validate_notice(notice)?;
        }
        Ok(notice)
    }

    /// Exact indexed occurrence evidence. Record graph edges retain their own
    /// existing owner and are not copied into this page.
    pub fn definition_reference_occurrences(
        &self,
        target: &DefinitionReferenceTarget,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<DefinitionReferenceInput>, u64), IndexError> {
        if !(1..=100).contains(&limit) {
            return Err(IndexError::Invalid(
                "definition occurrence page size must be 1..100".into(),
            ));
        }
        if self.reference_coverage() != ReferenceCoverage::Complete {
            return Err(IndexError::Unavailable(
                "glossary occurrence coverage requires explicit atlas setup".into(),
            ));
        }
        let (predicate, value, owners) = match target {
            DefinitionReferenceTarget::Record { key, owners } => (
                "target_record_key=?1 AND json_extract(evidence,'$.target.owners')=?2",
                key.to_string(),
                serde_json::to_string(owners)?,
            ),
            DefinitionReferenceTarget::Definition { key } => {
                if self.reference_entry(key)?.is_none() {
                    return Err(IndexError::Unavailable(
                        "definition occurrence target is unavailable".into(),
                    ));
                }
                (
                    "target_definition_key=?1 AND ?2=''",
                    key.to_string(),
                    String::new(),
                )
            }
        };
        let total=self.native()?.query_row(&format!("SELECT count(*) FROM definition_reference_occurrences WHERE visible=1 AND {predicate}"),params![value,owners],|r|r.get(0))?;
        let mut statement=self.native()?.prepare(&format!("SELECT occurrence_id,evidence FROM definition_reference_occurrences WHERE visible=1 AND {predicate} ORDER BY occurrence_id LIMIT ?3 OFFSET ?4"))?;
        let mut result = Vec::new();
        for row in statement.query_map(params![value, owners, limit, offset], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })? {
            let (id, json) = row?;
            let occurrence: DefinitionReferenceInput = serde_json::from_str(&json)?;
            if reference_id(&occurrence)? != id
                || occurrence.target != *target
                || !occurrence.visible
            {
                return Err(IndexError::Invalid(
                    "definition occurrence identity/target mismatch".into(),
                ));
            }
            result.push(occurrence);
        }
        Ok((result, total))
    }
}

pub(crate) fn read_entry(
    c: &Connection,
    key: &DefinitionKey,
) -> Result<Option<ReferenceEntry>, IndexError> {
    let row: Option<(String, String)> = c
        .query_row(
            "SELECT definition_json,definition_hash FROM reference_entries WHERE key=?",
            [key.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    row.map(|(json, hash)| {
        if crate::codec::sha256(json.as_bytes()) != hash {
            return Err(IndexError::Invalid(
                "reference definition checksum mismatch".into(),
            ));
        }
        let entry: ReferenceEntry = serde_json::from_str(&json)?;
        if entry.key != *key {
            return Err(IndexError::Invalid(
                "reference definition identity mismatch".into(),
            ));
        }
        validate_entry_metadata(c, &entry, None, false)?;
        Ok(entry)
    })
    .transpose()
}

pub(crate) fn validate_notice(notice: &ReferenceNotice) -> Result<(), IndexError> {
    if notice.id.trim().is_empty()
        || notice.text.is_empty()
        || crate::codec::sha256(notice.text.as_bytes()) != notice.sha256
    {
        return Err(IndexError::Invalid(
            "reference notice identity/checksum mismatch".into(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_entry(
    c: &Connection,
    entry: &ReferenceEntry,
    preparation: Option<&ReferencePreparation>,
) -> Result<(), IndexError> {
    validate_entry_metadata(c, entry, preparation, true)
}

fn validate_entry_metadata(
    c: &Connection,
    entry: &ReferenceEntry,
    preparation: Option<&ReferencePreparation>,
    inspect_retained_source: bool,
) -> Result<(), IndexError> {
    if entry.label.trim().is_empty()
        || entry.provenance.indexing_locale.is_empty()
        || entry.provenance.source_package.is_empty()
        || entry.provenance.recipe.is_empty()
        || entry.filter_bindings.is_empty()
    {
        return Err(IndexError::Invalid("incomplete reference metadata".into()));
    }
    if (entry.availability == ReferenceAvailability::Available)
        == matches!(&entry.backing, ReferenceBacking::None)
    {
        return Err(IndexError::Invalid(
            "reference availability differs from its backing".into(),
        ));
    }
    if entry
        .provenance
        .source_sha256
        .as_ref()
        .is_some_and(|hash| !crate::codec::checked_hash(hash))
    {
        return Err(IndexError::Invalid(
            "invalid reference source checksum".into(),
        ));
    }
    let mut bindings = std::collections::BTreeSet::new();
    for binding in &entry.filter_bindings {
        if binding.id.is_empty() || binding.relation.is_empty() || !bindings.insert(&binding.id) {
            return Err(IndexError::Invalid(
                "invalid or duplicate reference filter binding".into(),
            ));
        }
    }
    for grants in [
        &entry.permission.body_grants,
        &entry.permission.label_grants,
    ] {
        let mut unique = std::collections::BTreeSet::new();
        for grant in grants {
            if grant.license.trim().is_empty() || !unique.insert(serde_json::to_string(grant)?) {
                return Err(IndexError::Invalid(
                    "invalid or duplicate reference grant use".into(),
                ));
            }
        }
    }
    for notice in &entry.permission.notices {
        if !c.query_row(
            "SELECT EXISTS(SELECT 1 FROM reference_notices WHERE id=?)",
            [notice],
            |r| r.get::<_, bool>(0),
        )? {
            return Err(IndexError::Invalid(
                "reference permission notice is absent".into(),
            ));
        }
    }
    match &entry.backing {
        ReferenceBacking::None => {}
        ReferenceBacking::Record { key } => {
            if entry.permission.disposition != ReferencePermissionDisposition::ExistingRecordPolicy
                || !c.query_row(
                    "SELECT EXISTS(SELECT 1 FROM records WHERE key=?)",
                    [key.to_string()],
                    |r| r.get::<_, bool>(0),
                )?
            {
                return Err(IndexError::Invalid(
                    "reference record backing is unavailable or has invalid permission".into(),
                ));
            }
            if inspect_retained_source {
                let mapping = entry.provenance.record_mapping.as_ref().ok_or_else(|| {
                    IndexError::Invalid("record-backed reference lacks mapping evidence".into())
                })?;
                if !reference_pack_applies(preparation, &mapping.package, key.pack().as_str()) {
                    return Err(IndexError::Invalid(
                        "reference mapping captured package/pack is inapplicable".into(),
                    ));
                }
                validate_record_mapping(c, entry, key)?;
            }
        }
        ReferenceBacking::Prepared {
            authored_markup,
            content,
        } => {
            if !matches!(
                entry.permission.disposition,
                ReferencePermissionDisposition::ApprovedContent
                    | ReferencePermissionDisposition::ExistingTraitPolicy
            ) || content.key != entry.key
                || content.authored_markup_sha256
                    != crate::codec::sha256(authored_markup.as_bytes())
                || content.interpretation_version
                    != atlas_record::source_content::CONTENT_INTERPRETATION_VERSION
            {
                return Err(IndexError::Invalid(
                    "reference prepared-content identity/permission mismatch".into(),
                ));
            }
            atlas_record::source_content::validate_prepared_markers(
                &content.html,
                &content.interactions,
            )
            .map_err(|e| IndexError::Invalid(e.to_string()))?;
            atlas_record::source_content::validate_prepared_definition_reference_bindings(
                &content.html,
                &entry.key,
                &content.authored_markup_sha256,
                &content.references,
            )
            .map_err(|e| IndexError::Invalid(e.to_string()))?;
            if entry.permission.disposition == ReferencePermissionDisposition::ApprovedContent
                && (entry.permission.body_grants.is_empty()
                    || entry.permission.notices.is_empty()
                    || entry.provenance.content_locale.is_none()
                    || entry.provenance.source_sha256.is_none())
            {
                return Err(IndexError::Invalid(
                    "approved reference prose lacks source/locale/grant/notice evidence".into(),
                ));
            }
        }
    }
    Ok(())
}

fn validate_record_mapping(
    c: &Connection,
    entry: &ReferenceEntry,
    key: &RecordKey,
) -> Result<(), IndexError> {
    let evidence = entry.provenance.record_mapping.as_ref().ok_or_else(|| {
        IndexError::Invalid("record-backed reference lacks mapping evidence".into())
    })?;
    if evidence.package != entry.provenance.source_package || evidence.target != *key {
        return Err(IndexError::Invalid(
            "reference mapping package/target differs from source".into(),
        ));
    }
    let bytes: Vec<u8> = c.query_row(
        "SELECT b.snapshot FROM record_bodies b JOIN records r USING(record_id) WHERE r.key=?",
        [key.to_string()],
        |r| r.get(0),
    )?;
    let source = atlas_foundry_model::decode_snapshot(&crate::codec::gunzip(&bytes)?)
        .map_err(|e| IndexError::Invalid(e.to_string()))?;
    let record = atlas_record::source_record::SourceBackedRecord::new(key.pack().as_str(), source)
        .map_err(|_| IndexError::Invalid("reference mapping source identity differs".into()))?;
    if record.key() != key {
        return Err(IndexError::Invalid(
            "reference mapping source identity differs".into(),
        ));
    }
    let node = atlas_record::source_record::SourceNodeView::from(record.source());
    if node.document_kind() != evidence.expected_document_kind {
        return Err(IndexError::Invalid(
            "reference mapping document kind differs".into(),
        ));
    }
    if let Some(expected) = &evidence.expected_item_type {
        if !matches!(node,atlas_record::source_record::SourceNodeView::Item(item) if item.family()==expected)
        {
            return Err(IndexError::Invalid(
                "reference mapping item family differs".into(),
            ));
        }
    }
    if let Some(required) = &evidence.required_localization_key {
        let mut map = atlas_record::source_content::DefinitionLocalizationMap::new();
        map.entry(required.clone())
            .or_default()
            .insert(entry.key.clone());
        let audience = atlas_record::source_content::ContentAudience {
            include_gm: true,
            include_owner: true,
            implicit_check_dc: atlas_record::source_content::ContentVisibilityRule::All,
        };
        if !atlas_record::source_record::definition_record_localization_references(
            &record, audience, &map,
        )
        .iter()
        .any(|r| r.locator.owners.is_empty() && r.locator.field == "/system/description/value")
        {
            return Err(IndexError::Invalid(
                "reference mapping required localization evidence differs".into(),
            ));
        }
    }
    Ok(())
}

pub(crate) fn reference_id(reference: &DefinitionReferenceInput) -> Result<String, IndexError> {
    Ok(crate::codec::sha256(&serde_json::to_vec(reference)?))
}

pub(crate) fn insert_reference(
    c: &Connection,
    reference: &DefinitionReferenceInput,
) -> Result<(), IndexError> {
    if reference.occurrence_path.is_empty() || reference.evidence.is_empty() {
        return Err(IndexError::Invalid(
            "invalid definition reference occurrence".into(),
        ));
    }
    let (source_record, source_definition) = match &reference.source {
        DefinitionReferenceSource::Record { key, .. } => (Some(key.to_string()), None),
        DefinitionReferenceSource::Definition { key } => (None, Some(key.to_string())),
    };
    let (target_record, target_definition) = match &reference.target {
        DefinitionReferenceTarget::Record { key, .. } => (Some(key.to_string()), None),
        DefinitionReferenceTarget::Definition { key } => (None, Some(key.to_string())),
    };
    c.execute("INSERT INTO definition_reference_occurrences VALUES(?,?,?,?,?,?,?) ON CONFLICT(occurrence_id) DO NOTHING",params![reference_id(reference)?,source_record,source_definition,target_record,target_definition,reference.visible,serde_json::to_string(reference)?])?;
    Ok(())
}

pub(crate) fn write_reference_entry(
    c: &Connection,
    entry: &ReferenceEntry,
    preparation: Option<&ReferencePreparation>,
) -> Result<(), IndexError> {
    validate_entry(c, entry, preparation)?;
    let json = serde_json::to_string(entry)?;
    c.execute("INSERT INTO reference_entries VALUES(?,?,?) ON CONFLICT(key) DO UPDATE SET definition_json=excluded.definition_json,definition_hash=excluded.definition_hash WHERE definition_hash IS NOT excluded.definition_hash",params![entry.key.to_string(),json,crate::codec::sha256(json.as_bytes())])?;
    Ok(())
}

pub(crate) fn write_notices(
    c: &Connection,
    input: &crate::PublicationSpool,
) -> Result<(), IndexError> {
    let mut kept = std::collections::BTreeSet::new();
    input.reference_notices.visit(&mut |notice| {
        validate_notice(&notice)?;
        if !kept.insert(notice.id.clone()) { return Err(IndexError::Invalid("duplicate reference notice".into())); }
        c.execute("INSERT INTO reference_notices VALUES(?,?,?,?) ON CONFLICT(id) DO UPDATE SET sha256=excluded.sha256,text=excluded.text,source_url=excluded.source_url WHERE sha256 IS NOT excluded.sha256 OR source_url IS NOT excluded.source_url",params![notice.id,notice.sha256,notice.text,notice.source_url])?;
        Ok(())
    })?;
    // New definitions are validated before obsolete notices are removed. The
    // selected complete input still must carry every required notice explicitly.
    let old = c
        .prepare("SELECT id FROM reference_notices")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for id in old {
        if !kept.contains(&id) {
            c.execute("DELETE FROM reference_notices WHERE id=?", [id])?;
        }
    }
    Ok(())
}

pub(crate) fn write_entries(
    c: &Connection,
    input: &crate::PublicationSpool,
    outgoing: &mut Vec<DefinitionReferenceInput>,
) -> Result<std::collections::BTreeSet<String>, IndexError> {
    let mut kept = std::collections::BTreeSet::new();
    let roots = input
        .identities()
        .iter()
        .map(|i| &i.key)
        .collect::<std::collections::BTreeSet<_>>();
    input.reference_entries.visit(&mut |entry| {
        if entry.key.namespace() == DefinitionNamespace::Trait
            || !kept.insert(entry.key.to_string())
        {
            return Err(IndexError::Invalid(
                "duplicate or trait-specialized reference entry".into(),
            ));
        }
        if let ReferenceBacking::Record { key } = &entry.backing {
            if !roots.contains(key) {
                return Err(IndexError::Invalid(
                    "reference mapping target is absent from captured publication".into(),
                ));
            }
        }
        outgoing.extend(prepared_record_references(&entry));
        write_reference_entry(c, &entry, input.context.reference_preparation.as_ref())
    })?;
    let old = c
        .prepare("SELECT key FROM reference_entries WHERE key NOT LIKE 'trait:%'")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for key in old {
        if !kept.contains(&key) {
            c.execute("DELETE FROM reference_entries WHERE key=?", [key])?;
        }
    }
    Ok(kept)
}

pub(crate) fn write_occurrences(
    c: &Connection,
    input: &crate::PublicationSpool,
    outgoing: Vec<DefinitionReferenceInput>,
) -> Result<(), IndexError> {
    let mut desired = std::collections::BTreeMap::new();
    input.definition_references.visit(&mut |reference| {
        validate_reference(c, &reference, input.context.reference_preparation.as_ref())?;
        let id = reference_id(&reference)?;
        if desired.insert(id, reference).is_some() {
            return Err(IndexError::Invalid(
                "duplicate definition reference occurrence".into(),
            ));
        }
        Ok(())
    })?;
    for reference in outgoing {
        validate_reference(c, &reference, input.context.reference_preparation.as_ref())?;
        desired.insert(reference_id(&reference)?, reference);
    }
    let mut old = c
        .prepare("SELECT occurrence_id FROM definition_reference_occurrences")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
    for (id, reference) in desired {
        if !old.remove(&id) {
            insert_reference(c, &reference)?;
        }
    }
    for id in old {
        c.execute(
            "DELETE FROM definition_reference_occurrences WHERE occurrence_id=?",
            [id],
        )?;
    }
    Ok(())
}

fn validate_reference(
    c: &Connection,
    reference: &DefinitionReferenceInput,
    preparation: Option<&ReferencePreparation>,
) -> Result<(), IndexError> {
    if let DefinitionReferenceSource::Record { key, .. } = &reference.source {
        let is_trait = matches!(&reference.target, DefinitionReferenceTarget::Definition { key }
            if key.namespace() == DefinitionNamespace::Trait);
        if !is_trait && !reference_pack_applies(preparation, "pf2e", key.pack().as_str()) {
            return Err(IndexError::Invalid(
                "definition occurrence source package/pack is inapplicable".into(),
            ));
        }
    }
    match &reference.source {
        DefinitionReferenceSource::Definition { key } => {
            if !c.query_row(
                "SELECT EXISTS(SELECT 1 FROM reference_entries WHERE key=?)",
                [key.to_string()],
                |r| r.get::<_, bool>(0),
            )? {
                return Err(IndexError::Invalid(
                    "reference source definition is absent".into(),
                ));
            }
        }
        DefinitionReferenceSource::Record { key, .. } => {
            if !c.query_row(
                "SELECT EXISTS(SELECT 1 FROM records WHERE key=?)",
                [key.to_string()],
                |r| r.get::<_, bool>(0),
            )? {
                return Err(IndexError::Invalid(
                    "reference source record is absent".into(),
                ));
            }
        }
    }
    match &reference.target {
        DefinitionReferenceTarget::Definition { key } => {
            if !c.query_row(
                "SELECT EXISTS(SELECT 1 FROM reference_entries WHERE key=?)",
                [key.to_string()],
                |r| r.get::<_, bool>(0),
            )? {
                return Err(IndexError::Invalid(
                    "reference target definition is absent".into(),
                ));
            }
        }
        DefinitionReferenceTarget::Record { key, .. } => {
            if !c.query_row(
                "SELECT EXISTS(SELECT 1 FROM records WHERE key=?)",
                [key.to_string()],
                |r| r.get::<_, bool>(0),
            )? {
                return Err(IndexError::Invalid(
                    "reference target record is absent".into(),
                ));
            }
        }
    }
    Ok(())
}
