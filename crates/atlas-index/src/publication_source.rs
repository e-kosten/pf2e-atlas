//! Repeatable publication inputs. The writer consumes one bounded record at a time.
use crate::{
    IndexBuildPack, IndexError, SourceAliasInput, SourceArtifactBuildContext,
    SourceArtifactRecordInput, SourceLexicalUnitInput, SourceRemasterPairInput,
    SourceSearchProfileIdentity, TraitDefinition,
};
use atlas_domain::{
    OwnedContentLocator, RecordKey, SearchProfileId, SourcePassageAddress, TraitKey,
};
use atlas_embedding::NativeInputDescriptor;
use atlas_record::source_record::SourceSearchSelection;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
#[cfg(any(test, feature = "test-support"))]
use {crate::IndexBuildInput, atlas_record::source_record::select_record_search};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitSeedOccurrence {
    pub key: TraitKey,
    pub profile_id: SearchProfileId,
    pub input_hash: String,
    pub chunks: Vec<TraitChunkOccurrence>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitChunkOccurrence {
    pub input: String,
    pub descriptor: NativeInputDescriptor,
    pub address: SourcePassageAddress,
}

pub trait NativeValueSource {
    fn value<'a>(
        &'a self,
        model: &SourceSearchProfileIdentity,
        input: &NativeInputDescriptor,
    ) -> Result<Cow<'a, [f32]>, IndexError>;
}

