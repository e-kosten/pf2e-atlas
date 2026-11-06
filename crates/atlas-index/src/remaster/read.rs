//! Bounded current-reference read projection.
use super::*;

fn bound(limit: usize) -> Result<(), IndexError> {
    if !(1..=100).contains(&limit) {
        Err(IndexError::Limit(
            "remaster information limit must be1..100".into(),
        ))
    } else {
        Ok(())
    }
}

/// Read each distinct declaration and source input once for the whole request.
/// Aliases can share declarations, and declarations can share a source capture.
fn hydrate(
    reader: &SqliteIndexReader,
    ids: &BTreeSet<String>,
) -> Result<BTreeMap<String, SourceRemasterDeclaration>, IndexError> {
    if ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let c = reader.native()?;
    let wanted = serde_json::to_string(ids)?;
    reader.count_statement();
    let mut query = c.prepare(
        "SELECT d.declaration_id,d.payload_json FROM json_each(?1) w \
         JOIN remaster_declarations d ON d.declaration_id=w.value ORDER BY d.declaration_id",
    )?;
    let mut declarations = BTreeMap::new();
    for row in query.query_map([&wanted], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })? {
        let (id, json) = row?;
        let declaration: RemasterDeclaration = serde_json::from_str(&json)?;
        if declaration.id != id {
            return Err(invalid("remaster declaration key differs"));
        }
        declarations.insert(id, declaration);
    }
    if declarations.len() != ids.len() {
        return Err(invalid("remaster support declaration is unavailable"));
    }

    reader.count_statement();
    let mut query = c.prepare(
        "SELECT b.declaration_id,b.payload_json FROM json_each(?1) w \
         JOIN remaster_bindings b ON b.declaration_id=w.value \
         ORDER BY b.declaration_id,b.side,b.group_ordinal,b.reference_ordinal",
    )?;
    let mut bindings = BTreeMap::<String, Vec<RemasterBinding>>::new();
    for row in query.query_map([&wanted], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })? {
        let (id, json) = row?;
        let binding: RemasterBinding = serde_json::from_str(&json)?;
        if binding.declaration != id {
            return Err(invalid("remaster binding declaration differs"));
        }
        bindings.entry(id).or_default().push(binding);
    }

    reader.count_statement();
    let mut query = c.prepare(
        "SELECT c.declaration_id,c.payload_json FROM json_each(?1) w \
         JOIN remaster_citations c ON c.declaration_id=w.value \
         ORDER BY c.declaration_id,c.ordinal",
    )?;
    let mut source_citations = BTreeMap::<String, Vec<RemasterCitation>>::new();
    for row in query.query_map([&wanted], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })? {
        let (id, json) = row?;
        source_citations
            .entry(id)
            .or_default()
            .push(serde_json::from_str(&json)?);
    }
    for (id, declaration) in &declarations {
        if source_citations
            .get(id)
            .map(Vec::as_slice)
            .unwrap_or_default()
            != declaration.citations
        {
            return Err(invalid("remaster citation projection differs"));
        }
    }
    let citations = source_citations.values().flatten().collect::<Vec<_>>();
    let selected_fields = selected_fields(reader, &citations)?;
    let inputs = captured_inputs(reader, &citations)?;
    // The reader's context was authenticated when this bounded snapshot opened.
    let context = reader.context();
    let revision = context
        .source_revision
        .as_ref()
        .unwrap_or(&context.source_fingerprint);
    declarations
        .into_iter()
        .map(|(id, declaration)| {
            let statement = statement(&declaration);
            let citations = source_citations
                .remove(&id)
                .unwrap_or_default()
                .into_iter()
                .map(|citation| match citation {
                    RemasterCitation::Journal {
                        locator,
                        authored_sha256,
                        ..
                    } => {
                        let name = selected_fields
                            .get(&field_key(&locator, &authored_sha256)?)
                            .ok_or_else(|| invalid("remaster citation field is unavailable"))?;
                        Ok(SourceRemasterCitation {
                            origin: RemasterOrigin::Foundry,
                            attribution: format!("{name} — {}", declaration.context.section),
                            revision: revision.clone(),
                            statement: statement.clone(),
                            locator: Some(locator),
                            path: None,
                        })
                    }
                    RemasterCitation::CapturedFile { input_key, .. } => {
                        let input = inputs
                            .get(&input_key)
                            .ok_or_else(|| invalid("remaster source input unavailable"))?;
                        Ok(SourceRemasterCitation {
                            origin: input.origin,
                            attribution: match input.origin {
                                RemasterOrigin::Foundry => format!("Foundry — {}", input.path),
                                RemasterOrigin::AtlasReviewed => {
                                    "Atlas-reviewed correspondence".into()
                                }
                            },
                            revision: input.revision.clone(),
                            statement: statement.clone(),
                            locator: None,
                            path: Some(input.path.clone()),
                        })
                    }
                })
                .collect::<Result<Vec<_>, IndexError>>()?;
            Ok((
                id.clone(),
                SourceRemasterDeclaration {
                    declaration,
                    bindings: bindings.remove(&id).unwrap_or_default(),
                    citations,
                },
            ))
        })
        .collect()
}

