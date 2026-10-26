//! Exact preparation metadata complements existing HTML, occurrence and DTO storage.
//! This stores no source JSON, rendered HTML copy, plain text, or second DTO tree.
use crate::{
    IndexError, SourceArtifactBuildContext, SourceArtifactDiagnostic, SourceArtifactRecordInput,
};
use atlas_record::source_content::{
    ContentInterpretationDiagnostic, ContentReferenceOccurrence, PreparedSourceContent,
    SourceContentLocator,
};
use atlas_record::source_record::{
    FieldAvailability, SourceBackedRecord, SourceContentFormat, SourceContentOutcome,
    SourceContentStatus, SourceRelationshipOccurrence,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct PreparationMetadata {
    visibility_availability: FieldAvailability,
    status: PreparationStatus,
}
#[derive(Serialize, Deserialize)]
enum PreparationStatus {
    Prepared {
        hidden_references: Vec<ContentReferenceOccurrence>,
        diagnostics: Vec<ContentInterpretationDiagnostic>,
    },
    FormatUnavailable(FieldAvailability),
    UnsupportedFormat(SourceContentFormat),
    PreparationFailed(String),
}
impl PreparationMetadata {
    pub(crate) fn from_outcome(outcome: &SourceContentOutcome) -> Self {
        Self {
            visibility_availability: outcome.visibility_availability.clone(),
            status: match &outcome.status {
                SourceContentStatus::Prepared(p) => PreparationStatus::Prepared {
                    hidden_references: p
                        .references
                        .iter()
                        .filter(|r| !r.visible)
                        .cloned()
                        .collect(),
                    diagnostics: p.diagnostics.clone(),
                },
                SourceContentStatus::FormatUnavailable { availability, .. } => {
                    PreparationStatus::FormatUnavailable(availability.clone())
                }
                SourceContentStatus::UnsupportedFormat { format, .. } => {
                    PreparationStatus::UnsupportedFormat(*format)
                }
                SourceContentStatus::PreparationFailed { message, .. } => {
                    PreparationStatus::PreparationFailed(message.clone())
                }
            },
        }
    }
}

/// Caller owns an authenticated workspace session. Query-catalog compatibility is
/// deliberately irrelevant: these inputs also drive projection-only backfills.
pub fn read_persisted_inputs(
    c: &Connection,
    context: &SourceArtifactBuildContext,
) -> Result<Vec<SourceArtifactRecordInput>, IndexError> {
    read_selected_persisted_inputs(c, context, None)
}

pub(crate) fn read_persisted_input(
    c: &Connection,
    context: &SourceArtifactBuildContext,
    key: &atlas_domain::RecordKey,
) -> Result<Option<SourceArtifactRecordInput>, IndexError> {
    Ok(read_selected_persisted_inputs(c, context, Some(key))?
        .into_iter()
        .next())
}

fn read_selected_persisted_inputs(
    c: &Connection,
    context: &SourceArtifactBuildContext,
    key: Option<&atlas_domain::RecordKey>,
) -> Result<Vec<SourceArtifactRecordInput>, IndexError> {
    if context.source_contract != atlas_foundry_model::SOURCE_CONTRACT_ID
        || context.snapshot_version != atlas_foundry_model::SNAPSHOT_VERSION
    {
        return Err(IndexError::Unsupported(
            "persisted source admission contract differs".into(),
        ));
    }
    let preparation_hash = context.preparation_sha256()?;
    let mut statement = c.prepare("SELECT r.record_id,r.key,r.pack_id,r.source_path,r.content_hash,b.codec_version,b.encoding,b.snapshot FROM records r JOIN record_bodies b USING(record_id) WHERE (?1 IS NULL OR r.key=?1) ORDER BY r.record_id")?;
    let mut rows = statement.query([key.map(ToString::to_string)])?;
    let mut inputs = Vec::new();
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let key: String = row.get(1)?;
        let pack: String = row.get(2)?;
        if row.get::<_, u32>(5)? != atlas_foundry_model::SNAPSHOT_VERSION
            || row.get::<_, String>(6)? != "gzip6"
        {
            return Err(IndexError::Invalid(
                "persisted snapshot codec differs".into(),
            ));
        }
        let source = atlas_foundry_model::decode_snapshot(&crate::codec::gunzip(
            &row.get::<_, Vec<u8>>(7)?,
        )?)?;
        let record = SourceBackedRecord::new(&pack, source)
            .map_err(|_| IndexError::Invalid("persisted source has no identity".into()))?;
        if record.key().to_string() != key {
            return Err(IndexError::Invalid("persisted snapshot key differs".into()));
        }
        let mut references = crate::read_content::content_references_for_record(c, id)?;
        let mut content = Vec::new();
        let mut fields = c.prepare("SELECT owners_json,field_path,role,visibility,authored_markup_sha256,html_gzip,interactions_json,metadata_json,preparation_context_hash FROM prepared_content WHERE record_id=? ORDER BY content_id")?;
        let mut fields = fields.query([id])?;
        while let Some(field) = fields.next()? {
            let locator = SourceContentLocator {
                record: record.key().clone(),
                owners: serde_json::from_str(&field.get::<_, String>(0)?)?,
                field: field.get(1)?,
            };
            if field.get::<_, String>(8)? != preparation_hash {
                return Err(IndexError::Invalid(
                    "persisted field preparation identity differs".into(),
                ));
            }
            let authored = record
                .authored_content_at(&locator)
                .value()
                .ok_or_else(|| {
                    IndexError::Invalid("persisted content field is absent from DTO".into())
                })?;
            if crate::codec::sha256(authored.as_bytes()) != field.get::<_, String>(4)? {
                return Err(IndexError::Invalid(
                    "persisted authored content hash differs".into(),
                ));
            }
            let metadata: PreparationMetadata = serde_json::from_str(&field.get::<_, String>(7)?)?;
            let status = match metadata.status {
                PreparationStatus::Prepared {
                    mut hidden_references,
                    diagnostics,
                } => {
                    let html = field
                        .get::<_, Option<Vec<u8>>>(5)?
                        .map(|v| crate::codec::gunzip(&v))
                        .transpose()?
                        .map(String::from_utf8)
                        .transpose()
                        .map_err(|e| IndexError::Invalid(e.to_string()))?
                        .unwrap_or_default();
                    let mut occurrences = references
                        .remove(&crate::read_content::content_reference_key(&locator)?)
                        .unwrap_or_default();
                    occurrences.append(&mut hidden_references);
                    occurrences.sort_by_key(|r| r.ordinal);
                    SourceContentStatus::Prepared(Box::new(PreparedSourceContent {
                        locator: locator.clone(),
                        interpretation_version: context.content_interpretation_version.clone(),
                        authored_markup_sha256: field.get(4)?,
                        text: atlas_record::source_content::prepared_plain_text(&html)
                            .map_err(|e| IndexError::Invalid(e.to_string()))?,
                        html,
                        references: occurrences,
                        interactions: serde_json::from_str(&field.get::<_, String>(6)?)?,
                        diagnostics,
                    }))
                }
                PreparationStatus::FormatUnavailable(availability) => {
                    SourceContentStatus::FormatUnavailable {
                        locator: locator.clone(),
                        availability,
                    }
                }
                PreparationStatus::UnsupportedFormat(format) => {
                    SourceContentStatus::UnsupportedFormat {
                        locator: locator.clone(),
                        format,
                    }
                }
                PreparationStatus::PreparationFailed(message) => {
                    SourceContentStatus::PreparationFailed {
                        locator: locator.clone(),
                        message,
                    }
                }
            };
            content.push(SourceContentOutcome {
                role: serde_json::from_value(serde_json::Value::String(field.get(2)?))?,
                visibility: serde_json::from_value(serde_json::Value::String(field.get(3)?))?,
                visibility_availability: metadata.visibility_availability,
                status,
            });
        }
        if !references.is_empty() {
            return Err(IndexError::Invalid(
                "persisted content references lack field metadata".into(),
            ));
        }
        let mut relationships = Vec::new();
        let mut edges = c.prepare(&format!(
            "{} WHERE o.record_id=? AND o.origin='structured' ORDER BY o.ordinal",
            crate::read_content::RELATIONSHIP_SELECT
        ))?;
        for edge in edges.query_map([id], crate::read_content::relationship_row)? {
            let edge = edge?;
            relationships.push(SourceRelationshipOccurrence {
                locator: edge.locator,
                kind: serde_json::from_value(serde_json::Value::String(edge.kind))?,
                availability: edge
                    .details
                    .availability
                    .ok_or_else(|| IndexError::Invalid("structured availability missing".into()))?,
                authored_target: edge.authored_target,
                resolution: edge.resolution,
            });
        }
        let mut diagnostics = Vec::new();
        let mut ds=c.prepare("SELECT owners_json,field_path,stage,details_json FROM developer_diagnostics WHERE record_id=? AND stage='admission' ORDER BY diagnostic_id")?;
        let mut ds = ds.query([id])?;
        while let Some(d) = ds.next()? {
            let details: crate::read_content::DiagnosticDetails =
                serde_json::from_str(&d.get::<_, String>(3)?)?;
            diagnostics.push(SourceArtifactDiagnostic {
                owners: d
                    .get::<_, Option<String>>(0)?
                    .map(|s| serde_json::from_str(&s))
                    .transpose()?,
                field: d.get(1)?,
                stage: d.get(2)?,
                code: details.code,
                message: details.message,
                source_diagnostic: details.source_diagnostic,
            });
        }
        inputs.push(SourceArtifactRecordInput {
            record,
            source_path: row.get(3)?,
            content_hash: row.get(4)?,
            preparation_context_hash: preparation_hash.clone(),
            content,
            relationships,
            diagnostics,
        });
    }
    Ok(inputs)
}
