use std::num::NonZeroU32;

use atlas_app_model::{AppErrorCode, TableRollView};
use atlas_domain::RecordKey;
use atlas_record::RecordBody;
use atlas_search::{GetRecordRequest, RecordRetrieval};
use rand::Rng;

use crate::{AppServiceError, AppServiceResult, AtlasAppService};

impl AtlasAppService {
    pub fn roll_table(&self, record_key: &str) -> AppServiceResult<TableRollView> {
        self.roll_table_with_entropy(record_key, |sides| {
            rand::rng().random_range(1..=sides.get())
        })
    }

    pub(crate) fn roll_table_with_entropy<F>(
        &self,
        record_key: &str,
        entropy: F,
    ) -> AppServiceResult<TableRollView>
    where
        F: FnOnce(NonZeroU32) -> u32 + Send + 'static,
    {
        let record_key = RecordKey::parse(record_key).map_err(|error| {
            AppServiceError::new(AppErrorCode::InvalidRecordKey, error.to_string())
        })?;
        self.submit_retrieval(move |retrieval| {
            let record = retrieval
                .get_record(GetRecordRequest {
                    record_key: &record_key,
                })?
                .ok_or_else(|| {
                    AppServiceError::new(
                        AppErrorCode::RecordNotFound,
                        format!("record `{record_key}` was not found"),
                    )
                })?;
            let Some(RecordBody::RollTable(table)) = record.body.as_ref() else {
                return Err(AppServiceError::invalid_request(
                    "table-roll is only available for roll-table records",
                ));
            };
            let validated = match table.validate_rollability() {
                Ok(validated) => validated,
                Err(unavailable) => {
                    return Ok(TableRollView::Unavailable {
                        table_key: record_key.to_string(),
                        unavailable: crate::h8_surface::table_roll_unavailable(&unavailable),
                    });
                }
            };
            let formula = validated.normalized_formula();
            let total = entropy(validated.sides());
            let outcomes = validated.results_for_total(total).ok_or_else(|| {
                AppServiceError::new(
                    AppErrorCode::InternalError,
                    "table-roll entropy returned a value outside the validated formula domain",
                )
            })?;
            Ok(TableRollView::Available {
                table_key: record_key.to_string(),
                formula,
                total,
                outcomes: outcomes
                    .into_iter()
                    .map(crate::h8_surface::table_result)
                    .collect(),
            })
        })
    }
}
