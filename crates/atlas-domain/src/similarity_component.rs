//! Root identity and prose sections are distinct logical comparison components.
use crate::{SimilarityPart, SimilarityPartSelector, SimilarityRole};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// New typed-component comparisons do not relabel unchanged legacy prose scores.
pub const SIMILARITY_COMPONENT_RECIPE: &str = "selected-component-similarity/v1";

/// Identity is never represented as a fabricated prose field, ordinal or role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SimilarityComponent {
    Identity {
        selector: SimilarityPartSelector,
        label: String,
        owner_label: String,
        has_vectors: bool,
    },
    Section {
        part: SimilarityPart,
    },
}

impl SimilarityComponent {
    pub fn selector(&self) -> &SimilarityPartSelector {
        match self {
            Self::Identity { selector, .. } => selector,
            Self::Section { part } => &part.selector,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Identity { label, .. } => label,
            Self::Section { part } => &part.label,
        }
    }

    pub fn owner_label(&self) -> &str {
        match self {
            Self::Identity { owner_label, .. } => owner_label,
            Self::Section { part } => &part.owner_label,
        }
    }

    pub fn has_vectors(&self) -> bool {
        match self {
            Self::Identity { has_vectors, .. } => *has_vectors,
            Self::Section { part } => part.has_vectors,
        }
    }

    pub fn comparison_key(&self) -> SimilarityComparisonKey {
        match self {
            Self::Identity { .. } => SimilarityComparisonKey::Identity,
            Self::Section { part } => SimilarityComparisonKey::Role(part.role),
        }
    }

    pub fn part(&self) -> Option<&SimilarityPart> {
        match self {
            Self::Identity { .. } => None,
            Self::Section { part } => Some(part),
        }
    }

    pub fn is_identity(&self) -> bool {
        matches!(self, Self::Identity { .. })
    }
}

/// Correspondence keys extend the canonical scorer without changing prose roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    content = "role",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum SimilarityComparisonKey {
    Identity,
    Role(SimilarityRole),
}

impl SimilarityComparisonKey {
    pub fn corresponds_to(self, candidate: Self) -> bool {
        match (self, candidate) {
            (Self::Identity, Self::Identity) => true,
            (Self::Role(source), Self::Role(candidate)) => source.corresponds_to(candidate),
            _ => false,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Identity => "Name and identity",
            Self::Role(role) => role.label(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn identity_wire_shape_has_no_fabricated_section_metadata() {
        let component = SimilarityComponent::Identity {
            selector: SimilarityPartSelector("identity:checked-selector".into()),
            label: "Name and identity".into(),
            owner_label: "Lich".into(),
            has_vectors: true,
        };
        assert_eq!(
            serde_json::to_value(&component).unwrap(),
            json!({
                "kind":"identity", "selector":"identity:checked-selector",
                "label":"Name and identity", "owner_label":"Lich", "has_vectors":true
            })
        );
        assert!(component.part().is_none());
        let mut invalid = serde_json::to_value(component).unwrap();
        invalid["field"] = json!("/name");
        assert!(serde_json::from_value::<SimilarityComponent>(invalid).is_err());
    }

    #[test]
    fn identity_corresponds_only_to_identity_and_preserves_role_rules() {
        use SimilarityComparisonKey::{Identity, Role};
        assert!(Identity.corresponds_to(Identity));
        for (role, self_corresponds) in [
            (SimilarityRole::Description, true),
            (SimilarityRole::Ability, true),
            (SimilarityRole::Other, false),
            (SimilarityRole::TraitDefinition, false),
        ] {
            assert!(!Identity.corresponds_to(Role(role)));
            assert!(!Role(role).corresponds_to(Identity));
            assert_eq!(
                Role(role).corresponds_to(Role(role)),
                self_corresponds,
                "{role:?}"
            );
        }
        assert!(!Role(SimilarityRole::Description).corresponds_to(Role(SimilarityRole::Ability)));
    }
}
