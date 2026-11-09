use serde::{Deserialize, Serialize};

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

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DieOutcome {
    pub sides: u32,
    pub value: i32,
    pub kept: bool,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// A result tree preserves arithmetic context; mixed damage types are separate rolls.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
        #[cfg_attr(feature = "openapi", schema(no_recursion))]
        left: Box<RollExpression>,
        #[cfg_attr(feature = "openapi", schema(no_recursion))]
        right: Box<RollExpression>,
        total: i32,
    },
    Negate {
        #[cfg_attr(feature = "openapi", schema(no_recursion))]
        inner: Box<RollExpression>,
        total: i32,
    },
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollOutcome {
    pub formula: String,
    pub total: i32,
    pub expression: RollExpression,
}

/// A validated formula before rolling; carries no outcomes or kept/discarded claims.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
        #[cfg_attr(feature = "openapi", schema(no_recursion))]
        left: Box<RollPreviewExpression>,
        #[cfg_attr(feature = "openapi", schema(no_recursion))]
        right: Box<RollPreviewExpression>,
    },
    Negate {
        #[cfg_attr(feature = "openapi", schema(no_recursion))]
        inner: Box<RollPreviewExpression>,
    },
}
