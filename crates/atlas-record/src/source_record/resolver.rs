use super::traversal::visit_source_nodes;
use crate::source_content::{
    ContentReferenceResolver, ContentReferenceTarget, OwnedContentIdentity,
    ResolvedContentReference, SourceContentLocator,
};
use atlas_domain::RecordKey;
use atlas_foundry_model::FoundryDocumentSource;
use std::collections::{BTreeMap, BTreeSet};

/// A two-pass identity/name index, with no copied source or destination prose.
/// All collisions are unresolved; insertion order never chooses a winner.
#[derive(Debug, Default)]
pub struct SourceReferenceIndex {
    targets: BTreeMap<String, Option<ResolvedContentReference>>,
    names: BTreeMap<String, Option<ResolvedContentReference>>,
    roots: BTreeSet<RecordKey>,
    duplicate_roots: BTreeSet<RecordKey>,
    owned_item_families: BTreeMap<String, Option<&'static str>>,
}

impl SourceReferenceIndex {
    pub fn insert_source(&mut self, key: &RecordKey, source: &FoundryDocumentSource) {
        if !self.roots.insert(key.clone()) {
            self.duplicate_roots.insert(key.clone());
        }
        let root = format!(
            "Compendium.pf2e.{}.{}.{}",
            key.pack(),
            nodes_kind(source),
            key.id()
        );
        visit_source_nodes(source, |owners, node| {
            let target = if owners.is_empty() {
                ContentReferenceTarget::Record { key: key.clone() }
            } else {
                ContentReferenceTarget::OwnedNode {
                    key: key.clone(),
                    owners: owners.to_vec(),
                }
            };
            let resolved = ResolvedContentReference {
                target,
                display_name: node.name().value().cloned(),
            };
            let mut suffix = String::new();
            let mut stable = true;
            for owner in owners {
                let OwnedContentIdentity::Stable(id) = &owner.identity else {
                    stable = false;
                    break;
                };
                let kind = match owner.collection.as_str() {
                    "/items" | "/system/subitems" | "/system/spell" => "Item",
                    "/pages" => "JournalEntryPage",
                    "/results" => "TableResult",
                    _ => {
                        stable = false;
                        break;
                    }
                };
                suffix.push_str(&format!(".{kind}.{id}"));
            }
            if stable {
                if let super::SourceNodeView::Item(item) = node {
                    self.owned_item_families
                        .entry(format!("{root}{suffix}"))
                        .and_modify(|family| *family = None)
                        .or_insert(Some(item.family()));
                }
                self.insert(format!("{root}{suffix}"), resolved.clone());
                if owners.is_empty() {
                    // Legacy Compendium syntax identifies the root by ID or a
                    // unique authored name, including qualified builder input.
                    self.insert(
                        format!("pf2e.{}.{}", key.pack(), key.id()),
                        resolved.clone(),
                    );
                    if let Some(name) = node.name().value() {
                        for alias in name_aliases(key.pack().as_str(), nodes_kind(source), name) {
                            self.names
                                .entry(alias)
                                .and_modify(|value| *value = None)
                                .or_insert(Some(resolved.clone()));
                        }
                    }
                }
            }
        });
    }

    /// An unaddressable source can make a name ambiguous even though it cannot
    /// supply a destination. This excludes aliases only; exact IDs still win.
    pub fn exclude_root_name(&mut self, pack: &str, document_kind: &str, name: &str) {
        for alias in name_aliases(pack, document_kind, name) {
            self.names.insert(alias, None);
        }
    }

    fn insert(&mut self, target: String, resolved: ResolvedContentReference) {
        self.targets
            .entry(target)
            .and_modify(|entry| *entry = None)
            .or_insert(Some(resolved));
    }

