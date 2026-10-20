//! Incremental reference publication on the caller-owned workspace transaction.
use crate::{
    IndexBuildInput, IndexError, SourceArtifactBuildContext, SourceSearchProfileIdentity,
    codec::{checked_hash, gzip, sha256},
    input::*,
    persistence::{ProjectionRow, execute},
    projections::root_rows,
};
use atlas_domain::RecordKey;
use atlas_foundry_model::{SNAPSHOT_VERSION, SOURCE_CONTRACT_ID, encode_snapshot};
use atlas_record::{
    source_content::{
        CONTENT_INTERPRETATION_VERSION, ContentAudience, ContentReferenceKind,
        ContentReferenceResolution, ContentReferenceTarget, ContentVisibilityRule,
        validate_prepared_markers, validate_prepared_reference_bindings,
    },
    source_record::{FieldAvailability, SourceContentStatus},
};
use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension};
use std::{collections::BTreeMap, path::Path};

pub struct SqliteIndexWriter<'a, 'owner> {
    maintenance: &'a atlas_storage::MaintenanceSession<'owner>,
}
pub trait IndexArtifactWriter {
    fn output_path(&self) -> &Path;
    fn write(&self, input: &IndexBuildInput)
    -> Result<crate::ArtifactValidationReport, IndexError>;
}
impl<'a, 'owner> SqliteIndexWriter<'a, 'owner> {
    pub fn new(maintenance: &'a atlas_storage::MaintenanceSession<'owner>) -> Self {
        Self { maintenance }
    }
    pub fn write_job(
        &self,
        input: &IndexBuildInput,
        job: &str,
        target_hash: &str,
    ) -> Result<crate::ArtifactValidationReport, IndexError> {
        tracing::info!(target:"atlas_progress",phase="validate_build_input","Checking prepared records");
        validate_input(input)?;
        if input.context.sha256()? != target_hash {
            return Err(IndexError::Invalid(
                "publication input differs from update job target".into(),
            ));
        }
        self.maintenance.publish(|connection| {
            connection.execute_batch("PRAGMA defer_foreign_keys=ON")?;
            write_rows(connection, input)?;
            let report = crate::validation::validate_reference(connection)?;
            tracing::info!(target:"atlas_progress",phase="prune_checkpoints","Removing obsolete saved embeddings");
            crate::embedding_checkpoints::finish_reference_job(connection, job, target_hash)?;
            tracing::info!(target:"atlas_progress",phase="commit_reference","Finishing reference update");
            Ok(report)
        })
    }
}
impl IndexArtifactWriter for SqliteIndexWriter<'_, '_> {
    fn output_path(&self) -> &Path {
        self.maintenance.path()
    }
    fn write(
        &self,
        input: &IndexBuildInput,
    ) -> Result<crate::ArtifactValidationReport, IndexError> {
        validate_input(input)?;
        self.maintenance.publish(|connection| {
            // Cross-root edges may temporarily point at a root being replaced.
            // Validation and commit both require every final reference to exist.
            connection.execute_batch("PRAGMA defer_foreign_keys=ON")?;
            write_rows(connection, input)?;
            crate::validation::validate_reference(connection)
        })
    }
}
impl SourceSearchProfileIdentity {
    pub fn from_spec(profile: atlas_embedding::SearchProfileSpec) -> Self {
        let s = profile.model_spec();
        Self {
            profile_id: profile.profile_id(),
            provider: s.provider_family.into(),
            model_id: s.model_id.into(),
            revision: s.model_revision.into(),
            model_sha256: s.model_sha256.into(),
            tokenizer_sha256: s.tokenizer_sha256.into(),
            segmentation_tokenizer_sha256: atlas_embedding::BASELINE_SEGMENTATION_TOKENIZER_SHA256
                .into(),
            dimensions: s.dimensions,
            pooling: s.pooling.as_str().into(),
            normalization: s.normalization.as_str().into(),
            distance_metric: s.distance_metric.as_str().into(),
            query_prefix: s.query_prefix.into(),
            document_prefix: s.document_prefix.into(),
            token_limit: s.max_input_tokens.unwrap_or(512),
            unit_policy_version: atlas_embedding::EMBEDDING_UNIT_POLICY_VERSION.into(),
            tokenizer_id: s.tokenizer_id.into(),
            output_name: s.output_name.into(),
            dtype: s.dtype.as_str().into(),
            execution_policy: atlas_embedding::EMBEDDING_INFERENCE_POLICY_VERSION.into(),
            input_recipe: atlas_embedding::BASELINE_INPUT_RECIPE.into(),
            assets: s
                .required_assets
                .iter()
                .map(|(path, hash)| (path.to_string(), hash.to_string()))
                .collect(),
        }
    }
}
pub(crate) fn validate_context(c: &SourceArtifactBuildContext) -> Result<(), IndexError> {
    if c.trait_policy != "trait-definition-centroid/v1" {
        return Err(IndexError::Unsupported("trait preparation policy".into()));
    }
    if c.format_version != ARTIFACT_FORMAT_VERSION
        || c.snapshot_version != SNAPSHOT_VERSION
        || c.source_contract != SOURCE_CONTRACT_ID
    {
        return Err(IndexError::Unsupported(
            "source/artifact contract identity".into(),
        ));
    }
    if c.query_projection_version != SOURCE_PROJECTION_VERSION
        || c.query_catalog_version != crate::query::query_capabilities()?.version
        || c.lookup_version != SOURCE_LOOKUP_VERSION
        || c.lexical_selection_version != LEXICAL_SELECTION_VERSION
        || c.fts_tokenizer != "unicode61 remove_diacritics 2"
        || c.content_interpretation_version != CONTENT_INTERPRETATION_VERSION
        || c.content_selection_version
            != atlas_record::source_record::SOURCE_CONTENT_SELECTION_VERSION
        || c.localization_policy != LOCALIZATION_POLICY_VERSION
        || c.asset_policy != ASSET_POLICY_VERSION
        || c.relationship_policy != RELATIONSHIP_POLICY_VERSION
    {
        return Err(IndexError::Unsupported(
            "projection/lookup/content/FTS policy identity".into(),
        ));
    }
    for hash in [
        &c.source_fingerprint,
        &c.locale_catalog_sha256,
        &c.english_catalog_sha256,
    ] {
        if !checked_hash(hash) {
            return Err(IndexError::Invalid(
                "build context digest must be lowercase SHA-256".into(),
            ));
        }
    }
    if c.indexing_locale.is_empty()
        || c.used_trait_labels
            .iter()
            .any(|(id, label)| id.trim().is_empty() || label.trim().is_empty())
        || c.content_selection_version.is_empty()
        || c.localization_policy.is_empty()
        || c.asset_policy.is_empty()
        || c.relationship_policy.is_empty()
    {
        return Err(IndexError::Invalid(
            "build context policy or locale is empty".into(),
        ));
    }
    for profile in c.canonical_profiles()? {
        let spec = atlas_embedding::search_profile(&profile.profile_id)
            .map_err(|e| IndexError::Unsupported(e.to_string()))?;
        if profile.model_id != spec.model_spec().model_id
            || profile.dimensions != spec.model_spec().dimensions
            || profile.token_limit == 0
            || profile.token_limit > 32768
            || profile.assets.is_empty()
            || profile
                .assets
                .iter()
                .any(|(path, hash)| path.is_empty() || !checked_hash(hash))
            || !checked_hash(&profile.model_sha256)
            || !checked_hash(&profile.tokenizer_sha256)
            || !checked_hash(&profile.segmentation_tokenizer_sha256)
            || [
                &profile.provider,
                &profile.revision,
                &profile.pooling,
                &profile.normalization,
                &profile.distance_metric,
                &profile.unit_policy_version,
                &profile.tokenizer_id,
                &profile.output_name,
                &profile.dtype,
                &profile.execution_policy,
                &profile.input_recipe,
            ]
            .iter()
            .any(|s| s.is_empty())
        {
            return Err(IndexError::Unsupported(
                "invalid stored search profile execution or recipe identity".into(),
            ));
        }
        let expected_assets = spec
            .model_spec()
            .required_assets
            .iter()
            .map(|(path, _)| *path)
            .collect::<std::collections::BTreeSet<_>>();
        if profile
            .assets
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>()
            != expected_assets
            || profile.assets.get("onnx/model.onnx") != Some(&profile.model_sha256)
            || profile.assets.get("tokenizer.json") != Some(&profile.tokenizer_sha256)
            || profile.dtype != "f32"
            || profile.normalization != "l2"
            || profile.distance_metric != "cosine"
        {
            return Err(IndexError::Invalid(
                "incomplete or inconsistent stored profile asset/vector contract".into(),
            ));
        }
    }
    Ok(())
}
fn validate_input(input: &IndexBuildInput) -> Result<(), IndexError> {
    validate_context(&input.context)?;
    for profile in &input.context.search_profiles {
        let spec = atlas_embedding::search_profile(&profile.profile_id)
            .map_err(|e| IndexError::Unsupported(e.to_string()))?;
        if profile != &SourceSearchProfileIdentity::from_spec(spec) {
            return Err(IndexError::Unsupported(
                "writer requires the exact current search profile execution and recipe".into(),
            ));
        }
    }
    if input.context.source_file_count < input.records.len() {
        return Err(IndexError::Invalid(
            "source input count is smaller than admitted root count".into(),
        ));
    }
    if input.context.search_profiles.is_empty() && !input.semantic_units.is_empty() {
        return Err(IndexError::Invalid(
            "semantic units supplied without a model".into(),
        ));
    }
    let mut packs = BTreeMap::new();
    for p in &input.packs {
        if p.pack_id.is_empty() || packs.insert(p.pack_id.as_str(), p).is_some() {
            return Err(IndexError::Invalid(
                "duplicate or empty pack identity".into(),
            ));
        }
    }
    let context_hash = input.context.preparation_sha256()?;
    let mut keys = std::collections::BTreeSet::new();
    for r in &input.records {
        if !keys.insert(r.record.key()) {
            return Err(IndexError::Invalid(format!(
                "duplicate root {}",
                r.record.key()
            )));
        }
        let pack = packs
            .get(r.record.key().pack().as_str())
            .ok_or_else(|| IndexError::Invalid("record pack is absent".into()))?;
        if atlas_record::source_record::SourceNodeView::from(r.record.source()).document_kind()
            != pack.document_kind
        {
            return Err(IndexError::Invalid(
                "record document family disagrees with pack".into(),
            ));
        }
        if !checked_hash(&r.content_hash) || r.preparation_context_hash != context_hash {
            return Err(IndexError::Invalid(
                "source/preparation hash is invalid or context disagrees".into(),
            ));
        }
        let path = Path::new(&r.source_path);
        if r.source_path.is_empty()
            || path.is_absolute()
            || path
                .components()
                .any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return Err(IndexError::Invalid(
                "source path must be relative and contain no traversal".into(),
            ));
        }
    }
    Ok(())
}
fn write_rows(c: &Connection, input: &IndexBuildInput) -> Result<(), IndexError> {
    crate::traits::write_traits(c, input)?;
    tracing::info!(target:"atlas_progress",phase="artifact_identity","Updating record identities");
    let mut canonical = input.context.clone();
    canonical.search_profiles = input.context.canonical_profiles()?;
    let json = serde_json::to_string(&canonical)?;
    execute(
        c,
        "INSERT INTO artifact_context(singleton,format_version,context_hash,context_json) VALUES(1,?,?,?) ON CONFLICT(singleton) DO UPDATE SET format_version=excluded.format_version,context_hash=excluded.context_hash,context_json=excluded.context_json WHERE context_hash<>excluded.context_hash",
        &[
            Value::Integer(i64::from(ARTIFACT_FORMAT_VERSION)),
            Value::Text(sha256(json.as_bytes())),
            Value::Text(json),
        ],
    )?;
    for pack in &input.packs {
        let mut r = ProjectionRow::new("packs");
        r.text("pack_id", &pack.pack_id);
        r.text("label", &pack.label);
        crate::publication_delta::upsert(c, &r, "pack_id")?;
    }
    let root_ids = input
        .records
        .iter()
        .map(|r| {
            let name = match atlas_record::source_record::SourceNodeView::from(r.record.source())
                .name()
            {
                atlas_record::source_record::SourceFieldView::Value(name) => Some(name.as_str()),
                _ => None,
            };
            Ok((
                r.record.key().clone(),
                crate::identity::retain_source(c, r.record.key(), name)?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, IndexError>>()?;
    let product_ids = input
        .records
        .iter()
        .filter(|source| {
            !matches!(
                source.record.source(),
                atlas_foundry_model::FoundryDocumentSource::Macro(_)
            )
        })
        .map(|source| (source.record.key().clone(), root_ids[source.record.key()]))
        .collect::<BTreeMap<_, _>>();
    let pack_labels = input
        .packs
        .iter()
        .map(|p| (p.pack_id.as_str(), p.label.as_str()))
        .collect::<BTreeMap<_, _>>();
    tracing::info!(target:"atlas_progress",phase="artifact_snapshots","Updating records and filters");
    let mut changed = std::collections::BTreeSet::new();
    let mut live = std::collections::BTreeSet::new();
    for (ordinal, source) in input.records.iter().enumerate() {
        let id = id_for(&root_ids, source.record.key())?;
        let label = pack_labels
            .get(source.record.key().pack().as_str())
            .ok_or_else(|| IndexError::Invalid("missing pack label".into()))?;
        live.insert(id);
        let digest = crate::publication_delta::root_digest(source, label)?;
        let previous: Option<String> = c
            .query_row(
                "SELECT input_hash FROM projection_allocations WHERE record_id=?",
                [id],
                |r| r.get(0),
            )
            .optional()?;
        let exists: bool = c.query_row(
            "SELECT EXISTS(SELECT 1 FROM records WHERE record_id=?)",
            [id],
            |r| r.get(0),
        )?;
        if exists && previous.as_deref() == Some(&digest) {
            continue;
        }
        changed.insert(id);
        crate::publication_delta::clear_root_derived(c, id)?;
        let mut ids = crate::publication_delta::allocate(c, source, label, id, &digest)?;
        for row in root_rows(source, label, id, &mut ids)? {
            if row.table == "records" {
                crate::publication_delta::upsert(c, &row, "record_id")?;
            } else {
                row.insert(c)?;
            }
        }
        let bytes = gzip(&encode_snapshot(source.record.source())?)?;
        c.execute(
            "INSERT INTO record_bodies(record_id,codec_version,encoding,snapshot) VALUES(?,?,?,?)",
            rusqlite::params![id, i64::from(SNAPSHOT_VERSION), "gzip6", bytes],
        )?;
        if (ordinal + 1).is_multiple_of(500) || ordinal + 1 == input.records.len() {
            tracing::info!(target:"atlas_progress",phase="artifact_snapshots",current=(ordinal+1) as u64,total=input.records.len() as u64,unit="records","Saving records");
        }
    }
    tracing::info!(target:"atlas_progress",phase="artifact_content","Updating content and links");
    for (record_ordinal, source) in input.records.iter().enumerate() {
        let id = id_for(&root_ids, source.record.key())?;
        if !changed.contains(&id) {
            continue;
        }
        write_content(c, source, id, &root_ids, input.context.audience)?;
        for (ordinal, relationship) in source.relationships.iter().enumerate() {
            if relationship.locator.record != *source.record.key()
                || source
                    .record
                    .node_at(&relationship.locator.owners)
                    .is_none()
            {
                return Err(IndexError::Invalid(
                    "structured relationship locator disagrees with source".into(),
                ));
            }
            let kind = serde_json::from_str::<String>(&serde_json::to_string(&relationship.kind)?)?;
            write_relationship(
                c,
                id,
                &SourceStoredRelationship {
                    locator: relationship.locator.clone(),
                    ordinal,
                    origin: "structured".into(),
                    kind,
                    authored_target: relationship.authored_target.clone(),
                    occurrence_path: None,
                    resolution: relationship.resolution.clone(),
                    details: SourceRelationshipDetails {
                        availability: Some(relationship.availability.clone()),
                        reference_kind: None,
                        label: None,
                        audiences: Vec::new(),
                    },
                },
                &root_ids,
            )?;
        }
        for diagnostic in &source.diagnostics {
            write_diagnostic(c, id, diagnostic)?;
        }
        for diagnostic in crate::projections::projection_diagnostics(&source.record) {
            write_diagnostic(c, id, &diagnostic)?;
        }
        if (record_ordinal + 1).is_multiple_of(500) || record_ordinal + 1 == input.records.len() {
            tracing::info!(target:"atlas_progress",phase="artifact_content",current=(record_ordinal+1) as u64,total=input.records.len() as u64,unit="records","Saving content and links");
        }
    }
    crate::publication_delta::aliases(c, input, &product_ids)?;
    let definitions = crate::query::query_capabilities()?.fields;
    let existing = c
        .prepare("SELECT field FROM query_field_catalog")?
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for field in existing {
        if !definitions.iter().any(|definition| definition.id == field) {
            c.execute("DELETE FROM query_field_catalog WHERE field=?", [field])?;
        }
    }
    for definition in definitions {
        let mut r = ProjectionRow::new("query_field_catalog");
        r.text("field", &definition.id);
        r.text("definition_json", serde_json::to_string(&definition)?);
        crate::publication_delta::upsert(c, &r, "field")?;
    }
    tracing::info!(target:"atlas_progress",phase="artifact_search","Updating search indexes");
    crate::publication_delta::search_units(c, input, &product_ids)?;
    tracing::info!(target:"atlas_progress",phase="artifact_removals","Removing outdated records");
    crate::publication_delta::remove_absent_roots(c, &live)?;
    c.execute(
        "DELETE FROM packs WHERE pack_id NOT IN (SELECT pack_id FROM records)",
        [],
    )?;
    // Release dirty-page pressure before scanning newly populated vec0 chunks.
    // SQLite flushes pages within this transaction; it does not commit them.
    // Validation failure still rolls back the entire publication, including DDL.
    tracing::info!(target:"atlas_progress",phase="flush_reference","Flushing database writes");
    c.cache_flush()?;
    Ok(())
}
fn id_for(ids: &BTreeMap<RecordKey, i64>, key: &RecordKey) -> Result<i64, IndexError> {
    ids.get(key)
        .copied()
        .ok_or_else(|| IndexError::Invalid(format!("referenced root is absent: {key}")))
}
pub(crate) fn product_id(
    ids: &BTreeMap<RecordKey, i64>,
    key: &RecordKey,
) -> Result<i64, IndexError> {
    ids.get(key).copied().ok_or_else(|| {
        IndexError::Invalid(format!(
            "referenced product root is absent or excluded: {key}"
        ))
    })
}
pub(crate) fn permits(a: ContentAudience, v: ContentVisibilityRule) -> bool {
    match v {
        ContentVisibilityRule::All => true,
        ContentVisibilityRule::Gm => a.include_gm,
        ContentVisibilityRule::Owner => a.include_owner,
        ContentVisibilityRule::None => false,
    }
}
fn write_content(
    c: &Connection,
    source: &SourceArtifactRecordInput,
    id: i64,
    ids: &BTreeMap<RecordKey, i64>,
    audience: ContentAudience,
) -> Result<(), IndexError> {
    for outcome in &source.content {
        let locator = outcome.locator();
        if locator.record != *source.record.key() {
            return Err(IndexError::Invalid(
                "prepared content belongs to another root".into(),
            ));
        }
        let authored = source
            .record
            .authored_content_at(locator)
            .value()
            .ok_or_else(|| {
                IndexError::Invalid("prepared cache has no selected authored field".into())
            })?;
        let published = matches!(outcome.visibility_availability, FieldAvailability::Value)
            && permits(audience, outcome.visibility);
        let mut r = ProjectionRow::new("prepared_content");
        r.id("published", i64::from(published));
        r.text(
            "metadata_json",
            serde_json::to_string(&crate::persisted_inputs::PreparationMetadata::from_outcome(
                outcome,
            ))?,
        );
        r.id("record_id", id);
        r.text("owners_json", serde_json::to_string(&locator.owners)?);
        r.text("field_path", &locator.field);
        r.text(
            "role",
            serde_json::from_str::<String>(&serde_json::to_string(&outcome.role)?)?,
        );
        r.text(
            "visibility",
            serde_json::from_str::<String>(&serde_json::to_string(&outcome.visibility)?)?,
        );
        r.text("authored_markup_sha256", sha256(authored.as_bytes()));
        r.text("preparation_context_hash", &source.preparation_context_hash);
        let (status, html, interactions) = match &outcome.status {
            SourceContentStatus::Prepared(prepared) => {
                if prepared.interpretation_version != CONTENT_INTERPRETATION_VERSION
                    || prepared.authored_markup_sha256 != sha256(authored.as_bytes())
                {
                    return Err(IndexError::Invalid(
                        "prepared field input/interpreter identity disagrees".into(),
                    ));
                }
                validate_prepared_markers(&prepared.html, &prepared.interactions)
                    .map_err(|e| IndexError::Invalid(e.to_string()))?;
                validate_prepared_reference_bindings(
                    &prepared.html,
                    locator,
                    &prepared.authored_markup_sha256,
                    &prepared.references,
                )
                .map_err(|e| IndexError::Invalid(e.to_string()))?;
                for reference in &prepared.references {
                    if reference.visible && published {
                        let kind = match reference.kind {
                            ContentReferenceKind::Uuid => "uuid",
                            ContentReferenceKind::Compendium => "compendium",
                            ContentReferenceKind::Embed { .. } => "embed",
                            ContentReferenceKind::HtmlLink => "html_link",
                        };
                        write_relationship(
                            c,
                            id,
                            &SourceStoredRelationship {
                                locator: locator.clone(),
                                ordinal: reference.ordinal,
                                origin: "content".into(),
                                kind: kind.into(),
                                authored_target: Some(reference.authored_target.clone()),
                                occurrence_path: Some(reference.path.clone()),
                                resolution: reference.resolution.clone(),
                                details: SourceRelationshipDetails {
                                    availability: None,
                                    reference_kind: Some(reference.kind.clone()),
                                    label: Some(reference.label.clone()),
                                    audiences: reference.audiences.clone(),
                                },
                            },
                            ids,
                        )?;
                    }
                }
                for diagnostic in prepared.diagnostics.iter().filter(|_| published) {
                    write_diagnostic(
                        c,
                        id,
                        &SourceArtifactDiagnostic {
                            source_diagnostic: None,
                            owners: Some(locator.owners.clone()),
                            field: Some(locator.field.clone()),
                            stage: "preparation".into(),
                            code: format!("{:?}", diagnostic.code),
                            message: diagnostic.detail.clone(),
                        },
                    )?;
                }
                if prepared.html.is_empty() && prepared.interactions.is_empty() {
                    ("empty", Value::Null, "[]".into())
                } else {
                    (
                        "prepared",
                        Value::Blob(gzip(prepared.html.as_bytes())?),
                        serde_json::to_string(&prepared.interactions)?,
                    )
                }
            }
            SourceContentStatus::FormatUnavailable { .. } => {
                ("format_unavailable", Value::Null, "[]".into())
            }
            SourceContentStatus::UnsupportedFormat { .. } => {
                ("unsupported_format", Value::Null, "[]".into())
            }
            SourceContentStatus::PreparationFailed { .. } => {
                ("preparation_failed", Value::Null, "[]".into())
            }
        };
        r.text("outcome", status);
        if published && !matches!(status, "prepared" | "empty") {
            write_diagnostic(
                c,
                id,
                &SourceArtifactDiagnostic {
                    source_diagnostic: None,
                    owners: Some(locator.owners.clone()),
                    field: Some(locator.field.clone()),
                    stage: "preparation".into(),
                    code: status.into(),
                    message: match &outcome.status {
                        SourceContentStatus::PreparationFailed { message, .. } => message.clone(),
                        SourceContentStatus::FormatUnavailable { availability, .. } => {
                            format!("Content format unavailable: {availability:?}")
                        }
                        SourceContentStatus::UnsupportedFormat { format, .. } => {
                            format!("Content format unsupported: {format:?}")
                        }
                        SourceContentStatus::Prepared(_) => {
                            return Err(IndexError::Invalid(
                                "prepared content has failure status".into(),
                            ));
                        }
                    },
                },
            )?;
        }
        r.push("html_gzip", html);
        r.text("interactions_json", interactions);
        r.insert(c)?;
    }
    Ok(())
}
pub(crate) fn write_diagnostic(
    c: &Connection,
    id: i64,
    d: &SourceArtifactDiagnostic,
) -> Result<(), IndexError> {
    let mut r = ProjectionRow::new("developer_diagnostics");
    r.id("record_id", id);
    r.push(
        "owners_json",
        d.owners
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?
            .map(Value::Text)
            .unwrap_or(Value::Null),
    );
    r.push(
        "field_path",
        d.field
            .as_ref()
            .map(|s| Value::Text(s.clone()))
            .unwrap_or(Value::Null),
    );
    r.text("stage", &d.stage);
    r.text(
        "details_json",
        serde_json::to_string(&crate::read_content::DiagnosticDetails {
            source_diagnostic: d.source_diagnostic.clone(),
            code: d.code.clone(),
            message: d.message.clone(),
        })?,
    );
    r.insert(c)
}
fn write_relationship(
    c: &Connection,
    id: i64,
    occurrence: &SourceStoredRelationship,
    ids: &BTreeMap<RecordKey, i64>,
) -> Result<(), IndexError> {
    let SourceStoredRelationship {
        locator,
        ordinal,
        origin,
        kind,
        authored_target,
        occurrence_path,
        resolution,
        details,
    } = occurrence;
    let mut r = ProjectionRow::new("relationship_occurrences");
    r.id("record_id", id);
    r.text("owners_json", serde_json::to_string(&locator.owners)?);
    r.text("field_path", &locator.field);
    r.id("ordinal", *ordinal as i64);
    r.text("origin", origin);
    r.text("kind", kind);
    r.push(
        "authored_target",
        authored_target
            .as_deref()
            .map(|s| Value::Text(s.into()))
            .unwrap_or(Value::Null),
    );
    r.push(
        "occurrence_path",
        occurrence_path
            .as_deref()
            .map(|s| Value::Text(s.into()))
            .unwrap_or(Value::Null),
    );
    r.text("details_json", serde_json::to_string(&details)?);
    let (status, target, owners, url) = match resolution {
        ContentReferenceResolution::Resolved(ContentReferenceTarget::Record { key }) => {
            ("resolved_record", Some(id_for(ids, key)?), None, None)
        }
        ContentReferenceResolution::Resolved(ContentReferenceTarget::OwnedNode { key, owners }) => {
            (
                "resolved_owned",
                Some(id_for(ids, key)?),
                Some(serde_json::to_string(owners)?),
                None,
            )
        }
        ContentReferenceResolution::Resolved(ContentReferenceTarget::Url { url }) => {
            if !atlas_record::source_content::is_safe_content_url(url) {
                return Err(IndexError::Invalid(
                    "resolved relationship URL violates content policy".into(),
                ));
            }
            ("resolved_url", None, None, Some(url.clone()))
        }
        ContentReferenceResolution::UnverifiedUrl { url } => {
            if !atlas_record::source_content::is_safe_content_url(url) {
                return Err(IndexError::Invalid(
                    "unverified relationship URL violates content policy".into(),
                ));
            }
            ("unverified_url", None, None, Some(url.clone()))
        }
        ContentReferenceResolution::Unresolved => ("unresolved", None, None, None),
        ContentReferenceResolution::Blocked => ("blocked", None, None, None),
    };
    r.text("resolution", status);
    r.push(
        "target_record_id",
        target.map(Value::Integer).unwrap_or(Value::Null),
    );
    r.push(
        "target_owners_json",
        owners.map(Value::Text).unwrap_or(Value::Null),
    );
    r.push("target_url", url.map(Value::Text).unwrap_or(Value::Null));
    r.insert(c)
}
