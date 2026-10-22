//! Strict typed CEL literals for exact relationship selections; no JSON-string language.
use super::{cel::literal, validation::error};
use atlas_domain::{
    OwnedContentIdentity, OwnedContentLocator, QueryError, QueryLiteral, QueryRecordSelection,
    QueryRelationship, QueryRelationshipDirection, RecordKey, SourceByteRange,
    SourcePassageAddress,
};
use cel::common::ast::{EntryExpr, Expr, IdedExpr, LiteralValue};
use std::collections::BTreeMap;
fn invalid() -> QueryError {
    error(
        "relationship_literal",
        "Use a canonical record key or an exact typed selection map",
        None,
    )
}
fn map<'a>(
    e: &'a IdedExpr,
    allowed: &[&str],
) -> Result<BTreeMap<String, &'a IdedExpr>, QueryError> {
    let Expr::Map(m) = &e.expr else {
        return Err(invalid());
    };
    let mut out = BTreeMap::new();
    for entry in &m.entries {
        let EntryExpr::MapEntry(entry) = &entry.expr else {
            return Err(invalid());
        };
        let key = string(&entry.key)?;
        if entry.optional
            || !allowed.contains(&key.as_str())
            || out.insert(key, &entry.value).is_some()
        {
            return Err(invalid());
        }
    }
    Ok(out)
}
fn string(e: &IdedExpr) -> Result<String, QueryError> {
    match literal(e)? {
        QueryLiteral::String(s) => Ok(s),
        _ => Err(invalid()),
    }
}
fn unsigned(e: &IdedExpr) -> Result<usize, QueryError> {
    match literal(e)? {
        QueryLiteral::Number(n) => n
            .as_u64()
            .and_then(|v| usize::try_from(v).ok())
            .ok_or_else(invalid),
        _ => Err(invalid()),
    }
}
fn required<'a>(m: &BTreeMap<String, &'a IdedExpr>, key: &str) -> Result<&'a IdedExpr, QueryError> {
    m.get(key).copied().ok_or_else(invalid)
}
fn optional<'a>(m: &BTreeMap<String, &'a IdedExpr>, key: &str) -> Option<&'a IdedExpr> {
    m.get(key)
        .copied()
        .filter(|e| !matches!(e.expr, Expr::Literal(LiteralValue::Null)))
}
fn byte_range(e: &IdedExpr) -> Result<SourceByteRange, QueryError> {
    let m = map(e, &["start", "end"])?;
    Ok(SourceByteRange {
        start: unsigned(required(&m, "start")?)?,
        end: unsigned(required(&m, "end")?)?,
    })
}
fn passage(e: &IdedExpr) -> Result<SourcePassageAddress, QueryError> {
    let m = map(
        e,
        &[
            "kind",
            "prepared_html_sha256",
            "canonical_text_sha256",
            "source_text_sha256",
            "selection_version",
            "section_ordinal",
            "label",
            "chunk_bytes",
        ],
    )?;
    let kind = string(required(&m, "kind")?)?;
    if kind == "identity" {
        if m.len() != 1 {
            return Err(invalid());
        }
        return Ok(SourcePassageAddress::Identity {});
    }
    let selection_version = string(required(&m, "selection_version")?)?;
    let section_ordinal = unsigned(required(&m, "section_ordinal")?)?;
    let label = optional(&m, "label").map(string).transpose()?;
    let chunk_bytes = byte_range(required(&m, "chunk_bytes")?)?;
    match kind.as_str() {
        "html_section" if !m.contains_key("source_text_sha256") => {
            Ok(SourcePassageAddress::HtmlSection {
                prepared_html_sha256: string(required(&m, "prepared_html_sha256")?)?,
                canonical_text_sha256: string(required(&m, "canonical_text_sha256")?)?,
                selection_version,
                section_ordinal,
                label,
                chunk_bytes,
            })
        }
        "plain_section"
            if !m.contains_key("prepared_html_sha256")
                && !m.contains_key("canonical_text_sha256") =>
        {
            Ok(SourcePassageAddress::PlainSection {
                source_text_sha256: string(required(&m, "source_text_sha256")?)?,
                selection_version,
                section_ordinal,
                label,
                chunk_bytes,
            })
        }
        _ => Err(invalid()),
    }
}
fn owner(e: &IdedExpr) -> Result<OwnedContentLocator, QueryError> {
    let m = map(e, &["collection", "identity"])?;
    let identity = map(required(&m, "identity")?, &["Stable", "SnapshotLocal"])?;
    if identity.len() != 1 {
        return Err(invalid());
    }
    let identity = if let Some(value) = identity.get("Stable") {
        OwnedContentIdentity::Stable(string(value)?)
    } else {
        let local = map(required(&identity, "SnapshotLocal")?, &["index"])?;
        OwnedContentIdentity::SnapshotLocal {
            index: unsigned(required(&local, "index")?)?,
        }
    };
    Ok(OwnedContentLocator {
        collection: string(required(&m, "collection")?)?,
        identity,
    })
}
fn selection(e: &IdedExpr) -> Result<QueryRecordSelection, QueryError> {
    if matches!(e.expr, Expr::Literal(LiteralValue::String(_))) {
        return Ok(QueryRecordSelection {
            record_key: RecordKey::parse(&string(e)?).map_err(|_| invalid())?,
            owners: vec![],
            field: None,
            passage: None,
            source_fingerprint: None,
            spell_rank: None,
        });
    }
    let m = map(
        e,
        &[
            "record_key",
            "embedded_path",
            "field",
            "passage",
            "source_fingerprint",
            "spell_rank",
        ],
    )?;
    let owners = if let Some(e) = m.get("embedded_path") {
        let Expr::List(list) = &e.expr else {
            return Err(invalid());
        };
        if !list.optional_indices.is_empty() {
            return Err(invalid());
        }
        list.elements
            .iter()
            .map(owner)
            .collect::<Result<Vec<_>, _>>()?
    } else {
        vec![]
    };
    Ok(QueryRecordSelection {
        record_key: RecordKey::parse(&string(required(&m, "record_key")?)?)
            .map_err(|_| invalid())?,
        owners,
        field: optional(&m, "field").map(string).transpose()?,
        passage: optional(&m, "passage").map(passage).transpose()?,
        source_fingerprint: optional(&m, "source_fingerprint").map(string).transpose()?,
        spell_rank: optional(&m, "spell_rank")
            .map(|e| u8::try_from(unsigned(e)?).map_err(|_| invalid()))
            .transpose()?,
    })
}
pub(super) fn lower(name: &str, args: &[IdedExpr]) -> Result<QueryRelationship, QueryError> {
    if !(1..=2).contains(&args.len()) {
        return Err(invalid());
    }
    let include_contents = if args.len() == 1 {
        true
    } else {
        match string(&args[1])?.as_str() {
            "whole_entry" => true,
            "main_entry" => false,
            _ => {
                return Err(error(
                    "relationship_scope",
                    "Use whole_entry or main_entry for link scope",
                    None,
                ));
            }
        }
    };
    Ok(QueryRelationship {
        direction: if name == "links_to" {
            QueryRelationshipDirection::Incoming
        } else {
            QueryRelationshipDirection::Outgoing
        },
        selection: selection(&args[0])?,
        include_contents,
    })
}

