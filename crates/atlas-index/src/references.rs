//! Canonical vocabulary content and exact source evidence, without record copies.
use crate::{IndexError, SqliteIndexReader};
use atlas_domain::{DefinitionKey, DefinitionNamespace, RecordKey};
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
}

/// Metadata-only view of a canonical entry. The prepared body is not decoded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceSummary {
    pub key: DefinitionKey,
    pub label: String,
    pub backing: ReferenceSummaryBacking,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReferenceSummaryBacking {
    Prepared {
        /// Derived from the retained canonical body, not separately persisted.
        #[serde(default)]
        readable: bool,
    },
    Record {
        key: RecordKey,
    },
    None,
}

impl From<&ReferenceEntry> for ReferenceSummary {
    fn from(entry: &ReferenceEntry) -> Self {
        Self {
            key: entry.key.clone(),
            label: entry.label.clone(),
            backing: match &entry.backing {
                ReferenceBacking::Prepared { content, .. } => ReferenceSummaryBacking::Prepared {
                    readable: atlas_record::source_content::has_prepared_reading_content(
                        &content.text,
                        &content.html,
                        content.interactions.len(),
                    ),
                },
                ReferenceBacking::Record { key } => {
                    ReferenceSummaryBacking::Record { key: key.clone() }
                }
                ReferenceBacking::None => ReferenceSummaryBacking::None,
            },
        }
    }
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
    SourceInapplicable,
    PermissionUnresolved,
    UnavailableUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceNotice {
    pub id: String,
    pub sha256: String,
    pub text: String,
    pub source_url: Option<String>,
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

impl SqliteIndexReader {
    pub fn reference_coverage(&self) -> ReferenceCoverage {
        self.context()
            .reference_preparation
            .as_ref()
            .map_or(ReferenceCoverage::TraitsOnly, |p| p.coverage)
    }

    pub fn reference_entry(
        &self,
        key: &DefinitionKey,
    ) -> Result<Option<ReferenceEntry>, IndexError> {
        self.require_reference_key_coverage(key)?;
        read_entry(self.native()?, key)
    }

    /// Reads the checksummed canonical envelope without hydrating its prepared body.
    pub fn reference_summary(
        &self,
        key: &DefinitionKey,
    ) -> Result<Option<ReferenceSummary>, IndexError> {
        Ok(self.reference_summaries(std::slice::from_ref(key))?.pop())
    }

    /// At most 100 keys; returned entries need not be in request order.
    pub fn reference_summaries(
        &self,
        keys: &[DefinitionKey],
    ) -> Result<Vec<ReferenceSummary>, IndexError> {
        if keys.len() > 100 {
            return Err(IndexError::Invalid(
                "reference metadata batch must contain at most 100 keys".into(),
            ));
        }
        for key in keys {
            self.require_reference_key_coverage(key)?;
        }
        if keys.is_empty() {
            return Ok(vec![]);
        }
        let c = self.native()?;
        let mut statement = c.prepare_cached(&format!(
            "SELECT key,definition_json,definition_hash,{} FROM reference_entries
             WHERE key IN (SELECT value FROM json_each(?))",
            prepared_reading_projection_sql(),
        ))?;
        let mut rows = statement.query([serde_json::to_string(keys)?])?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next()? {
            let key: String = row.get(0)?;
            let json: String = row.get(1)?;
            let hash: String = row.get(2)?;
            entries.push(decode_reference_summary(
                c,
                &key,
                &json,
                &hash,
                row.get(3)?,
            )?);
        }
        Ok(entries)
    }

    fn require_reference_key_coverage(&self, key: &DefinitionKey) -> Result<(), IndexError> {
        if key.namespace() != DefinitionNamespace::Trait
            && self.reference_coverage() != ReferenceCoverage::Complete
        {
            return Err(IndexError::Unavailable(
                "glossary coverage requires explicit atlas setup".into(),
            ));
        }
        Ok(())
    }

    pub fn reference_notices(&self, ids: &[String]) -> Result<Vec<ReferenceNotice>, IndexError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let mut statement = self.native()?.prepare_cached(
            "SELECT n.id,n.sha256,n.text,n.source_url
             FROM json_each(?) requested JOIN reference_notices n ON n.id=requested.value
             ORDER BY requested.key",
        )?;
        statement
            .query_map([serde_json::to_string(ids)?], read_notice_row)?
            .map(|row| {
                let notice = row?;
                validate_notice(&notice)?;
                Ok(notice)
            })
            .collect()
    }
}

pub(crate) fn read_entry(
    c: &Connection,
    key: &DefinitionKey,
) -> Result<Option<ReferenceEntry>, IndexError> {
    let key = key.to_string();
    let row: Option<(String, String)> = c
        .query_row(
            "SELECT definition_json,definition_hash FROM reference_entries WHERE key=?",
            [&key],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    row.map(|(json, hash)| {
        let entry = decode_entry(&key, &json, &hash)?;
        if entry.label.trim().is_empty() {
            return Err(IndexError::Invalid("empty reference label".into()));
        }
        validate_backing(c, &entry)?;
        Ok(entry)
    })
    .transpose()
}

fn decode_entry(key: &str, json: &str, hash: &str) -> Result<ReferenceEntry, IndexError> {
    check_envelope_hash(json, hash)?;
    let entry: ReferenceEntry = serde_json::from_str(json)?;
    check_entry_key(&entry.key, key)?;
    Ok(entry)
}

fn check_envelope_hash(json: &str, hash: &str) -> Result<(), IndexError> {
    if crate::codec::sha256(json.as_bytes()) != hash {
        return Err(IndexError::Invalid(
            "reference definition checksum mismatch".into(),
        ));
    }
    Ok(())
}

fn check_entry_key(entry: &DefinitionKey, stored: &str) -> Result<(), IndexError> {
    if entry.to_string() != stored {
        return Err(IndexError::Invalid(
            "reference definition identity mismatch".into(),
        ));
    }
    Ok(())
}

fn validate_notice(notice: &ReferenceNotice) -> Result<(), IndexError> {
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

fn validate_entry(
    c: &Connection,
    entry: &ReferenceEntry,
    preparation: Option<&ReferencePreparation>,
) -> Result<(), IndexError> {
    validate_reference_metadata(c, entry)?;
    if let ReferenceBacking::Record { key } = &entry.backing {
        let mapping = entry.provenance.record_mapping.as_ref().ok_or_else(|| {
            IndexError::Invalid("record-backed reference lacks mapping evidence".into())
        })?;
        if !reference_pack_applies(preparation, &mapping.package, key.pack().as_str()) {
            return Err(IndexError::Invalid(
                "reference mapping captured package/pack is inapplicable".into(),
            ));
        }
        validate_record_mapping(c, entry, key, mapping)
    } else {
        validate_backing(c, entry)
    }
}

/// Equivalent to the record-owned presence predicate over canonical prepared
/// fields. SQLite returns only the capability bit; Rust never decodes the body.
fn prepared_reading_projection_sql() -> String {
    format!(
        "(coalesce(length(json_extract(definition_json,'$.backing.content.text')),0)>0 OR coalesce(json_array_length(definition_json,'$.backing.content.interactions'),0)>0 OR instr(coalesce(json_extract(definition_json,'$.backing.content.html'),''),'{}')>0)",
        atlas_record::source_content::PREPARED_IMAGE_TAG_PREFIX,
    )
}

fn decode_reference_summary(
    c: &Connection,
    key: &str,
    json: &str,
    hash: &str,
    readable: bool,
) -> Result<ReferenceSummary, IndexError> {
    // The canonical checksum covers the entire envelope, even though serde skips
    // prepared content. A second persisted metadata copy is unnecessary.
    check_envelope_hash(json, hash)?;
    let mut summary: ReferenceSummary = serde_json::from_str(json)?;
    if let ReferenceSummaryBacking::Prepared {
        readable: capability,
    } = &mut summary.backing
    {
        *capability = readable;
    }
    check_entry_key(&summary.key, key)?;
    if summary.label.trim().is_empty() {
        return Err(IndexError::Invalid("empty reference label".into()));
    }
    if let ReferenceSummaryBacking::Record { key } = &summary.backing {
        validate_record_target(c, key)?;
    }
    Ok(summary)
}

fn validate_record_target(c: &Connection, key: &RecordKey) -> Result<(), IndexError> {
    let exists = c.query_row(
        "SELECT EXISTS(SELECT 1 FROM records WHERE key=?)",
        [key.to_string()],
        |r| r.get::<_, bool>(0),
    )?;
    if !exists {
        return Err(IndexError::Invalid(
            "reference record backing is unavailable".into(),
        ));
    }
    Ok(())
}

fn validate_reference_metadata(c: &Connection, entry: &ReferenceEntry) -> Result<(), IndexError> {
    if entry.label.trim().is_empty()
        || entry.provenance.indexing_locale.is_empty()
        || entry.provenance.source_package.is_empty()
        || entry.provenance.recipe.is_empty()
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
        ReferenceBacking::Record { .. } => {
            if entry.permission.disposition != ReferencePermissionDisposition::ExistingRecordPolicy
            {
                return Err(IndexError::Invalid(
                    "reference record backing has invalid permission".into(),
                ));
            }
        }
        ReferenceBacking::Prepared { .. } => {
            if !matches!(
                entry.permission.disposition,
                ReferencePermissionDisposition::ApprovedContent
                    | ReferencePermissionDisposition::ExistingTraitPolicy
            ) {
                return Err(IndexError::Invalid(
                    "reference prepared-content permission mismatch".into(),
                ));
            }
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
        ReferenceBacking::None => {}
    }
    Ok(())
}

fn validate_backing(c: &Connection, entry: &ReferenceEntry) -> Result<(), IndexError> {
    match &entry.backing {
        ReferenceBacking::None => {}
        ReferenceBacking::Record { key } => validate_record_target(c, key)?,
        ReferenceBacking::Prepared {
            authored_markup,
            content,
        } => {
            if content.key != entry.key
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
        }
    }
    Ok(())
}

fn validate_record_mapping(
    c: &Connection,
    entry: &ReferenceEntry,
    key: &RecordKey,
    evidence: &ReferenceRecordMapping,
) -> Result<(), IndexError> {
    if evidence.package != entry.provenance.source_package || evidence.target != *key {
        return Err(IndexError::Invalid(
            "reference mapping package/target differs from source".into(),
        ));
    }
    let bytes: Vec<u8> = c
        .query_row(
            "SELECT b.snapshot FROM record_bodies b JOIN records r USING(record_id) WHERE r.key=?",
            [key.to_string()],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| {
            IndexError::Invalid("reference mapping target has no retained snapshot".into())
        })?;
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
    if let Some(expected) = &evidence.expected_item_type
        && !matches!(node,atlas_record::source_record::SourceNodeView::Item(item) if item.family()==expected)
    {
        return Err(IndexError::Invalid(
            "reference mapping item family differs".into(),
        ));
    }
    if let Some(required) = &evidence.required_localization_key {
        let audience = atlas_record::source_content::ContentAudience {
            include_gm: true,
            include_owner: true,
            implicit_check_dc: atlas_record::source_content::ContentVisibilityRule::All,
        };
        if !atlas_record::source_record::root_description_localizations(&record, audience)
            .any(|key| key == *required)
        {
            return Err(IndexError::Invalid(
                "reference mapping required localization evidence differs".into(),
            ));
        }
    }
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
) -> Result<std::collections::BTreeSet<String>, IndexError> {
    let mut kept = std::collections::BTreeSet::new();
    input.reference_entries.visit(&mut |entry| {
        if entry.key.namespace() == DefinitionNamespace::Trait
            || !kept.insert(entry.key.to_string())
        {
            return Err(IndexError::Invalid(
                "duplicate or trait-specialized reference entry".into(),
            ));
        }
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

pub(crate) fn audit_catalog(
    c: &Connection,
    context: &crate::SourceArtifactBuildContext,
) -> Result<crate::OwnedAbilityCatalog, IndexError> {
    let preparation = context.reference_preparation.as_ref();
    if let Some(preparation) = preparation
        && (preparation.recipe.is_empty() || !crate::codec::checked_hash(&preparation.input_hash))
    {
        return Err(IndexError::Invalid(
            "invalid reference preparation identity".into(),
        ));
    }
    let mut keys = Vec::new();
    let mut catalog = crate::OwnedAbilityCatalog::default();
    let mut statement = c.prepare(
        "SELECT key,definition_json,definition_hash FROM reference_entries ORDER BY key",
    )?;
    let mut rows = statement.query([])?;
    while let Some(row) = rows.next()? {
        let key: String = row.get(0)?;
        let json: String = row.get(1)?;
        let hash: String = row.get(2)?;
        let entry = decode_entry(&key, &json, &hash)?;
        validate_entry(c, &entry, preparation)?;
        if entry.key.namespace() == DefinitionNamespace::Trait {
            let definition = entry.trait_view()?;
            if definition.indexing_locale != context.indexing_locale
                || definition.source_revision != context.source_revision
            {
                return Err(IndexError::Invalid(
                    "trait publication provenance mismatch".into(),
                ));
            }
            crate::traits::validate_definition_targets(c, &definition)?;
        }
        keys.push(key);
        catalog.add_entry(&entry);
    }
    validate_inventory(preparation, keys)?;
    let mut notices = c.prepare("SELECT id,sha256,text,source_url FROM reference_notices")?;
    for row in notices.query_map([], read_notice_row)? {
        validate_notice(&row?)?;
    }
    Ok(catalog)
}

fn read_notice_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ReferenceNotice> {
    Ok(ReferenceNotice {
        id: row.get(0)?,
        sha256: row.get(1)?,
        text: row.get(2)?,
        source_url: row.get(3)?,
    })
}
