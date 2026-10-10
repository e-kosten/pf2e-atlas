use super::traversal::visit_source_nodes;
use super::{
    ItemSourceView, SourceFieldView, SourceIdentityError, SourceNodeView, source_record_key,
};
use crate::source_content::{OwnedContentLocator, SourceContentLocator};
use atlas_domain::RecordKey;
use atlas_foundry_model::FoundryDocumentSource;

/// The source body uses atlas-foundry-model's checked snapshot codec. Deliberately
/// no envelope Serde: it must not bypass that codec's identity and stack guards.
/// Identity and the authored body are read-only after checked construction.
///
/// ```compile_fail
/// use atlas_record::source_record::SourceBackedRecord;
/// use atlas_domain::RecordKey;
/// fn replace_key(record: &mut SourceBackedRecord, key: RecordKey) {
///     record.key = key;
/// }
/// ```
///
/// ```compile_fail
/// use atlas_record::source_record::SourceBackedRecord;
/// use atlas_foundry_model::FoundryDocumentSource;
/// fn replace_body(record: &mut SourceBackedRecord, source: FoundryDocumentSource) {
///     record.source = source;
/// }
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct SourceBackedRecord {
    key: RecordKey,
    source: FoundryDocumentSource,
}

/// Failed construction returns the unchanged body for retention by its owner.
#[derive(Debug, PartialEq, Eq)]
pub struct SourceRecordConstructionError {
    pub reason: SourceIdentityError,
    pub source: FoundryDocumentSource,
}

impl SourceBackedRecord {
    /// Derive identity from the admitted body; corpus-wide collision exclusion
    /// remains the caller's responsibility before building the reference index.
    pub fn new(
        pack: &str,
        source: FoundryDocumentSource,
    ) -> Result<Self, SourceRecordConstructionError> {
        let key = match source_record_key(pack, &source) {
            Ok(key) => key,
            Err(reason) => return Err(SourceRecordConstructionError { reason, source }),
        };
        Ok(Self { key, source })
    }

    pub fn key(&self) -> &RecordKey {
        &self.key
    }

    pub fn source(&self) -> &FoundryDocumentSource {
        &self.source
    }

    /// Resolve a checked chain in this snapshot. Snapshot-local selectors must
    /// only be used with the artifact that supplied them, never to reconcile
    /// saved user state after a rebuild. Empty chains address the root.
    pub fn node_at(&self, owners: &[OwnedContentLocator]) -> Option<SourceNodeView<'_>> {
        let mut found = None;
        visit_source_nodes(self.source(), |candidate, node| {
            if candidate == owners {
                found = Some(node);
            }
        });
        found
    }

    /// Borrow an authored selected content field from this exact source snapshot.
    /// Foreign root addresses and unknown fields never become raw-path lookups.
    pub fn authored_content_at<'a>(
        &'a self,
        locator: &SourceContentLocator,
    ) -> SourceFieldView<'a, &'a str> {
        self.content_selection_at(locator)
            .map(|selection| selection.text.map(String::as_str))
            .unwrap_or(SourceFieldView::NotApplicable)
    }

    pub fn content_selection_at<'a>(
        &'a self,
        locator: &SourceContentLocator,
    ) -> Option<super::content::SourceContentSelection<'a>> {
        if &locator.record != self.key() {
            return None;
        }
        let node = self.node_at(&locator.owners)?;
        super::content::select_content(node)
            .into_iter()
            .find(|selection| selection.field == locator.field)
    }

    /// Enumerate the declared content fields using the same checked source
    /// owner traversal and selection policy as preparation and lookup.
    pub fn visit_content_selections<'a>(
        &'a self,
        mut visit: impl FnMut(&[OwnedContentLocator], super::content::SourceContentSelection<'a>),
    ) {
        visit_source_nodes(self.source(), |owners, node| {
            for selection in super::content::select_content(node) {
                visit(owners, selection);
            }
        });
    }

    /// Stream the immediate Actor Items with the same checked addressing policy
    /// used by content and relationships. No child bodies or address inventory
    /// are retained; unavailable whole collections yield no salvaged neighbors.
    pub fn visit_immediate_actor_items<'a>(
        &'a self,
        mut visit: impl FnMut(usize, &OwnedContentLocator, ItemSourceView<'a>),
    ) -> SourceFieldView<'a, ()> {
        SourceNodeView::from(self.source()).actor_items().map(|_| {
            let mut original_index = 0;
            visit_source_nodes(self.source(), |owners, node| {
                if let ([owner], SourceNodeView::Item(item)) = (owners, node)
                    && owner.collection == "/items"
                {
                    visit(original_index, owner, item);
                    original_index += 1;
                }
            });
        })
    }
    /// Immediate authored Journal pages with checked stable or snapshot-local addresses.
    pub fn visit_immediate_journal_pages<'a>(
        &'a self,
        mut visit: impl FnMut(&OwnedContentLocator, SourceNodeView<'a>),
    ) -> SourceFieldView<'a, ()> {
        let SourceNodeView::Journal(source) = SourceNodeView::from(self.source()) else {
            return SourceFieldView::NotApplicable;
        };
        SourceFieldView::from(&source.pages).map(|_| {
            visit_source_nodes(self.source(), |owners, node| {
                if let ([owner], SourceNodeView::JournalPage(_)) = (owners, node) {
                    visit(owner, node);
                }
            });
        })
    }
    /// Immediate authored RollTable results; no generic node inventory is retained.
    pub fn visit_immediate_table_results<'a>(
        &'a self,
        mut visit: impl FnMut(&OwnedContentLocator, SourceNodeView<'a>),
    ) -> SourceFieldView<'a, ()> {
        let SourceNodeView::Table(source) = SourceNodeView::from(self.source()) else {
            return SourceFieldView::NotApplicable;
        };
        SourceFieldView::from(&source.results).map(|_| {
            visit_source_nodes(self.source(), |owners, node| {
                if let ([owner], SourceNodeView::Result(_)) = (owners, node) {
                    visit(owner, node);
                }
            });
        })
    }
}

/// Developer reporting uses the same traversal as content and reference indexing.
/// No owned-document inventory is retained on the authoritative record.
pub fn source_owned_document_count(record: &SourceBackedRecord) -> usize {
    let mut count = 0usize;
    visit_source_nodes(record.source(), |owners, _| {
        count += usize::from(!owners.is_empty());
    });
    count
}
