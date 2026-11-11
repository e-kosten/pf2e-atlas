//! Typed vocabulary retrieval on the same admitted reference snapshot.
use crate::{AtlasRetrievalService, SearchError, SearchPage, SearchPageInfo};
use atlas_domain::TraitKey;
pub use atlas_index::{ReferenceSourceLocation, TraitDefinition, TraitMembership};
pub struct TraitAssociationRequest<'a> {
    pub trait_key: &'a TraitKey,
    pub page: SearchPage,
}
pub struct TraitMembershipResult {
    pub entries: Vec<TraitMembership>,
    pub page: SearchPageInfo,
}
impl AtlasRetrievalService {
    pub fn trait_definition(&self, key: &TraitKey) -> Result<Option<TraitDefinition>, SearchError> {
        Ok(self.index.trait_definition(key)?)
    }
    pub fn trait_memberships(
        &self,
        request: TraitAssociationRequest<'_>,
    ) -> Result<TraitMembershipResult, SearchError> {
        let (entries, total) = self.index.trait_memberships(
            request.trait_key,
            request.page.size(),
            request.page.offset()?,
        )?;
        let page = SearchPageInfo::from_page(request.page, entries.len(), total)?;
        Ok(TraitMembershipResult { entries, page })
    }
}
