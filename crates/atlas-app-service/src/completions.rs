//! Completion reads deliberately skip startup migrations and never load inference.
use crate::{AppServiceResult, AtlasAppServiceOptions};
use atlas_runtime::AtlasRuntime;

pub fn complete_character_keys(
    options: &AtlasAppServiceOptions,
    prefix: &str,
) -> AppServiceResult<Vec<(String, String)>> {
    let runtime = AtlasRuntime::resolve(crate::service::runtime_options(options))?;
    let session = runtime.read_state()?;
    Ok(session
        .local_state()?
        .characters()
        .list()?
        .into_iter()
        .filter(|c| c.character_key.starts_with(prefix))
        .map(|c| (c.character_key, c.name))
        .take(100)
        .collect())
}

pub fn complete_party_keys(
    options: &AtlasAppServiceOptions,
    prefix: &str,
) -> AppServiceResult<Vec<(String, String)>> {
    let runtime = AtlasRuntime::resolve(crate::service::runtime_options(options))?;
    let session = runtime.read_state()?;
    Ok(session
        .local_state()?
        .parties()
        .list()?
        .into_iter()
        .filter(|p| p.party_key.starts_with(prefix))
        .map(|p| (p.party_key, p.name))
        .take(100)
        .collect())
}

pub fn complete_saved_list_slugs(
    options: &AtlasAppServiceOptions,
    prefix: &str,
) -> AppServiceResult<Vec<String>> {
    let runtime = AtlasRuntime::resolve(crate::service::runtime_options(options))?;
    let session = runtime.read_state()?;
    Ok(session
        .local_state()?
        .saved_lists()
        .list(true)?
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