fn statement(declaration: &RemasterDeclaration) -> String {
    format!(
        "{}{}{}",
        declaration
            .before
            .iter()
            .map(|side| side.text.as_str())
            .collect::<Vec<_>>()
            .join("; "),
        if declaration.before.is_empty() {
            ""
        } else {
            " → "
        },
        declaration
            .after
            .iter()
            .map(|side| side.text.as_str())
            .collect::<Vec<_>>()
            .join("; "),
    )
}

type FieldKey = (String, String, String, String);
fn field_key(locator: &SourceContentLocator, hash: &str) -> Result<FieldKey, IndexError> {
    Ok((
        locator.record.to_string(),
        serde_json::to_string(&locator.owners)?,
        locator.field.clone(),
        hash.to_owned(),
    ))
}
fn selected_fields(
    reader: &SqliteIndexReader,
    citations: &[&RemasterCitation],
) -> Result<BTreeMap<FieldKey, String>, IndexError> {
    let wanted = citations
        .iter()
        .filter_map(|citation| match citation {
            RemasterCitation::Journal {
                locator,
                authored_sha256,
                ..
            } => Some(field_key(locator, authored_sha256)),
            _ => None,
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    if wanted.is_empty() {
        return Ok(BTreeMap::new());
    }
    reader.count_statement();
    let mut query = reader.native()?.prepare(
        "SELECT r.key,p.owners_json,p.field_path,p.authored_markup_sha256,r.name \
         FROM json_each(?1) w JOIN records r ON r.key=json_extract(w.value,'$[0]') \
         JOIN prepared_content p ON p.record_id=r.record_id \
         AND p.owners_json=json_extract(w.value,'$[1]') \
         AND p.field_path=json_extract(w.value,'$[2]') \
         AND p.authored_markup_sha256=json_extract(w.value,'$[3]') \
         WHERE r.document_kind<>'Macro' AND p.published=1 AND p.outcome='prepared'",
    )?;
    let rows = query
        .query_map([serde_json::to_string(&wanted)?], |row| {
            Ok((
                (
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ),
                row.get::<_, String>(4)?,
            ))
        })?
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    if rows.len() != wanted.len() {
        return Err(invalid("remaster citation field is unavailable"));
    }
    Ok(rows)
}

struct CapturedInput {
    origin: RemasterOrigin,
    revision: String,
    path: String,
}
fn captured_inputs(
    reader: &SqliteIndexReader,
    citations: &[&RemasterCitation],
) -> Result<BTreeMap<String, CapturedInput>, IndexError> {
    let wanted = citations
        .iter()
        .filter_map(|citation| match citation {
            RemasterCitation::CapturedFile { input_key, .. } => Some(input_key),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    if wanted.is_empty() {
        return Ok(BTreeMap::new());
    }
    reader.count_statement();
    let mut query = reader.native()?.prepare(
        "SELECT i.input_key,i.payload_json FROM json_each(?1) w \
         JOIN remaster_source_inputs i ON i.input_key=w.value ORDER BY i.input_key",
    )?;
    let mut result = BTreeMap::new();
    for row in query.query_map([serde_json::to_string(&wanted)?], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })? {
        let (key, json) = row?;
        // Decode/hash once per distinct captured input, then retain only the
        // attribution metadata used by every declaration sharing that input.
        let input: RemasterSourceInput = serde_json::from_str(&json)?;
        if input.key != key || input.sha256 != hash(&input.bytes) {
            return Err(invalid("remaster source input differs"));
        }
        result.insert(
            key,
            CapturedInput {
                origin: input.origin,
                revision: input.revision,
                path: input.path,
            },
        );
    }
    if result.len() != wanted.len() {
        return Err(invalid("remaster source input unavailable"));
    }
    Ok(result)
}

fn information(
    reader: &SqliteIndexReader,
    ids: Vec<String>,
    limit: usize,
) -> Result<SourceRemasterInformation, IndexError> {
    let truncated = ids.len() > limit;
    let selected = ids.into_iter().take(limit).collect::<BTreeSet<_>>();
    Ok(SourceRemasterInformation {
        declarations: hydrate(reader, &selected)?.into_values().collect(),
        truncated,
    })
}
impl SqliteIndexReader {
    pub fn remaster_record_information(
        &self,
        key: &RecordKey,
        owners: &[OwnedContentLocator],
        limit: usize,
    ) -> Result<SourceRemasterInformation, IndexError> {
        bound(limit)?;
        self.count_statement();
        // A Discussion is context on the exact journal/page that authored it,
        // not an association inferred from the prose to other game records.
        let mut query = self.native()?.prepare(
            "SELECT declaration_id FROM remaster_bindings WHERE record_key=?1 AND owners_json=?2 \
             UNION SELECT c.declaration_id FROM remaster_citations c \
             JOIN records r ON r.record_id=c.source_record_id \
             JOIN remaster_declarations d ON d.declaration_id=c.declaration_id \
             WHERE r.key=?1 AND json_extract(c.payload_json,'$.kind')='journal' \
             AND (?2='[]' OR json_extract(c.payload_json,'$.locator.owners')=json(?2)) \
             AND EXISTS(SELECT 1 FROM json_each(d.payload_json,'$.kinds') WHERE value='discussion') \
             ORDER BY declaration_id LIMIT ?3",
        )?;
        let ids = query
            .query_map(
                params![
                    key.to_string(),
                    serde_json::to_string(owners)?,
                    limit as i64 + 1
                ],
                |row| row.get::<_, String>(0),
            )?
            .collect::<Result<Vec<_>, _>>()?;
        information(self, ids, limit)
    }
    pub fn remaster_term_information(
        &self,
        term: &str,
        limit: usize,
    ) -> Result<SourceRemasterInformation, IndexError> {
        bound(limit)?;
        if term.chars().count() > 256 || term.trim().is_empty() {
            return Err(IndexError::Limit(
                "remaster term must contain1..256characters".into(),
            ));
        }
        self.count_statement();
        let mut query = self.native()?.prepare(
            "SELECT DISTINCT declaration_id FROM remaster_terms WHERE term=? ORDER BY declaration_id LIMIT ?",
        )?;
        let ids = query
            .query_map(
                params![normalize_record_name(term), limit as i64 + 1],
                |row| row.get::<_, String>(0),
            )?
            .collect::<Result<Vec<_>, _>>()?;
        information(self, ids, limit)
    }
    pub fn remaster_alias_information(
        &self,
        aliases: &[(RecordKey, String)],
        limit: usize,
    ) -> Result<BTreeMap<(RecordKey, String), SourceRemasterInformation>, IndexError> {
        bound(limit)?;
        if aliases.len() > 10240 {
            return Err(IndexError::Limit("too many remaster alias requests".into()));
        }
        if aliases.is_empty() {
            return Ok(BTreeMap::new());
        }
        self.count_statement();
        let wanted = aliases
            .iter()
            .map(|(key, alias)| (key.to_string(), alias))
            .collect::<BTreeSet<_>>();
        let mut query = self.native()?.prepare(
            "WITH wanted AS (SELECT json_extract(value,'$[0]') AS key,json_extract(value,'$[1]') AS alias FROM json_each(?1)), \
             ranked AS (SELECT r.key,s.alias,s.declaration_id,row_number() OVER(PARTITION BY r.key,s.alias ORDER BY s.declaration_id) AS ordinal \
             FROM wanted w JOIN records r ON r.key=w.key JOIN remaster_alias_support s ON s.record_id=r.record_id AND s.alias=w.alias) \
             SELECT key,alias,declaration_id FROM ranked WHERE ordinal<=?2 ORDER BY key,alias,declaration_id",
        )?;
        let mut grouped = BTreeMap::<(RecordKey, String), Vec<String>>::new();
        for row in query.query_map(
            params![serde_json::to_string(&wanted)?, limit as i64 + 1],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )? {
            let (key, alias, id) = row?;
            grouped
                .entry((RecordKey::parse(&key).map_err(invalid)?, alias))
                .or_default()
                .push(id);
        }
        let selected = grouped
            .values()
            .flat_map(|ids| ids.iter().take(limit).cloned())
            .collect::<BTreeSet<_>>();
        let declarations = hydrate(self, &selected)?;
        grouped
            .into_iter()
            .map(|(key, ids)| {
                let truncated = ids.len() > limit;
                let declarations =
                    ids.into_iter()
                        .take(limit)
                        .map(|id| {
                            declarations.get(&id).cloned().ok_or_else(|| {
                                invalid("remaster support declaration is unavailable")
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                Ok((
                    key,
                    SourceRemasterInformation {
                        declarations,
                        truncated,
                    },
                ))
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "read_tests.rs"]
mod tests;
