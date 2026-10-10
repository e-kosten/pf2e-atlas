//! Native FTS/vector queries use one executable-owned eligibility relation.
use crate::{
    IndexError, SourceLexicalUnitKind, SourceUnitLocation, SqliteIndexReader,
    query::{ValidatedQuery, compile_predicate},
};
use atlas_domain::{RecordKey, SourcePassageAddress};
use atlas_record::source_content::OwnedContentLocator;
use rusqlite::{OptionalExtension, params_from_iter, types::Value};
use serde::{Deserialize, Serialize};
pub(crate) const NAME_LOOKUP_SQL: &str = "SELECT r.key,NULL,NULL FROM records r WHERE r.document_kind<>'Macro' AND r.name_state='value' AND r.name_lookup_key=?1 UNION ALL SELECT r.key,a.alias,a.evidence_json FROM verified_aliases a JOIN records r ON r.record_id=a.record_id WHERE r.document_kind<>'Macro' AND a.alias_lookup_key=?1 ORDER BY 1,2";
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceLexicalHit {
    pub location: SourceUnitLocation,
    pub rank: f64,
    pub exact_label: bool,
    pub label: Option<String>,
    pub unit_kind: crate::SourceLexicalUnitKind,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceLexicalRootHit {
    pub record: RecordKey,
    pub best_rank: f64,
    pub witnesses: Vec<SourceLexicalHit>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceVectorHit {
    pub location: SourceUnitLocation,
    pub distance: f64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceKeyPage {
    pub keys: Vec<RecordKey>,
    pub total: usize,
}
#[derive(Debug, Clone)]
pub struct SourceIdentityMatch {
    pub key: RecordKey,
    pub alias: Option<String>,
    pub evidence: Option<String>,
}
#[derive(Debug, Clone)]
pub struct SourceRemasterPair {
    pub legacy: RecordKey,
    pub remaster: RecordKey,
    pub evidence: String,
}
#[derive(Debug, Clone)]
pub struct SourceIdentityVector {
    pub model: crate::SourceSemanticModelIdentity,
    pub vector: Vec<f32>,
}

pub(crate) fn eligible_sql(
    query: &ValidatedQuery,
    keys: Option<&[RecordKey]>,
) -> Result<(String, Vec<Value>), IndexError> {
    let compiled = compile_predicate(query)?;
    let mut params = compiled.parameters;
    let mut sql = format!(
        "SELECT r.record_id FROM records r WHERE r.document_kind<>'Macro' AND ({})=1",
        compiled.expression
    );
    if let Some(keys) = keys {
        if keys.is_empty() {
            sql.push_str(" AND 0");
        } else {
            sql.push_str(&format!(
                " AND r.key IN ({})",
                vec!["?"; keys.len()].join(",")
            ));
            params.extend(keys.iter().map(|k| Value::Text(k.to_string())));
        }
    }
    Ok((sql, params))
}
pub(crate) fn parse_key(s: String) -> Result<RecordKey, IndexError> {
    RecordKey::parse(&s).map_err(|e| IndexError::Invalid(e.to_string()))
}
fn location(row: &rusqlite::Row<'_>) -> rusqlite::Result<SourceUnitLocation> {
    let decode = |e: Box<dyn std::error::Error + Send + Sync>| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, e)
    };
    let key: String = row.get(1)?;
    let owners: String = row.get(2)?;
    let section: Option<String> = row.get(4)?;
    Ok(SourceUnitLocation {
        unit_id: row.get(0)?,
        record: RecordKey::parse(&key).map_err(|e| decode(Box::new(e)))?,
        owners: serde_json::from_str::<Vec<OwnedContentLocator>>(&owners)
            .map_err(|e| decode(Box::new(e)))?,
        field: row.get(3)?,
        address: section
            .map(|s| serde_json::from_str::<SourcePassageAddress>(&s))
            .transpose()
            .map_err(|e| decode(Box::new(e)))?,
    })
}
impl SqliteIndexReader {
    pub fn eligible_keys(
        &self,
        query: &ValidatedQuery,
        keys: Option<&[RecordKey]>,
        prefer_remaster: bool,
        offset: usize,
        limit: usize,
    ) -> Result<SourceKeyPage, IndexError> {
        if limit > 1024 {
            return Err(IndexError::Invalid("browse page exceeds 1024 roots".into()));
        }
        let (eligible, mut params) = eligible_sql(query, keys)?;
        let preference = if prefer_remaster {
            " AND NOT EXISTS(SELECT 1 FROM remaster_pairs rp WHERE rp.legacy_record_id=r.record_id AND rp.remaster_record_id IN (SELECT record_id FROM eligible))"
        } else {
            ""
        };
        let base = format!(
            "WITH eligible AS ({eligible}) SELECT r.key FROM records r WHERE r.record_id IN(SELECT record_id FROM eligible){preference}"
        );
        let c = self.native()?;
        self.count_statement();
        let total: i64 = c.query_row(
            &format!("SELECT count(*) FROM ({base})"),
            params_from_iter(params.iter()),
            |r| r.get(0),
        )?;
        params.push(Value::Integer(
            i64::try_from(limit).map_err(|e| IndexError::Invalid(e.to_string()))?,
        ));
        params.push(Value::Integer(
            i64::try_from(offset).map_err(|e| IndexError::Invalid(e.to_string()))?,
        ));
        self.count_statement();
        let mut statement = c.prepare(&format!(
            "{base} ORDER BY r.name_lookup_key,r.key LIMIT ? OFFSET ?"
        ))?;
        let values = statement
            .query_map(params_from_iter(params.iter()), |r| r.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(SourceKeyPage {
            keys: values
                .into_iter()
                .map(parse_key)
                .collect::<Result<Vec<_>, _>>()?,
            total: usize::try_from(total).map_err(|e| IndexError::Invalid(e.to_string()))?,
        })
    }
    pub fn lexical_candidates(
        &self,
        text: &str,
        query: &ValidatedQuery,
        keys: Option<&[RecordKey]>,
        limit: usize,
    ) -> Result<Vec<SourceLexicalHit>, IndexError> {
        if limit > 65536 {
            return Err(IndexError::Invalid(
                "lexical candidate window exceeds 65536 units".into(),
            ));
        }
        let expression = literal_fts_query(text)?;
        if expression.is_empty() {
            return Ok(Vec::new());
        }
        let (eligible, mut params) = eligible_sql(query, keys)?;
        let mut all = Vec::new();
        all.append(&mut params);
        let term_param = all.len() + 1;
        all.push(Value::Text(expression));
        all.push(Value::Integer(limit as i64));
        let limit_param = all.len();
        let sql = format!(
            "WITH eligible AS ({eligible}) SELECT lu.unit_id,r.key,lu.owners_json,lu.field_path,lu.section_json,bm25(lexical_fts,10.0,8.0,1.0,6.0),lu.unit_kind,lu.identity_terms,lu.definition_terms,r.name_state FROM lexical_fts JOIN lexical_units lu ON lu.unit_id=lexical_fts.rowid JOIN records r ON r.record_id=lu.record_id WHERE lexical_fts MATCH ?{term_param} AND lu.record_id IN (SELECT record_id FROM eligible) ORDER BY 6,lu.unit_id LIMIT ?{limit_param}"
        );
        let c = self.native()?;
        self.count_statement();
        let mut statement = c.prepare(&sql)?;
        Ok(statement
            .query_map(params_from_iter(all.iter()), |r| lexical_hit(r, text))?
            .collect::<Result<Vec<_>, _>>()?)
    }
    pub fn vector_candidates(
        &self,
        vector: &[f32],
        query: &ValidatedQuery,
        keys: Option<&[RecordKey]>,
        limit: usize,
    ) -> Result<Vec<SourceVectorHit>, IndexError> {
        self.vector_candidates_inner(vector, query, keys, None, limit)
    }
    pub fn vector_candidates_excluding(
        &self,
        vector: &[f32],
        query: &ValidatedQuery,
        keys: Option<&[RecordKey]>,
        exclude: &RecordKey,
        limit: usize,
    ) -> Result<Vec<SourceVectorHit>, IndexError> {
        self.vector_candidates_inner(vector, query, keys, Some(exclude), limit)
    }
    fn vector_candidates_inner(
        &self,
        vector: &[f32],
        query: &ValidatedQuery,
        keys: Option<&[RecordKey]>,
        exclude: Option<&RecordKey>,
        limit: usize,
    ) -> Result<Vec<SourceVectorHit>, IndexError> {
        if limit == 0 || limit > 4096 {
            return Err(IndexError::Invalid(
                "sqlite-vec candidate window must be 1..=4096".into(),
            ));
        }
        let model = self
            .context()
            .semantic_model
            .as_ref()
            .ok_or_else(|| IndexError::Unavailable("artifact has no semantic model".into()))?;
        crate::validation::check_vector(vector, model.dimensions)?;
        self.require_vectors()?;
        let (mut eligible, mut params) = eligible_sql(query, keys)?;
        if let Some(exclude) = exclude {
            eligible.push_str(&format!(" AND r.key<>?{}", params.len() + 1));
            params.push(Value::Text(exclude.to_string()));
        }
        let mut all = params;
        let vector_param = all.len() + 1;
        all.push(Value::Blob(vector_bytes(vector)));
        let limit_param = all.len() + 1;
        all.push(Value::Integer(limit as i64));
        let sql = format!(
            "WITH eligible AS ({eligible}) SELECT su.unit_id,r.key,su.owners_json,su.field_path,su.section_json,knn.distance FROM (SELECT unit_id,distance FROM semantic_vectors WHERE embedding MATCH ?{vector_param} AND k=?{limit_param} AND unit_id IN (SELECT su.unit_id FROM semantic_units su WHERE su.record_id IN (SELECT record_id FROM eligible))) knn JOIN semantic_units su ON su.unit_id=knn.unit_id JOIN records r ON r.record_id=su.record_id ORDER BY knn.distance,su.unit_id"
        );
        let c = self.native()?;
        self.count_statement();
        let mut statement = c.prepare(&sql)?;
        Ok(statement
            .query_map(params_from_iter(all.iter()), |r| {
                Ok(SourceVectorHit {
                    location: location(r)?,
                    distance: r.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?)
    }
    pub fn seed_identity_vector(
        &self,
        key: &RecordKey,
    ) -> Result<Option<SourceIdentityVector>, IndexError> {
        let Some(model) = self.context().semantic_model.as_ref() else {
            return Ok(None);
        };
        self.require_vectors()?;
        let c = self.native()?;
        self.count_statement();
        let bytes:Option<Vec<u8>>=c.query_row("SELECT sv.embedding FROM semantic_vectors sv JOIN semantic_units su ON su.unit_id=sv.unit_id JOIN records r ON r.record_id=su.record_id WHERE r.key=? AND r.document_kind<>'Macro' AND su.unit_kind='identity'",[key.to_string()],|r|r.get(0)).optional()?;
        bytes
            .map(|b| {
                let vector = decode_vector(&b)?;
                crate::validation::check_vector(&vector, model.dimensions)?;
                Ok(SourceIdentityVector {
                    model: model.clone(),
                    vector,
                })
            })
            .transpose()
    }
    pub fn lookup_name_or_alias(&self, text: &str) -> Result<Vec<SourceIdentityMatch>, IndexError> {
        let key = atlas_domain::normalize_record_name(text);
        let c = self.native()?;
        self.count_statement();
        let mut statement = c.prepare(NAME_LOOKUP_SQL)?;
        let values = statement
            .query_map([&key], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        values
            .into_iter()
            .map(|(key, alias, evidence)| {
                Ok(SourceIdentityMatch {
                    key: parse_key(key)?,
                    alias,
                    evidence: evidence.map(|s| serde_json::from_str(&s)).transpose()?,
                })
            })
            .collect()
    }
    /// Preference evidence is returned only where both requested roots matched.
    pub fn matched_remaster_pairs(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<SourceRemasterPair>, IndexError> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }
        let requested = keys.iter().collect::<std::collections::BTreeSet<_>>();
        let c = self.native()?;
        self.count_statement();
        let mut statement = c.prepare("SELECT l.key,m.key,rp.evidence_json FROM remaster_pairs rp JOIN records l ON l.record_id=rp.legacy_record_id JOIN records m ON m.record_id=rp.remaster_record_id WHERE l.document_kind<>'Macro' AND m.document_kind<>'Macro' ORDER BY l.key,m.key")?;
        let values = statement
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        values
            .into_iter()
            .map(|(legacy, remaster, evidence)| {
                Ok(SourceRemasterPair {
                    legacy: parse_key(legacy)?,
                    remaster: parse_key(remaster)?,
                    evidence: serde_json::from_str(&evidence)?,
                })
            })
            .filter(|pair| match pair {
                Ok(pair) => requested.contains(&pair.legacy) && requested.contains(&pair.remaster),
                Err(_) => true,
            })
            .collect()
    }
    /// Bounded indexed candidates; the caller verifies the declared name convention and typed category/group.
    pub fn variant_candidates(
        &self,
        pack: &str,
        family: &str,
        base_prefix: &str,
        limit: usize,
    ) -> Result<Vec<RecordKey>, IndexError> {
        if limit == 0 || limit > 256 {
            return Err(IndexError::Invalid(
                "variant candidate limit must be 1..=256".into(),
            ));
        }
        let prefix = atlas_domain::normalize_record_name(base_prefix);
        let upper = format!("{prefix}\u{10ffff}");
        let c = self.native()?;
        self.count_statement();
        let mut statement=c.prepare("SELECT key FROM records WHERE pack_id=? AND source_type=? AND document_kind<>'Macro' AND name_lookup_key>=? AND name_lookup_key<? ORDER BY name_lookup_key,key LIMIT ?")?;
        statement
            .query_map(
                rusqlite::params![pack, family, prefix, upper, limit as i64],
                |r| r.get::<_, String>(0),
            )?
            .map(|r| parse_key(r?))
            .collect()
    }
}
pub(crate) fn vector_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|n| n.to_le_bytes()).collect()
}
pub(crate) fn decode_vector(bytes: &[u8]) -> Result<Vec<f32>, IndexError> {
    if !bytes.len().is_multiple_of(4) {
        return Err(IndexError::Invalid("vector blob is not f32-aligned".into()));
    }
    Ok(bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect())
}
fn literal_fts_query(text: &str) -> Result<String, IndexError> {
    if text.len() > 16_384 {
        return Err(IndexError::Invalid(
            "lexical text exceeds 16384 bytes".into(),
        ));
    }
    let mut terms = Vec::new();
    let mut buffer = String::new();
    let mut quoted = false;
    for c in text.chars() {
        if c == '"' {
            if !buffer.trim().is_empty() {
                terms.push(std::mem::take(&mut buffer));
            }
            quoted = !quoted;
        } else if c.is_whitespace() && !quoted {
            if !buffer.is_empty() {
                terms.push(std::mem::take(&mut buffer));
            }
        } else {
            buffer.push(c);
        }
    }
    if quoted {
        return Err(IndexError::Invalid("unclosed lexical phrase quote".into()));
    }
    if !buffer.trim().is_empty() {
        terms.push(buffer);
    }
    Ok(terms
        .into_iter()
        .filter(|s| s.chars().any(char::is_alphanumeric))
        .map(|s| format!("\"{}\"", s.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" AND "))
}

#[derive(Debug, Clone)]
pub struct SourceReuseCandidate {
    pub location: SourceUnitLocation,
    pub chunk_ordinal: usize,
    pub input_token_count: usize,
    pub input_hash: String,
    pub vector: Vec<f32>,
}
impl SqliteIndexReader {
    /// Untrusted reuse candidates. Ingest must reconstruct each old input with
    /// the pinned tokenizer and verify its complete metadata before accepting it.
    pub fn read_reuse_candidates(
        &self,
        hashes: &[String],
        model: &crate::SourceSemanticModelIdentity,
        after_unit_id: i64,
        limit: usize,
    ) -> Result<Vec<SourceReuseCandidate>, IndexError> {
        if hashes.len() > 1024 || !(1..=1024).contains(&limit) || after_unit_id < 0 {
            return Err(IndexError::Invalid(
                "vector reuse batch exceeds 1024 hashes".into(),
            ));
        }
        if hashes.is_empty() || self.context().semantic_model.as_ref() != Some(model) {
            return Ok(vec![]);
        }
        self.require_vectors()?;
        let c = self.native()?;
        let sql = format!(
            "SELECT su.unit_id,r.key,su.owners_json,su.field_path,su.section_json,su.chunk_ordinal,su.input_token_count,su.input_hash,sv.embedding FROM semantic_units su JOIN semantic_vectors sv USING(unit_id) JOIN records r USING(record_id) WHERE su.unit_id>? AND su.input_hash IN ({}) ORDER BY su.unit_id LIMIT ?",
            vec!["?"; hashes.len()].join(",")
        );
        self.count_statement();
        let mut s = c.prepare(&sql)?;
        let mut parameters = vec![Value::Integer(after_unit_id)];
        parameters.extend(hashes.iter().cloned().map(Value::Text));
        parameters.push(Value::Integer(limit as i64));
        let mut output = vec![];
        let mut rows = s.query(params_from_iter(parameters))?;
        while let Some(row) = rows.next()? {
            let bytes: Vec<u8> = row.get(8)?;
            let vector = decode_vector(&bytes)?;
            crate::validation::check_vector(&vector, model.dimensions)?;
            output.push(SourceReuseCandidate {
                location: location(row)?,
                chunk_ordinal: row.get(5)?,
                input_token_count: row.get(6)?,
                input_hash: row.get(7)?,
                vector,
            });
        }
        Ok(output)
    }
}
fn lexical_hit(row: &rusqlite::Row<'_>, text: &str) -> rusqlite::Result<SourceLexicalHit> {
    let location = location(row)?;
    let kind: String = row.get(6)?;
    let identity: String = row.get(7)?;
    let definition: String = row.get(8)?;
    let name_state: String = row.get(9)?;
    let label = match kind.as_str() {
        "root_name" if name_state == "value" => Some(identity.as_str()),
        "owned_name" | "heading" | "definition_label" => Some(definition.as_str()),
        _ => None,
    };
    let comparison = text.trim();
    let comparison = comparison
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .filter(|s| !s.contains('"'))
        .unwrap_or(comparison);
    let exact_label = label.is_some_and(|label| {
        atlas_domain::normalize_record_name(label)
            == atlas_domain::normalize_record_name(comparison)
    });
    Ok(SourceLexicalHit {
        location,
        rank: row.get(5)?,
        exact_label,
        label: label.map(str::to_owned),
        unit_kind: serde_json::from_str(&format!("\"{kind}\"")).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, Box::new(e))
        })?,
    })
}
impl SqliteIndexReader {
    /// Exhaustive lexical root membership. Only retained witnesses are bounded.
    pub fn lexical_root_candidates(
        &self,
        text: &str,
        query: &ValidatedQuery,
        keys: Option<&[RecordKey]>,
    ) -> Result<Vec<SourceLexicalRootHit>, IndexError> {
        let expression = literal_fts_query(text)?;
        if expression.is_empty() {
            return Ok(vec![]);
        }
        let (eligible, mut params) = eligible_sql(query, keys)?;
        let text_param = params.len() + 1;
        params.push(Value::Text(expression));
        let sql = format!(
            "WITH eligible AS ({eligible}) SELECT lu.unit_id,r.key,lu.owners_json,lu.field_path,lu.section_json,bm25(lexical_fts,10.0,8.0,1.0,6.0),lu.unit_kind,lu.identity_terms,lu.definition_terms,r.name_state FROM lexical_fts JOIN lexical_units lu ON lu.unit_id=lexical_fts.rowid JOIN records r ON r.record_id=lu.record_id WHERE lexical_fts MATCH ?{text_param} AND lu.record_id IN (SELECT record_id FROM eligible) ORDER BY 6,lu.unit_id"
        );
        let c = self.native()?;
        self.count_statement();
        let mut statement = c.prepare(&sql)?;
        let mut roots = std::collections::BTreeMap::<RecordKey, SourceLexicalRootHit>::new();
        for row in statement.query_map(params_from_iter(&params), |row| lexical_hit(row, text))? {
            let hit = row?;
            let root =
                roots
                    .entry(hit.location.record.clone())
                    .or_insert_with(|| SourceLexicalRootHit {
                        record: hit.location.record.clone(),
                        best_rank: hit.rank,
                        witnesses: vec![],
                    });
            root.witnesses.push(hit);
            root.witnesses.sort_by(|a, b| {
                lexical_witness_priority(a)
                    .cmp(&lexical_witness_priority(b))
                    .then_with(|| a.rank.total_cmp(&b.rank))
                    .then_with(|| a.location.unit_id.cmp(&b.location.unit_id))
            });
            root.witnesses.truncate(3);
        }
        let mut values = roots.into_values().collect::<Vec<_>>();
        values.sort_by(|a, b| {
            a.best_rank
                .total_cmp(&b.best_rank)
                .then_with(|| a.record.cmp(&b.record))
        });
        Ok(values)
    }
    pub fn read_remaster_links(
        &self,
        seed: &RecordKey,
    ) -> Result<Vec<SourceRemasterPair>, IndexError> {
        let c = self.native()?;
        self.count_statement();
        let mut s=c.prepare("SELECT l.key,m.key,p.evidence_json FROM remaster_pairs p JOIN records l ON l.record_id=p.legacy_record_id JOIN records m ON m.record_id=p.remaster_record_id WHERE l.document_kind<>'Macro' AND m.document_kind<>'Macro' AND (l.key=? OR m.key=?) ORDER BY l.key,m.key")?;
        s.query_map([seed.to_string(), seed.to_string()], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?
        .map(|r| {
            let (l, m, e) = r?;
            Ok(SourceRemasterPair {
                legacy: parse_key(l)?,
                remaster: parse_key(m)?,
                evidence: serde_json::from_str(&e)?,
            })
        })
        .collect()
    }
}

fn lexical_witness_priority(hit: &SourceLexicalHit) -> (u8, bool) {
    let tier = match hit.unit_kind {
        SourceLexicalUnitKind::RootName if hit.exact_label => 0,
        SourceLexicalUnitKind::OwnedName if hit.exact_label => 1,
        SourceLexicalUnitKind::Heading => 2,
        SourceLexicalUnitKind::DefinitionLabel => 3,
        SourceLexicalUnitKind::RootName | SourceLexicalUnitKind::OwnedName => 4,
    };
    (tier, !hit.exact_label)
}
