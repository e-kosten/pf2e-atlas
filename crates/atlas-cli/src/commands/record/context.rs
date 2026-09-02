use atlas_app_model::{AppError, AppErrorCode};
use atlas_record::{
    RecordJson, RecordJsonContext, RecordJsonOptions, RetrievedRecord, record_json_with_context,
};
use atlas_search::RemasterLinksResult;

use crate::client::AtlasClient;

pub(crate) fn project_record(
    client: &impl AtlasClient,
    retrieved: &RetrievedRecord,
    options: RecordJsonOptions,
) -> Result<RecordJson, AppError> {
    let record_key = retrieved.record.identity.key.clone();
    let remaster = client.remaster_links(record_key.clone())?.ok_or_else(|| {
        AppError::new(
            AppErrorCode::QueryFailed,
            format!("canonical record `{record_key}` disappeared during edition lookup"),
        )
    })?;
    project_record_with_remaster(retrieved, options, &remaster)
}

pub(crate) fn project_record_with_remaster(
    retrieved: &RetrievedRecord,
    options: RecordJsonOptions,
    remaster: &RemasterLinksResult,
) -> Result<RecordJson, AppError> {
    let record_key = &retrieved.record.identity.key;
    let edition = remaster.record_edition_lookup().map_err(|error| {
        AppError::new(
            AppErrorCode::QueryFailed,
            format!("edition lookup failed for `{record_key}`: {error}"),
        )
    })?;
    let context =
        RecordJsonContext::without_lookups(&retrieved.record).with_edition_lookup(edition);
    record_json_with_context(retrieved, options, context).map_err(|error| {
        AppError::new(
            AppErrorCode::QueryFailed,
            format!("record projection failed for `{record_key}`: {error}"),
        )
    })
}
