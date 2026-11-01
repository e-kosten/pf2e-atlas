//! Metadata-only section projection rebuilt from retained checked records.
use crate::{IndexError, SourceArtifactBuildContext, SourceArtifactRecordInput};
use atlas_domain::{SIMILARITY_ROLE_RECIPE, SimilarityPart, SimilarityPartSelector};
use atlas_record::source_record::{
    SourceBackedRecord, SourceSearchSelection, select_record_search, similarity_content_role,
};
use rusqlite::{Connection, params};

pub const SIMILARITY_SCHEMA_SQL: &str = "
CREATE TABLE similarity_parts(
 selector TEXT PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES records ON DELETE CASCADE,
 owners_json TEXT NOT NULL CHECK(json_valid(owners_json)),
 field_path TEXT NOT NULL,
 section_ordinal INTEGER NOT NULL,
 body_hash TEXT NOT NULL,
 role TEXT NOT NULL,
 metadata_json TEXT NOT NULL CHECK(json_valid(metadata_json)),
 recipe TEXT NOT NULL,
 UNIQUE(record_id,owners_json,field_path,section_ordinal)
) STRICT;
CREATE INDEX similarity_parts_role ON similarity_parts(role,record_id);
";

pub fn backfill_similarity_parts(c: &Connection) -> atlas_storage::StorageResult<()> {
    backfill(c).map_err(|error| atlas_storage::StorageError::Migration(error.to_string()))
}

fn backfill(c: &Connection) -> Result<(), IndexError> {
    let Some(context) = crate::read_stored_reference_context(c)? else {
        return Ok(());
    };
    if context.snapshot_version != atlas_foundry_model::SNAPSHOT_VERSION
        || context.source_contract != atlas_foundry_model::SOURCE_CONTRACT_ID
        || context.content_selection_version
            != atlas_record::source_record::SOURCE_CONTENT_SELECTION_VERSION
        || context.content_interpretation_version
            != atlas_record::source_content::CONTENT_INTERPRETATION_VERSION
    {
        // Existing setup admission owns replacement of retired checked codecs.
        return Ok(());
    }
    if matches!(
        crate::classify_reference_upgrade(c)?,
        crate::ReferenceUpgrade::SourcePreparation | crate::ReferenceUpgrade::SourceReplacement
    ) {
        // Setup owns replacing retired preparation outputs (localization, assets,
        // authored relationships, lookup, content selection and trait policy).
        // Reuse its classifier before reading any record rows or checked bodies.
        return Ok(());
    }
    let mut after = 0i64;
    loop {
        let roots = c.prepare("SELECT record_id,key FROM records WHERE record_id>? AND document_kind<>'Macro' ORDER BY record_id LIMIT 128")?
            .query_map([after], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        if roots.is_empty() {
            break;
        }
        for (id, key) in roots {
            after = id;
            let key = key.parse().map_err(|error| {
                IndexError::Invalid(format!("invalid similarity record key: {error}"))
            })?;
            let source = crate::persisted_inputs::read_persisted_input(c, &context, &key)?
                .ok_or_else(|| {
                    IndexError::Invalid("similarity backfill lost its checked source".into())
                })?;
            write_record_parts(c, &context, &source)?;
        }
    }
    Ok(())
}

#[cfg(any(test, feature = "test-support"))]
pub(crate) fn write_similarity_parts(
    c: &Connection,
    input: &crate::IndexBuildInput,
) -> Result<(), IndexError> {
    for source in &input.records {
        write_record_parts(c, &input.context, source)?;
    }
    Ok(())
}

fn write_record_parts(
    c: &Connection,
    context: &SourceArtifactBuildContext,
    source: &SourceArtifactRecordInput,
) -> Result<(), IndexError> {
    let (record_id, pack_label): (i64, String) = c.query_row(
        "SELECT r.record_id,p.label FROM records r JOIN packs p USING(pack_id) WHERE r.key=?",
        [source.record.key().to_string()],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    c.execute(
        "DELETE FROM similarity_parts WHERE record_id=?",
        [record_id],
    )?;
    let selection = select_record_search(
        &source.record,
        &source.content,
        context.audience,
        &pack_label,
        &context.used_trait_labels,
    )
    .map_err(|error| IndexError::Invalid(error.to_string()))?;
    for (part, body_hash) in project_parts(
        &source.record,
        &selection,
        &source.content_hash,
        &source.preparation_context_hash,
    )? {
        c.execute(
            "INSERT INTO similarity_parts VALUES(?,?,?,?,?,?,?,?,?)",
            params![
                part.selector.0,
                record_id,
                serde_json::to_string(&part.owners)?,
                part.field,
                part.section_ordinal,
                body_hash,
                serde_json::to_value(part.role)?.as_str(),
                serde_json::to_string(&part)?,
                SIMILARITY_ROLE_RECIPE
            ],
        )?;
    }
    Ok(())
}

fn project_parts(
    record: &SourceBackedRecord,
    selection: &SourceSearchSelection,
    content_hash: &str,
    preparation_hash: &str,
) -> Result<Vec<(SimilarityPart, String)>, IndexError> {
    let mut projected = Vec::new();
    for field in &selection.fields {
        let role = similarity_content_role(record, &field.locator, field.role)
            .ok_or_else(|| IndexError::Invalid("similarity field owner is unavailable".into()))?;
        let owner_label = record
            .node_at(&field.locator.owners)
            .and_then(|node| node.name().value().cloned())
            .unwrap_or_else(|| record.key().to_string());
        for section in &field.sections {
            let body_hash = crate::codec::sha256(section.text.as_bytes());
            let selector = SimilarityPartSelector(format!(
                "part:{}",
                crate::codec::sha256(&serde_json::to_vec(&(
                    record.key(),
                    &field.locator.owners,
                    &field.locator.field,
                    section.section_ordinal,
                    content_hash,
                    preparation_hash,
                    &body_hash
                ))?)
            ));
            let label =
                section
                    .label
                    .clone()
                    .unwrap_or_else(|| match field.locator.field.as_str() {
                        "/system/details/publicNotes" => "Public notes".into(),
                        "/system/details/privateNotes" => "Private notes".into(),
                        "/system/details/blurb" => "Blurb".into(),
                        _ => role.label().into(),
                    });
            let part = SimilarityPart {
                selector,
                label,
                owner_label: owner_label.clone(),
                owners: field.locator.owners.clone(),
                field: field.locator.field.clone(),
                section_ordinal: section.section_ordinal,
                role,
                has_vectors: false,
                label_only: false,
            };
            projected.push((part, body_hash));
        }
    }
    Ok(projected)
}
