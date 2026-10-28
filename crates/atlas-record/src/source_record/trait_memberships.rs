//! Exact authored trait membership without search identity or name requirements.
#[cfg(test)]
use atlas_domain::OwnedContentIdentity;
use atlas_domain::OwnedContentLocator;
use atlas_domain::TraitKey;

use super::{
    SourceBackedRecord, SourceContentRole, SourceQueryView, content::select_content,
    traversal::visit_source_nodes,
};
use crate::source_content::ContentAudience;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTraitMembership {
    pub trait_key: TraitKey,
    pub owners: Vec<OwnedContentLocator>,
    /// Exact authored name of this source; never inherited from an ancestor.
    pub source_label: Option<String>,
}

/// Visit every supported source owner, including unnamed embedded documents.
/// Name visibility admits the identity surface independently of name availability.
/// Unknown authored identifiers retain their exact identity and owner.
pub fn trait_record_memberships(
    record: &SourceBackedRecord,
    audience: ContentAudience,
) -> Vec<SourceTraitMembership> {
    let mut memberships = Vec::new();
    visit_source_nodes(record.source(), |owners, node| {
        let visible = select_content(node)
            .into_iter()
            .find(|selected| selected.role == SourceContentRole::Name)
            .is_some_and(|selected| {
                selected
                    .visibility
                    .value()
                    .is_some_and(|rule| audience.permits(rule))
            });
        if !visible {
            return;
        }
        let query = SourceQueryView {
            source: node,
            pack_id: record.key().pack().as_str(),
            pack_label: "",
        };
        if let Some(traits) = query.traits().value() {
            for identifier in traits {
                if let Ok(trait_key) = TraitKey::new(identifier) {
                    memberships.push(SourceTraitMembership {
                        trait_key,
                        owners: owners.to_vec(),
                        source_label: node.name().value().cloned(),
                    });
                }
            }
        }
    });
    memberships
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_content::ContentVisibilityRule;
    use atlas_foundry_model::{SourceContext, admit_document_source};
    use serde_json::json;

    fn audience() -> ContentAudience {
        ContentAudience {
            include_gm: false,
            include_owner: false,
            implicit_check_dc: ContentVisibilityRule::All,
        }
    }
    fn record(value: serde_json::Value) -> SourceBackedRecord {
        let bytes = serde_json::to_vec(&value).unwrap();
        let source =
            admit_document_source("Item", SourceContext::new("fixture", "test", "$"), &bytes)
                .unwrap()
                .model
                .unwrap();
        SourceBackedRecord::new("equipment", source).unwrap()
    }
    fn owner(collection: &str, id: &str) -> OwnedContentLocator {
        OwnedContentLocator {
            collection: collection.into(),
            identity: OwnedContentIdentity::Stable(id.into()),
        }
    }

    #[test]
    fn consumable_spell_memberships_preserve_exact_named_and_nameless_source() {
        for name in [Some("Jump"), None] {
            let mut spell = json!({"_id":"bbbbbbbbbbbbbbbb","type":"spell","system":{"traits":{"value":["move","custom-unknown"]}}});
            if let Some(name) = name {
                spell["name"] = name.into();
            }
            let source = record(
                json!({"_id":"aaaaaaaaaaaaaaaa","name":"Wand","type":"consumable","system":{"traits":{"value":["wand"]},"spell":spell}}),
            );
            let memberships = trait_record_memberships(&source, audience());
            assert_eq!(memberships.len(), 3);
            let root = memberships
                .iter()
                .find(|m| m.trait_key.identifier() == "wand")
                .unwrap();
            assert!(root.owners.is_empty());
            for identifier in ["move", "custom-unknown"] {
                let embedded = memberships
                    .iter()
                    .find(|m| m.trait_key.identifier() == identifier)
                    .unwrap();
                assert_eq!(
                    embedded.owners,
                    vec![owner("/system/spell", "bbbbbbbbbbbbbbbb")]
                );
                assert_eq!(embedded.source_label.as_deref(), name);
                assert!(source.node_at(&embedded.owners).is_some());
            }
            assert!(
                !memberships
                    .iter()
                    .any(|m| m.trait_key.identifier() == "move" && m.owners.is_empty())
            );
        }
    }

    #[test]
    fn nested_subitems_preserve_names_missing_ids_and_duplicate_id_positions() {
        let source = record(
            json!({"_id":"aaaaaaaaaaaaaaaa","name":"Weapon","type":"weapon","system":{"traits":{"value":["agile"]},"subitems":[
                {"_id":"bbbbbbbbbbbbbbbb","name":"Fitted part","type":"equipment","system":{"traits":{"value":["magical"]},"subitems":[{"type":"equipment","system":{"traits":{"value":["custom-nameless"]}}}]}},
                {"_id":"cccccccccccccccc","type":"equipment","system":{"traits":{"value":["first"]}}},
                {"_id":"cccccccccccccccc","type":"equipment","system":{"traits":{"value":["second"]}}}
            ]}}),
        );
        let memberships = trait_record_memberships(&source, audience());
        assert_eq!(memberships.len(), 5);
        let named = memberships
            .iter()
            .find(|m| m.trait_key.identifier() == "magical")
            .unwrap();
        assert_eq!(named.source_label.as_deref(), Some("Fitted part"));
        let unnamed = memberships
            .iter()
            .find(|m| m.trait_key.identifier() == "custom-nameless")
            .unwrap();
        assert_eq!(unnamed.source_label, None);
        assert_eq!(
            unnamed.owners,
            vec![
                owner("/system/subitems", "bbbbbbbbbbbbbbbb"),
                OwnedContentLocator {
                    collection: "/system/subitems".into(),
                    identity: OwnedContentIdentity::SnapshotLocal { index: 0 }
                }
            ]
        );
        for (identifier, index) in [("first", 1), ("second", 2)] {
            let membership = memberships
                .iter()
                .find(|m| m.trait_key.identifier() == identifier)
                .unwrap();
            assert_eq!(
                membership.owners,
                vec![OwnedContentLocator {
                    collection: "/system/subitems".into(),
                    identity: OwnedContentIdentity::SnapshotLocal { index }
                }]
            );
            assert!(source.node_at(&membership.owners).is_some());
        }
    }
}
