use crate::{ContentSourceKind, ContentVisibility};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceGraphMode {
    Default,
    WithEmbedded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReferenceEdgeFacts {
    pub source_kind: ContentSourceKind,
    pub visibility: ContentVisibility,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReferenceGraphPolicy {
    pub excluded_source_kinds: &'static [ContentSourceKind],
}

pub const DEFAULT_EXCLUDED_SOURCE_KINDS: &[ContentSourceKind] = &[
    ContentSourceKind::EmbeddedItemDescription,
    ContentSourceKind::EmbeddedGmDescription,
    ContentSourceKind::EmbeddedSpellDescription,
];

pub const fn reference_graph_policy(mode: ReferenceGraphMode) -> ReferenceGraphPolicy {
    match mode {
        ReferenceGraphMode::Default => ReferenceGraphPolicy {
            excluded_source_kinds: DEFAULT_EXCLUDED_SOURCE_KINDS,
        },
        ReferenceGraphMode::WithEmbedded => ReferenceGraphPolicy {
            excluded_source_kinds: &[],
        },
    }
}

pub fn reference_edge_matches_mode(edge: ReferenceEdgeFacts, mode: ReferenceGraphMode) -> bool {
    reference_graph_policy(mode).matches(edge)
}

impl ReferenceGraphPolicy {
    pub fn matches(self, edge: ReferenceEdgeFacts) -> bool {
        !self.excluded_source_kinds.contains(&edge.source_kind)
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
    fn default_graph_includes_non_embedded_edges_across_visibility_classes() {
        assert!(reference_edge_matches_mode(
            edge(ContentSourceKind::Description, ContentVisibility::Public),
            ReferenceGraphMode::Default
        ));
        assert!(reference_edge_matches_mode(
            edge(ContentSourceKind::GmNotes, ContentVisibility::GmOnly),
            ReferenceGraphMode::Default
        ));
        assert!(reference_edge_matches_mode(
            edge(ContentSourceKind::PrivateNotes, ContentVisibility::Private),
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
    fn default_graph_excludes_embedded_edges_only_for_duplicate_control() {
        assert!(!reference_edge_matches_mode(
            edge(
                ContentSourceKind::EmbeddedItemDescription,
                ContentVisibility::Public
            ),
            ReferenceGraphMode::Default
        ));
        assert!(!reference_edge_matches_mode(
            edge(
                ContentSourceKind::EmbeddedGmDescription,
                ContentVisibility::GmOnly
            ),
            ReferenceGraphMode::Default
        ));
    }

    #[test]
    fn expanded_graph_includes_embedded_edges_across_visibility_classes() {
        assert!(reference_edge_matches_mode(
            edge(
                ContentSourceKind::EmbeddedSpellDescription,
                ContentVisibility::Public
            ),
            ReferenceGraphMode::WithEmbedded
        ));
        assert!(reference_edge_matches_mode(
            edge(
                ContentSourceKind::EmbeddedGmDescription,
                ContentVisibility::GmOnly
            ),
            ReferenceGraphMode::WithEmbedded
        ));
    }
}
