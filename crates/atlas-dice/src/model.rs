use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, thiserror::Error)]
pub enum DiceError {
    #[error("{0}")]
    Invalid(String),
    #[error("This dice syntax is not supported")]
    Unsupported,
    #[error("Roll exceeds the supported size or arithmetic limits")]
    Limit,
    #[error("The dice evaluator returned an unsupported result")]
    ResultShape,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct DieOutcome {
    pub sides: u32,
    pub value: i32,
    pub kept: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum RollOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// A result tree preserves arithmetic context; mixed damage types are separate rolls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RollExpression {
    Number {
        value: i32,
    },
    Dice {
        dice: Vec<DieOutcome>,
        total: i32,
    },
    Binary {
        operator: RollOperator,
        left: Box<RollExpression>,
        right: Box<RollExpression>,
        total: i32,
    },
    Negate {
        inner: Box<RollExpression>,
        total: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct RollOutcome {
    pub formula: String,
    pub total: i32,
    pub expression: RollExpression,
}

/// A validated formula before rolling; carries no outcomes or kept/discarded claims.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RollPreviewExpression {
    Number {
        value: i32,
    },
    Dice {
        sides: Vec<u32>,
    },
    Binary {
        operator: RollOperator,
        left: Box<RollPreviewExpression>,
        right: Box<RollPreviewExpression>,
    },
    Negate {
        inner: Box<RollPreviewExpression>,
    },
}
