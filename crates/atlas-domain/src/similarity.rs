//! Shared, stored-vector similarity options and exact selectable content parts.
use crate::{OwnedContentLocator, RecordKey, TraitKey};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SIMILARITY_RECIPE: &str = "selected-content-similarity/v1";
pub const SIMILARITY_ROLE_RECIPE: &str = "typed-content-correspondence/v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SimilaritySeed {
    Record { record_key: RecordKey },
    Trait { trait_key: TraitKey },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(transparent)]
pub struct SimilarityPartSelector(pub String);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SimilarityCandidateParts {
    #[default]
    Any,
    Corresponding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SimilarityRole {
    Description,
    Ability,
    Disable,
    Routine,
    Reset,
    Stealth,
    Prerequisites,
    Notes,
    JournalText,
    TableResult,
    TraitDefinition,
    Other,
}
impl SimilarityRole {
    pub fn corresponds_to(self, candidate: Self) -> bool {
        self == candidate && !matches!(self, Self::Other | Self::TraitDefinition)
    }
    pub const fn label(self) -> &'static str {
        match self {
            Self::Description => "Description and background",
            Self::Ability => "Ability",
            Self::Disable => "Disable",
            Self::Routine => "Routine",
            Self::Reset => "Reset",
            Self::Stealth => "Stealth",
            Self::Prerequisites => "Prerequisites",
            Self::Notes => "Notes",
            Self::JournalText => "Journal text",
            Self::TableResult => "Table result",
            Self::TraitDefinition => "Trait definition",
            Self::Other => "Other content",
        }
    }
}

pub fn default_similarity_decay() -> f64 {
    0.5
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "method", rename_all = "kebab-case", deny_unknown_fields)]
pub enum SimilarityOptions {
    Passages {
        #[serde(default = "default_similarity_decay")]
        decay: f64,
        #[serde(default)]
        parts: Option<Vec<SimilarityPartSelector>>,
        #[serde(default)]
        candidate_parts: SimilarityCandidateParts,
    },
    ContentAverage {
        #[serde(default)]
        parts: Option<Vec<SimilarityPartSelector>>,
        #[serde(default)]
        candidate_parts: SimilarityCandidateParts,
    },
    Baseline {},
}
impl Default for SimilarityOptions {
    fn default() -> Self {
        Self::Passages {
            decay: default_similarity_decay(),
            parts: None,
            candidate_parts: SimilarityCandidateParts::Any,
        }
    }
}
impl SimilarityOptions {
    pub fn parts(&self) -> Option<&[SimilarityPartSelector]> {
        match self {
            Self::Passages { parts, .. } | Self::ContentAverage { parts, .. } => parts.as_deref(),
            Self::Baseline {} => None,
        }
    }
    pub fn candidate_parts(&self) -> SimilarityCandidateParts {
        match self {
            Self::Passages {
                candidate_parts, ..
            }
            | Self::ContentAverage {
                candidate_parts, ..
            } => *candidate_parts,
            Self::Baseline {} => SimilarityCandidateParts::Any,
        }
    }
    pub fn validate(&self, seed: &SimilaritySeed) -> Result<(), String> {
        if let Self::Passages { decay, .. } = self
            && (!decay.is_finite() || !(0.0..=1.0).contains(decay))
        {
            return Err("match decay must be a finite number from 0 through 1".into());
        }
        if self.parts().is_some_and(<[_]>::is_empty) {
            return Err("select at least one comparison part".into());
        }
        if matches!(seed, SimilaritySeed::Trait { .. })
            && self.candidate_parts() == SimilarityCandidateParts::Corresponding
        {
            return Err(
                "trait definitions support any candidate content, not corresponding parts".into(),
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SimilarityPart {
    pub selector: SimilarityPartSelector,
    pub label: String,
    pub owner_label: String,
    pub owners: Vec<OwnedContentLocator>,
    pub field: String,
    pub section_ordinal: usize,
    pub role: SimilarityRole,
    pub has_vectors: bool,
    pub label_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SimilarityParts {
    pub seed: SimilaritySeed,
    pub parts: Vec<SimilarityPart>,
    pub role_recipe: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn baseline_wire_options_reject_method_specific_fields() {
        assert_eq!(
            serde_json::from_value::<SimilarityOptions>(serde_json::json!({"method":"baseline"}))
                .unwrap(),
            SimilarityOptions::Baseline {}
        );
        for key in ["decay", "parts", "candidate_parts", "unknown"] {
            let mut value = serde_json::json!({"method":"baseline"});
            value[key] = serde_json::json!(null);
            assert!(
                serde_json::from_value::<SimilarityOptions>(value).is_err(),
                "Baseline must reject {key}"
            );
        }
    }
}
