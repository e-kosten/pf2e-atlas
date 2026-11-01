//! Canonical selected source sections, shared by reading, lexical and semantic units.
use crate::{IndexError, SourceArtifactBuildContext};
use atlas_domain::{
    SIMILARITY_ROLE_RECIPE, SimilarityPart, SimilarityPartSelector, SourceByteRange,
    SourcePassageAddress,
};
use atlas_record::source_record::{
    SourceBackedRecord, SourceSearchSelection, similarity_content_role,
};
use rusqlite::{Connection, OptionalExtension, Row, params};

mod migration;
pub use migration::backfill_selected_sections;
pub const SELECTED_SECTIONS_RECIPE: &str = "canonical-selected-sections/v1";
pub const SELECTED_SECTIONS_SCHEMA_SQL: &str = include_str!("selected_sections/schema.sql");
pub(crate) const COLUMNS: &str = "ss.section_id,ss.record_id,ss.selector,ss.owners_json,ss.field_path,ss.section_ordinal,ss.body_hash,ss.address_kind,ss.prepared_html_sha256,ss.selection_version,ss.section_label,ss.byte_length,ss.role,ss.owner_label,ss.recipe";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectedSection {
    pub id: i64,
    pub record_id: i64,
    pub part: SimilarityPart,
    pub address: SourcePassageAddress,
}
fn invalid(message: &str) -> IndexError {
    IndexError::Invalid(message.into())
}
fn sql_error(error: impl std::error::Error + Send + Sync + 'static) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}
fn label(field: &str, authored: Option<&str>, role: atlas_domain::SimilarityRole) -> String {
    authored.map(str::to_owned).unwrap_or_else(|| match field {
        "/system/details/publicNotes" => "Public notes".into(),
        "/system/details/privateNotes" => "Private notes".into(),
        "/system/details/blurb" => "Blurb".into(),
        _ => role.label().into(),
    })
}
impl SelectedSection {
    pub(crate) fn read(row: &Row<'_>, offset: usize) -> rusqlite::Result<Self> {
        let recipe: String = row.get(offset + 14)?;
        if recipe != SIMILARITY_ROLE_RECIPE {
            return Err(sql_error(invalid("selected section role recipe differs")));
        }
        let role = serde_json::from_value(serde_json::Value::String(row.get(offset + 12)?))
            .map_err(sql_error)?;
        let field: String = row.get(offset + 4)?;
        let ordinal = row.get(offset + 5)?;
        let authored_label: Option<String> = row.get(offset + 10)?;
        let address = address(
            row.get(offset + 7)?,
            row.get(offset + 8)?,
            row.get(offset + 6)?,
            row.get(offset + 9)?,
            ordinal,
            authored_label.clone(),
            SourceByteRange {
                start: 0,
                end: row.get(offset + 11)?,
            },
        )
        .map_err(sql_error)?;
        Ok(Self {
            id: row.get(offset)?,
            record_id: row.get(offset + 1)?,
            part: SimilarityPart {
                selector: SimilarityPartSelector(row.get(offset + 2)?),
                owners: serde_json::from_str(&row.get::<_, String>(offset + 3)?)
                    .map_err(sql_error)?,
                label: label(&field, authored_label.as_deref(), role),
                owner_label: row.get(offset + 13)?,
                field,
                section_ordinal: ordinal,
                role,
                has_vectors: false,
                label_only: false,
            },
            address,
        })
    }
    pub(crate) fn chunk_address(
        &self,
        range: SourceByteRange,
    ) -> Result<SourcePassageAddress, IndexError> {
        let mut address = self.address.clone();
        let whole = range_mut(&mut address)?;
        if range.start > range.end || range.end > whole.end {
            return Err(invalid("selected section chunk exceeds whole extent"));
        }
        *whole = range;
        Ok(address)
    }
}
pub(crate) fn range_mut(
    address: &mut SourcePassageAddress,
) -> Result<&mut SourceByteRange, IndexError> {
    match address {
        SourcePassageAddress::HtmlSection { chunk_bytes, .. }
        | SourcePassageAddress::PlainSection { chunk_bytes, .. } => Ok(chunk_bytes),
        SourcePassageAddress::Identity {} => Err(invalid("identity has no section extent")),
    }
}
pub(crate) fn address(
    kind: String,
    html: Option<String>,
    body: String,
    version: String,
    ordinal: usize,
    label: Option<String>,
    range: SourceByteRange,
) -> Result<SourcePassageAddress, IndexError> {
    if version != atlas_record::source_record::SOURCE_CONTENT_SELECTION_VERSION
        || !crate::codec::checked_hash(&body)
        || range.start >= range.end
    {
        return Err(invalid("selected section address identity is invalid"));
    }
    match (kind.as_str(), html) {
        ("html", Some(html)) if crate::codec::checked_hash(&html) => {
            Ok(SourcePassageAddress::HtmlSection {
                prepared_html_sha256: html,
                canonical_text_sha256: body,
                selection_version: version,
                section_ordinal: ordinal,
                label,
                chunk_bytes: range,
            })
        }
        ("plain", None) => Ok(SourcePassageAddress::PlainSection {
            source_text_sha256: body,
            selection_version: version,
            section_ordinal: ordinal,
            label,
            chunk_bytes: range,
        }),
        _ => Err(invalid("selected section format/hash disagree")),
    }
}
pub(crate) fn project(
    record: &SourceBackedRecord,
    selection: &SourceSearchSelection,
    content_hash: &str,
    preparation_hash: &str,
) -> Result<Vec<SelectedSection>, IndexError> {
    let mut output = Vec::new();
    for field in &selection.fields {
        let role = similarity_content_role(record, &field.locator, field.role)
            .ok_or_else(|| invalid("selected section owner is unavailable"))?;
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
            output.push(SelectedSection {
                id: 0,
                record_id: 0,
                part: SimilarityPart {
                    selector,
                    label: label(&field.locator.field, section.label.as_deref(), role),
                    owner_label: owner_label.clone(),
                    owners: field.locator.owners.clone(),
                    field: field.locator.field.clone(),
                    section_ordinal: section.section_ordinal,
                    role,
                    has_vectors: false,
                    label_only: false,
                },
                address: field.section_address(section),
            });
        }
    }
    Ok(output)
}
pub(crate) fn upsert(
    c: &Connection,
    id: i64,
    section: &SelectedSection,
) -> Result<i64, IndexError> {
    let (kind, html, body, version, ordinal, authored, range) = match &section.address {
        SourcePassageAddress::HtmlSection {
            prepared_html_sha256,
            canonical_text_sha256,
            selection_version,
            section_ordinal,
            label,
            chunk_bytes,
        } => (
            "html",
            Some(prepared_html_sha256.as_str()),
            canonical_text_sha256.as_str(),
            selection_version.as_str(),
            *section_ordinal,
            label,
            *chunk_bytes,
        ),
        SourcePassageAddress::PlainSection {
            source_text_sha256,
            selection_version,
            section_ordinal,
            label,
            chunk_bytes,
        } => (
            "plain",
            None,
            source_text_sha256.as_str(),
            selection_version.as_str(),
            *section_ordinal,
            label,
            *chunk_bytes,
        ),
        SourcePassageAddress::Identity {} => {
            return Err(invalid("identity cannot be stored as a selected section"));
        }
    };
    if range.start != 0 {
        return Err(invalid("canonical section must have its whole extent"));
    }
    let owners = serde_json::to_string(&section.part.owners)?;
    let role = serde_json::to_value(section.part.role)?;
    c.execute("INSERT INTO selected_sections(record_id,selector,owners_json,field_path,section_ordinal,body_hash,address_kind,prepared_html_sha256,selection_version,section_label,byte_length,role,owner_label,recipe) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?) ON CONFLICT(record_id,owners_json,field_path,section_ordinal) DO UPDATE SET selector=excluded.selector,body_hash=excluded.body_hash,address_kind=excluded.address_kind,prepared_html_sha256=excluded.prepared_html_sha256,selection_version=excluded.selection_version,section_label=excluded.section_label,byte_length=excluded.byte_length,role=excluded.role,owner_label=excluded.owner_label,recipe=excluded.recipe WHERE selected_sections.selector IS NOT excluded.selector OR selected_sections.body_hash IS NOT excluded.body_hash OR selected_sections.address_kind IS NOT excluded.address_kind OR selected_sections.prepared_html_sha256 IS NOT excluded.prepared_html_sha256 OR selected_sections.selection_version IS NOT excluded.selection_version OR selected_sections.section_label IS NOT excluded.section_label OR selected_sections.byte_length IS NOT excluded.byte_length OR selected_sections.role IS NOT excluded.role OR selected_sections.owner_label IS NOT excluded.owner_label OR selected_sections.recipe IS NOT excluded.recipe",params![id,section.part.selector.0,owners,section.part.field,ordinal,body,kind,html,version,authored,range.end,role.as_str(),section.part.owner_label,SIMILARITY_ROLE_RECIPE])?;
    Ok(c.query_row("SELECT section_id FROM selected_sections WHERE record_id=? AND owners_json=? AND field_path=? AND section_ordinal=?",params![id,owners,section.part.field,ordinal],|r|r.get(0))?)
}
pub(crate) fn find(
    c: &Connection,
    record: i64,
    owners: &str,
    field: &str,
    ordinal: usize,
) -> Result<SelectedSection, IndexError> {
    c.query_row(&format!("SELECT {COLUMNS} FROM selected_sections ss WHERE record_id=? AND owners_json=? AND field_path=? AND section_ordinal=?"),params![record,owners,field,ordinal],|row|SelectedSection::read(row,0)).optional()?.ok_or_else(||invalid("unit belongs to an unavailable selected section"))
}
pub(crate) fn validate_record(
    c: &Connection,
    id: i64,
    record: &SourceBackedRecord,
    context: &SourceArtifactBuildContext,
    selection: &SourceSearchSelection,
) -> Result<(), IndexError> {
    let hash: String = c.query_row(
        "SELECT content_hash FROM records WHERE record_id=?",
        [id],
        |r| r.get(0),
    )?;
    let mut expected = project(record, selection, &hash, &context.preparation_sha256()?)?
        .into_iter()
        .map(|row| (row.part.selector.clone(), row))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut statement = c.prepare(&format!(
        "SELECT {COLUMNS} FROM selected_sections ss WHERE record_id=?"
    ))?;
    let mut rows = statement.query([id])?;
    while let Some(row) = rows.next()? {
        let mut actual = SelectedSection::read(row, 0)?;
        actual.id = 0;
        actual.record_id = 0;
        if expected.remove(&actual.part.selector).as_ref() != Some(&actual) {
            return Err(invalid(
                "selected section differs from checked source, whole extent or role policy",
            ));
        }
    }
    if !expected.is_empty() {
        return Err(invalid("canonical selected section coverage is incomplete"));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
