use atlas_app_model::{AppError, AppErrorCode, FieldEdit};

use crate::AppServiceResult;

#[derive(Default)]
pub(crate) struct SavedEditCheck {
    fields: Vec<&'static str>,
}

impl SavedEditCheck {
    pub(crate) fn value<T: PartialEq>(
        &mut self,
        field: &'static str,
        current: &T,
        edit: FieldEdit<T>,
    ) -> T {
        if &edit.expected != current {
            self.fields.push(field);
        }
        edit.value
    }

    pub(crate) fn finish(self) -> AppServiceResult<()> {
        if self.fields.is_empty() {
            return Ok(());
        }
        let mut error = AppError::new(
            AppErrorCode::SavedEditConflict,
            "Saved values changed since editing began.",
        )
        .with_details(serde_json::json!({ "fields": self.fields }));
        error.retryable = Some(false);
        Err(error.into())
    }
}
