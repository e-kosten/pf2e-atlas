use super::*;
use crate::source_content::{PreparedSourceContent, prepared_plain_text};
use scraper::{ElementRef, Html, Selector};

fn selector(value: &str) -> Result<Selector, RemasterError> {
    Selector::parse(value).map_err(|e| RemasterError::Invalid(e.to_string()))
}
fn plain(value: &str) -> Result<String, RemasterError> {
    prepared_plain_text(value).map_err(|e| RemasterError::Invalid(e.to_string()))
}
fn descriptor(target: &str) -> &str {
    [".Item.", ".Actor.", ".JournalEntryPage.", ".JournalEntry."]
        .iter()
        .find_map(|marker| target.rsplit_once(marker).map(|(_, tail)| tail))
        .unwrap_or(target)
}
fn text(
    element: ElementRef<'_>,
    content: &PreparedSourceContent,
    refs: &mut Vec<RemasterReference>,
    out: &mut String,
) -> Result<(), RemasterError> {
    if let Some(value) = element.value().attr("data-atlas-reference") {
        let ordinal = value
            .parse::<usize>()
            .map_err(|_| RemasterError::Invalid("invalid remaster reference ordinal".into()))?;
        let reference = content
            .references
            .iter()
            .find(|r| r.ordinal == ordinal && r.visible)
            .ok_or_else(|| RemasterError::Invalid("remaster reference is not selected".into()))?;
        let label = if reference.label.explicit {
            plain(&element.html())?
        } else {
            descriptor(&reference.authored_target).to_owned()
        };
        out.push_str(&label);
        refs.push(RemasterReference {
            target: reference.authored_target.clone(),
            label,
            occurrence: Some(ordinal),
            explicit_key: None,
        });
        return Ok(());
    }
    for node in element.children() {
        if let Some(child) = ElementRef::wrap(node) {
            out.push(' ');
            text(child, content, refs, out)?;
            out.push(' ');
        } else if let scraper::node::Node::Text(value) = node.value() {
            out.push_str(&value.text);
        }
    }
    Ok(())
}
fn side(
    element: ElementRef<'_>,
    content: &PreparedSourceContent,
) -> Result<RemasterSide, RemasterError> {
    let mut references = vec![];
    let mut value = String::new();
    text(element, content, &mut references, &mut value)?;
    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let terms = if references.is_empty() {
        (!value.is_empty() && value != "—")
            .then(|| value.clone())
            .into_iter()
            .collect()
    } else {
        references.iter().map(|r| r.label.clone()).collect()
    };
    Ok(RemasterSide {
        text: value,
        terms,
        references,
    })
}
fn kinds(status: &str) -> Vec<RemasterChangeKind> {
    let mut result = vec![];
    for token in status.split(',').map(|s| s.trim().to_ascii_lowercase()) {
        let kind = match token.as_str() {
            "rename" | "renamed" => RemasterChangeKind::Renamed,
            "merged" => RemasterChangeKind::Merged,
            "replaced" => RemasterChangeKind::Replaced,
            "removed" => RemasterChangeKind::Removed,
            "altered mechanics" => RemasterChangeKind::MechanicsChanged,
            _ => RemasterChangeKind::Unclassified,
        };
        if !result.contains(&kind) {
            result.push(kind);
        }
    }
    result
}
fn namespace(section: &str) -> &str {
    match section {
        "Class Features" => "class_feature",
        "Feats" => "feat",
        "Spells" => "spell",
        "Equipment" => "equipment",
        "Bestiaries" => "creature",
        "Languages" => "language",
        "Domains" => "domain",
        "Pantheons" => "pantheon",
        _ => "rules",
    }
}
fn context(section: &str, qualifier: Option<String>) -> RemasterContext {
    RemasterContext {
        section: section.into(),
        namespace: namespace(section).into(),
        qualifier,
        source_type: None,
    }
}
fn declaration(
    content: &PreparedSourceContent,
    context: RemasterContext,
    selector: RemasterSelector,
    status: Option<String>,
    change: Vec<RemasterChangeKind>,
    before: Vec<RemasterSide>,
    after: Vec<RemasterSide>,
) -> Result<RemasterDeclaration, RemasterError> {
    RemasterDeclaration {
        id: String::new(),
        kinds: change,
        authored_status: status,
        context,
        before,
        after,
        citations: vec![RemasterCitation::Journal {
            locator: content.locator.clone(),
            selector,
            authored_sha256: content.authored_markup_sha256.clone(),
        }],
    }
    .seal()
}

