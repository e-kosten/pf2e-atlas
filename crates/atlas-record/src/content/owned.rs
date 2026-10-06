use atlas_domain::RecordKey;
use sha2::{Digest, Sha256};

use super::{
    ContentSourceKind, ContentVisibility, FoundryLinkBehavior, RichDocument, RichLinkTarget,
    iter_foundry_links, render_plain_text,
};
use crate::{
    CreatureEntityFamily, CreatureEntityId, CreatureOccurrenceId, HazardEntityFamily,
    HazardEntityId, HazardOccurrenceId,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OwnedRichContent {
    pub documents: Vec<OwnedRichContentDocument>,
    pub exclusions: Vec<ContentExclusion>,
}

impl OwnedRichContent {
    pub fn document(&self, id: &ContentId) -> Option<&OwnedRichContentDocument> {
        self.documents.iter().find(|document| &document.id == id)
    }

    pub fn documents_for_owner(
        &self,
        owner: &ContentOwner,
    ) -> impl Iterator<Item = &OwnedRichContentDocument> {
        self.documents
            .iter()
            .filter(move |document| &document.owner == owner)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentId {
    pub parent_record_key: RecordKey,
    pub content_key: ContentKey,
}

impl ContentId {
    pub fn new(parent_record_key: RecordKey, content_key: ContentKey) -> Self {
        Self {
            parent_record_key,
            content_key,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentKey(String);

impl ContentKey {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidContentKey> {
        let value = value.into();
        if value.trim().is_empty() || value.chars().any(char::is_whitespace) {
            return Err(InvalidContentKey);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidContentKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContentIdentityStability {
    StableSourceIdentity,
    UnstableAuthoredOrdinal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentOwner {
    Record(RecordKey),
    CreatureEntity(CreatureEntityId),
    CreatureOccurrence(CreatureOccurrenceId),
    HazardEntity(HazardEntityId),
    HazardOccurrence(HazardOccurrenceId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContentRole {
    PrimaryDescription,
    Summary,
    SupplementalRules,
    EmbeddedCapability,
    JournalPage,
    TableResult,
    GeneratedNarrative,
    Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentOrigin {
    RecordField {
        source_kind: ContentSourceKind,
        relative_source_path: String,
    },
    EmbeddedEntityField {
        family: CreatureEntityFamily,
        nested_source_id: Option<String>,
        relative_source_path: String,
    },
    HazardEmbeddedEntityField {
        family: HazardEntityFamily,
        nested_source_id: Option<String>,
        relative_source_path: String,
    },
    Generated {
        source_kind: ContentSourceKind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentProvenance {
    pub source_record_key: RecordKey,
    pub relative_source_path: String,
    pub field_or_pointer_family: String,
    pub nested_source_id: Option<String>,
    pub authored_ordinal_or_range: Option<String>,
    pub authored_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateContentStatus {
    Unique,
    CopiedFromCanonicalTarget { target_record_key: RecordKey },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedRichContentDocument {
    pub id: ContentId,
    pub identity_stability: ContentIdentityStability,
    pub owner: ContentOwner,
    pub role: ContentRole,
    pub origin: ContentOrigin,
    pub visibility: ContentVisibility,
    pub provenance: ContentProvenance,
    pub source_kind: ContentSourceKind,
    pub authored_order: u32,
    pub label: Option<String>,
    pub document: RichDocument,
    pub content_hash: ContentHash,
    pub duplicate_status: DuplicateContentStatus,
    pub diagnostics: Vec<ContentDiagnostic>,
    pub reference_occurrences: Vec<ContentReferenceOccurrence>,
}

impl OwnedRichContentDocument {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: ContentId,
        identity_stability: ContentIdentityStability,
        owner: ContentOwner,
        role: ContentRole,
        origin: ContentOrigin,
        visibility: ContentVisibility,
        provenance: ContentProvenance,
        source_kind: ContentSourceKind,
        authored_order: u32,
        label: Option<String>,
        document: RichDocument,
        duplicate_status: DuplicateContentStatus,
        diagnostics: Vec<ContentDiagnostic>,
    ) -> Self {
        let mut owned = Self {
            id,
            identity_stability,
            owner,
            role,
            origin,
            visibility,
            provenance,
            source_kind,
            authored_order,
            label,
            content_hash: ContentHash::for_document(&document),
            duplicate_status,
            diagnostics,
            reference_occurrences: Vec::new(),
            document,
        };
        owned.refresh_derived_state();
        owned
    }

    pub fn refresh_derived_state(&mut self) {
        self.content_hash = ContentHash::for_document(&self.document);
        self.diagnostics
            .retain(|diagnostic| diagnostic.kind != ContentDiagnosticKind::UnresolvedLink);
        self.reference_occurrences = iter_foundry_links(&self.document)
            .enumerate()
            .map(|(ordinal, link)| {
                if let RichLinkTarget::Unresolved { target, .. } = &link.target {
                    self.diagnostics.push(ContentDiagnostic {
                        kind: ContentDiagnosticKind::UnresolvedLink,
                        subject: target.clone(),
                        detail: Some(link.source.authored_target.clone()),
                    });
                }
                ContentReferenceOccurrence {
                    source_content_id: self.id.clone(),
                    ordinal: ordinal as u32,
                    owner: self.owner.clone(),
                    role: self.role,
                    origin: self.origin.clone(),
                    visibility: self.visibility,
                    provenance: self.provenance.clone(),
                    target: link.target.clone(),
                    label: link
                        .label
                        .as_ref()
                        .map(|label| render_plain_text(&RichDocument::new(label.clone())))
                        .filter(|label| !label.trim().is_empty()),
                    relation_kind: match link.behavior {
                        FoundryLinkBehavior::Reference => super::ReferenceRelationKind::Reference,
                        FoundryLinkBehavior::Embed { .. } => super::ReferenceRelationKind::Embed,
                    },
                }
            })
            .collect();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentHash(String);

impl ContentHash {
    pub fn for_document(document: &RichDocument) -> Self {
        let mut hasher = Sha256::new();
        hash_nodes(&mut hasher, &document.nodes);
        Self(format!("{:x}", hasher.finalize()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentSemanticInputHash(String);

impl ContentSemanticInputHash {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentReferenceOccurrence {
    pub source_content_id: ContentId,
    pub ordinal: u32,
    pub owner: ContentOwner,
    pub role: ContentRole,
    pub origin: ContentOrigin,
    pub visibility: ContentVisibility,
    pub provenance: ContentProvenance,
    pub target: RichLinkTarget,
    pub label: Option<String>,
    pub relation_kind: super::ReferenceRelationKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentDiagnostic {
    pub kind: ContentDiagnosticKind,
    pub subject: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContentDiagnosticKind {
    UnsupportedTag,
    UnsupportedAttribute,
    UnknownFoundryMacro,
    UnresolvedLink,
    UnstableIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentExclusion {
    pub parent_record_key: RecordKey,
    pub content_key: ContentKey,
    pub relative_source_path: String,
    pub label: Option<String>,
    pub reason: ContentExclusionReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContentExclusionReason {
    DeferredEntityFamily,
    MissingTypedOwner,
}

fn hash_nodes(hasher: &mut Sha256, nodes: &[super::RichNode]) {
    hash_len(hasher, nodes.len());
    for node in nodes {
        match node {
            super::RichNode::Text { text } => {
                hash_str(hasher, "text");
                hash_str(hasher, text);
            }
            super::RichNode::HtmlElement {
                tag,
                attributes,
                children,
            } => {
                hash_str(hasher, "html");
                hash_str(hasher, tag);
                hash_len(hasher, attributes.len());
                for (name, value) in attributes {
                    hash_str(hasher, name);
                    hash_optional_str(hasher, value.as_deref());
                }
                hash_nodes(hasher, children);
            }
            super::RichNode::FoundryLink { link } => {
                hash_str(hasher, "link");
                hash_link(hasher, link);
            }
            super::RichNode::Foundry { node } => {
                hash_str(hasher, "foundry");
                hash_foundry_node(hasher, node);
            }
        }
    }
}

fn hash_link(hasher: &mut Sha256, link: &super::FoundryLink) {
    match &link.target {
        RichLinkTarget::Record { key, name } => {
            hash_str(hasher, "record");
            hash_str(hasher, &key.to_string());
            hash_str(hasher, name);
        }
        RichLinkTarget::LocalContent { content_key, label } => {
            hash_str(hasher, "local_content");
            hash_str(hasher, content_key);
            hash_optional_str(hasher, label.as_deref());
        }
        RichLinkTarget::External { target, label } => {
            hash_str(hasher, "external");
            hash_str(hasher, target);
            hash_optional_str(hasher, label.as_deref());
        }
        RichLinkTarget::Unresolved {
            target,
            fallback_label,
        } => {
            hash_str(hasher, "unresolved");
            hash_str(hasher, target);
            hash_str(hasher, fallback_label);
        }
    }
    hash_optional_nodes(hasher, link.label.as_deref());
    hash_str(
        hasher,
        match link.source.macro_kind {
            super::FoundryLinkMacroKind::Uuid => "uuid",
            super::FoundryLinkMacroKind::Compendium => "compendium",
            super::FoundryLinkMacroKind::Embed => "embed",
        },
    );
    hash_str(hasher, &link.source.authored_target);
    hash_optional_str(hasher, link.source.relation.as_deref());
    match &link.behavior {
        FoundryLinkBehavior::Reference => hash_str(hasher, "reference"),
        FoundryLinkBehavior::Embed {
            inline,
            hr,
            options,
        } => {
            hash_str(hasher, "embed");
            hash_bool(hasher, *inline);
            hash_optional_bool(hasher, *hr);
            hash_map(hasher, options);
        }
    }
}

fn hash_foundry_node(hasher: &mut Sha256, node: &super::FoundryNode) {
    match node {
        super::FoundryNode::Check {
            statistic,
            options,
            label,
        } => {
            hash_str(hasher, "check");
            hash_optional_str(hasher, statistic.as_deref());
            hash_map(hasher, options);
            hash_optional_nodes(hasher, label.as_deref());
        }
        super::FoundryNode::Damage {
            formula,
            options,
            damage_parts,
            label,
        } => {
            hash_str(hasher, "damage");
            hash_str(hasher, formula);
            hash_map(hasher, options);
            hash_len(hasher, damage_parts.len());
            for part in damage_parts {
                hash_str(hasher, &part.formula);
                hash_optional_str(hasher, part.damage_type.as_deref());
            }
            hash_optional_nodes(hasher, label.as_deref());
        }
        super::FoundryNode::InlineCommand {
            command,
            arguments,
            options,
            label,
        } => {
            hash_str(hasher, "inline_command");
            hash_str(hasher, command);
            hash_str(hasher, arguments);
            hash_map(hasher, options);
            hash_optional_nodes(hasher, label.as_deref());
        }
        super::FoundryNode::Template {
            shape,
            options,
            label,
        } => {
            hash_str(hasher, "template");
            hash_optional_str(hasher, shape.as_deref());
            hash_map(hasher, options);
            hash_optional_nodes(hasher, label.as_deref());
        }
        super::FoundryNode::ActionGlyph { action } => {
            hash_str(hasher, "action_glyph");
            hash_str(hasher, action);
        }
        super::FoundryNode::Trait { traits, label } => {
            hash_str(hasher, "trait");
            hash_strings(hasher, traits);
            hash_optional_nodes(hasher, label.as_deref());
        }
        super::FoundryNode::Localize {
            key,
            label,
            resolved,
        } => {
            hash_str(hasher, "localize");
            hash_str(hasher, key);
            hash_optional_nodes(hasher, label.as_deref());
            hash_optional_nodes(hasher, resolved.as_deref());
        }
        super::FoundryNode::UnknownFoundry {
            name,
            body,
            label,
            raw,
        } => {
            hash_str(hasher, "unknown_foundry");
            hash_str(hasher, name);
            hash_optional_str(hasher, body.as_deref());
            hash_optional_nodes(hasher, label.as_deref());
            hash_str(hasher, raw);
        }
    }
}

fn hash_len(hasher: &mut Sha256, value: usize) {
    hasher.update(value.to_le_bytes());
}

fn hash_str(hasher: &mut Sha256, value: &str) {
    hash_len(hasher, value.len());
    hasher.update(value.as_bytes());
}

fn hash_optional_str(hasher: &mut Sha256, value: Option<&str>) {
    match value {
        Some(value) => {
            hasher.update([1]);
            hash_str(hasher, value);
        }
        None => hasher.update([0]),
    }
}

fn hash_optional_nodes(hasher: &mut Sha256, nodes: Option<&[super::RichNode]>) {
    match nodes {
        Some(nodes) => {
            hasher.update([1]);
            hash_nodes(hasher, nodes);
        }
        None => hasher.update([0]),
    }
}

fn hash_map(hasher: &mut Sha256, values: &std::collections::BTreeMap<String, String>) {
    hash_len(hasher, values.len());
    for (key, value) in values {
        hash_str(hasher, key);
        hash_str(hasher, value);
    }
}

fn hash_strings(hasher: &mut Sha256, values: &[String]) {
    hash_len(hasher, values.len());
    for value in values {
        hash_str(hasher, value);
    }
}

fn hash_bool(hasher: &mut Sha256, value: bool) {
    hasher.update([u8::from(value)]);
}

fn hash_optional_bool(hasher: &mut Sha256, value: Option<bool>) {
    match value {
        Some(value) => {
            hasher.update([1]);
            hash_bool(hasher, value);
        }
        None => hasher.update([0]),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use atlas_domain::RecordKey;

    use super::*;
    use crate::{
        FoundryLink, FoundryLinkBehavior, FoundryLinkMacroKind, FoundryLinkSource, RichNode,
    };

    #[test]
    fn source_identity_survives_edits_while_content_hash_changes() {
        let record_key = RecordKey::parse("bestiary:night-hag").expect("record key");
        let id = ContentId::new(
            record_key,
            ContentKey::new("description").expect("content key"),
        );
        let first = RichDocument::new(vec![paragraph("First wording")]);
        let second = RichDocument::new(vec![paragraph("Edited wording")]);

        assert_eq!(id.content_key.as_str(), "description");
        assert_ne!(
            ContentHash::for_document(&first),
            ContentHash::for_document(&second)
        );
        assert_eq!(id, id.clone());
        assert_ne!(
            ContentHash::for_document(&first).as_str(),
            ContentSemanticInputHash::new("semantic-cache-input").as_str()
        );
    }

    #[test]
    fn reference_occurrences_preserve_typed_source_parity_and_order() {
        let record_key = RecordKey::parse("bestiary:night-hag").expect("record key");
        let target = RecordKey::parse("spells:nightmare").expect("target key");
        let document = RichDocument::new(vec![RichNode::HtmlElement {
            tag: "p".to_string(),
            attributes: BTreeMap::new(),
            children: vec![
                link(target.clone(), "First"),
                RichNode::Text {
                    text: " then ".to_string(),
                },
                link(target, "Second"),
            ],
        }]);
        let owner = ContentOwner::Record(record_key.clone());
        let provenance = ContentProvenance {
            source_record_key: record_key.clone(),
            relative_source_path: "packs/bestiary/night-hag.json".to_string(),
            field_or_pointer_family: "$.system.description.value".to_string(),
            nested_source_id: None,
            authored_ordinal_or_range: None,
            authored_label: None,
        };
        let content = OwnedRichContentDocument::new(
            ContentId::new(record_key, ContentKey::new("description").expect("key")),
            ContentIdentityStability::StableSourceIdentity,
            owner.clone(),
            ContentRole::PrimaryDescription,
            ContentOrigin::RecordField {
                source_kind: ContentSourceKind::Description,
                relative_source_path: "$.system.description.value".to_string(),
            },
            ContentVisibility::GmOnly,
            provenance.clone(),
            ContentSourceKind::Description,
            0,
            None,
            document,
            DuplicateContentStatus::Unique,
            Vec::new(),
        );

        assert_eq!(content.reference_occurrences.len(), 2);
        assert_eq!(content.reference_occurrences[0].ordinal, 0);
        assert_eq!(content.reference_occurrences[1].ordinal, 1);
        assert_eq!(content.reference_occurrences[0].owner, owner);
        assert_eq!(
            content.reference_occurrences[0].visibility,
            ContentVisibility::GmOnly
        );
        assert_eq!(content.reference_occurrences[0].provenance, provenance);
    }

    fn paragraph(text: &str) -> RichNode {
        RichNode::HtmlElement {
            tag: "p".to_string(),
            attributes: BTreeMap::new(),
            children: vec![RichNode::Text {
                text: text.to_string(),
            }],
        }
    }

    fn link(target: RecordKey, label: &str) -> RichNode {
        RichNode::FoundryLink {
            link: FoundryLink {
                target: RichLinkTarget::Record {
                    key: target,
                    name: label.to_string(),
                },
                label: Some(vec![RichNode::Text {
                    text: label.to_string(),
                }]),
                source: FoundryLinkSource {
                    macro_kind: FoundryLinkMacroKind::Uuid,
                    authored_target: label.to_string(),
                    relation: None,
                },
                behavior: FoundryLinkBehavior::Reference,
            },
        }
    }
}
