//! Complete temporary artifact builds, validated before atomic publication.
use crate::{
    IndexBuildInput, IndexError, SourceArtifactBuildContext, SourceSemanticModelIdentity,
    codec::{checked_hash, gzip, sha256},
    input::*,
    persistence::{ProjectionRow, execute, writable},
    projections::{ProjectionIds, root_rows},
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
use diesel::{Connection, ExpressionMethods, RunQueryDsl, connection::SimpleConnection};
use rusqlite::types::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub struct SqliteIndexWriter {
    path: PathBuf,
}
pub trait IndexArtifactWriter {
    fn output_path(&self) -> &Path;
    fn write(&self, input: &IndexBuildInput)
    -> Result<crate::ArtifactValidationReport, IndexError>;
}
impl SqliteIndexWriter {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}
impl IndexArtifactWriter for SqliteIndexWriter {
    fn output_path(&self) -> &Path {
        &self.path
    }
    fn write(
        &self,
        input: &IndexBuildInput,
    ) -> Result<crate::ArtifactValidationReport, IndexError> {
        validate_input(input)?;
        if input.context.semantic_model.is_some() {
            atlas_sqlite_vec::register_sqlite_vec_auto_extension()
                .map_err(|e| IndexError::Operation(e.to_string()))?;
        }
        let parent = self
            .path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent)?;
        let temporary = tempfile::NamedTempFile::new_in(parent)?;
        let mut c = writable(temporary.path())?;
        c.transaction::<_, IndexError, _>(|c| {
            c.batch_execute(include_str!(
                "../migrations/00000000000001_create_artifact/up.sql"
            ))?;
            c.batch_execute(&format!("PRAGMA user_version={ARTIFACT_FORMAT_VERSION}"))?;
            write_rows(c, input)?;
            c.batch_execute("INSERT INTO lexical_fts(lexical_fts) VALUES('rebuild');")?;
            Ok(())
        })?;
        drop(c);
        let report = crate::validation::validate_artifact(temporary.path())?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(&self.path)
            .map_err(|e| IndexError::Operation(format!("cannot atomically publish artifact (it may be in use); prior artifact preserved: {e}")))?;
        #[cfg(unix)]
        {
            std::fs::File::open(parent)?.sync_all()?;
        }
        Ok(report)
    }
}
impl SourceSemanticModelIdentity {
    pub fn current() -> Self {
        let s = atlas_embedding::default_embedding_model_spec();
        Self {
            provider: s.provider_family.into(),
            model_id: s.model_id.into(),
            revision: s.model_revision.into(),
            model_sha256: s.model_sha256.into(),
            tokenizer_sha256: s.tokenizer_sha256.into(),
            dimensions: s.dimensions,
            pooling: s.pooling.as_str().into(),
            normalization: s.normalization.as_str().into(),
            distance_metric: s.distance_metric.as_str().into(),
            query_prefix: s.query_prefix.into(),
            document_prefix: s.document_prefix.into(),
            token_limit: s.max_input_tokens.unwrap_or(512),
            unit_policy_version: atlas_embedding::EMBEDDING_UNIT_POLICY_VERSION.into(),
        }
    }
}
pub(crate) fn validate_context(c: &SourceArtifactBuildContext) -> Result<(), IndexError> {
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
    if let Some(model) = &c.semantic_model
        && model != &SourceSemanticModelIdentity::current()
    {
        return Err(IndexError::Unsupported(
            "semantic model or unit policy identity".into(),
        ));
    }
    Ok(())
}
fn validate_input(input: &IndexBuildInput) -> Result<(), IndexError> {
    validate_context(&input.context)?;
    if input.context.source_file_count < input.records.len() {
        return Err(IndexError::Invalid(
            "source input count is smaller than admitted root count".into(),
        ));
    }
    if input.context.semantic_model.is_none() && !input.semantic_units.is_empty() {
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
    let context_hash = input.context.sha256()?;
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
fn write_rows(c: &mut diesel::SqliteConnection, input: &IndexBuildInput) -> Result<(), IndexError> {
    let json = serde_json::to_string(&input.context)?;
    execute(
        c,
        "INSERT INTO artifact_context(singleton,format_version,context_hash,context_json) VALUES(1,?,?,?)",
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
        r.insert(c)?;
    }
    let root_ids = input
        .records
        .iter()
        .enumerate()
        .map(|(i, r)| {
            Ok((
                r.record.key().clone(),
                i64::try_from(i + 1).map_err(|e| IndexError::Invalid(e.to_string()))?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, IndexError>>()?;
    let pack_labels = input
        .packs
        .iter()
        .map(|p| (p.pack_id.as_str(), p.label.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut ids = ProjectionIds::default();
    for source in &input.records {
        let id = id_for(&root_ids, source.record.key())?;
        let label = pack_labels
            .get(source.record.key().pack().as_str())
            .ok_or_else(|| IndexError::Invalid("missing pack label".into()))?;
        for row in root_rows(source, label, id, &mut ids)? {
            row.insert(c)?;
        }
        let bytes = gzip(&encode_snapshot(source.record.source())?)?;
        diesel::insert_into(crate::schema::record_bodies::table)
            .values((
                crate::schema::record_bodies::record_id.eq(id),
                crate::schema::record_bodies::codec_version.eq(i64::from(SNAPSHOT_VERSION)),
                crate::schema::record_bodies::encoding.eq("gzip6"),
                crate::schema::record_bodies::snapshot.eq(bytes),
            ))
            .execute(c)?;
    }
    for source in &input.records {
        let id = id_for(&root_ids, source.record.key())?;
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
    }
    for alias in &input.aliases {
        if alias.alias.trim().is_empty() || alias.evidence.trim().is_empty() {
            return Err(IndexError::Invalid(
                "alias needs authored text and explicit evidence".into(),
            ));
        }
        let id = product_id(input, &root_ids, &alias.record)?;
        let mut r = ProjectionRow::new("verified_aliases");
        r.id("record_id", id);
        r.text("alias", &alias.alias);
        r.text(
            "alias_lookup_key",
            atlas_domain::normalize_record_name(&alias.alias),
        );
        r.text("evidence_json", serde_json::to_string(&alias.evidence)?);
        r.insert(c)?;
    }
    for pair in &input.remaster_pairs {
        let mut r = ProjectionRow::new("remaster_pairs");
        r.id(
            "legacy_record_id",
            product_id(input, &root_ids, &pair.legacy)?,
        );
        r.id(
            "remaster_record_id",
            product_id(input, &root_ids, &pair.remaster)?,
        );
        if pair.evidence.trim().is_empty() {
            return Err(IndexError::Invalid("remaster pair has no evidence".into()));
        }
        r.text("evidence_json", serde_json::to_string(&pair.evidence)?);
        r.insert(c)?;
    }
    for definition in crate::query::query_capabilities()?.fields {
        let mut r = ProjectionRow::new("query_field_catalog");
        r.text("field", &definition.id);
        r.text("definition_json", serde_json::to_string(&definition)?);
        r.insert(c)?;
    }
    for (i, unit) in input.lexical_units.iter().enumerate() {
        let id = product_id(input, &root_ids, &unit.record)?;
        let mut r = ProjectionRow::new("lexical_units");
        r.id("unit_id", (i + 1) as i64);
        r.id("record_id", id);
        r.text("owners_json", serde_json::to_string(&unit.owners)?);
        r.push(
            "field_path",
            unit.field
                .as_ref()
                .map(|s| Value::Text(s.clone()))
                .unwrap_or(Value::Null),
        );
        r.push(
            "section_json",
            unit.address
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?
                .map(Value::Text)
                .unwrap_or(Value::Null),
        );
        r.text("unit_kind", unit.kind.as_str());
        r.text("identity_terms", &unit.identity_terms);
        r.text("alias_terms", &unit.alias_terms);
        r.text("structured_terms", &unit.structured_terms);
        r.text("definition_terms", &unit.definition_terms);
        r.insert(c)?;
    }
    if let Some(model) = &input.context.semantic_model {
        let mut r = ProjectionRow::new("semantic_models");
        r.id("model_id", 1);
        r.text("identity_json", serde_json::to_string(model)?);
        r.id("dimensions", model.dimensions as i64);
        r.id("unit_policy_version", 1);
        r.insert(c)?;
        c.batch_execute(&format!("CREATE VIRTUAL TABLE semantic_vectors USING vec0(unit_id INTEGER PRIMARY KEY,embedding FLOAT[{}] distance_metric=cosine);",model.dimensions))?;
        for (i, unit) in input.semantic_units.iter().enumerate() {
            crate::validation::check_vector(&unit.vector, model.dimensions)?;
            let mut r = ProjectionRow::new("semantic_units");
            r.id("unit_id", (i + 1) as i64);
            r.id("record_id", product_id(input, &root_ids, &unit.record)?);
            r.id("model_id", 1);
            r.text("owners_json", serde_json::to_string(&unit.owners)?);
            r.push(
                "field_path",
                unit.field
                    .as_ref()
                    .map(|s| Value::Text(s.clone()))
                    .unwrap_or(Value::Null),
            );
            r.text("section_json", serde_json::to_string(&unit.address)?);
            r.text(
                "unit_kind",
                if matches!(
                    unit.address,
                    atlas_domain::SourcePassageAddress::Identity {}
                ) {
                    "identity"
                } else {
                    "passage"
                },
            );
            r.id("chunk_ordinal", unit.chunk_ordinal as i64);
            r.id("input_token_count", unit.input_token_count as i64);
            r.text("input_hash", &unit.input_hash);
            r.insert(c)?;
            execute(
                c,
                "INSERT INTO semantic_vectors(unit_id,embedding) VALUES(?,?)",
                &[
                    Value::Integer((i + 1) as i64),
                    Value::Blob(crate::read_units::vector_bytes(&unit.vector)),
                ],
            )?;
        }
    }
    Ok(())
}
fn id_for(ids: &BTreeMap<RecordKey, i64>, key: &RecordKey) -> Result<i64, IndexError> {
    ids.get(key)
        .copied()
        .ok_or_else(|| IndexError::Invalid(format!("referenced root is absent: {key}")))
}
fn product_id(
    input: &IndexBuildInput,
    ids: &BTreeMap<RecordKey, i64>,
    key: &RecordKey,
) -> Result<i64, IndexError> {
    let id = id_for(ids, key)?;
    let index = usize::try_from(id - 1).map_err(|e| IndexError::Invalid(e.to_string()))?;
    if matches!(
        input.records[index].record.source(),
        atlas_foundry_model::FoundryDocumentSource::Macro(_)
    ) {
        return Err(IndexError::Invalid(
            "Macro cannot be a product identity/search unit".into(),
        ));
    }
    Ok(id)
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
    c: &mut diesel::SqliteConnection,
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
        if !matches!(outcome.visibility_availability, FieldAvailability::Value)
            || !permits(audience, outcome.visibility)
        {
            continue;
        }
        let mut r = ProjectionRow::new("prepared_content");
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
                    if reference.visible {
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
                                    audiences: reference.audiences.clone(),
                                },
                            },
                            ids,
                        )?;
                    }
                }
                for diagnostic in &prepared.diagnostics {
                    write_diagnostic(
                        c,
                        id,
                        &SourceArtifactDiagnostic {
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
        if !matches!(status, "prepared" | "empty") {
            write_diagnostic(
                c,
                id,
                &SourceArtifactDiagnostic {
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
fn write_diagnostic(
    c: &mut diesel::SqliteConnection,
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
            code: d.code.clone(),
            message: d.message.clone(),
        })?,
    );
    r.insert(c)
}
fn write_relationship(
    c: &mut diesel::SqliteConnection,
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
