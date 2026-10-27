//! Same-snapshot, bounded reading composition of authored Item inclusions.
use crate::{AppServiceResult, prepared_content::prepared_fields, projection::relationship_views};
use atlas_app_model::*;
use atlas_record::source_content::{
    ContentReferenceKind, ContentReferenceResolution, ContentReferenceTarget,
    PreparedHtmlInclusion, PreparedInclusionSourceLink, SourceContentLocator,
    compose_prepared_inclusions, escape_content_text as escape, terminal_prepared_reference,
};
use atlas_record::source_record::{
    InlineEmbedOptions, ItemSourceView, SourceContentRole, SourceFieldView, SourceNodeView,
    inline_embed_fields,
};
use atlas_search::{AtlasRetrievalService, GetRecordRequest, SourceStoredRelationship};
use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

const INCLUSION_LIMIT: usize = 64;
const DEPTH_LIMIT: usize = 4;
const HTML_LIMIT: usize = 256 * 1024;
const FIELD_LIMIT: usize = 256;
const REFERENCE_LIMIT: usize = 1024;

pub(crate) fn compose_inline_content<'a>(
    r: &AtlasRetrievalService,
    fields: impl Iterator<Item = &'a mut PreparedContentFieldView>,
    relationships: &[SourceStoredRelationship],
) -> AppServiceResult<()> {
    let fields: Vec<_> = fields.collect();
    let mut composer = Composer {
        r,
        count: 0,
        namespace: 0,
        bytes: fields
            .iter()
            .map(|f| match &f.body {
                PreparedFieldBodyView::Html { html, .. } => html.len(),
                _ => 0,
            })
            .sum(),
        fields: fields.len(),
        references: relationships.len(),
        records: HashMap::new(),
    };
    for field in fields {
        let locator = SourceContentLocator {
            record: field.locator.record.clone(),
            owners: field.locator.owners.clone(),
            field: field.locator.field.clone(),
        };
        let refs: Vec<_> = relationships
            .iter()
            .filter(|e| e.locator == locator)
            .cloned()
            .collect();
        let mut active = vec![(locator.record.to_string(), locator.owners.clone())];
        if let PreparedFieldBodyView::Html {
            html, inclusions, ..
        } = &mut field.body
        {
            *html = composer.expand(html, &refs, &mut active, inclusions)?;
        }
    }
    Ok(())
}
struct Composer<'a> {
    r: &'a AtlasRetrievalService,
    count: usize,
    namespace: usize,
    bytes: usize,
    fields: usize,
    references: usize,
    records: HashMap<String, Rc<atlas_search::SourceRecordDetail>>,
}
impl Composer<'_> {
    fn expand(
        &mut self,
        html: &str,
        references: &[SourceStoredRelationship],
        active: &mut Vec<(String, Vec<atlas_domain::OwnedContentLocator>)>,
        bindings: &mut Vec<PreparedInclusionView>,
    ) -> AppServiceResult<String> {
        let mut insertions = BTreeMap::new();
        for reference in references {
            let Some(ContentReferenceKind::Embed { options }) = &reference.details.reference_kind
            else {
                continue;
            };
            let options = if self.count >= INCLUSION_LIMIT
                || active.len() > DEPTH_LIMIT
                || self.bytes >= HTML_LIMIT
            {
                Err("More content is available in the source record.")
            } else {
                InlineEmbedOptions::parse(options)
            };
            self.count += 1;
            let insertion = match options {
                Err(reason) => Err(reason),
                Ok(options) => self.insert(
                    reference,
                    options,
                    references
                        .iter()
                        .filter(|other| {
                            other.resolution == reference.resolution
                                && matches!(
                                    other.details.reference_kind,
                                    Some(
                                        ContentReferenceKind::Uuid
                                            | ContentReferenceKind::Compendium
                                            | ContentReferenceKind::HtmlLink
                                    )
                                )
                        })
                        .map(|other| other.ordinal)
                        .collect(),
                    active,
                    bindings,
                )?,
            };
            let html = match insertion {
                Ok(html) => html,
                Err(reason) => PreparedHtmlInclusion {
                    source_link: None,
                    html: format!(
                        "<section class=\"prepared-inclusion-unavailable\"><p>{} <span data-atlas-reference=\"{}\">Open reference</span></p></section>",
                        escape(reason),
                        reference.ordinal
                    ),
                },
            };
            insertions.insert(reference.ordinal, html);
        }
        Ok(if insertions.is_empty() {
            html.to_owned()
        } else {
            compose_prepared_inclusions(html, &insertions, false)
        })
    }
    fn insert(
        &mut self,
        reference: &SourceStoredRelationship,
        options: InlineEmbedOptions,
        matching_references: Vec<usize>,
        active: &mut Vec<(String, Vec<atlas_domain::OwnedContentLocator>)>,
        bindings: &mut Vec<PreparedInclusionView>,
    ) -> AppServiceResult<Result<PreparedHtmlInclusion, &'static str>> {
        let (key, owners) = match &reference.resolution {
            ContentReferenceResolution::Resolved(ContentReferenceTarget::Record { key }) => {
                (key, vec![])
            }
            ContentReferenceResolution::Resolved(ContentReferenceTarget::OwnedNode {
                key,
                owners,
            }) => (key, owners.clone()),
            _ => return Ok(Err("Included content is unavailable.")),
        };
        let identity = (key.to_string(), owners.clone());
        if active.contains(&identity) {
            return Ok(Err(
                "This content refers back to a record already included here.",
            ));
        }
        let record = if let Some(record) = self.records.get(&key.to_string()) {
            record.clone()
        } else {
            let Some(record) = self.r.get_record(GetRecordRequest {
                record_key: key,
                selected_content: &[],
            })?
            else {
                return Ok(Err("Included content is unavailable."));
            };
            let record = Rc::new(record);
            self.records.insert(key.to_string(), record.clone());
            record
        };
        let Some(node) = record.source.node_at(&owners) else {
            return Ok(Err("Included content is unavailable."));
        };
        let Some(selected) = inline_embed_fields(node) else {
            return Ok(Err("This record cannot be included inline."));
        };
        let selected: Vec<_> = selected
            .into_iter()
            .filter(|s| {
                s.role != SourceContentRole::Prerequisite
                    || !matches!(s.text, SourceFieldView::Missing | SourceFieldView::Null)
                        && !s.text.value().is_some_and(|v| v.trim().is_empty())
            })
            .collect();
        if selected.is_empty() {
            return Ok(Err("Included content is unavailable."));
        }
        if self.fields + selected.len() > FIELD_LIMIT {
            return Ok(Err("More content is available in the source record."));
        }
        self.fields += selected.len();
        let locators: Vec<_> = selected
            .iter()
            .filter(|s| matches!(s.text, SourceFieldView::Value(_)))
            .map(|s| SourceContentLocator {
                record: key.clone(),
                owners: owners.clone(),
                field: s.field.to_string(),
            })
            .collect();
        let content = self.r.read_attributed_content(&locators)?;
        if self.references + content.relationships.len() > REFERENCE_LIMIT {
            return Ok(Err("More content is available in the source record."));
        }
        self.references += content.relationships.len();
        let mut fields = prepared_fields(
            key,
            &owners,
            &selected,
            &content.content.fields,
            &content.bounded,
            self.r.artifact_context().audience,
            &crate::prepared_content::PreparedContentContext::source(
                self.r.source_fingerprint(),
                &record.source,
                &owners,
                None,
            ),
        )?;
        let added_bytes: usize = fields
            .iter()
            .map(|f| match &f.body {
                PreparedFieldBodyView::Html { html, .. } => html.len(),
                _ => 0,
            })
            .sum();
        if self.bytes + added_bytes > HTML_LIMIT {
            return Ok(Err("More content is available in the source record."));
        }
        self.bytes += added_bytes;
        let is_feat = matches!(node, SourceNodeView::Item(ItemSourceView::FeatSource(_)));
        self.namespace += 1;
        let source_namespace = self.namespace;
        // This synthetic navigation marker has no authored ordinal: it lives only
        // in its own transient namespace and retains the exact checked target.
        let mut source =
            crate::projection::relationship_view(reference.clone(), self.r.source_fingerprint());
        source.ordinal = 0;
        source.effect_label = None;
        bindings.push(PreparedInclusionView {
            namespace: source_namespace,
            source: reference.locator.clone().into(),
            controls: vec![],
            references: vec![source],
        });
        let title = node
            .name()
            .value()
            .map(String::as_str)
            .unwrap_or("Source record");
        let mut out = format!(
            "<section class=\"prepared-inclusion\" data-atlas-inclusion=\"{source_namespace}\">"
        );
        if is_feat && options.header {
            let level = if let SourceNodeView::Item(item) = node {
                item.item_level().value().map(ToString::to_string)
            } else {
                None
            };
            out.push_str(&format!(
                "<h4><span data-atlas-reference=\"0\">{}</span>{}</h4>",
                escape(title),
                level
                    .map(|n| format!(" <small>Level {}</small>", escape(&n)))
                    .unwrap_or_default()
            ));
        }
        if is_feat
            && options.traits
            && let SourceNodeView::Item(item) = node
        {
            let mut traits: Vec<String> = item
                .traits()
                .value()
                .unwrap_or_default()
                .iter()
                .map(|t| self.r.trait_label(t).unwrap_or(t).to_owned())
                .collect();
            traits.sort();
            if let Some(rarity) = item.rarity().value() {
                let rarity = serde_json::to_value(rarity)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_owned));
                if let Some(rarity) = rarity.filter(|v| v != "common") {
                    traits.insert(0, self.r.trait_label(&rarity).unwrap_or(&rarity).into());
                }
            }
            if !traits.is_empty() {
                out.push_str("<ul class=\"prepared-inclusion__traits\">");
                for t in traits {
                    out.push_str(&format!("<li>{}</li>", escape(&t)));
                }
                out.push_str("</ul>");
            }
        }
        active.push(identity);
        let mut had_prerequisite = false;
        for field in &mut fields {
            let own_refs: Vec<_> = content
                .relationships
                .iter()
                .filter(|r| {
                    r.locator.record == field.locator.record
                        && r.locator.owners == field.locator.owners
                        && r.locator.field == field.locator.field
                })
                .cloned()
                .collect();
            let is_prerequisite = field.role == "prerequisite";
            if !is_prerequisite && had_prerequisite {
                out.push_str("</div>");
                if options.prerequisite_rule {
                    out.push_str("<hr>");
                }
            }
            if is_prerequisite {
                if had_prerequisite {
                    out.push_str(", ");
                } else {
                    out.push_str("<div class=\"prepared-inclusion__prerequisite\"><strong>Prerequisites</strong> ");
                }
            }
            had_prerequisite = is_prerequisite;
            let text = match &mut field.body {
                PreparedFieldBodyView::Html { html, controls, .. } => {
                    if is_feat
                        && !options.journal_link
                        && !is_prerequisite
                        && terminal_prepared_reference(html).is_some_and(|ordinal| {
                            own_refs.iter().any(|r| {
                                r.ordinal == ordinal
                                    && matches!(
                                        r.details.reference_kind,
                                        Some(ContentReferenceKind::Uuid)
                                    )
                            })
                        })
                    {
                        *html = compose_prepared_inclusions(html, &BTreeMap::new(), true);
                    }
                    let html = self.expand(html, &own_refs, active, bindings)?;
                    self.namespace += 1;
                    let namespace = self.namespace;
                    bindings.push(PreparedInclusionView {
                        namespace,
                        source: field.locator.clone(),
                        controls: controls.clone(),
                        references: relationship_views(own_refs, self.r.source_fingerprint()),
                    });
                    format!("<div data-atlas-inclusion=\"{namespace}\">{html}</div>")
                }
                PreparedFieldBodyView::Plain { text } => format!("<p>{}</p>", escape(text)),
                PreparedFieldBodyView::Unavailable { .. } => {
                    "<p>Included text unavailable.</p>".into()
                }
                PreparedFieldBodyView::Bounded { .. } => {
                    "<p>More content is available in the source record.</p>".into()
                }
            };
            out.push_str(&text);
        }
        if had_prerequisite {
            out.push_str("</div>");
        }
        active.pop();
        if is_feat
            && options.publication
            && let SourceNodeView::Item(item) = node
            && let Some(title) = item
                .publication()
                .and_then(|p| (&p.title).into())
                .value()
                .filter(|t| !t.is_empty())
        {
            out.push_str(&format!(
                "<p class=\"muted record-reference__source\">Source: {}</p>",
                escape(title)
            ));
        }
        out.push_str("</section>");
        self.bytes += out.len().saturating_sub(added_bytes);
        if self.bytes > HTML_LIMIT {
            return Ok(Err("More content is available in the source record."));
        }
        Ok(Ok(PreparedHtmlInclusion {
            html: out,
            source_link: (!(is_feat && options.header)).then_some(PreparedInclusionSourceLink {
                namespace: source_namespace,
                matching_references,
                action: matches!(node, SourceNodeView::Item(ItemSourceView::AbilitySource(_))),
            }),
        }))
    }
}
