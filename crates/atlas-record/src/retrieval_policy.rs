use std::collections::BTreeSet;

use atlas_domain::{RecordKey, RecordKind};

use crate::{AtlasRecord, RecordVisibilityReason, RemasterLink};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordRole {
    Source,
    SourceInstance,
    Canonical,
}

impl RecordRole {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::SourceInstance => "source_instance",
            Self::Canonical => "canonical",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrievalDisposition {
    Ordinary,
    DirectOnly,
    InspectionOnly,
}

impl RetrievalDisposition {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ordinary => "ordinary",
            Self::DirectOnly => "direct_only",
            Self::InspectionOnly => "inspection_only",
        }
    }

    pub const fn is_ordinary(self) -> bool {
        matches!(self, Self::Ordinary)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrievalRationale {
    SourceRecord,
    GeneratedCanonical,
    DuplicateSourceInstance,
    CanonicalEditionDuplicate,
    ToolingNoAddressableProductMeaning,
}

impl RetrievalRationale {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceRecord => "source_record",
            Self::GeneratedCanonical => "generated_canonical",
            Self::DuplicateSourceInstance => "duplicate_source_instance",
            Self::CanonicalEditionDuplicate => "canonical_edition_duplicate",
            Self::ToolingNoAddressableProductMeaning => "tooling_no_addressable_product_meaning",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetrievalPolicyDecision {
    pub role: RecordRole,
    pub disposition: RetrievalDisposition,
    pub rationale: RetrievalRationale,
}

#[derive(Debug, Clone)]
pub struct ProductRetrievalPolicy {
    demoted_legacy_record_keys: BTreeSet<RecordKey>,
}

impl ProductRetrievalPolicy {
    pub fn from_remaster_links(remaster_links: &[RemasterLink]) -> Self {
        Self {
            demoted_legacy_record_keys: remaster_links
                .iter()
                .map(|link| link.legacy_record_key.clone())
                .collect(),
        }
    }

    pub fn decision(&self, record: &AtlasRecord) -> RetrievalPolicyDecision {
        if record.classification.kind == RecordKind::Tooling {
            return RetrievalPolicyDecision {
                role: RecordRole::Source,
                disposition: RetrievalDisposition::InspectionOnly,
                rationale: RetrievalRationale::ToolingNoAddressableProductMeaning,
            };
        }
        if self
            .demoted_legacy_record_keys
            .contains(&record.identity.key)
        {
            return RetrievalPolicyDecision {
                role: RecordRole::Source,
                disposition: RetrievalDisposition::DirectOnly,
                rationale: RetrievalRationale::CanonicalEditionDuplicate,
            };
        }
        match record.visibility.reason() {
            RecordVisibilityReason::GeneratedInstance => RetrievalPolicyDecision {
                role: RecordRole::SourceInstance,
                disposition: RetrievalDisposition::DirectOnly,
                rationale: RetrievalRationale::DuplicateSourceInstance,
            },
            RecordVisibilityReason::GeneratedCanonical => RetrievalPolicyDecision {
                role: RecordRole::Canonical,
                disposition: RetrievalDisposition::Ordinary,
                rationale: RetrievalRationale::GeneratedCanonical,
            },
            RecordVisibilityReason::SourceRecord => RetrievalPolicyDecision {
                role: RecordRole::Source,
                disposition: RetrievalDisposition::Ordinary,
                rationale: RetrievalRationale::SourceRecord,
            },
        }
    }

    pub fn is_ordinary(&self, record: &AtlasRecord) -> bool {
        self.decision(record).disposition.is_ordinary()
    }
}

#[cfg(test)]
mod tests {
    use atlas_domain::{PackName, RecordId, RemasterLinkSource};

    use super::*;
    use crate::{
        FoundryDocumentType, FoundryRecordInfo, FoundryRecordType, RecordClassification,
        RecordIdentity, RecordProvenance, RecordVisibility,
    };

    #[test]
    fn retrieval_policy_has_one_exhaustive_typed_decision_matrix() {
        let ordinary = record(
            "ordinary",
            RecordKind::Rule,
            RecordVisibilityReason::SourceRecord,
        );
        let canonical = record(
            "canonical",
            RecordKind::Affliction,
            RecordVisibilityReason::GeneratedCanonical,
        );
        let source_instance = record(
            "source-instance",
            RecordKind::Affliction,
            RecordVisibilityReason::GeneratedInstance,
        );
        let tooling = record(
            "tooling",
            RecordKind::Tooling,
            RecordVisibilityReason::SourceRecord,
        );
        let legacy = record(
            "legacy",
            RecordKind::Rule,
            RecordVisibilityReason::SourceRecord,
        );
        let remaster = record(
            "remaster",
            RecordKind::Rule,
            RecordVisibilityReason::SourceRecord,
        );
        let policy = ProductRetrievalPolicy::from_remaster_links(&[RemasterLink {
            remaster_record_key: remaster.identity.key.clone(),
            legacy_record_key: legacy.identity.key.clone(),
            source: RemasterLinkSource::Migration,
            source_ref: "fixture".to_string(),
        }]);

        let cases = [
            (
                ordinary,
                RetrievalPolicyDecision {
                    role: RecordRole::Source,
                    disposition: RetrievalDisposition::Ordinary,
                    rationale: RetrievalRationale::SourceRecord,
                },
            ),
            (
                canonical,
                RetrievalPolicyDecision {
                    role: RecordRole::Canonical,
                    disposition: RetrievalDisposition::Ordinary,
                    rationale: RetrievalRationale::GeneratedCanonical,
                },
            ),
            (
                source_instance,
                RetrievalPolicyDecision {
                    role: RecordRole::SourceInstance,
                    disposition: RetrievalDisposition::DirectOnly,
                    rationale: RetrievalRationale::DuplicateSourceInstance,
                },
            ),
            (
                tooling,
                RetrievalPolicyDecision {
                    role: RecordRole::Source,
                    disposition: RetrievalDisposition::InspectionOnly,
                    rationale: RetrievalRationale::ToolingNoAddressableProductMeaning,
                },
            ),
            (
                legacy,
                RetrievalPolicyDecision {
                    role: RecordRole::Source,
                    disposition: RetrievalDisposition::DirectOnly,
                    rationale: RetrievalRationale::CanonicalEditionDuplicate,
                },
            ),
        ];

        for (record, expected) in cases {
            assert_eq!(policy.decision(&record), expected);
            assert_eq!(
                policy.is_ordinary(&record),
                expected.disposition.is_ordinary()
            );
        }
        assert_eq!(
            policy.decision(&remaster).disposition,
            RetrievalDisposition::Ordinary
        );
    }

    #[test]
    fn authored_visibility_classification_does_not_change_retrieval_disposition() {
        let mut record = record(
            "authored-hidden",
            RecordKind::Rule,
            RecordVisibilityReason::SourceRecord,
        );
        record.visibility = RecordVisibility::hidden(RecordVisibilityReason::SourceRecord);

        assert_eq!(
            ProductRetrievalPolicy::from_remaster_links(&[])
                .decision(&record)
                .disposition,
            RetrievalDisposition::Ordinary
        );
    }

    fn record(id: &str, kind: RecordKind, reason: RecordVisibilityReason) -> AtlasRecord {
        let pack_name = PackName::new("test-pack").expect("pack parses");
        let id = RecordId::new(id).expect("id parses");
        let mut record = AtlasRecord::new(
            RecordIdentity::new(RecordKey::new(pack_name, id), "Test Record"),
            RecordClassification::new(kind),
            FoundryRecordInfo::new(
                "Test Pack",
                FoundryDocumentType::Item,
                FoundryRecordType::Action,
            ),
            RecordProvenance::new("test.json").with_raw_json("{}"),
        );
        record.visibility = match reason {
            RecordVisibilityReason::SourceRecord | RecordVisibilityReason::GeneratedCanonical => {
                RecordVisibility::visible(reason)
            }
            RecordVisibilityReason::GeneratedInstance => RecordVisibility::hidden(reason),
        };
        record
    }
}