    fn lookup(&self, target: &str) -> Option<ResolvedContentReference> {
        // Exact identity wins over a name that happens to resemble that ID.
        // The pinned pack builder resolves qualified authored names; we retain
        // the authored target and resolve it without rewriting the DTO.
        let result = self
            .targets
            .get(target)
            .or_else(|| self.names.get(target))?
            .as_ref()?;
        let key = match &result.target {
            ContentReferenceTarget::Record { key }
            | ContentReferenceTarget::OwnedNode { key, .. } => key,
            ContentReferenceTarget::Url { .. } => return None,
        };
        (!self.duplicate_roots.contains(key)).then(|| result.clone())
    }

    fn lookup_compendium(&self, target: &str) -> Option<ResolvedContentReference> {
        if let Some(resolved) = self.lookup(target) {
            return Some(resolved);
        }
        // Authored journal-page UUIDs may identify a heading with #fragment.
        // Resolve the page identity only; the occurrence keeps the full target
        // for later navigation. This does not validate the heading's existence.
        let (page, _) = target.split_once('#')?;
        let resolved = self.lookup(page)?;
        match &resolved.target {
            ContentReferenceTarget::OwnedNode { owners, .. }
                if owners
                    .last()
                    .is_some_and(|owner| owner.collection == "/pages") =>
            {
                Some(resolved)
            }
            _ => None,
        }
    }

    pub(super) fn resolve_local_item(
        &self,
        source: &SourceContentLocator,
        id: &str,
        expected_family: Option<&str>,
    ) -> Option<ResolvedContentReference> {
        let target = format!(
            "Compendium.pf2e.{}.Actor.{}.Item.{id}",
            source.record.pack(),
            source.record.id()
        );
        let family = self.owned_item_families.get(&target)?.as_ref()?;
        if expected_family.is_some_and(|expected| expected != *family) {
            return None;
        }
        self.lookup(&target)
    }
}

fn nodes_kind(source: &FoundryDocumentSource) -> &'static str {
    super::SourceNodeView::from(source).document_kind()
}

fn name_aliases(pack: &str, document_kind: &str, name: &str) -> [String; 3] {
    [
        format!("pf2e.{pack}.{name}"),
        format!("Compendium.pf2e.{pack}.{document_kind}.{name}"),
        format!("pf2e.{pack}.{document_kind}.{name}"),
    ]
}

impl ContentReferenceResolver for SourceReferenceIndex {
    fn resolve_reference(
        &self,
        source: &SourceContentLocator,
        target: &str,
    ) -> Option<ResolvedContentReference> {
        if let Some(legacy) = target.strip_prefix("Compendium.") {
            // Qualified authored root names are supported by PF2e's pinned
            // build/lib/compendium-pack.ts LINK_PATTERNS and #finalize.
            let parts = legacy.split('.').collect::<Vec<_>>();
            if parts.len() == 3 {
                return self.lookup_compendium(legacy);
            }
            return self.lookup_compendium(target);
        }
        if target.starts_with("pf2e.") {
            return self.lookup_compendium(target);
        }
        if let Some(relative) = target.strip_prefix('.') {
            // Pinned journal prose uses .PAGE_ID for a sibling page in the
            // current JournalEntry. Do not interpret arbitrary relative paths
            // or search other journals when this destination is unavailable.
            let id = relative.split_once('#').map_or(relative, |(id, _)| id);
            if source.owners.len() == 1
                && source.owners[0].collection == "/pages"
                && super::identity::valid_source_id(id)
            {
                return self.lookup_compendium(&format!(
                    "Compendium.pf2e.{}.JournalEntry.{}.JournalEntryPage.{relative}",
                    source.record.pack(),
                    source.record.id()
                ));
            }
            return None;
        }
        if let Some(id) = target.strip_prefix("Item.").filter(|id| !id.contains('.')) {
            return self.lookup(&format!(
                "Compendium.pf2e.{}.Actor.{}.Item.{id}",
                source.record.pack(),
                source.record.id()
            ));
        }
        None
    }
}
