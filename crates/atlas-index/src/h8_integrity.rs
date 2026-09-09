use std::collections::{BTreeMap, BTreeSet};

use atlas_domain::RecordKey;
use atlas_record::{
    ContentChildKind, ContentChildLocator, ContentOrigin, ContentOwner, FactValue, H8FieldValue,
    JournalPageEntry, OwnedRichContent, RecordBody, ReferenceEdge, RichLinkTarget,
    TableResultEntry, iter_foundry_links,
};

#[derive(Debug, Default)]
pub(crate) struct H8ChildCatalog {
    children: BTreeMap<RecordKey, BTreeSet<ContentChildLocator>>,
}

impl H8ChildCatalog {
    pub(crate) fn from_bodies<'a>(
        bodies: impl IntoIterator<Item = &'a RecordBody>,
    ) -> Result<Self, String> {
        let mut catalog = Self::default();
        for body in bodies {
            catalog.insert_body(body)?;
        }
        Ok(catalog)
    }

    pub(crate) fn validate_body_targets<'a>(
        &self,
        bodies: impl IntoIterator<Item = &'a RecordBody>,
    ) -> Result<(), String> {
        for body in bodies {
            let owned = match body {
                RecordBody::Journal(journal) => Some(&journal.content),
                RecordBody::RollTable(table) => Some(&table.content),
                RecordBody::Creature(_) | RecordBody::Hazard(_) | RecordBody::Spell(_) => None,
            };
            if let Some(owned) = owned {
                self.validate_owned_content_targets(owned)?;
            }
        }
        Ok(())
    }

    pub(crate) fn validate_reference_edge(&self, edge: &ReferenceEdge) -> Result<(), String> {
        self.validate_edge_locators(
            &edge.from_record_key,
            edge.source_child.as_ref(),
            &edge.to_record_key,
            edge.target_child.as_ref(),
        )
    }

    pub(crate) fn validate_edge_locators(
        &self,
        from: &RecordKey,
        source_child: Option<&ContentChildLocator>,
        to: &RecordKey,
        target_child: Option<&ContentChildLocator>,
    ) -> Result<(), String> {
        if let Some(locator) = source_child {
            self.require_child(from, locator, "reference source")?;
        }
        if let Some(locator) = target_child {
            self.require_child(to, locator, "reference target")?;
        }
        Ok(())
    }

    pub(crate) fn target_parent_keys<'a>(
        bodies: impl IntoIterator<Item = &'a RecordBody>,
    ) -> BTreeSet<RecordKey> {
        let mut keys = BTreeSet::new();
        for body in bodies {
            let owned = match body {
                RecordBody::Journal(journal) => Some(&journal.content),
                RecordBody::RollTable(table) => Some(&table.content),
                RecordBody::Creature(_) | RecordBody::Hazard(_) | RecordBody::Spell(_) => None,
            };
            let Some(owned) = owned else {
                continue;
            };
            for document in &owned.documents {
                for link in iter_foundry_links(&document.document) {
                    if let RichLinkTarget::RecordChild { key, .. } = &link.target {
                        keys.insert(key.clone());
                    }
                }
                for occurrence in &document.reference_occurrences {
                    if let RichLinkTarget::RecordChild { key, .. } = &occurrence.target {
                        keys.insert(key.clone());
                    }
                }
            }
        }
        keys
    }

    fn insert_body(&mut self, body: &RecordBody) -> Result<(), String> {
        match body {
            RecordBody::Journal(journal) => {
                let parent = &journal.identity.record_key;
                let mut locators = BTreeSet::new();
                if let FactValue::Value(H8FieldValue::Known(pages)) = &journal.pages {
                    for entry in pages {
                        let locator = match entry {
                            JournalPageEntry::Page(page) => &page.locator,
                            JournalPageEntry::Unsupported(page) => &page.locator,
                        };
                        require_local_child(
                            parent,
                            ContentChildKind::JournalPage,
                            locator,
                            "journal page",
                        )?;
                        if !locators.insert(locator.clone()) {
                            return Err(format!(
                                "journal `{parent}` contains duplicate child locator `{}`",
                                atlas_record::encode_content_child_locator(locator)
                            ));
                        }
                    }
                }
                self.insert_parent(parent, locators)?;
                self.validate_owned_content_local(parent, &journal.content)?;
            }
            RecordBody::RollTable(table) => {
                let parent = &table.identity.record_key;
                let mut locators = BTreeSet::new();
                if let FactValue::Value(H8FieldValue::Known(results)) = &table.results {
                    for entry in results {
                        let locator = match entry {
                            TableResultEntry::Result(result) => &result.locator,
                            TableResultEntry::Unsupported(result) => &result.locator,
                        };
                        require_local_child(
                            parent,
                            ContentChildKind::TableResult,
                            locator,
                            "table result",
                        )?;
                        if !locators.insert(locator.clone()) {
                            return Err(format!(
                                "roll table `{parent}` contains duplicate child locator `{}`",
                                atlas_record::encode_content_child_locator(locator)
                            ));
                        }
                    }
                }
                self.insert_parent(parent, locators)?;
                self.validate_owned_content_local(parent, &table.content)?;
            }
            RecordBody::Creature(_) | RecordBody::Hazard(_) | RecordBody::Spell(_) => {}
        }
        Ok(())
    }

    fn insert_parent(
        &mut self,
        parent: &RecordKey,
        locators: BTreeSet<ContentChildLocator>,
    ) -> Result<(), String> {
        if self.children.insert(parent.clone(), locators).is_some() {
            return Err(format!("duplicate canonical H8 body `{parent}`"));
        }
        Ok(())
    }

    fn validate_owned_content_local(
        &self,
        parent: &RecordKey,
        owned: &OwnedRichContent,
    ) -> Result<(), String> {
        for document in &owned.documents {
            if &document.id.parent_record_key != parent {
                return Err(format!(
                    "H8 content `{}` is indexed under `{}` instead of `{parent}`",
                    document.id.content_key.as_str(),
                    document.id.parent_record_key
                ));
            }
            match (&document.owner, &document.origin) {
                (ContentOwner::Record(owner), ContentOrigin::RecordField { .. })
                    if owner == parent => {}
                (
                    ContentOwner::Child(owner),
                    ContentOrigin::ChildField {
                        locator: origin, ..
                    },
                ) if owner == origin => {
                    self.require_child(parent, owner, "content owner and origin")?;
                }
                (ContentOwner::Child(owner), ContentOrigin::ChildField { locator, .. }) => {
                    return Err(format!(
                        "H8 content `{}` owner locator `{}` disagrees with origin locator `{}`",
                        document.id.content_key.as_str(),
                        atlas_record::encode_content_child_locator(owner),
                        atlas_record::encode_content_child_locator(locator)
                    ));
                }
                _ => {
                    return Err(format!(
                        "H8 content `{}` has an invalid canonical owner/origin pair",
                        document.id.content_key.as_str()
                    ));
                }
            }
            if document.provenance.source_record_key != *parent {
                return Err(format!(
                    "H8 content `{}` provenance belongs to `{}` instead of `{parent}`",
                    document.id.content_key.as_str(),
                    document.provenance.source_record_key
                ));
            }
        }
        Ok(())
    }

    fn validate_owned_content_targets(&self, owned: &OwnedRichContent) -> Result<(), String> {
        for document in &owned.documents {
            for link in iter_foundry_links(&document.document) {
                self.validate_target(&link.target)?;
            }
            for occurrence in &document.reference_occurrences {
                self.validate_target(&occurrence.target)?;
            }
        }
        Ok(())
    }

    fn validate_target(&self, target: &RichLinkTarget) -> Result<(), String> {
        if let RichLinkTarget::RecordChild { key, locator, .. } = target {
            self.require_child(key, locator, "record-child target")?;
        }
        Ok(())
    }

    fn require_child(
        &self,
        expected_parent: &RecordKey,
        locator: &ContentChildLocator,
        subject: &str,
    ) -> Result<(), String> {
        if &locator.parent != expected_parent {
            return Err(format!(
                "{subject} key `{expected_parent}` disagrees with locator parent `{}`",
                locator.parent
            ));
        }
        if !self
            .children
            .get(expected_parent)
            .is_some_and(|children| children.contains(locator))
        {
            return Err(format!(
                "{subject} locator `{}` does not identify a canonical child",
                atlas_record::encode_content_child_locator(locator)
            ));
        }
        Ok(())
    }
}

fn require_local_child(
    parent: &RecordKey,
    kind: ContentChildKind,
    locator: &ContentChildLocator,
    subject: &str,
) -> Result<(), String> {
    if &locator.parent != parent {
        return Err(format!(
            "{subject} locator parent `{}` does not match owning record `{parent}`",
            locator.parent
        ));
    }
    if locator.kind != kind {
        return Err(format!(
            "{subject} locator has kind `{:?}` instead of `{:?}`",
            locator.kind, kind
        ));
    }
    Ok(())
}
