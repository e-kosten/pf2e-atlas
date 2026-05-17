use crate::{ContentSourceKind, ContentVisibility};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceGraphMode {
    Default,
    PublicWithEmbedded,
    AllVisible,
    Internal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReferenceEdgeFacts {
    pub source_kind: ContentSourceKind,
    pub visibility: ContentVisibility,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceVisibilityPolicy {
    Only(ContentVisibility),
    Exclude(ContentVisibility),
    Any,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReferenceGraphPolicy {
    pub visibility: ReferenceVisibilityPolicy,
    pub excluded_source_kinds: &'static [ContentSourceKind],
}

pub const DEFAULT_EXCLUDED_SOURCE_KINDS: &[ContentSourceKind] = &[
    ContentSourceKind::EmbeddedItemDescription,
    ContentSourceKind::EmbeddedSpellDescription,
];

pub const fn reference_graph_policy(mode: ReferenceGraphMode) -> ReferenceGraphPolicy {
    match mode {
        ReferenceGraphMode::Default => ReferenceGraphPolicy {
            visibility: ReferenceVisibilityPolicy::Only(ContentVisibility::Public),
            excluded_source_kinds: DEFAULT_EXCLUDED_SOURCE_KINDS,
        },
        ReferenceGraphMode::PublicWithEmbedded => ReferenceGraphPolicy {
            visibility: ReferenceVisibilityPolicy::Only(ContentVisibility::Public),
            excluded_source_kinds: &[],
        },
        ReferenceGraphMode::AllVisible => ReferenceGraphPolicy {
            visibility: ReferenceVisibilityPolicy::Exclude(ContentVisibility::Internal),
            excluded_source_kinds: &[],
        },
        ReferenceGraphMode::Internal => ReferenceGraphPolicy {
            visibility: ReferenceVisibilityPolicy::Any,
            excluded_source_kinds: &[],
        },
    }
}

pub fn reference_edge_matches_mode(edge: ReferenceEdgeFacts, mode: ReferenceGraphMode) -> bool {
    reference_graph_policy(mode).matches(edge)
}

impl ReferenceGraphPolicy {
    pub fn matches(self, edge: ReferenceEdgeFacts) -> bool {
        let visibility_matches = match self.visibility {
            ReferenceVisibilityPolicy::Only(visibility) => edge.visibility == visibility,
            ReferenceVisibilityPolicy::Exclude(visibility) => edge.visibility != visibility,
            ReferenceVisibilityPolicy::Any => true,
        };
        visibility_matches && !self.excluded_source_kinds.contains(&edge.source_kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edge(source_kind: ContentSourceKind, visibility: ContentVisibility) -> ReferenceEdgeFacts {
        ReferenceEdgeFacts {
            source_kind,
            visibility,
        }
    }

    #[test]
    fn default_graph_includes_public_non_embedded_edges() {
        assert!(reference_edge_matches_mode(
            edge(ContentSourceKind::Description, ContentVisibility::Public),
            ReferenceGraphMode::Default
        ));
        assert!(reference_edge_matches_mode(
            edge(
                ContentSourceKind::GeneratedAffliction,
                ContentVisibility::Public
            ),
            ReferenceGraphMode::Default
        ));
    }

    #[test]
    fn default_graph_excludes_embedded_and_non_public_edges() {
        assert!(!reference_edge_matches_mode(
            edge(
                ContentSourceKind::EmbeddedItemDescription,
                ContentVisibility::Public
            ),
            ReferenceGraphMode::Default
        ));
        assert!(!reference_edge_matches_mode(
            edge(ContentSourceKind::Description, ContentVisibility::Private),
            ReferenceGraphMode::Default
        ));
        assert!(!reference_edge_matches_mode(
            edge(ContentSourceKind::Description, ContentVisibility::Internal),
            ReferenceGraphMode::Default
        ));
    }

    #[test]
    fn expanded_modes_include_expected_edge_sets() {
        assert!(reference_edge_matches_mode(
            edge(
                ContentSourceKind::EmbeddedSpellDescription,
                ContentVisibility::Public
            ),
            ReferenceGraphMode::PublicWithEmbedded
        ));
        assert!(reference_edge_matches_mode(
            edge(ContentSourceKind::Description, ContentVisibility::Private),
            ReferenceGraphMode::AllVisible
        ));
        assert!(!reference_edge_matches_mode(
            edge(ContentSourceKind::Description, ContentVisibility::Internal),
            ReferenceGraphMode::AllVisible
        ));
        assert!(reference_edge_matches_mode(
            edge(ContentSourceKind::Description, ContentVisibility::Internal),
            ReferenceGraphMode::Internal
        ));
    }
}
