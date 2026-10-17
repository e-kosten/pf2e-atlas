use super::{SourceFieldView, SourceNodeView};
use atlas_domain::{PackName, RecordId, RecordKey};
use atlas_foundry_model::FoundryDocumentSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceIdentityError {
    InvalidPack,
    MissingId,
    NullId,
    RejectedId,
    InvalidId,
    DuplicateId,
    UnsupportedRoot,
}

/// Source-record addressing requires 16 ASCII alphanumeric characters. This
/// policy is narrower than Atlas's general RecordId and does not change it.
pub(super) fn valid_source_id(id: &str) -> bool {
    id.len() == 16 && id.bytes().all(|c| c.is_ascii_alphanumeric())
}

pub fn source_record_key(
    pack: &str,
    source: &FoundryDocumentSource,
) -> Result<RecordKey, SourceIdentityError> {
    if pack.contains(':') {
        return Err(SourceIdentityError::InvalidPack);
    }
    let pack = PackName::new(pack).map_err(|_| SourceIdentityError::InvalidPack)?;
    let id = match SourceNodeView::from(source).id() {
        SourceFieldView::Value(id) if valid_source_id(id) => id,
        SourceFieldView::Value(_) => return Err(SourceIdentityError::InvalidId),
        SourceFieldView::Missing => return Err(SourceIdentityError::MissingId),
        SourceFieldView::Null => return Err(SourceIdentityError::NullId),
        SourceFieldView::Invalid(_) | SourceFieldView::ProjectionInvalid { .. } => {
            return Err(SourceIdentityError::RejectedId);
        }
        SourceFieldView::NotApplicable => return Err(SourceIdentityError::UnsupportedRoot),
    };
    Ok(RecordKey::new(
        pack,
        RecordId::new(id).map_err(|_| SourceIdentityError::InvalidId)?,
    ))
}
