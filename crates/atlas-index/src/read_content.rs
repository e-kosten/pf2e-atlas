//! Bounded attributed content/edge reads; neither operation decodes a source body.
use crate::{
    IndexError, SourceArtifactDiagnostic, SourcePreparedContent, SourceRelationshipDetails,
    SourceStoredRelationship, SqliteIndexReader,
};
use atlas_domain::{RecordKey, SourcePassageAddress};
use atlas_record::source_content::{
    ContentInteraction, ContentReferenceOccurrence, ContentReferenceResolution,
    ContentReferenceTarget, ContentVisibilityRule, OwnedContentLocator, SourceContentLocator,
};
use rusqlite::{params_from_iter, types::Value};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(crate) type ContentReferenceKey = (String, String, String);
pub(crate) fn content_reference_key(
    locator: &SourceContentLocator,
) -> Result<ContentReferenceKey, IndexError> {
    Ok((
        locator.record.to_string(),
        serde_json::to_string(&locator.owners)?,
        locator.field.clone(),
    ))
}
pub(crate) fn content_reference(
    edge: SourceStoredRelationship,
) -> Result<ContentReferenceOccurrence, IndexError> {
    if edge.origin != "content" || edge.details.availability.is_some() {
        return Err(IndexError::Invalid(
            "invalid content reference attribution".into(),
        ));
    }
    let required = |value: Option<String>| {
        value.ok_or_else(|| {
            IndexError::Invalid("content reference lacks authored target/path".into())
        })
    };
    let kind = edge
        .details
        .reference_kind
        .ok_or_else(|| IndexError::Invalid("content reference lacks kind".into()))?;
    let expected_kind = match &kind {
        atlas_record::source_content::ContentReferenceKind::Uuid => "uuid",
        atlas_record::source_content::ContentReferenceKind::Compendium => "compendium",
        atlas_record::source_content::ContentReferenceKind::Embed { .. } => "embed",
        atlas_record::source_content::ContentReferenceKind::HtmlLink => "html_link",
    };
    if edge.kind != expected_kind {
        return Err(IndexError::Invalid(
            "content reference kind mismatch".into(),
        ));
    }
    Ok(ContentReferenceOccurrence {
        ordinal: edge.ordinal,
        path: required(edge.occurrence_path)?,
        kind,
        authored_target: required(edge.authored_target)?,
        resolution: edge.resolution,
        visible: true,
        audiences: edge.details.audiences,
    })
}
pub(crate) fn content_references_for_record(
    c: &rusqlite::Connection,
    id: i64,
) -> Result<BTreeMap<ContentReferenceKey, Vec<ContentReferenceOccurrence>>, IndexError> {
    let mut statement = c.prepare(&format!(
        "{RELATIONSHIP_SELECT} WHERE o.record_id=? AND o.origin='content'"
    ))?;
    let mut grouped = BTreeMap::<ContentReferenceKey, Vec<ContentReferenceOccurrence>>::new();
    for edge in statement.query_map([id], relationship_row)? {
        let edge = edge?;
        grouped
            .entry(content_reference_key(&edge.locator)?)
            .or_default()
            .push(content_reference(edge)?);
    }
    Ok(grouped)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceContentBundle {
    pub fields: Vec<SourcePreparedContent>,
    pub missing: Vec<SourceContentLocator>,
    pub truncated: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceRelationshipDirection {
    Outgoing,
    Incoming,
}
#[derive(Debug, Clone)]
pub struct SourceRelationshipRequest {
    pub record: RecordKey,
    pub owners: Option<Vec<OwnedContentLocator>>,
    pub field: Option<String>,
    pub direction: SourceRelationshipDirection,
    pub limit: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRelationshipBundle {
    pub occurrences: Vec<SourceStoredRelationship>,
    pub truncated: bool,
}
const CONTENT_SELECT: &str = "SELECT r.key,c.owners_json,c.field_path,c.role,c.visibility,c.outcome,c.html_gzip,c.interactions_json,c.authored_markup_sha256 FROM prepared_content c JOIN records r USING(record_id) WHERE r.document_kind<>'Macro'";
fn decode_error(e: impl std::error::Error + Send + Sync + 'static) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
}
fn content_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<(SourcePreparedContent, String)> {
    let key: String = r.get(0)?;
    let chain: String = r.get(1)?;
    let visibility: String = r.get(4)?;
    let bytes: Option<Vec<u8>> = r.get(6)?;
    let controls: String = r.get(7)?;
    let html = bytes
        .map(|b| {
            crate::codec::gunzip(&b)
                .map_err(decode_error)
                .and_then(|b| String::from_utf8(b).map_err(decode_error))
        })
        .transpose()?;
    let interactions: Vec<ContentInteraction> =
        serde_json::from_str(&controls).map_err(decode_error)?;
    if let Some(html) = &html {
        atlas_record::source_content::validate_prepared_markers(html, &interactions)
            .map_err(decode_error)?;
    }
    Ok((
        SourcePreparedContent {
            locator: SourceContentLocator {
                record: RecordKey::parse(&key).map_err(decode_error)?,
                owners: serde_json::from_str(&chain).map_err(decode_error)?,
                field: r.get(2)?,
            },
            role: r.get(3)?,
            visibility: serde_json::from_str::<ContentVisibilityRule>(&format!("\"{visibility}\""))
                .map_err(decode_error)?,
            outcome: r.get(5)?,
            html,
            interactions,
        },
        r.get(8)?,
    ))
}
pub(crate) const RELATIONSHIP_SELECT: &str = "SELECT r.key,o.owners_json,o.field_path,o.ordinal,o.origin,o.kind,o.authored_target,o.occurrence_path,o.resolution,t.key,o.target_owners_json,o.target_url,o.details_json FROM relationship_occurrences o JOIN records r ON r.record_id=o.record_id LEFT JOIN records t ON t.record_id=o.target_record_id";
pub(crate) fn relationship_row(
    r: &rusqlite::Row<'_>,
) -> rusqlite::Result<SourceStoredRelationship> {
    let key: String = r.get(0)?;
    let owners: String = r.get(1)?;
    let status: String = r.get(8)?;
    let target: Option<String> = r.get(9)?;
    let chain: Option<String> = r.get(10)?;
    let url: Option<String> = r.get(11)?;
    let resolution = match status.as_str() {
        "resolved_record" => ContentReferenceResolution::Resolved(ContentReferenceTarget::Record {
            key: RecordKey::parse(
                &target.ok_or_else(|| decode_error(std::io::Error::other("missing target")))?,
            )
            .map_err(decode_error)?,
        }),
        "resolved_owned" => {
            ContentReferenceResolution::Resolved(ContentReferenceTarget::OwnedNode {
                key: RecordKey::parse(
                    &target.ok_or_else(|| decode_error(std::io::Error::other("missing target")))?,
                )
                .map_err(decode_error)?,
                owners: serde_json::from_str(
                    &chain.ok_or_else(|| {
                        decode_error(std::io::Error::other("missing target owners"))
                    })?,
                )
                .map_err(decode_error)?,
            })
        }
        "resolved_url" => ContentReferenceResolution::Resolved(ContentReferenceTarget::Url {
            url: url.ok_or_else(|| decode_error(std::io::Error::other("missing target URL")))?,
        }),
        "unverified_url" => ContentReferenceResolution::UnverifiedUrl {
            url: url.ok_or_else(|| decode_error(std::io::Error::other("missing URL")))?,
        },
        "unresolved" => ContentReferenceResolution::Unresolved,
        "blocked" => ContentReferenceResolution::Blocked,
        _ => return Err(decode_error(std::io::Error::other("unknown resolution"))),
    };
    Ok(SourceStoredRelationship {
        locator: SourceContentLocator {
            record: RecordKey::parse(&key).map_err(decode_error)?,
            owners: serde_json::from_str(&owners).map_err(decode_error)?,
            field: r.get(2)?,
        },
        ordinal: r.get(3)?,
        origin: r.get(4)?,
        kind: r.get(5)?,
        authored_target: r.get(6)?,
        occurrence_path: r.get(7)?,
        resolution,
        details: serde_json::from_str::<SourceRelationshipDetails>(&r.get::<_, String>(12)?)
            .map_err(decode_error)?,
    })
}
impl SqliteIndexReader {
    fn checked_content_fields(
        &self,
        c: &rusqlite::Connection,
        fields: Vec<(SourcePreparedContent, String)>,
    ) -> Result<Vec<SourcePreparedContent>, IndexError> {
        if fields.is_empty() {
            return Ok(vec![]);
        }
        let clauses =
            vec!["(r.key=? AND o.owners_json=? AND o.field_path=?)"; fields.len()].join(" OR ");
        let mut values = vec![];
        for (field, _) in &fields {
            let (key, owners, path) = content_reference_key(&field.locator)?;
            values.extend([Value::Text(key), Value::Text(owners), Value::Text(path)]);
        }
        self.count_statement();
        let mut statement = c.prepare(&format!(
            "{RELATIONSHIP_SELECT} WHERE o.origin='content' AND ({clauses})"
        ))?;
        let mut references =
            BTreeMap::<ContentReferenceKey, Vec<ContentReferenceOccurrence>>::new();
        for edge in statement.query_map(params_from_iter(values), relationship_row)? {
            let edge = edge?;
            references
                .entry(content_reference_key(&edge.locator)?)
                .or_default()
                .push(content_reference(edge)?);
        }
        let mut checked = Vec::with_capacity(fields.len());
        for (field, hash) in fields {
            let occurrences = references
                .remove(&content_reference_key(&field.locator)?)
                .unwrap_or_default();
            if let Some(html) = &field.html {
                atlas_record::source_content::validate_prepared_reference_bindings(
                    html,
                    &field.locator,
                    &hash,
                    &occurrences,
                )
                .map_err(|e| IndexError::Invalid(e.to_string()))?;
            } else if !occurrences.is_empty() {
                return Err(IndexError::Invalid(
                    "content references lack rendered field".into(),
                ));
            }
            checked.push(field);
        }
        Ok(checked)
    }
    pub fn read_content(
        &self,
        locators: &[SourceContentLocator],
    ) -> Result<SourceContentBundle, IndexError> {
        if locators.len() > 256 {
            return Err(IndexError::Invalid(
                "selected content batch exceeds 256 fields".into(),
            ));
        }
        if locators.is_empty() {
            return Ok(SourceContentBundle {
                fields: vec![],
                missing: vec![],
                truncated: false,
            });
        }
        let clauses =
            vec!["(r.key=? AND c.owners_json=? AND c.field_path=?)"; locators.len()].join(" OR ");
        let sql = format!("{CONTENT_SELECT} AND ({clauses})");
        let mut params = vec![];
        for l in locators {
            params.push(Value::Text(l.record.to_string()));
            params.push(Value::Text(serde_json::to_string(&l.owners)?));
            params.push(Value::Text(l.field.clone()));
        }
        let c = self.native()?;
        self.count_statement();
        let mut statement = c.prepare(&sql)?;
        #[cfg(any(test, feature = "test-support"))]
        self.count_content_batch();
        let fields = statement
            .query_map(params_from_iter(params), content_row)?
            .collect::<Result<Vec<_>, _>>()?;
        let mut fields = self.checked_content_fields(c, fields)?;
        let missing = locators
            .iter()
            .filter(|l| !fields.iter().any(|f| &f.locator == *l))
            .cloned()
            .collect();
        fields.sort_by_key(|f| {
            locators
                .iter()
                .position(|l| l == &f.locator)
                .unwrap_or(usize::MAX)
        });
        Ok(SourceContentBundle {
            fields,
            missing,
            truncated: false,
        })
    }
    pub fn read_content_for_record(
        &self,
        key: &RecordKey,
        limit: usize,
    ) -> Result<SourceContentBundle, IndexError> {
        if !(1..=256).contains(&limit) {
            return Err(IndexError::Invalid(
                "record content limit must be 1..=256".into(),
            ));
        }
        let c = self.native()?;
        self.count_statement();
        let mut s = c.prepare(&format!(
            "{CONTENT_SELECT} AND r.key=? ORDER BY c.content_id LIMIT ?"
        ))?;
        let mut fields = s
            .query_map(rusqlite::params![key.to_string(), limit + 1], content_row)?
            .collect::<Result<Vec<_>, _>>()?;
        let truncated = fields.len() > limit;
        fields.truncate(limit);
        let fields = self.checked_content_fields(c, fields)?;
        Ok(SourceContentBundle {
            fields,
            missing: vec![],
            truncated,
        })
    }
    pub fn read_relationships(
        &self,
        request: &SourceRelationshipRequest,
    ) -> Result<SourceRelationshipBundle, IndexError> {
        if !(1..=1024).contains(&request.limit) {
            return Err(IndexError::Invalid(
                "relationship limit must be 1..=1024".into(),
            ));
        }
        let mut params = vec![Value::Text(request.record.to_string())];
        let mut sql = format!(
            "{RELATIONSHIP_SELECT} WHERE r.document_kind<>'Macro' AND (t.document_kind IS NULL OR t.document_kind<>'Macro') AND ",
        );
        sql.push_str(match request.direction {
            SourceRelationshipDirection::Outgoing => "r.key=?",
            SourceRelationshipDirection::Incoming => "t.key=?",
        });
        if let Some(chain) = &request.owners {
            sql.push_str(match request.direction {
                SourceRelationshipDirection::Outgoing => " AND o.owners_json=?",
                SourceRelationshipDirection::Incoming => {
                    " AND COALESCE(o.target_owners_json,'[]')=?"
                }
            });
            params.push(Value::Text(serde_json::to_string(chain)?));
        }
        if let Some(field) = &request.field {
            if request.direction == SourceRelationshipDirection::Incoming {
                return Err(IndexError::Invalid(
                    "incoming edges have no target field selector".into(),
                ));
            }
            sql.push_str(" AND o.field_path=?");
            params.push(Value::Text(field.clone()));
        }
        sql.push_str(" ORDER BY o.occurrence_id LIMIT ?");
        params.push(Value::Integer((request.limit + 1) as i64));
        let c = self.native()?;
        self.count_statement();
        let mut s = c.prepare(&sql)?;
        let mut occurrences = s
            .query_map(params_from_iter(params), relationship_row)?
            .collect::<Result<Vec<_>, _>>()?;
        let truncated = occurrences.len() > request.limit;
        occurrences.truncate(request.limit);
        Ok(SourceRelationshipBundle {
            occurrences,
            truncated,
        })
    }
    pub fn recover_passage(
        &self,
        location: &crate::SourceUnitLocation,
    ) -> Result<Option<String>, IndexError> {
        let Some(address) = &location.address else {
            return Ok(None);
        };
        if matches!(address, SourcePassageAddress::Identity {}) {
            return Ok(None);
        };
        let field = location
            .field
            .clone()
            .ok_or_else(|| IndexError::Invalid("passage has no field".into()))?;
        let locator = SourceContentLocator {
            record: location.record.clone(),
            owners: location.owners.clone(),
            field,
        };
        let content = self.read_content(std::slice::from_ref(&locator))?;
        let html = content.fields.first().and_then(|f| f.html.as_deref());
        let record = if matches!(address, SourcePassageAddress::PlainSection { .. }) {
            self.read_source_record(&location.record)?
        } else {
            None
        };
        let text = record
            .as_ref()
            .and_then(|r| r.authored_content_at(&locator).value());
        Ok(Some(
            atlas_record::source_record::recover_source_passage(html, text, address)
                .map_err(|e| IndexError::Invalid(e.to_string()))?,
        ))
    }
    pub fn field_counts(
        &self,
        request: &crate::QueryCountsRequest,
    ) -> Result<atlas_domain::QueryFieldCounts, IndexError> {
        crate::discovery::field_counts(self.native()?, request)
    }
    pub fn field_values(
        &self,
        request: &crate::QueryValuesRequest,
    ) -> Result<atlas_domain::QueryValueOptions, IndexError> {
        crate::discovery::field_values(self.native()?, request)
    }
    pub fn inspect_diagnostics(
        &self,
        key: &RecordKey,
        limit: usize,
    ) -> Result<Vec<SourceArtifactDiagnostic>, IndexError> {
        if !(1..=4096).contains(&limit) {
            return Err(IndexError::Invalid(
                "diagnostic limit must be 1..=4096".into(),
            ));
        }
        let c = self.native()?;
        self.count_statement();
        let mut s=c.prepare("SELECT d.owners_json,d.field_path,d.stage,d.details_json FROM developer_diagnostics d JOIN records r USING(record_id) WHERE r.key=? ORDER BY diagnostic_id LIMIT ?")?;
        Ok(s.query_map(rusqlite::params![key.to_string(), limit], |r| {
            let details: String = r.get(3)?;
            let details: DiagnosticDetails =
                serde_json::from_str(&details).map_err(decode_error)?;
            let owners: Option<String> = r.get(0)?;
            Ok(SourceArtifactDiagnostic {
                owners: owners
                    .map(|s| serde_json::from_str::<Vec<OwnedContentLocator>>(&s))
                    .transpose()
                    .map_err(decode_error)?,
                field: r.get(1)?,
                stage: r.get(2)?,
                code: details.code,
                message: details.message,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?)
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DiagnosticDetails {
    pub code: String,
    pub message: String,
}
