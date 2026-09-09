use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::TableResultView;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "state", rename_all = "snake_case")]
#[ts(tag = "state", rename_all = "snake_case")]
pub enum TableRollCapabilityView {
    Available {
        formula: String,
        #[ts(type = "number")]
        sides: u32,
    },
    Unavailable {
        unavailable: TableRollUnavailableView,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct TableRollUnavailableView {
    pub reason: TableRollUnavailableReasonView,
    pub subject: TableRollSubjectView,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum TableRollUnavailableReasonView {
    FormulaMissing,
    FormulaNull,
    FormulaUnsupported,
    FormulaInvalid,
    ResultsMissing,
    ResultsNull,
    ResultsUnsupported,
    UnsupportedResult,
    RangeMissing,
    RangeNull,
    RangeUnsupported,
    RangeOutOfDomain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "subject_type", rename_all = "snake_case")]
#[ts(tag = "subject_type", rename_all = "snake_case")]
pub enum TableRollSubjectView {
    Table,
    Result {
        locator: String,
        #[ts(type = "number")]
        source_ordinal: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "state", rename_all = "snake_case")]
#[ts(tag = "state", rename_all = "snake_case")]
pub enum TableRollView {
    Available {
        table_key: String,
        formula: String,
        #[ts(type = "number")]
        total: u32,
        outcomes: Vec<TableResultView>,
    },
    Unavailable {
        table_key: String,
        unavailable: TableRollUnavailableView,
    },
}
