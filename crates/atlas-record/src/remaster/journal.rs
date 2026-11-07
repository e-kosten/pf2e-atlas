use super::*;
use crate::source_content::{
    ContentReferenceResolution, ContentReferenceTarget, PreparedSourceContent, prepared_plain_text,
};
use scraper::{ElementRef, Html, Selector};

fn selector(value: &str) -> Result<Selector, String> {
    Selector::parse(value).map_err(|e| e.to_string())
}
fn plain(element: ElementRef<'_>) -> Result<String, String> {
    prepared_plain_text(&element.html()).map_err(|e| e.to_string())
}
fn direct_references(element: ElementRef<'_>) -> bool {
    fn residual(element: ElementRef<'_>, out: &mut String) {
        if element.value().attr("data-atlas-reference").is_some() {
            return;
        }
        for node in element.children() {
            if let Some(child) = ElementRef::wrap(node) {
                residual(child, out);
            } else if let scraper::node::Node::Text(text) = node.value() {
                out.push_str(&text.text);
            }
        }
    }
    let mut text = String::new();
    residual(element, &mut text);
    text.to_ascii_lowercase()
        .split(|c: char| !c.is_alphabetic())
        .all(|word| matches!(word, "" | "and" | "or"))
}
fn targets(
    element: ElementRef<'_>,
    content: &PreparedSourceContent,
) -> Result<Vec<RemasterTarget>, String> {
    let mut result = Vec::new();
    for element in element.select(&selector("[data-atlas-reference]")?) {
        let Some(ordinal) = element
            .value()
            .attr("data-atlas-reference")
            .and_then(|v| v.parse::<usize>().ok())
        else {
            continue;
        };
        let Some(reference) = content
            .references
            .iter()
            .find(|r| r.ordinal == ordinal && r.visible)
        else {
            continue;
        };
        let (record, owners) = match &reference.resolution {
            ContentReferenceResolution::Resolved(ContentReferenceTarget::Record { key }) => {
                (Some(key.clone()), vec![])
            }
            ContentReferenceResolution::Resolved(ContentReferenceTarget::OwnedNode {
                key,
                owners,
            }) => (Some(key.clone()), owners.clone()),
            _ => (None, vec![]),
        };
        let name = if reference.label.explicit {
            plain(element)?
        } else {
            [".Item.", ".Actor.", ".JournalEntryPage.", ".JournalEntry."]
                .iter()
                .find_map(|m| {
                    reference
                        .authored_target
                        .rsplit_once(m)
                        .map(|(_, tail)| tail)
                })
                .unwrap_or(&reference.authored_target)
                .to_owned()
        };
        result.push(RemasterTarget {
            name,
            record,
            owners,
        });
    }
    if result.is_empty() {
        let name = plain(element)?;
        if !name.is_empty() && name != "—" {
            result.push(RemasterTarget {
                name,
                record: None,
                owners: vec![],
            });
        }
    } else if !direct_references(element) {
        fn flush(text: &mut String, parts: &mut Vec<RemasterTarget>) {
            let name = text
                .trim()
                .trim_matches(|c: char| matches!(c, ',' | '(' | ')'))
                .trim();
            let name = name
                .strip_suffix(" and")
                .or_else(|| name.strip_suffix(" or"))
                .unwrap_or(name)
                .trim();
            let name = name
                .strip_prefix("and ")
                .or_else(|| name.strip_prefix("or "))
                .unwrap_or(name)
                .trim();
            if !name.is_empty() && !matches!(name, "and" | "or") {
                parts.push(RemasterTarget {
                    name: name.into(),
                    record: None,
                    owners: vec![],
                });
            }
            text.clear();
        }
        fn walk(
            element: ElementRef<'_>,
            refs: &mut std::vec::IntoIter<RemasterTarget>,
            parts: &mut Vec<RemasterTarget>,
            text: &mut String,
        ) {
            if element.value().attr("data-atlas-reference").is_some() {
                flush(text, parts);
                if let Some(target) = refs.next() {
                    parts.push(target);
                }
                return;
            }
            for node in element.children() {
                if let Some(child) = ElementRef::wrap(node) {
                    walk(child, refs, parts, text);
                } else if let scraper::node::Node::Text(value) = node.value() {
                    text.push_str(&value.text);
                }
            }
        }
        let mut refs = result.into_iter();
        let mut parts = vec![];
        let mut text = String::new();
        walk(element, &mut refs, &mut parts, &mut text);
        flush(&mut text, &mut parts);
        result = parts;
    }
    Ok(result)
}
fn add(
    result: &mut Vec<RecordRemasterContext>,
    old: Vec<RemasterTarget>,
    new: Vec<RemasterTarget>,
    relation: RemasterRelation,
    scope: &str,
    content: &PreparedSourceContent,
) {
    let reverse = match relation {
        RemasterRelation::FormerName => RemasterRelation::RenamedTo,
        RemasterRelation::MergedFrom => RemasterRelation::MergedInto,
        _ => RemasterRelation::ReplacedBy,
    };
    for (anchors, targets, relation) in [(&new, &old, relation), (&old, &new, reverse)] {
        for anchor in anchors {
            if let Some(record) = &anchor.record {
                let targets = targets
                    .iter()
                    .filter(|target| {
                        target.record.as_ref() != Some(record) || target.owners != anchor.owners
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                if !targets.is_empty() {
                    result.push(RecordRemasterContext {
                        record: record.clone(),
                        owners: anchor.owners.clone(),
                        relation,
                        targets,
                        journal: Some(content.locator.clone()),
                        scope: scope.into(),
                    });
                }
            }
        }
    }
}
pub fn journal_context(
    section: &str,
    content: &PreparedSourceContent,
) -> Result<Vec<RecordRemasterContext>, String> {
    let html = Html::parse_fragment(&content.html);
    let mut result = Vec::new();
    let cells = selector("td")?;
    for row in html.select(&selector("tr")?) {
        let values = row.select(&cells).collect::<Vec<_>>();
        let Some(table) = row
            .ancestors()
            .filter_map(ElementRef::wrap)
            .find(|e| e.value().name() == "table")
        else {
            continue;
        };
        let headers = table
            .select(&selector("th")?)
            .map(plain)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|s| {
                s.to_ascii_lowercase()
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric())
                    .collect::<String>()
            })
            .collect::<Vec<_>>();
        if headers.len() != values.len()
            || !headers
                .first()
                .is_some_and(|v| matches!(v.as_str(), "old" | "oldname" | "itemname" | "spellname"))
            || !headers
                .last()
                .is_some_and(|v| matches!(v.as_str(), "new" | "newname"))
        {
            continue;
        }
        let (status, qualifier) = match headers.as_slice() {
            [_, _] => ("renamed".into(), String::new()),
            [_, s, _] if s == "status" => (plain(values[1])?, String::new()),
            [_, q, s, _] if matches!(q.as_str(), "class" | "maintrait") && s == "status" => {
                (plain(values[2])?, plain(values[1])?)
            }
            _ => continue,
        };
        let status = status.to_ascii_lowercase();
        let has = |kind: &str| status.split(',').any(|token| token.trim() == kind);
        let relation = if has("replaced") {
            RemasterRelation::Replaces
        } else if has("merged") {
            RemasterRelation::MergedFrom
        } else if has("rename") || has("renamed") {
            RemasterRelation::FormerName
        } else {
            continue;
        };
        let Some(last) = values.last() else { continue };
        if !direct_references(*last) {
            continue;
        }
        let old = targets(values[0], content)?;
        let new = targets(*last, content)?;
        if new.len() != 1 {
            continue;
        }
        add(
            &mut result,
            old,
            new,
            relation,
            &format!("{section}:{qualifier}"),
            content,
        );
    }
    for item in html.select(&selector("li")?) {
        let value = plain(item)?;
        let lower = value.to_ascii_lowercase();
        let split = [
            " are merged into ",
            " is merged into ",
            " are now ",
            " is now ",
        ]
        .iter()
        .find_map(|d| lower.find(d).map(|i| (i, *d)));
        let Some((at, delimiter)) = split else {
            continue;
        };
        let old = value[..at]
            .replace(" and ", ", ")
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|name| RemasterTarget {
                name: name.into(),
                record: None,
                owners: vec![],
            })
            .collect();
        let new = targets(item, content)?
            .into_iter()
            .filter(|t| t.record.is_some())
            .collect::<Vec<_>>();
        if !new.is_empty() {
            add(
                &mut result,
                old,
                new,
                if delimiter.contains("merged") {
                    RemasterRelation::MergedFrom
                } else {
                    RemasterRelation::FormerName
                },
                section,
                content,
            );
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        source_content::{ContentAudience, ContentVisibilityRule},
        source_record::{
            SourceBackedRecord, SourceContentStatus, SourceReferenceIndex, prepare_record_content,
        },
    };
    fn record(pack: &str, kind: &str, value: serde_json::Value) -> SourceBackedRecord {
        let source = atlas_foundry_model::admit_document_source(
            kind,
            atlas_foundry_model::SourceContext::new(pack, "fixture.json", "$"),
            value.to_string().as_bytes(),
        )
        .unwrap()
        .model
        .unwrap();
        SourceBackedRecord::new(pack, source).unwrap()
    }
    fn extract(markup: &str) -> Vec<RecordRemasterContext> {
        let new = record(
            "items",
            "Item",
            serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","name":"New name","type":"action"}),
        );
        let journal = record(
            "journals",
            "JournalEntry",
            serde_json::json!({"_id":"bbbbbbbbbbbbbbbb","name":"Remaster Changes","pages":[{"_id":"cccccccccccccccc","name":"Class Features","type":"text","text":{"format":1,"content":markup}}]}),
        );
        let mut index = SourceReferenceIndex::default();
        index.insert_source(new.key(), new.source());
        let old = record(
            "items",
            "Item",
            serde_json::json!({"_id":"dddddddddddddddd","name":"Old B","type":"action"}),
        );
        index.insert_source(old.key(), old.source());
        let audience = ContentAudience {
            include_gm: false,
            include_owner: false,
            implicit_check_dc: ContentVisibilityRule::None,
        };
        let outcomes = prepare_record_content(&journal, audience, None, Some(&index));
        outcomes
            .into_iter()
            .filter_map(|o| {
                if let SourceContentStatus::Prepared(content) = o.status {
                    Some(journal_context("Class Features", &content).unwrap())
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }
    const TARGET: &str = "@UUID[Compendium.pf2e.items.Item.aaaaaaaaaaaaaaaa]{New name}";
    #[test]
    fn selected_direct_rename_keeps_plain_old_name_and_journal_without_qualifier_copy() {
        let rows = extract(&format!(
            "<table><tr><th>Old</th><th>Class</th><th>Status</th><th>New</th></tr><tr><td>Old name</td><td>Runelord Class Archetype</td><td>Renamed, altered mechanics</td><td>{TARGET}</td></tr><tr data-visibility='gm'><td>Secret name</td><td>Hidden</td><td>Renamed</td><td>{TARGET}</td></tr></table>"
        ));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].relation, RemasterRelation::FormerName);
        assert_eq!(rows[0].targets[0].name, "Old name");
        assert!(rows[0].targets[0].record.is_none());
        assert!(rows[0].journal.is_some());
        assert_eq!(rows[0].scope, "Class Features:Runelord Class Archetype");
    }
    #[test]
    fn prose_references_and_collective_destinations_are_not_individual_renames() {
        for new in [
            format!("Removed - see {TARGET}"),
            format!("See {TARGET}"),
            format!("Marvelous Miniatures ({TARGET}, {TARGET})"),
            format!("{TARGET} and {TARGET}"),
        ] {
            assert!(extract(&format!("<table><tr><th>Old</th><th>New</th></tr><tr><td>Old group</td><td>{new}</td></tr></table>")).is_empty(),"{new}");
        }
    }
    #[test]
    fn heritage_merge_keeps_all_predecessors_once() {
        let rows = extract(&format!(
            "<ul><li>Aasimar, Aphorite, Ganzi, and Tiefling are merged into {TARGET}.</li></ul>"
        ));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].relation, RemasterRelation::MergedFrom);
        assert_eq!(
            rows[0]
                .targets
                .iter()
                .map(|t| t.name.as_str())
                .collect::<Vec<_>>(),
            ["Aasimar", "Aphorite", "Ganzi", "Tiefling"]
        );
    }
    #[test]
    fn mixed_plain_and_linked_predecessors_keep_authored_order_and_exact_identity() {
        let rows = extract(&format!(
            "<table><tr><th>Old</th><th>New</th></tr><tr><td>Old A and @UUID[Compendium.pf2e.items.Item.dddddddddddddddd]{{Old B}}</td><td>{TARGET}</td></tr></table>"
        ));
        let row = rows
            .iter()
            .find(|c| c.relation == RemasterRelation::FormerName)
            .unwrap();
        assert_eq!(
            row.targets
                .iter()
                .map(|t| t.name.as_str())
                .collect::<Vec<_>>(),
            ["Old A", "Old B"]
        );
        assert!(row.targets[0].record.is_none());
        assert_eq!(
            row.targets[1].record.as_ref().unwrap().to_string(),
            "items:dddddddddddddddd"
        );
    }
    #[test]
    fn singular_rename_status_preserves_the_pinned_nocturnal_grippli_alias() {
        let rows = extract(&format!(
            "<table><tr><th>Old</th><th>Main Trait</th><th>Status</th><th>New</th></tr><tr><td>Nocturnal Grippli</td><td>Grippli</td><td>Rename</td><td>{TARGET}</td></tr></table>"
        ));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].relation, RemasterRelation::FormerName);
        assert_eq!(rows[0].targets[0].name, "Nocturnal Grippli");
    }
}