#[cfg(test)]
mod tests {
    use atlas_domain::{QueryExpression, QueryRelationshipDirection};
    #[test]
    fn cel_link_literals_are_canonical_typed_and_strict() {
        let query=crate::parse_where(r#"links_to({"record_key":"items:aaaaaaaaaaaaaaaa","embedded_path":[{"collection":"/items","identity":{"Stable":"bbbbbbbbbbbbbbbb"}}],"field":"/system/description/value","passage":null,"source_fingerprint":null}, "main_entry")"#).unwrap();
        let QueryExpression::Relationship { relationship } = &query.predicate().expression else {
            panic!("expected link leaf")
        };
        assert_eq!(relationship.direction, QueryRelationshipDirection::Incoming);
        assert_eq!(relationship.selection.owners.len(), 1);
        assert!(!relationship.include_contents);
        assert!(query.predicate().clause_id.is_some());
        let root = crate::parse_where("linked_from('items:aaaaaaaaaaaaaaaa')").unwrap();
        let QueryExpression::Relationship { relationship } = &root.predicate().expression else {
            panic!("expected link leaf")
        };
        assert_eq!(relationship.direction, QueryRelationshipDirection::Outgoing);
        assert!(relationship.include_contents);
        for source in [
            "links_to('Leather Armor')",
            "links_to('items:aaaaaaaaaaaaaaaa','nearby')",
            "links_to({'record_key':'items:aaaaaaaaaaaaaaaa','unknown':null})",
            "links_to({'record_key':'items:aaaaaaaaaaaaaaaa','record_key':'items:bbbbbbbbbbbbbbbb'})",
            "links_to({'record_key':'items:aaaaaaaaaaaaaaaa','embedded_path':null})",
            "links_to({'record_key':'items:aaaaaaaaaaaaaaaa','spell_rank':1.5})",
            "links_to({'record_key':'items:aaaaaaaaaaaaaaaa','embedded_path':[{'collection':'/items','identity':{'Stable':'x','SnapshotLocal':{'index':0}}}]})",
        ] {
            assert!(crate::parse_where(source).is_err(), "{source}");
        }
    }
}
