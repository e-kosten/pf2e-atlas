//! Versioned, ephemeral source-backed search selection and passage recovery.
use super::{
    SourceBackedRecord, SourceContentOutcome, SourceContentRole, SourceContentStatus,
    SourceFieldView, SourceNodeView, SourceQueryView,
};
use super::{content::select_content, traversal::visit_source_nodes};
use crate::source_content::{ContentAudience, OwnedContentLocator, SourceContentLocator};
use atlas_domain::{RecordKey, SourceByteRange, SourcePassageAddress};
use scraper::{ElementRef, Html};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fmt};

pub const SOURCE_CONTENT_SELECTION_VERSION: &str = "structural-source-sections/v2";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceSelectedSectionKind {
    Unlabeled,
    Heading,
    DefinitionLabel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSelectedSection {
    pub kind: SourceSelectedSectionKind,
    pub section_ordinal: usize,
    pub label: Option<String>,
    pub heading_chain: Vec<String>,
    pub text: String,
}
impl SourceSelectedSection {
    /// Only substantive headings and leading definition labels form lexical units.
    pub fn is_lexical_definition(&self) -> bool {
        self.kind != SourceSelectedSectionKind::Unlabeled
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSelectedField {
    pub locator: SourceContentLocator,
    pub role: SourceContentRole,
    pub prepared_html_sha256: Option<String>,
    pub sections: Vec<SourceSelectedSection>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSelectedIdentity {
    pub owners: Vec<OwnedContentLocator>,
    pub field: Option<String>,
    pub name: String,
    pub vocabulary: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSearchSelection {
    pub root: RecordKey,
    pub pack_label: String,
    pub identities: Vec<SourceSelectedIdentity>,
    pub fields: Vec<SourceSelectedField>,
    /// Kept separate so the embedding owner can bound optional vocabulary.
    pub identity_required: String,
    pub identity_optional: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSelectionError(pub String);
impl fmt::Display for SourceSelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
impl std::error::Error for SourceSelectionError {}
fn hash(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}
fn plain(html: &str) -> Result<String, SourceSelectionError> {
    crate::source_content::prepared_plain_text(html)
        .map_err(|error| SourceSelectionError(error.to_string()))
}

/// Select admitted values and compatible prepared HTML; unknown visibility,
/// unavailable formats and failed fields have no fabricated search text.
pub fn select_record_search(
    record: &SourceBackedRecord,
    content: &[SourceContentOutcome],
    audience: ContentAudience,
    pack_label: &str,
    trait_labels: &BTreeMap<String, String>,
) -> Result<SourceSearchSelection, SourceSelectionError> {
    let root_node = SourceNodeView::from(record.source());
    let mut result = SourceSearchSelection {
        root: record.key().clone(),
        pack_label: pack_label.into(),
        identities: vec![],
        fields: vec![],
        identity_required: String::new(),
        identity_optional: vec![],
    };
    if matches!(root_node, SourceNodeView::Macro(_)) {
        return Ok(result);
    }
    visit_source_nodes(record.source(), |owners, node| {
        let query = SourceQueryView {
            source: node,
            pack_id: record.key().pack().as_str(),
            pack_label,
        };
        let mut vocabulary = Vec::new();
        if let Some(family) = node.source_type().value() {
            vocabulary.push(family.to_owned());
        }
        if owners.is_empty() && !pack_label.is_empty() {
            vocabulary.push(pack_label.to_owned());
        }
        if let Some(publication) = query.publication_title().value() {
            vocabulary.push(publication.to_owned());
        }
        if let Some(traits) = query.traits().value() {
            for identifier in traits {
                vocabulary.push(identifier.clone());
                if let Some(label) = trait_labels.get(identifier) {
                    vocabulary.push(label.clone());
                }
            }
        }
        let semantic_vocabulary = vocabulary.clone();
        if let Some(rarity) = query.rarity().value() {
            use atlas_foundry_model::generated::ItemTraitsItemTraitRarity::*;
            vocabulary.push(
                match rarity {
                    Common => "common",
                    Uncommon => "uncommon",
                    Rare => "rare",
                    Unique => "unique",
                }
                .into(),
            );
        }
        if let SourceNodeView::Item(item) = node
            && let Some(traditions) = item.spell_traditions().value()
        {
            use atlas_foundry_model::generated::PatchSpellOverlayOverrideSystemTraitsTraditionsEntry::*;
            vocabulary.extend(traditions.iter().map(|tradition| {
                match tradition {
                    Arcane => "arcane",
                    Divine => "divine",
                    Occult => "occult",
                    Primal => "primal",
                }
                .into()
            }));
        }
        vocabulary.sort();
        vocabulary.dedup();
        let visible_name = select_content(node)
            .into_iter()
            .find(|selected| selected.role == SourceContentRole::Name)
            .is_some_and(|selected| {
                selected
                    .visibility
                    .value()
                    .is_some_and(|visibility| audience.permits(visibility))
            });
        if let Some(name) = node.name().value().filter(|_| visible_name) {
            result.identities.push(SourceSelectedIdentity {
                owners: owners.to_vec(),
                field: Some("/name".into()),
                name: name.to_owned(),
                vocabulary: vocabulary.clone(),
            });
        } else if owners.is_empty() && visible_name {
            result.identities.push(SourceSelectedIdentity {
                owners: vec![],
                field: None,
                name: record.key().to_string(),
                vocabulary: vocabulary.clone(),
            });
        }
        if owners.is_empty() {
            let name = node
                .name()
                .value()
                .cloned()
                .unwrap_or_else(|| record.key().to_string());
            let family = node.source_type().value().unwrap_or(match node {
                SourceNodeView::Actor(_) => "Actor",
                SourceNodeView::Item(_) => "Item",
                SourceNodeView::Journal(_) => "JournalEntry",
                SourceNodeView::Table(_) => "RollTable",
                _ => "",
            });
            result.identity_required = name;
            result.identity_optional = [family, pack_label]
                .into_iter()
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .chain(semantic_vocabulary)
                .collect();
        }
    });
    let mut failure = None;
    visit_source_nodes(record.source(), |owners, node| {
        for selection in select_content(node) {
            if selection.role == SourceContentRole::Name
                || !matches!(
                    selection.format,
                    SourceFieldView::Value(super::SourceContentFormat::Plain)
                )
            {
                continue;
            }
            let (Some(text), Some(visibility)) =
                (selection.text.value(), selection.visibility.value())
            else {
                continue;
            };
            if !audience.permits(visibility) || text.trim().is_empty() {
                continue;
            }
            result.fields.push(SourceSelectedField {
                locator: SourceContentLocator {
                    record: record.key().clone(),
                    owners: owners.to_vec(),
                    field: selection.field.into(),
                },
                role: selection.role,
                prepared_html_sha256: None,
                sections: vec![SourceSelectedSection {
                    kind: SourceSelectedSectionKind::Unlabeled,
                    section_ordinal: 0,
                    label: None,
                    heading_chain: vec![],
                    text: text.to_owned(),
                }],
            });
        }
    });
    for outcome in content {
        let SourceContentStatus::Prepared(prepared) = &outcome.status else {
            continue;
        };
        if !audience.permits(outcome.visibility)
            || outcome.visibility_availability != super::FieldAvailability::Value
            || outcome.role == SourceContentRole::Name
        {
            continue;
        }
        if prepared.locator.record != *record.key()
            || record
                .authored_content_at(&prepared.locator)
                .value()
                .is_none_or(|markup| hash(markup) != prepared.authored_markup_sha256)
        {
            return Err(SourceSelectionError(
                "prepared content belongs to a different source field or snapshot".into(),
            ));
        }
        match select_prepared_html_sections(&prepared.html) {
            Ok(sections) if !sections.is_empty() => result.fields.push(SourceSelectedField {
                locator: prepared.locator.clone(),
                role: outcome.role,
                prepared_html_sha256: Some(hash(&prepared.html)),
                sections,
            }),
            Ok(_) => {}
            Err(error) => failure = Some(error),
        }
    }
    if let Some(error) = failure {
        return Err(error);
    }
    Ok(result)
}

fn generic_label(label: &str) -> bool {
    let lower = label.trim().trim_end_matches(':').to_lowercase();
    let base = lower
        .split_once('(')
        .map(|(base, _)| base.trim())
        .unwrap_or(&lower);
    [
        "trigger",
        "effect",
        "requirements",
        "requirement",
        "frequency",
        "saving throw",
        "duration",
        "onset",
        "maximum duration",
        "special",
        "critical success",
        "success",
        "failure",
        "critical failure",
        "activation",
        "activate",
        "cost",
        "sustain",
        "level",
        "rank",
    ]
    .contains(&base)
        || base == "stage"
        || base.starts_with("stage ")
        || base == "heightened"
        || base.starts_with("heightened ")
}
fn heading_level(tag: &str) -> Option<u8> {
    match tag {
        "h1" => Some(1),
        "h2" => Some(2),
        "h3" => Some(3),
        "h4" => Some(4),
        "h5" => Some(5),
        "h6" => Some(6),
        _ => None,
    }
}
fn leading_definition(element: ElementRef<'_>) -> Option<String> {
    if !matches!(element.value().name(), "p" | "li" | "dt") {
        return None;
    }
    let first = element.children().find(|node| {
        !node
            .value()
            .as_text()
            .is_some_and(|text| text.trim().is_empty())
    })?;
    let bold = ElementRef::wrap(first)?;
    if !matches!(bold.value().name(), "strong" | "b") {
        return None;
    }
    let label = bold.text().collect::<String>();
    let remainder = first
        .next_siblings()
        .map(|node| match node.value() {
            scraper::Node::Text(text) => text.to_string(),
            _ => String::new(),
        })
        .collect::<String>();
    let has_colon = label.trim_end().ends_with(':') || remainder.trim_start().starts_with(':');
    let label = label.trim().trim_end_matches(':').trim().to_owned();
    (has_colon && !label.is_empty() && !generic_label(&label)).then_some(label)
}

/// Partition prose in document order at substantive headings/definition labels.
/// Heading chains retain nested context; generic mechanics headings stay in the
/// current section. Every emitted block occurs in exactly one section.
pub fn select_prepared_html_sections(
    html: &str,
) -> Result<Vec<SourceSelectedSection>, SourceSelectionError> {
    let document = Html::parse_fragment(html);
    let mut blocks = Vec::new();
    collect_blocks(document.root_element(), &mut blocks);
    let mut output = Vec::new();
    let mut current_html = String::new();
    let mut label = None;
    let mut heading_chain: Vec<(u8, String)> = Vec::new();
    let mut current_chain = Vec::new();
    let mut kind = SourceSelectedSectionKind::Unlabeled;
    for block in blocks {
        let heading = block
            .element
            .and_then(|element| heading_level(element.value().name()).map(|level| (element, level)))
            .and_then(|(element, level)| {
                let label = element.text().collect::<String>().trim().to_owned();
                (!label.is_empty() && !generic_label(&label)).then_some((level, label))
            });
        let definition = block.element.and_then(leading_definition);
        if heading.is_some() || definition.is_some() {
            finish_section(
                &mut output,
                &mut current_html,
                label.take(),
                std::mem::take(&mut current_chain),
                kind,
            )?;
            if let Some((level, heading)) = heading {
                kind = SourceSelectedSectionKind::Heading;
                heading_chain.retain(|(ancestor, _)| *ancestor < level);
                heading_chain.push((level, heading.clone()));
                label = Some(heading);
            } else {
                kind = SourceSelectedSectionKind::DefinitionLabel;
                label = definition;
            }
            current_chain = heading_chain
                .iter()
                .map(|(_, label)| label.clone())
                .collect();
        }
        current_html.push_str(&block.html);
    }
    finish_section(&mut output, &mut current_html, label, current_chain, kind)?;
    if output.is_empty() {
        let text = plain(html)?;
        if !text.is_empty() {
            output.push(SourceSelectedSection {
                kind: SourceSelectedSectionKind::Unlabeled,
                section_ordinal: 0,
                label: None,
                heading_chain: vec![],
                text,
            });
        }
    }
    Ok(output)
}
struct HtmlBlock<'a> {
    element: Option<ElementRef<'a>>,
    html: String,
}
fn collect_blocks<'a>(element: ElementRef<'a>, output: &mut Vec<HtmlBlock<'a>>) {
    for child in element.children() {
        if let Some(element) = ElementRef::wrap(child) {
            if matches!(
                element.value().name(),
                "html"
                    | "body"
                    | "div"
                    | "section"
                    | "article"
                    | "main"
                    | "ul"
                    | "ol"
                    | "dl"
                    | "blockquote"
            ) {
                collect_blocks(element, output);
            } else {
                output.push(HtmlBlock {
                    html: element.html(),
                    element: Some(element),
                });
            }
        } else if let scraper::Node::Text(text) = child.value() {
            output.push(HtmlBlock {
                element: None,
                html: text
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;"),
            });
        }
    }
}
fn finish_section(
    output: &mut Vec<SourceSelectedSection>,
    html: &mut String,
    label: Option<String>,
    heading_chain: Vec<String>,
    kind: SourceSelectedSectionKind,
) -> Result<(), SourceSelectionError> {
    let text = plain(html)?;
    html.clear();
    if !text.is_empty() {
        output.push(SourceSelectedSection {
            kind,
            section_ordinal: output.len(),
            label,
            heading_chain,
            text,
        });
    }
    Ok(())
}

/// Reconstruct an addressed section with the same selection policy before
/// consuming a byte range. Source/prepared hashes prevent stale snippet use.
pub fn recover_source_passage(
    html: Option<&str>,
    plain_source: Option<&str>,
    address: &SourcePassageAddress,
) -> Result<String, SourceSelectionError> {
    match address {
        SourcePassageAddress::Identity {} => {
            Err(SourceSelectionError("identity has no prose passage".into()))
        }
        SourcePassageAddress::HtmlSection {
            prepared_html_sha256,
            canonical_text_sha256,
            selection_version,
            section_ordinal,
            label,
            chunk_bytes,
        } => {
            let html =
                html.ok_or_else(|| SourceSelectionError("prepared HTML is unavailable".into()))?;
            if selection_version != SOURCE_CONTENT_SELECTION_VERSION
                || hash(html) != *prepared_html_sha256
            {
                return Err(SourceSelectionError(
                    "HTML passage preparation/selection identity changed".into(),
                ));
            }
            let sections = select_prepared_html_sections(html)?;
            let section = sections.get(*section_ordinal).ok_or_else(|| {
                SourceSelectionError("HTML section ordinal is unavailable".into())
            })?;
            if hash(&section.text) != *canonical_text_sha256 || section.label != *label {
                return Err(SourceSelectionError(
                    "HTML section source identity changed".into(),
                ));
            }
            extract(&section.text, *chunk_bytes)
        }
        SourcePassageAddress::PlainSection {
            source_text_sha256,
            selection_version,
            section_ordinal,
            label,
            chunk_bytes,
        } => {
            let text = plain_source
                .ok_or_else(|| SourceSelectionError("plain source is unavailable".into()))?;
            if selection_version != SOURCE_CONTENT_SELECTION_VERSION
                || *section_ordinal != 0
                || label.is_some()
                || hash(text) != *source_text_sha256
            {
                return Err(SourceSelectionError(
                    "plain passage source/selection identity changed".into(),
                ));
            }
            extract(text, *chunk_bytes)
        }
    }
}
fn extract(text: &str, range: SourceByteRange) -> Result<String, SourceSelectionError> {
    range
        .extract(text)
        .map(str::to_owned)
        .map_err(|error| SourceSelectionError(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_content::{ContentVisibilityRule, validate_prepared_markers};
    use crate::source_record::prepare_record_content;
    use atlas_foundry_model::{SourceContext, admit_document_source};
    fn audience() -> ContentAudience {
        ContentAudience {
            include_gm: true,
            include_owner: true,
            implicit_check_dc: ContentVisibilityRule::Gm,
        }
    }
    fn make_record(kind: &str, pack: &str, json: serde_json::Value) -> SourceBackedRecord {
        let bytes = serde_json::to_vec(&json).unwrap();
        let source =
            admit_document_source(kind, SourceContext::new("fixture", "test", "$"), &bytes)
                .unwrap()
                .model
                .unwrap();
        SourceBackedRecord::new(pack, source).unwrap()
    }
    #[test]
    fn sections_preserve_nested_context_all_prose_and_duplicate_titles() {
        let html = "bare leading prose<div><h2>Soul Degradation (Curse 7)</h2><p>The PC's sense of self fades.</p><p><strong>Saving Throw</strong>: DC 23 Fortitude</p><p><strong>Stage 1</strong>: one</p><p><strong>Stage 6</strong>: final stage</p><h3>Recovery</h3><p>Recovery details</p></div><p><b>Special Ability:</b> definition body</p><h2>Recovery</h2><p>Different definition</p>tail prose";
        let sections = select_prepared_html_sections(html).unwrap();
        assert_eq!(sections.len(), 5);
        assert!(sections[0].text.contains("bare leading prose"));
        assert!(sections[1].text.contains("Stage 6"));
        assert!(sections[1].text.contains("DC 23"));
        assert_eq!(
            sections[2].heading_chain,
            vec!["Soul Degradation (Curse 7)", "Recovery"]
        );
        assert!(sections[4].text.contains("tail prose"));
        assert_ne!(sections[2].section_ordinal, sections[4].section_ordinal);
        assert_eq!(sections[3].label.as_deref(), Some("Special Ability"));
        let joined = sections
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        for phrase in [
            "bare leading prose",
            "sense of self",
            "Stage 1",
            "Stage 6",
            "Recovery details",
            "definition body",
            "Different definition",
            "tail prose",
        ] {
            assert!(joined.contains(phrase), "{phrase}");
        }
    }
    #[test]
    fn lexical_vocabulary_contains_typed_rarity_and_traditions_without_expanding_semantic_identity()
    {
        let record = make_record(
            "Item",
            "spells",
            serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","name":"Fire Story","type":"spell","system":{"traits":{"value":["fire","new-open-trait"],"rarity":"rare","traditions":["divine"]},"description":{"value":"<p>Explanatory prose</p>"}}}),
        );
        let content = prepare_record_content(&record, audience(), None, None);
        let selection =
            select_record_search(&record, &content, audience(), "Spells", &BTreeMap::new())
                .unwrap();
        let vocabulary = &selection.identities[0].vocabulary;
        for term in ["rare", "divine", "fire", "new-open-trait"] {
            assert!(vocabulary.iter().any(|value| value == term));
        }
        assert!(
            !selection
                .identity_optional
                .iter()
                .any(|value| value == "rare" || value == "divine")
        );
        let sections = select_prepared_html_sections(
            "<ul><li><strong>Named Ability:</strong> complete body tail</li></ul>",
        )
        .unwrap();
        assert_eq!(sections[0].kind, SourceSelectedSectionKind::DefinitionLabel);
        assert!(sections[0].text.ends_with("tail"));
    }
    #[test]
    fn borrowed_authored_content_rejects_foreign_unknown_and_stale_addresses() {
        let record = make_record(
            "Item",
            "test",
            serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","name":"Test","type":"effect","system":{"description":{"value":"<h2>Valid</h2><p>Body</p>"}}}),
        );
        let content = prepare_record_content(&record, audience(), None, None);
        let locator = content[0].locator().clone();
        assert_eq!(
            record.authored_content_at(&locator).value(),
            Some("<h2>Valid</h2><p>Body</p>")
        );
        let mut wrong = locator.clone();
        wrong.record = "other:aaaaaaaaaaaaaaaa".parse().unwrap();
        assert!(matches!(
            record.authored_content_at(&wrong),
            SourceFieldView::NotApplicable
        ));
        wrong = locator.clone();
        wrong.field = "/system/arbitrary/additional".into();
        assert!(matches!(
            record.authored_content_at(&wrong),
            SourceFieldView::NotApplicable
        ));
        let mut stale = content;
        let SourceContentStatus::Prepared(prepared) = &mut stale[0].status else {
            panic!("expected prepared content")
        };
        prepared.authored_markup_sha256 = hash("another source");
        assert!(
            select_record_search(&record, &stale, audience(), "Test", &BTreeMap::new()).is_err()
        );
    }
    #[test]
    fn passage_recovery_checks_hashes_versions_utf8_and_section_ordinals() {
        let html = "<h2>Title</h2><p>Body 🐉 tail</p>";
        let section = select_prepared_html_sections(html).unwrap().remove(0);
        let address = SourcePassageAddress::HtmlSection {
            prepared_html_sha256: hash(html),
            canonical_text_sha256: hash(&section.text),
            selection_version: SOURCE_CONTENT_SELECTION_VERSION.into(),
            section_ordinal: 0,
            label: section.label,
            chunk_bytes: SourceByteRange {
                start: 0,
                end: section.text.len(),
            },
        };
        assert_eq!(
            recover_source_passage(Some(html), None, &address).unwrap(),
            section.text
        );
        assert!(recover_source_passage(Some("<p>changed</p>"), None, &address).is_err());
        assert!(recover_source_passage(None, None, &address).is_err());
        let SourcePassageAddress::HtmlSection { chunk_bytes, .. } = &address else {
            panic!("expected HTML")
        };
        let mut bad = address.clone();
        if let SourcePassageAddress::HtmlSection {
            section_ordinal, ..
        } = &mut bad
        {
            *section_ordinal = 99;
        }
        assert!(recover_source_passage(Some(html), None, &bad).is_err());
        let mut bad = address.clone();
        if let SourcePassageAddress::HtmlSection {
            chunk_bytes: range, ..
        } = &mut bad
        {
            range.end = section.text.find('🐉').unwrap() + 1;
        }
        assert!(recover_source_passage(Some(html), None, &bad).is_err());
        assert!(chunk_bytes.end > 0);
    }
    #[test]
    fn known_empty_unknown_visibility_macros_and_partial_source_remain_distinct() {
        let record = make_record(
            "Item",
            "test",
            serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","name":"Test","type":"effect","system":{"description":{"value":"<p>Public body</p>","gm":"<h2>Hidden Definition</h2><p>Private body</p>"}}}),
        );
        let mut content = prepare_record_content(&record, audience(), None, None);
        content[1].visibility_availability = super::super::FieldAvailability::Missing;
        let selection =
            select_record_search(&record, &content, audience(), "Test", &BTreeMap::new()).unwrap();
        assert_eq!(selection.fields.len(), 1);
        assert_eq!(selection.identity_required, "Test");
        for outcome in content {
            if let SourceContentStatus::Prepared(content) = outcome.status {
                validate_prepared_markers(&content.html, &content.interactions).unwrap();
            }
        }
        let macro_record = make_record(
            "Macro",
            "test",
            serde_json::json!({"_id":"bbbbbbbbbbbbbbbb","name":"Helper","type":"script","command":"alert('no')"}),
        );
        let selection =
            select_record_search(&macro_record, &[], audience(), "Test", &BTreeMap::new()).unwrap();
        assert!(
            selection.identities.is_empty()
                && selection.fields.is_empty()
                && selection.identity_required.is_empty()
        );
    }
    #[test]
    #[ignore = "requires the pinned PF2e source export; run explicitly"]
    fn actual_owned_ghoul_fever_and_gm_soul_degradation_have_source_navigation() {
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .map(|parent| parent.join("scratch/source-contracts-full/pf2e"))
            .find(|path| path.join("packs/pathfinder-bestiary/ghoul.json").is_file())
            .expect("pinned source export required");
        for (kind, pack, file, expected) in [
            (
                "Actor",
                "pathfinder-bestiary",
                "packs/pathfinder-bestiary/ghoul.json",
                "Ghoul Fever",
            ),
            (
                "Item",
                "campaign-effects",
                "packs/campaign-effects/season-of-ghosts/effect-spirit-powers.json",
                "Soul Degradation",
            ),
        ] {
            let bytes = std::fs::read(source.join(file)).unwrap();
            let dto = admit_document_source(kind, SourceContext::new("corpus", file, "$"), &bytes)
                .unwrap()
                .model
                .unwrap();
            let record = SourceBackedRecord::new(pack, dto).unwrap();
            let content = prepare_record_content(&record, audience(), None, None);
            let selection =
                select_record_search(&record, &content, audience(), pack, &BTreeMap::new())
                    .unwrap();
            if expected == "Ghoul Fever" {
                let identity = selection
                    .identities
                    .iter()
                    .find(|identity| identity.name == expected)
                    .unwrap();
                assert!(!identity.owners.is_empty());
                assert!(
                    selection
                        .fields
                        .iter()
                        .filter(|field| field.locator.owners == identity.owners)
                        .flat_map(|field| &field.sections)
                        .any(|section| section.text.contains("Stage 1")
                            && section.text.contains("DC 15"))
                );
            } else {
                let field = selection
                    .fields
                    .iter()
                    .find(|field| {
                        field.sections.iter().any(|section| {
                            section
                                .label
                                .as_deref()
                                .is_some_and(|label| label.starts_with(expected))
                                && section.text.contains("Stage 6")
                        })
                    })
                    .unwrap();
                assert_eq!(field.locator.field, "/system/description/gm");
                let section = field
                    .sections
                    .iter()
                    .find(|section| section.text.contains("Stage 6"))
                    .unwrap();
                assert!(section.text.contains("DC 23"));
                let address = SourcePassageAddress::HtmlSection {
                    prepared_html_sha256: field.prepared_html_sha256.clone().unwrap(),
                    canonical_text_sha256: hash(&section.text),
                    selection_version: SOURCE_CONTENT_SELECTION_VERSION.into(),
                    section_ordinal: section.section_ordinal,
                    label: section.label.clone(),
                    chunk_bytes: SourceByteRange {
                        start: 0,
                        end: section.text.len(),
                    },
                };
                let prepared = content
                    .iter()
                    .find(|outcome| outcome.locator() == &field.locator)
                    .unwrap();
                let SourceContentStatus::Prepared(prepared) = &prepared.status else {
                    panic!("expected prepared")
                };
                assert_eq!(
                    recover_source_passage(Some(&prepared.html), None, &address).unwrap(),
                    section.text
                );
            }
        }
    }
}