/// Interpret only an audience-selected prepared field. Its attributed occurrences
/// supply authored reference text; current resolutions never enter canonical data.
pub fn journal_declarations(
    section: &str,
    content: &PreparedSourceContent,
) -> Result<Vec<RemasterDeclaration>, RemasterError> {
    let html = Html::parse_fragment(&content.html);
    let mut declarations = vec![];
    let cells = selector("td")?;
    for (ordinal, row) in html.select(&selector("tr")?).enumerate() {
        let values = row
            .select(&cells)
            .map(|e| side(e, content))
            .collect::<Result<Vec<_>, _>>()?;
        if values.is_empty() {
            continue;
        }
        let table = row
            .ancestors()
            .filter_map(ElementRef::wrap)
            .find(|e| e.value().name() == "table");
        let headers = table
            .map(|table| {
                table
                    .select(&selector("th")?)
                    .map(|h| {
                        plain(&h.html()).map(|v| {
                            v.to_ascii_lowercase()
                                .chars()
                                .filter(|c| c.is_ascii_alphanumeric())
                                .collect::<String>()
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default();
        let known_section = matches!(
            section,
            "Class Features" | "Feats" | "Spells" | "Equipment" | "Bestiaries"
        );
        let known_first = headers
            .first()
            .is_some_and(|v| matches!(v.as_str(), "old" | "oldname" | "itemname" | "spellname"));
        let known_last = headers
            .last()
            .is_some_and(|v| matches!(v.as_str(), "new" | "newname"));
        let supported = known_section
            && known_first
            && known_last
            && headers.len() == values.len()
            && match headers.as_slice() {
                [_, _] => true,
                [_, status, _] => status == "status",
                [_, qualifier, status, _] => {
                    matches!(qualifier.as_str(), "class" | "maintrait") && status == "status"
                }
                _ => false,
            };
        let (status, qualifier) = if supported {
            match values.len() {
                4 => (Some(values[2].text.clone()), Some(values[1].text.clone())),
                3 => (Some(values[1].text.clone()), None),
                _ => (None, None),
            }
        } else {
            (None, None)
        };
        let change = if supported {
            status
                .as_deref()
                .map(kinds)
                .unwrap_or_else(|| vec![RemasterChangeKind::Renamed])
        } else {
            vec![RemasterChangeKind::Unclassified]
        };
        let before = values.first().cloned().into_iter().collect();
        let after = if supported {
            values.last().cloned().into_iter().collect()
        } else {
            values.iter().skip(1).cloned().collect()
        };
        declarations.push(declaration(
            content,
            context(section, qualifier),
            RemasterSelector::TableRow { ordinal },
            status,
            change,
            before,
            after,
        )?);
    }
    let mut current_section = section.to_owned();
    let mut material_context = false;
    let mut list_ordinal = 0usize;
    let mut discussion_ordinal = 0usize;
    for element in html
        .root_element()
        .descendants()
        .filter_map(ElementRef::wrap)
    {
        match element.value().name() {
            "h2" => {
                current_section = plain(&element.html())?;
                material_context = false;
            }
            "li" => {
                let mut value = side(element, content)?;
                let lower = value.text.to_ascii_lowercase();
                let split = [
                    " are merged into ",
                    " is merged into ",
                    " are now ",
                    " is now ",
                ]
                .iter()
                .find_map(|delimiter| lower.find(delimiter).map(|p| (p, *delimiter)));
                if let Some((position, delimiter)) = split {
                    let old = value.text[..position].to_owned();
                    let new = value.text[position + delimiter.len()..].to_owned();
                    let terms = old
                        .replace(" and ", ", ")
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::to_owned)
                        .collect();
                    let before = RemasterSide {
                        text: old,
                        terms,
                        references: vec![],
                    };
                    value.text = new;
                    if value.references.is_empty() {
                        value.terms = vec![value.text.trim_end_matches('.').to_owned()];
                    }
                    let kind = if delimiter.contains("merged") {
                        RemasterChangeKind::Merged
                    } else {
                        RemasterChangeKind::Renamed
                    };
                    let mut item = declaration(
                        content,
                        context(&current_section, None),
                        RemasterSelector::ListItem {
                            ordinal: list_ordinal,
                        },
                        Some(delimiter.trim().into()),
                        vec![kind],
                        vec![before],
                        vec![value],
                    )?;
                    if material_context {
                        item.context.namespace = "material".into();
                        item = item.seal()?;
                    }
                    declarations.push(item);
                } else if !value.text.is_empty() {
                    value.terms.clear();
                    declarations.push(declaration(
                        content,
                        context(&current_section, None),
                        RemasterSelector::ListItem {
                            ordinal: list_ordinal,
                        },
                        None,
                        vec![RemasterChangeKind::Unclassified],
                        vec![],
                        vec![value],
                    )?);
                }
                list_ordinal += 1;
            }
            "p" if element
                .ancestors()
                .filter_map(ElementRef::wrap)
                .all(|p| !matches!(p.value().name(), "table" | "li")) =>
            {
                let mut value = side(element, content)?;
                if section == "Equipment" && value.text == "Some materials have been renamed." {
                    material_context = true;
                }
                if value.text.is_empty() {
                    continue;
                }
                value.terms.clear();
                declarations.push(declaration(
                    content,
                    context(&current_section, None),
                    RemasterSelector::Discussion {
                        ordinal: discussion_ordinal,
                    },
                    None,
                    vec![RemasterChangeKind::Discussion],
                    vec![],
                    vec![value],
                )?);
                discussion_ordinal += 1;
            }
            _ => {}
        }
    }
    Ok(declarations)
}
