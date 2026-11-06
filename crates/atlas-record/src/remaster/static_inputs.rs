use super::*;
use atlas_domain::RecordKey;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpellEnd {
    _id: String,
    name: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpellRedirect {
    from: SpellEnd,
    to: SpellEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewedCorrespondences {
    pub entries: Vec<ReviewedCorrespondence>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewedCorrespondence {
    pub before: RecordKey,
    pub after: RecordKey,
    pub before_label: String,
    pub after_label: String,
    pub namespace: String,
    pub rationale: String,
    pub source_type: String,
}
fn literal_json(source: &str) -> Result<String, RemasterError> {
    let mut result = String::new();
    let mut chars = source.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            continue;
        }
        match c {
            '[' | ']' | '{' | '}' | ':' => result.push(c),
            ',' => {
                while chars.peek().is_some_and(|c| c.is_whitespace()) {
                    chars.next();
                }
                if !chars.peek().is_some_and(|c| matches!(*c, '}' | ']')) {
                    result.push(',');
                }
            }
            '"' => {
                result.push(c);
                let mut escaped = false;
                let mut closed = false;
                for value in chars.by_ref() {
                    result.push(value);
                    if value == '"' && !escaped {
                        closed = true;
                        break;
                    }
                    escaped = value == '\\' && !escaped;
                }
                if !closed {
                    return Err(invalid_literal());
                }
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let mut key = c.to_string();
                while chars
                    .peek()
                    .is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_')
                {
                    if let Some(c) = chars.next() {
                        key.push(c);
                    }
                }
                if !matches!(key.as_str(), "from" | "to" | "_id" | "name") {
                    return Err(invalid_literal());
                }
                result.push_str(&serde_json::to_string(&key)?);
            }
            _ => return Err(invalid_literal()),
        }
    }
    Ok(result)
}
fn invalid_literal() -> RemasterError {
    RemasterError::Invalid("remaster input is not the supported typed literal".into())
}
fn record_ref(key: &RecordKey, kind: &str, label: String) -> RemasterReference {
    RemasterReference {
        target: format!("Compendium.pf2e.{}.{kind}.{}", key.pack(), key.id()),
        label,
        occurrence: None,
        explicit_key: Some(key.clone()),
    }
}
fn single(label: String, target: RemasterReference) -> RemasterSide {
    RemasterSide {
        text: label.clone(),
        terms: vec![label],
        references: vec![target],
    }
}
fn make(
    input: &RemasterSourceInput,
    ordinal: usize,
    before: RemasterSide,
    after: RemasterSide,
    context: RemasterContext,
    status: &str,
) -> Result<RemasterDeclaration, RemasterError> {
    RemasterDeclaration {
        id: String::new(),
        kinds: vec![RemasterChangeKind::Replaced],
        authored_status: Some(status.into()),
        context,
        before: vec![before],
        after: vec![after],
        citations: vec![RemasterCitation::CapturedFile {
            input_key: input.key.clone(),
            selector: RemasterSelector::LiteralEntry { ordinal },
        }],
    }
    .seal()
}

/// Parse closed source literals only; source programs are never evaluated.
pub fn file_declarations(
    input: &RemasterSourceInput,
) -> Result<Vec<RemasterDeclaration>, RemasterError> {
    if input.bytes.len() > 1024 * 1024
        || input.sha256 != hash(&input.bytes)
        || input.key
            != RemasterSourceInput::new(
                input.kind,
                input.origin,
                input.revision.clone(),
                input.path.clone(),
                input.bytes.clone(),
            )
            .key
    {
        return Err(RemasterError::Invalid(
            "remaster source capture identity differs".into(),
        ));
    }
    let source =
        std::str::from_utf8(&input.bytes).map_err(|e| RemasterError::Invalid(e.to_string()))?;
    match input.kind {
        RemasterInputKind::Migration850 => {
            if input.origin != RemasterOrigin::Foundry {
                return Err(invalid_literal());
            }
            let compact = source.split_whitespace().collect::<String>();
            if !compact.contains(r#"#oldNamePattern=newRegExp(/\bFlat-Footed\b/g);"#)
                || !compact.contains(r#"#newName="Off-Guard";"#)
                || !source.contains("AJh5ex99aV6VTggg")
            {
                return Err(invalid_literal());
            }
            let key: RecordKey = "conditionitems:AJh5ex99aV6VTggg"
                .parse()
                .map_err(|e| RemasterError::Invalid(format!("{e}")))?;
            let mut result = make(
                input,
                0,
                RemasterSide {
                    text: "Flat-Footed".into(),
                    terms: vec!["Flat-Footed".into()],
                    references: vec![],
                },
                single(
                    "Off-Guard".into(),
                    record_ref(&key, "Item", "Off-Guard".into()),
                ),
                RemasterContext {
                    section: "Condition name change".into(),
                    namespace: "condition".into(),
                    qualifier: None,
                    source_type: Some("condition".into()),
                },
                "Renamed",
            )?;
            result.kinds = vec![RemasterChangeKind::Renamed];
            Ok(vec![result.seal()?])
        }
        RemasterInputKind::Migration887 => {
            if input.origin != RemasterOrigin::Foundry {
                return Err(invalid_literal());
            }
            let (_, tail) = source.split_once("#spells").ok_or_else(invalid_literal)?;
            let start = tail.find('[').ok_or_else(invalid_literal)?;
            let end = tail[start..]
                .find(']')
                .map(|x| start + x)
                .ok_or_else(invalid_literal)?;
            let redirects: Vec<SpellRedirect> =
                serde_json::from_str(&literal_json(&tail[start..=end])?)?;
            redirects
                .into_iter()
                .enumerate()
                .map(|(ordinal, row)| {
                    let old: RecordKey = format!("spells-srd:{}", row.from._id)
                        .parse()
                        .map_err(|e| RemasterError::Invalid(format!("{e}")))?;
                    let new: RecordKey = format!("spells-srd:{}", row.to._id)
                        .parse()
                        .map_err(|e| RemasterError::Invalid(format!("{e}")))?;
                    make(
                        input,
                        ordinal,
                        single(
                            row.from.name.clone(),
                            record_ref(&old, "Item", row.from.name),
                        ),
                        single(row.to.name.clone(), record_ref(&new, "Item", row.to.name)),
                        RemasterContext {
                            section: "Spell replacement".into(),
                            namespace: "spell".into(),
                            qualifier: None,
                            source_type: Some("spell".into()),
                        },
                        "Replaced",
                    )
                })
                .collect()
        }
        RemasterInputKind::AtlasCorrespondence => {
            if input.origin != RemasterOrigin::AtlasReviewed {
                return Err(invalid_literal());
            }
            let entries: ReviewedCorrespondences = serde_json::from_str(source)?;
            entries
                .entries
                .into_iter()
                .enumerate()
                .map(|(ordinal, row)| {
                    if row.source_type.is_empty()
                        || row.before == row.after
                        || row.rationale.trim().is_empty()
                    {
                        return Err(invalid_literal());
                    }
                    make(
                        input,
                        ordinal,
                        single(
                            row.before_label.clone(),
                            record_ref(&row.before, "Item", row.before_label),
                        ),
                        single(
                            row.after_label.clone(),
                            record_ref(&row.after, "Item", row.after_label),
                        ),
                        RemasterContext {
                            section: "Atlas-reviewed correspondence".into(),
                            namespace: row.namespace,
                            qualifier: Some(row.rationale),
                            source_type: Some(row.source_type),
                        },
                        "Atlas-reviewed counterpart",
                    )
                })
                .collect()
        }
    }
}
