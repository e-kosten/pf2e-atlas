#![deny(unsafe_code)]

mod evaluation;
mod model;
mod validation;

pub use evaluation::{PreparedRoll, prepare, roll};
pub use model::{
    DiceError, DieOutcome, RollExpression, RollOperator, RollOutcome, RollPreviewExpression,
};
