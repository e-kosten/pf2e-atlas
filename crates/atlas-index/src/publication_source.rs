//! Repeatable publication inputs. The writer consumes one bounded record at a time.
use crate::{SourceArtifactRecordInput, SourceLexicalUnitInput};
use atlas_domain::{OwnedContentLocator, RecordKey, SearchProfileId, SourcePassageAddress};
use atlas_embedding::NativeInputDescriptor;
use atlas_record::source_record::SourceSearchSelection;
use serde::{Deserialize, Serialize};

pub struct PreparedRecordRef<'a> {
    pub source: &'a SourceArtifactRecordInput,
    pub snapshot: &'a [u8],
    pub selection: &'a SourceSearchSelection,
    pub lexical: &'a [SourceLexicalUnitInput],
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticOccurrence {
    pub profile_id: SearchProfileId,
    pub record: RecordKey,
    pub owners: Vec<OwnedContentLocator>,
    pub field: Option<String>,
    pub address: SourcePassageAddress,
    pub chunk_ordinal: usize,
    pub input: NativeInputDescriptor,
}
#[cfg(test)]
mod tests {
    #[test]
    fn fixture_values_require_exact_agreement_across_duplicate_native_inputs() {
        let mut input = crate::foundation_tests::fixture(true);
        let model = input.context.search_profiles[0].clone();
        let unit = input.semantic_units[0].clone();
        input.semantic_units.push(unit.clone());
        let values = crate::test_support::fixture_native_values(&input).unwrap();
        assert_eq!(
            values
                .iter()
                .find(|value| value.input.input_sha256 == unit.input_hash)
                .unwrap()
                .vector,
            unit.vector
        );
        input.semantic_units.last_mut().unwrap().input_token_count += 1;
        assert!(
            crate::test_support::fixture_native_values(&input)
                .unwrap_err()
                .to_string()
                .contains("values disagree")
        );
        let duplicate = input.semantic_units.last_mut().unwrap();
        duplicate.input_token_count = unit.input_token_count;
        duplicate.vector = vec![0.; model.dimensions];
        duplicate.vector[17] = 1.;
        assert!(
            crate::test_support::fixture_native_values(&input)
                .unwrap_err()
                .to_string()
                .contains("values disagree")
        );
    }
}
