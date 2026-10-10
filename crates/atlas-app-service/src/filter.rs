//! Structured requests are validated by the retrieval owner without a second grammar.
use crate::{AppServiceResult, AtlasAppService};
use atlas_app_model::FilterValidationResult;
use atlas_domain::QueryPredicate;
impl AtlasAppService {
    pub fn validate_filter(
        &self,
        predicate: QueryPredicate,
    ) -> AppServiceResult<FilterValidationResult> {
        self.submit_retrieval(
            move |retrieval| match retrieval.validate_filter(&predicate) {
                Ok(_) => Ok(FilterValidationResult {
                    predicate: Some(predicate),
                    errors: vec![],
                }),
                Err(error) => match error.query_error.as_deref() {
                    Some(query) => Ok(FilterValidationResult {
                        predicate: None,
                        errors: vec![query.clone()],
                    }),
                    None => Err(error.into()),
                },
            },
        )
    }
}
