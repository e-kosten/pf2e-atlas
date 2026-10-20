//! Completion reads deliberately skip startup migrations and never load inference.
use crate::{AppServiceResult, AtlasAppServiceOptions};
use atlas_runtime::AtlasRuntime;

pub fn complete_saved_list_slugs(
    options: &AtlasAppServiceOptions,
    prefix: &str,
) -> AppServiceResult<Vec<String>> {
    let runtime = AtlasRuntime::resolve(crate::service::runtime_options(options))?;
    let session = runtime.read_state()?;
    Ok(session
        .local_state()
        .saved_lists()
        .list()?
        .into_iter()
        .map(|list| list.slug)
        .filter(|slug| slug.starts_with(prefix))
        .take(100)
        .collect())
}

pub fn complete_record_keys(
    options: &AtlasAppServiceOptions,
    prefix: &str,
) -> AppServiceResult<Vec<String>> {
    let runtime = AtlasRuntime::resolve(crate::service::runtime_options(options))?;
    let session = runtime.read_workspace(None)?;
    Ok(session.retrieval().complete_record_keys(prefix)?)
}
