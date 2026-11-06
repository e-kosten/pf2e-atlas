//! Complete glossary filter choices remain artifact-scoped and model-free.
use crate::{AtlasRetrievalService, SearchError};
pub use atlas_index::{
    DefinitionFilterOption, DefinitionFilterOptionsPage, DefinitionFilterOptionsRequest,
    DefinitionFilterSelection, DefinitionFilterSelectionAvailability,
};
impl AtlasRetrievalService {
    pub fn definition_filter_options(
        &self,
        request: DefinitionFilterOptionsRequest,
    ) -> Result<DefinitionFilterOptionsPage, SearchError> {
        Ok(self.index.definition_filter_options(request)?)
    }
}
