use std::collections::BTreeMap;

use atlas_domain::RecordKey;
use serde::{Deserialize, Serialize};

/// Interpretation identity, independent of source DTO or artifact versions.
pub const CONTENT_INTERPRETATION_VERSION: &str = "foundry-content/v7";

/// A source field, never an output hash, display label or HTML node ordinal.
/// Callers establish identity from their admitted source and retain this locator
/// alongside the authored DTO. Snapshot-local owners must not reconcile user
/// state across rebuilds by position.
/// `field` is relative to the last owner, for example /system/description/value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct SourceContentLocator {
    pub record: RecordKey,
    pub owners: Vec<OwnedContentLocator>,
    pub field: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(rename = "EmbeddedContentLocator")]
pub struct OwnedContentLocator {
    pub collection: String,
    pub identity: OwnedContentIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(rename = "EmbeddedContentIdentity")]
pub enum OwnedContentIdentity {
    /// A source ID whose validity and uniqueness were checked by the caller.
    Stable(String),
    /// Missing/duplicate/invalid IDs retain content without claiming stable identity.
    SnapshotLocal { index: usize },
}

/// Explicit projection inputs, not a Foundry permission/runtime emulator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentAudience {
    pub include_gm: bool,
    pub include_owner: bool,
    pub implicit_check_dc: ContentVisibilityRule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentVisibilityRule {
    All,
    Gm,
    Owner,
    None,
}

impl ContentVisibilityRule {
    pub(crate) fn from_source(value: &str) -> Option<Self> {
        match value {
            "all" => Some(Self::All),
            "gm" => Some(Self::Gm),
            "owner" => Some(Self::Owner),
            "none" => Some(Self::None),
            _ => None,
        }
    }
}

impl ContentAudience {
    pub(crate) fn permits(self, rule: ContentVisibilityRule) -> bool {
        match rule {
            ContentVisibilityRule::All => true,
            ContentVisibilityRule::Gm => self.include_gm,
            ContentVisibilityRule::Owner => self.include_owner,
            ContentVisibilityRule::None => false,
        }
    }
}

/// Resolver supplies identities, not product routes or copied destination prose.
pub trait ContentReferenceResolver {
    fn resolve_reference(
        &self,
        source: &SourceContentLocator,
        authored_target: &str,
    ) -> Option<ResolvedContentReference>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedContentReference {
    pub target: ContentReferenceTarget,
    pub display_name: Option<String>,
    /// Family of the exact typed destination, including embedded items.
    pub item_family: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentReferenceTarget {
    Record {
        key: RecordKey,
    },
    OwnedNode {
        key: RecordKey,
        owners: Vec<OwnedContentLocator>,
    },
    Url {
        url: String,
    },
}

/// Preparation output carries no authored body or generic markup tree.
/// Complete source occurrences include hidden content; their visible flag must
/// be respected by audience-facing consumers. This is not an HTTP response DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedSourceContent {
    pub locator: SourceContentLocator,
    pub interpretation_version: String,
    /// Hash of the exact authored field interpreted by this preparation call.
    pub authored_markup_sha256: String,
    pub html: String,
    /// Unwrapped plain text derived from the same audience-filtered HTML.
    pub text: String,
    pub references: Vec<ContentReferenceOccurrence>,
    /// Only visible interactions, referenced by data-atlas-interaction markers.
    pub interactions: Vec<ContentInteraction>,
    pub diagnostics: Vec<ContentInterpretationDiagnostic>,
}

/// Prepared localization-backed term content with its exact vocabulary identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedTraitDefinition {
    pub key: atlas_domain::TraitKey,
    pub interpretation_version: String,
    /// Hash of the exact authored field interpreted by this preparation call.
    pub authored_markup_sha256: String,
    pub html: String,
    /// Unwrapped plain text derived from the same audience-filtered HTML.
    pub text: String,
    pub references: Vec<ContentReferenceOccurrence>,
    /// Only visible interactions, referenced by data-atlas-interaction markers.
    pub interactions: Vec<ContentInteraction>,
    pub diagnostics: Vec<ContentInterpretationDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentReferenceOccurrence {
    pub ordinal: usize,
    /// Interpretation-local path, not a durable content identity.
    pub path: String,
    pub kind: ContentReferenceKind,
    pub authored_target: String,
    pub resolution: ContentReferenceResolution,
    pub visible: bool,
    pub audiences: Vec<String>,
    pub label: ContentReferenceLabel,
}

/// Source facts for presentation; no inferred role or redundancy classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentReferenceLabel {
    pub explicit: bool,
    /// Nearest parsed block path, scoped to this content field; empty at its root.
    pub block: String,
    pub target_name: Option<String>,
    pub item_family: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentReferenceKind {
    Uuid,
    Compendium,
    Embed { options: BTreeMap<String, String> },
    HtmlLink,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentReferenceResolution {
    Resolved(ContentReferenceTarget),
    /// Relative/absolute URL preserved by the sanitizer, not checked for reachability.
    UnverifiedUrl {
        url: String,
    },
    Unresolved,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentInteraction {
    pub ordinal: usize,
    pub path: String,
    pub kind: ContentInteractionKind,
}

/// Authored parameters for concrete controls; expressions are not evaluated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentInteractionKind {
    Check {
        statistic: Option<String>,
        options: BTreeMap<String, String>,
    },
    Damage {
        formula: String,
        options: BTreeMap<String, String>,
    },
    Command {
        command: String,
        arguments: String,
        options: BTreeMap<String, String>,
    },
    Template {
        shape: Option<String>,
        options: BTreeMap<String, String>,
    },
}

impl ContentInteractionKind {
    pub fn marker_kind(&self) -> &'static str {
        match self {
            Self::Check { .. } => "check",
            Self::Damage { .. } => "damage",
            Self::Command { .. } => "command",
            Self::Template { .. } => "template",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentInterpretationDiagnostic {
    pub path: String,
    pub code: ContentDiagnosticCode,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentDiagnosticCode {
    UnknownMacro,
    MalformedSyntax,
    UnresolvedLocalization,
    IgnoredLocalizationLabel,
    UnresolvedReference,
    BlockedDestination,
    EmbedNotExpanded,
    RuntimeContextRequired,
    UnknownActionGlyph,
    UnknownVisibility,
    ExcludedHtmlContent,
    UnavailableImageAsset,
    InvalidTemplate,
    MissingCheckType,
}
