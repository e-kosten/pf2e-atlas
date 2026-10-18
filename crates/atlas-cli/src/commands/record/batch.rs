//! Ordered CLI batch outcomes. Runtime failures stop work without inventing misses.
use atlas_app_model::AppError;
use serde::Serialize;

use super::super::product::exit_class;

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub(super) enum Outcome<T> {
    Ok {
        input: String,
        value: T,
    },
    Error {
        input: String,
        error: AppError,
    },
    Unprocessed {
        input: String,
        reason: UnprocessedReason,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum UnprocessedReason {
    PriorRuntimeFailure,
}

#[derive(Serialize)]
pub(super) struct Batch<T> {
    pub results: Vec<Outcome<T>>,
    #[serde(skip)]
    pub exit: u8,
}

impl<T> Batch<T> {
    /// Borrow successful values for transport presentation without changing outcomes.
    pub fn map_values<'a, U>(&'a self, map: impl Fn(&'a T) -> U) -> Batch<U> {
        Batch {
            exit: self.exit,
            results: self
                .results
                .iter()
                .map(|outcome| match outcome {
                    Outcome::Ok { input, value } => Outcome::Ok {
                        input: input.clone(),
                        value: map(value),
                    },
                    Outcome::Error { input, error } => Outcome::Error {
                        input: input.clone(),
                        error: error.clone(),
                    },
                    Outcome::Unprocessed { input, .. } => Outcome::Unprocessed {
                        input: input.clone(),
                        reason: UnprocessedReason::PriorRuntimeFailure,
                    },
                })
                .collect(),
        }
    }

    pub fn status(&self) -> atlas_cli_support::JsonStatus {
        if self.exit == 0 {
            atlas_cli_support::JsonStatus::Ok
        } else if self
            .results
            .iter()
            .any(|result| matches!(result, Outcome::Ok { .. }))
        {
            atlas_cli_support::JsonStatus::Partial
        } else {
            atlas_cli_support::JsonStatus::Error
        }
    }

    pub fn run(inputs: Vec<String>, mut action: impl FnMut(&str) -> Result<T, AppError>) -> Self {
        let mut batch = Self {
            results: Vec::with_capacity(inputs.len()),
            exit: 0,
        };
        for input in inputs {
            if batch.exit == 3 {
                batch.results.push(Outcome::Unprocessed {
                    input,
                    reason: UnprocessedReason::PriorRuntimeFailure,
                });
                continue;
            }
            match action(&input) {
                Ok(value) => {
                    batch.results.push(Outcome::Ok { input, value });
                }
                Err(error) => {
                    batch.exit = batch.exit.max(exit_class(error.code));
                    batch.results.push(Outcome::Error { input, error });
                }
            }
        }
        batch
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_app_model::AppErrorCode;

    #[test]
    fn mixed_failures_keep_highest_exit_and_stop_before_unprocessed_inputs() {
        let mut visited = Vec::new();
        let batch = Batch::<()>::run(
            ["invalid", "missing", "runtime", "unprocessed"]
                .map(str::to_owned)
                .to_vec(),
            |input| {
                visited.push(input.to_owned());
                Err(AppError::new(
                    match input {
                        "invalid" => AppErrorCode::InvalidRecordKey,
                        "missing" => AppErrorCode::RecordNotFound,
                        _ => AppErrorCode::QueryFailed,
                    },
                    input,
                ))
            },
        );
        assert_eq!(visited, ["invalid", "missing", "runtime"]);
        assert_eq!(batch.exit, 3);
        assert_eq!(batch.status(), atlas_cli_support::JsonStatus::Error);
        let json = serde_json::to_value(&batch).unwrap();
        assert_eq!(json["results"][3]["reason"], "prior_runtime_failure");
        assert!(json.get("counts").is_none());
        assert!(json.get("partial").is_none());
        assert!(json.get("exit").is_none());
    }

    #[test]
    fn runtime_failure_leaves_tail_unprocessed_and_keeps_prior_success() {
        let batch = Batch::run(vec!["ok".into(), "fail".into(), "tail".into()], |input| {
            if input == "ok" {
                Ok(42)
            } else {
                Err(AppError::new(AppErrorCode::QueryFailed, "unavailable"))
            }
        });
        assert_eq!(batch.exit, 3);
        assert_eq!(batch.status(), atlas_cli_support::JsonStatus::Partial);
        assert!(matches!(&batch.results[0], Outcome::Ok { value: 42, .. }));
        assert!(matches!(&batch.results[1], Outcome::Error { .. }));
        assert!(matches!(&batch.results[2], Outcome::Unprocessed { input, .. } if input == "tail"));
    }
}
