use std::collections::BTreeMap;

use atlas_domain::RecordKey;
use serde::{Deserialize, Serialize};

/// Interpretation identity, independent of source DTO or artifact versions.
pub const CONTENT_INTERPRETATION_VERSION: &str = "foundry-content/v1";

/// A source field, never an output hash, display label or HTML node ordinal.
/// Callers establish identity from their admitted source and retain this locator
/// alongside the authored DTO. Snapshot-local owners must not reconcile user
/// state across rebuilds by position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceContentLocator {
    pub record: RecordKey,
    pub owners: Vec<OwnedContentLocator>,
    /// Field path relative to the last owner (for example /system/description/value).
    pub field: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedContentLocator {
    pub collection: String,
    pub identity: OwnedContentIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnedContentIdentity {
    /// A source ID whose validity and uniqueness were checked by the caller.
    Stable(String),
    /// Missing/duplicate/invalid IDs retain content without claiming stable identity.
    SnapshotLocal { index: usize },
}

/// Explicit projection inputs, not a Foundry permission/runtime emulator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentReferenceTarget {
    Record { key: RecordKey },
    LocalContent { content_key: String },
    Url { url: String },
}

/// Preparation output carries no authored body or generic markup tree.
/// Complete source occurrences include hidden content; their visible flag must
/// be respected by audience-facing consumers. This is not an HTTP response DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedSourceContent {
    pub locator: SourceContentLocator,
    pub interpretation_version: String,
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
    InvalidTemplate,
    MissingCheckType,
}
