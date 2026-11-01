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
    pub(crate) fn is_prepared(&self) -> bool {
        matches!(self.status, PreparationStatus::Prepared { .. })
    }

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

/// Fixture-only eager collection; production maintenance visits bounded roots.
#[cfg(any(test, feature = "test-support"))]
pub fn collect_persisted_inputs_for_fixture(
    c: &Connection,
    context: &SourceArtifactBuildContext,
) -> Result<Vec<SourceArtifactRecordInput>, IndexError> {
    let mut inputs = Vec::new();
    crate::visit_persisted_inputs(c, context, &mut |input| {
        inputs.push(input);
        Ok(())
    })?;
    Ok(inputs)
}

/// Caller owns an authenticated workspace session. Query-catalog compatibility is
/// deliberately irrelevant: this also supplies projection-only backfills.
pub(crate) fn read_attested_persisted_input(
    c: &Connection,
    context: &SourceArtifactBuildContext,
    key: &atlas_domain::RecordKey,
) -> Result<Option<SourceArtifactRecordInput>, IndexError> {
    let Some(decoded) = decode_persisted_input(c, context, key)? else {
        return Ok(None);
    };
    crate::component_digests::validate_retained_payload(
        c,
        decoded.id,
        &decoded.source,
        &decoded.snapshot,
    )?;
    Ok(Some(decoded.source))
}
/// Sole owner: the immutable schema20 similarity backfill. Preserve its private
/// historical call name; all current readers use the attested entry above.
pub(crate) fn read_persisted_input(
    c: &Connection,
    context: &SourceArtifactBuildContext,
    key: &atlas_domain::RecordKey,
) -> Result<Option<SourceArtifactRecordInput>, IndexError> {
    let version: u32 = c.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let present: bool = c.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='reference_component_digests')",
        [],
        |r| r.get(0),
    )?;
    if version > 20 || present {
        return Err(IndexError::Invalid(
            "unattested payload entry is restricted to historical migration20".into(),
        ));
    }
    Ok(decode_persisted_input(c, context, key)?.map(|decoded| decoded.source))
}
/// Only schema27's historical callback can read before attestations exist. This
/// is private migration composition, never a current reader fallback.
pub(crate) fn read_pre_component_digest_migration_input(
    c: &Connection,
    context: &SourceArtifactBuildContext,
    key: &atlas_domain::RecordKey,
) -> Result<Option<SourceArtifactRecordInput>, IndexError> {
    let version: u32 = c.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let present: bool = c.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='reference_component_digests')",
        [],
        |r| r.get(0),
    )?;
    if version >= 28 || present {
        return Err(IndexError::Invalid(
            "pre-component payload decoder is restricted to historical migration27".into(),
        ));
    }
    Ok(decode_persisted_input(c, context, key)?.map(|decoded| decoded.source))
}
struct DecodedPersistedInput {
    id: i64,
    source: SourceArtifactRecordInput,
    snapshot: Vec<u8>,
}
fn decode_persisted_input(
    c: &Connection,
    context: &SourceArtifactBuildContext,
    key: &atlas_domain::RecordKey,
) -> Result<Option<DecodedPersistedInput>, IndexError> {
    if context.source_contract != atlas_foundry_model::SOURCE_CONTRACT_ID
        || context.snapshot_version != atlas_foundry_model::SNAPSHOT_VERSION
    {
        return Err(IndexError::Unsupported(
            "persisted source admission contract differs".into(),
        ));
    }
    let preparation_hash = context.preparation_sha256()?;
    let mut statement = c.prepare_cached(RECORD_INPUT_SQL)?;
    let mut rows = statement.query([key.to_string()])?;
    let Some(row) = rows.next()? else {
        return Ok(None);
    };
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
    let snapshot = crate::codec::gunzip(&row.get::<_, Vec<u8>>(7)?)?;
    let source = atlas_foundry_model::decode_snapshot(&snapshot)?;
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
    Ok(Some(DecodedPersistedInput {
        id,
        snapshot,
        source: SourceArtifactRecordInput {
            record,
            source_path: row.get(3)?,
            content_hash: row.get(4)?,
            preparation_context_hash: preparation_hash,
            content,
            relationships,
            diagnostics,
        },
    }))
}

// An optional-key OR predicate would prevent using the unique key index.
const RECORD_INPUT_SQL: &str = "SELECT r.record_id,r.key,r.pack_id,r.source_path,r.content_hash,b.codec_version,b.encoding,b.snapshot FROM records r JOIN record_bodies b USING(record_id) WHERE r.key=?1 ORDER BY r.record_id";

#[cfg(test)]
mod tests {
    #[test]
    fn selected_checked_input_uses_key_lookup_instead_of_a_corpus_scan() {
        let input = crate::foundation_tests::fixture(false);
        let (_directory, path) = crate::foundation_tests::build(&input);
        let connection = rusqlite::Connection::open(&path).unwrap();
        let plan = connection
            .prepare(&format!("EXPLAIN QUERY PLAN {}", super::RECORD_INPUT_SQL))
            .unwrap()
            .query_map([input.records[0].record.key().to_string()], |row| {
                row.get::<_, String>(3)
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(
            plan.iter()
                .any(|step| step.starts_with("SEARCH r ") && step.contains("key=?")),
            "{plan:?}"
        );
        assert!(
            !plan.iter().any(|step| step.starts_with("SCAN r")),
            "{plan:?}"
        );
        let expected =
            super::collect_persisted_inputs_for_fixture(&connection, &input.context).unwrap();
        let selected = super::read_attested_persisted_input(
            &connection,
            &input.context,
            input.records[0].record.key(),
        )
        .unwrap()
        .unwrap();
        assert_eq!(selected.record, expected[0].record);
        assert_eq!(selected.content, expected[0].content);
        let missing = "actors:zzzzzzzzzzzzzzzz".parse().unwrap();
        assert!(
            super::read_attested_persisted_input(&connection, &input.context, &missing)
                .unwrap()
                .is_none()
        );
    }
}
