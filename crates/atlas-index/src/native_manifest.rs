//! Exact all-profile native membership, independent of occurrence storage IDs.
use crate::{IndexError, PublicationSource, SourceSearchProfileIdentity};
use atlas_embedding::NativeInputDescriptor;
use sha2::{Digest, Sha256};

/// Bounds numerical lookup and checkpoint buffers independently of corpus size.
pub const NATIVE_VALUE_BATCH_LIMIT: usize = 128;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct NativeManifest {
    pub digest: Vec<u8>,
    pub occurrences: usize,
}

/// Preserve occurrence order in the proof while SQLite deduplicates membership.
/// All requested profiles are covered before this proof can authorize promotion.
pub(crate) fn visit_native_manifest(
    source: &dyn PublicationSource,
    visitor: &mut dyn FnMut(
        &SourceSearchProfileIdentity,
        &NativeInputDescriptor,
    ) -> Result<(), IndexError>,
) -> Result<NativeManifest, IndexError> {
    source.check_seal()?;
    let mut hash = Sha256::new();
    hash.update(source.context().sha256()?.as_bytes());
    let mut occurrences = 0usize;
    let mut visit = |model: &SourceSearchProfileIdentity, input: &NativeInputDescriptor| {
        crate::embedding_values::digest(&input.input_sha256)?;
        if input.token_count == 0 || input.token_count > model.token_limit {
            return Err(IndexError::Invalid(
                "invalid native membership token count".into(),
            ));
        }
        let encoded = serde_json::to_vec(&(model, input))?;
        hash.update((encoded.len() as u64).to_le_bytes());
        hash.update(encoded);
        occurrences = occurrences
            .checked_add(1)
            .ok_or_else(|| IndexError::Invalid("native membership count overflow".into()))?;
        visitor(model, input)
    };
    let mut semantic = 0usize;
    source.semantic(&mut |occurrence| {
        semantic += 1;
        visit(
            source.context().search_profile(&occurrence.profile_id)?,
            &occurrence.input,
        )
    })?;
    if semantic != source.semantic_count() {
        return Err(IndexError::Invalid(
            "native semantic membership count differs from sealed source".into(),
        ));
    }
    source.trait_seeds(&mut |seed| {
        let model = source.context().search_profile(&seed.profile_id)?;
        for chunk in &seed.chunks {
            visit(model, &chunk.descriptor)?;
        }
        Ok(())
    })?;
    source.check_seal()?;
    Ok(NativeManifest {
        digest: hash.finalize().to_vec(),
        occurrences,
    })
}
