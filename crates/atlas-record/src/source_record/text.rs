use super::content::select_content;
use super::traversal::visit_source_nodes;
use super::{SourceBackedRecord, SourceFieldView, SourceNodeView, SourceQueryView};
use crate::source_content::{ContentAudience, ContentVisibilityRule, OwnedContentLocator};
use atlas_domain::RecordKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceTextKind {
    Name,
    Publication,
    Trait,
    Content,
}
#[derive(Debug)]
pub struct SourceTextSource<'a> {
    pub record: &'a RecordKey,
    pub owners: Vec<OwnedContentLocator>,
    pub field: &'a str,
    pub kind: SourceTextKind,
    pub visibility: ContentVisibilityRule,
    pub text: &'a str,
}
impl SourceBackedRecord {
    /// Combine source-backed plain fields with compatible prepared prose.
    /// Callers supply output from this record's exact snapshot and audience;
    /// preparation evidence is never exposed as audience-facing search text.
    pub fn text_sources<'a>(
        &'a self,
        content: &'a [super::SourceContentOutcome],
        audience: ContentAudience,
        pack_label: &'a str,
    ) -> Vec<SourceTextSource<'a>> {
        let mut out = Vec::new();
        visit_source_nodes(self.source(), |owners, source| {
            for selection in select_content(source) {
                if !matches!(
                    selection.format,
                    SourceFieldView::Value(super::SourceContentFormat::Plain)
                ) {
                    continue;
                }
                let (Some(text), Some(visibility)) =
                    (selection.text.value(), selection.visibility.value())
                else {
                    continue;
                };
                if !audience.permits(visibility) {
                    continue;
                }
                out.push(SourceTextSource {
                    record: self.key(),
                    owners: owners.to_vec(),
                    field: selection.field,
                    kind: if selection.role == super::SourceContentRole::Name {
                        SourceTextKind::Name
                    } else {
                        SourceTextKind::Content
                    },
                    visibility,
                    text,
                });
            }
        });
        let query = SourceQueryView::new(self.source(), self.key().pack().as_str(), pack_label);
        if let Some(title) = query.publication_title().value() {
            out.push(SourceTextSource {
                record: self.key(),
                owners: vec![],
                field: if matches!(query.source, SourceNodeView::Actor(_)) {
                    "/system/details/publication/title"
                } else {
                    "/system/publication/title"
                },
                kind: SourceTextKind::Publication,
                visibility: ContentVisibilityRule::All,
                text: title,
            });
        }
        if let Some(traits) = query.traits().value() {
            for text in traits {
                out.push(SourceTextSource {
                    record: self.key(),
                    owners: vec![],
                    field: "/system/traits/value",
                    kind: SourceTextKind::Trait,
                    visibility: ContentVisibilityRule::All,
                    text,
                });
            }
        }
        for field in content {
            if let super::SourceContentStatus::Prepared(content) = &field.status {
                out.push(SourceTextSource {
                    record: self.key(),
                    owners: content.locator.owners.clone(),
                    field: &content.locator.field,
                    kind: if field.role == super::SourceContentRole::Name {
                        SourceTextKind::Name
                    } else {
                        SourceTextKind::Content
                    },
                    visibility: field.visibility,
                    text: &content.text,
                });
            }
        }
        out
    }
}
