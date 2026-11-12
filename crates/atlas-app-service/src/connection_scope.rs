//! Static connected discovery, resolved in the consuming workspace snapshot.
use crate::{AppServiceError, AppServiceResult};
use atlas_app_model::{
    AppRecoverableAction, ConnectionScopeUnavailableReason, ConnectionScopeView,
    SearchConnectionScope,
};
use atlas_domain::{QueryFacetOutcome, QuerySelfExclusion, RecordKey, exclude_query_selection};
use atlas_search::RecordScope;

pub(crate) fn check_generation(expected: Option<&str>, actual: &str) -> AppServiceResult<()> {
    if let Some(expected) = expected
        && expected != actual
    {
        return Err(AppServiceError::artifact_generation_changed(
            expected,
            actual,
            AppRecoverableAction::RefreshResults,
        ));
    }
    Ok(())
}

pub(crate) struct ResolvedConnectionScope {
    pub keys: Option<Vec<RecordKey>>,
    pub view: ConnectionScopeView,
}

pub(crate) fn resolve_connection_scope(
    workspace: &mut atlas_runtime::WorkspaceRead,
    context: &SearchConnectionScope,
) -> AppServiceResult<ResolvedConnectionScope> {
    validate_connection_scope(workspace.retrieval(), context)?;
    resolve(workspace, context)
}
pub(crate) fn validate_connection_scope(
    retrieval: &atlas_search::AtlasRetrievalService,
    context: &SearchConnectionScope,
) -> AppServiceResult<()> {
    let mut current = Some(context);
    let mut depth = 0;
    while let Some(scope) = current {
        depth += 1;
        if depth > 4 {
            return Err(AppServiceError::invalid_request(
                "Link pickers may be nested at most four levels",
            ));
        }
        let parent = &scope.parent;
        check_generation(
            parent.expected_artifact_generation.as_deref(),
            &retrieval.artifact_generation()?,
        )?;
        if let Some(filter) = &parent.filter {
            retrieval.validate_filter(filter)?;
        }
        if let Some(id) = &parent.exclude_clause_id {
            let (target, member) = parent
                .filter
                .as_ref()
                .and_then(|filter| atlas_domain::addressed_query_clause(filter, id))
                .ok_or_else(|| {
                    AppServiceError::invalid_request("The addressed link condition does not exist")
                })?;
            if member.is_some()
                || !matches!(
                    target.expression,
                    atlas_domain::QueryExpression::Relationship { .. }
                        | atlas_domain::QueryExpression::BooleanConstant { value: true }
                )
            {
                return Err(AppServiceError::invalid_request(
                    "Link self-exclusion requires a root relationship condition",
                ));
            }
        }
        current = parent.connection_scope.as_deref();
    }
    Ok(())
}
fn intersect(keys: &mut Option<Vec<RecordKey>>, restriction: Vec<RecordKey>) {
    *keys = Some(match keys.take() {
        Some(keys) => {
            let allowed: std::collections::BTreeSet<_> = restriction.into_iter().collect();
            keys.into_iter()
                .filter(|key| allowed.contains(key))
                .collect()
        }
        None => restriction,
    });
}
fn resolve(
    workspace: &mut atlas_runtime::WorkspaceRead,
    context: &SearchConnectionScope,
) -> AppServiceResult<ResolvedConnectionScope> {
    let parent = &context.parent;
    let inherited = parent
        .connection_scope
        .as_deref()
        .map(|scope| resolve(workspace, scope))
        .transpose()?;
    let mut keys = inherited.and_then(|scope| scope.keys);
    if let Some(list_keys) =
        crate::filters::scoped_list_keys(workspace, parent.list_ref.as_deref())?
    {
        intersect(&mut keys, list_keys);
    }
    let retrieval = workspace.retrieval_mut();
    let broad = |reason| ResolvedConnectionScope {
        keys: None,
        view: ConnectionScopeView::AllEntries { reason },
    };
    if parent.incomplete {
        return Ok(broad(ConnectionScopeUnavailableReason::IncompleteDraft));
    }
    let filter = if let Some(id) = &parent.exclude_clause_id {
        let filter = parent.filter.as_ref().ok_or_else(|| {
            AppServiceError::invalid_request("Self-exclusion requires an addressed condition")
        })?;
        match exclude_query_selection(filter, QuerySelfExclusion::Clause(id)) {
            None => {
                return Err(AppServiceError::invalid_request(
                    "The addressed link condition does not exist",
                ));
            }
            Some(QueryFacetOutcome::Unavailable { .. }) => {
                return Ok(broad(
                    ConnectionScopeUnavailableReason::NonConjunctiveContext,
                ));
            }
            Some(QueryFacetOutcome::Available { value }) => Some(value.predicate),
        }
    } else {
        parent.filter.clone()
    };
    let keys = retrieval.connected_record_keys(
        filter.as_ref(),
        keys.as_deref()
            .map(RecordScope::exact)
            .unwrap_or(RecordScope::catalog(None)),
        context.direction,
        context.include_contents,
    )?;
    Ok(ResolvedConnectionScope {
        keys: Some(keys),
        view: ConnectionScopeView::Connected,
    })
}