pub trait PublicationSource {
    fn context(&self) -> &SourceArtifactBuildContext;
    fn packs(&self) -> &[IndexBuildPack];
    fn inventory(&self) -> &crate::CompleteSourceInventory;
    fn record_count(&self) -> usize;
    fn semantic_count(&self) -> usize;
    fn expected_instance(&self) -> Option<&crate::ArtifactInstance> {
        None
    }
    fn check_seal(&self) -> Result<(), IndexError>;
    fn records(
        &self,
        visitor: &mut dyn FnMut(PreparedRecordRef<'_>) -> Result<(), IndexError>,
    ) -> Result<(), IndexError>;
    fn lexical(
        &self,
        visitor: &mut dyn FnMut(&SourceLexicalUnitInput) -> Result<(), IndexError>,
    ) -> Result<(), IndexError>;
    fn semantic(
        &self,
        visitor: &mut dyn FnMut(&SemanticOccurrence) -> Result<(), IndexError>,
    ) -> Result<(), IndexError>;
    fn aliases(
        &self,
        visitor: &mut dyn FnMut(&SourceAliasInput) -> Result<(), IndexError>,
    ) -> Result<(), IndexError>;
    fn remaster_pairs(
        &self,
        visitor: &mut dyn FnMut(&SourceRemasterPairInput) -> Result<(), IndexError>,
    ) -> Result<(), IndexError>;
    fn traits(
        &self,
        visitor: &mut dyn FnMut(&TraitDefinition) -> Result<(), IndexError>,
    ) -> Result<(), IndexError>;
    fn trait_seeds(
        &self,
        visitor: &mut dyn FnMut(&TraitSeedOccurrence) -> Result<(), IndexError>,
    ) -> Result<(), IndexError>;
}

#[cfg(any(test, feature = "test-support"))]
impl PublicationSource for IndexBuildInput {
    fn context(&self) -> &SourceArtifactBuildContext {
        &self.context
    }
    fn packs(&self) -> &[IndexBuildPack] {
        &self.packs
    }
    fn inventory(&self) -> &crate::CompleteSourceInventory {
        &self.inventory
    }
    fn record_count(&self) -> usize {
        self.records.len()
    }
    fn semantic_count(&self) -> usize {
        self.semantic_units.len()
    }
    fn check_seal(&self) -> Result<(), IndexError> {
        Ok(())
    }
    fn records(
        &self,
        visitor: &mut dyn FnMut(PreparedRecordRef<'_>) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        for source in &self.records {
            let label = self
                .packs
                .iter()
                .find(|p| p.pack_id == source.record.key().pack().as_str())
                .ok_or_else(|| IndexError::Invalid("record pack is absent".into()))?;
            let selection = select_record_search(
                &source.record,
                &source.content,
                self.context.audience,
                &label.label,
                &self.context.used_trait_labels,
            )
            .map_err(|e| IndexError::Invalid(e.to_string()))?;
            let snapshot = atlas_foundry_model::encode_snapshot(source.record.source())?;
            let lexical = self
                .lexical_units
                .iter()
                .filter(|row| row.record == *source.record.key())
                .cloned()
                .collect::<Vec<_>>();
            visitor(PreparedRecordRef {
                source,
                snapshot: &snapshot,
                selection: &selection,
                lexical: &lexical,
            })?;
        }
        Ok(())
    }
    fn lexical(
        &self,
        visitor: &mut dyn FnMut(&SourceLexicalUnitInput) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        for row in &self.lexical_units {
            visitor(row)?;
        }
        Ok(())
    }
    fn semantic(
        &self,
        visitor: &mut dyn FnMut(&SemanticOccurrence) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        for row in &self.semantic_units {
            visitor(&SemanticOccurrence {
                profile_id: row.profile_id.clone(),
                record: row.record.clone(),
                owners: row.owners.clone(),
                field: row.field.clone(),
                address: row.address.clone(),
                chunk_ordinal: row.chunk_ordinal,
                input: NativeInputDescriptor {
                    input_sha256: row.input_hash.clone(),
                    token_count: row.input_token_count,
                },
            })?;
        }
        Ok(())
    }
    fn aliases(
        &self,
        visitor: &mut dyn FnMut(&SourceAliasInput) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        for row in &self.aliases {
            visitor(row)?;
        }
        Ok(())
    }
    fn remaster_pairs(
        &self,
        visitor: &mut dyn FnMut(&SourceRemasterPairInput) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        for row in &self.remaster_pairs {
            visitor(row)?;
        }
        Ok(())
    }
    fn traits(
        &self,
        visitor: &mut dyn FnMut(&TraitDefinition) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        for row in &self.traits {
            visitor(row)?;
        }
        Ok(())
    }
    fn trait_seeds(
        &self,
        visitor: &mut dyn FnMut(&TraitSeedOccurrence) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        for row in &self.trait_vectors {
            let definition = self
                .traits
                .iter()
                .find(|d| d.key == row.key)
                .ok_or_else(|| IndexError::Invalid("trait seed has no definition".into()))?;
            crate::traits::validate_seed(
                definition,
                self.context.search_profile(&row.profile_id)?,
                &row.input_hash,
                &row.vector,
                &row.chunks,
            )?;
            visitor(&TraitSeedOccurrence {
                key: row.key.clone(),
                profile_id: row.profile_id.clone(),
                input_hash: row.input_hash.clone(),
                chunks: row
                    .chunks
                    .iter()
                    .map(|c| TraitChunkOccurrence {
                        input: c.input.clone(),
                        descriptor: NativeInputDescriptor {
                            input_sha256: c.input_hash.clone(),
                            token_count: c.token_count,
                        },
                        address: c.address.clone(),
                    })
                    .collect(),
            })?;
        }
        Ok(())
    }
}
#[cfg(any(test, feature = "test-support"))]
impl NativeValueSource for IndexBuildInput {
    fn value<'a>(
        &'a self,
        model: &SourceSearchProfileIdentity,
        input: &NativeInputDescriptor,
    ) -> Result<Cow<'a, [f32]>, IndexError> {
        if self.context.search_profile(&model.profile_id)? != model {
            return Err(IndexError::Invalid(
                "native execution identity differs".into(),
            ));
        }
        let mut selected: Option<&[f32]> = None;
        let mut accept = |tokens: usize, vector: &'a [f32]| -> Result<(), IndexError> {
            if tokens != input.token_count {
                return Err(IndexError::Invalid(
                    "duplicate native input token counts disagree".into(),
                ));
            }
            crate::validation::check_vector(vector, model.dimensions)?;
            if selected.is_some_and(|previous| {
                previous.len() != vector.len()
                    || previous
                        .iter()
                        .zip(vector)
                        .any(|(a, b)| a.to_bits() != b.to_bits())
            }) {
                return Err(IndexError::Invalid(
                    "duplicate native input values disagree".into(),
                ));
            }
            selected = Some(vector);
            Ok(())
        };
        for row in &self.semantic_units {
            if row.profile_id == model.profile_id && row.input_hash == input.input_sha256 {
                accept(row.input_token_count, &row.vector)?;
            }
        }
        for row in &self.trait_vectors {
            if row.profile_id == model.profile_id {
                for chunk in &row.chunks {
                    if chunk.input_hash == input.input_sha256 {
                        accept(chunk.token_count, &chunk.vector)?;
                    }
                }
            }
        }
        selected
            .map(Cow::Borrowed)
            .ok_or_else(|| IndexError::Invalid("exact native input value is absent".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixture_values_require_exact_agreement_across_duplicate_native_inputs() {
        let mut input = crate::foundation_tests::fixture(true);
        let model = input.context.search_profiles[0].clone();
        let unit = input.semantic_units[0].clone();
        let descriptor = NativeInputDescriptor {
            input_sha256: unit.input_hash.clone(),
            token_count: unit.input_token_count,
        };
        input.semantic_units.push(unit.clone());
        assert_eq!(
            input.value(&model, &descriptor).unwrap().as_ref(),
            unit.vector
        );
        input.semantic_units.last_mut().unwrap().input_token_count += 1;
        assert!(
            input
                .value(&model, &descriptor)
                .unwrap_err()
                .to_string()
                .contains("token counts")
        );
        let duplicate = input.semantic_units.last_mut().unwrap();
        duplicate.input_token_count = unit.input_token_count;
        duplicate.vector = vec![0.; model.dimensions];
        duplicate.vector[17] = 1.;
        assert!(
            input
                .value(&model, &descriptor)
                .unwrap_err()
                .to_string()
                .contains("values disagree")
        );
        let mut wrong_model = model.clone();
        wrong_model.input_recipe.push_str("-different");
        assert!(input.value(&wrong_model, &descriptor).is_err());
    }
}
