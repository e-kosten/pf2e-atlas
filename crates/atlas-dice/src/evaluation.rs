use crate::{DiceError, DieOutcome, RollExpression, RollOperator, RollOutcome};
use diceng::{BinaryOp, Expression, ParseResult, RollResult};

/// Parsed and bounded before any random outcomes are generated.
pub struct PreparedRoll {
    formula: String,
    expression: Expression,
}

pub fn prepare(formula: &str) -> Result<PreparedRoll, DiceError> {
    // Bound parser recursion/allocation as well as evaluated work.
    if formula.is_empty() || formula.len() > 256 {
        return Err(DiceError::Limit);
    }
    let expression = match diceng::parse(formula) {
        ParseResult::Success(expr) => expr,
        ParseResult::Failure(errors) => {
            return Err(DiceError::Invalid(
                errors
                    .first()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "Invalid dice expression".into()),
            ));
        }
    };
    crate::validation::validate(&expression, 0, &mut 0)?;
    Ok(PreparedRoll {
        formula: formula.into(),
        expression,
    })
}

pub fn roll(formula: &str) -> Result<RollOutcome, DiceError> {
    prepare(formula)?.roll()
}

impl PreparedRoll {
    pub fn roll(&self) -> Result<RollOutcome, DiceError> {
        self.evaluate(diceng::RandomRng::new())
    }

    fn evaluate(&self, rng: impl diceng::DiceRng) -> Result<RollOutcome, DiceError> {
        let result = diceng::Roller::new(rng).roll(&self.expression);
        Ok(RollOutcome {
            formula: self.formula.clone(),
            total: result.value(),
            expression: project(&result)?,
        })
    }
}

fn project(result: &RollResult) -> Result<RollExpression, DiceError> {
    Ok(match result {
        RollResult::Literal { value } => RollExpression::Number { value: *value },
        RollResult::Die { .. } | RollResult::DicePool { .. } | RollResult::Filtered { .. } => {
            let mut dice = Vec::new();
            collect_dice(result, true, &mut dice)?;
            RollExpression::Dice {
                dice,
                total: result.value(),
            }
        }
        RollResult::BinaryOp {
            op,
            left,
            right,
            value,
        } => RollExpression::Binary {
            operator: match op {
                BinaryOp::Add => RollOperator::Add,
                BinaryOp::Sub => RollOperator::Subtract,
                BinaryOp::Mul => RollOperator::Multiply,
                BinaryOp::Div => RollOperator::Divide,
            },
            left: Box::new(project(left)?),
            right: Box::new(project(right)?),
            total: *value,
        },
        RollResult::UnaryMinus { inner, value } => RollExpression::Negate {
            inner: Box::new(project(inner)?),
            total: *value,
        },
        _ => return Err(DiceError::ResultShape),
    })
}
fn collect_dice(
    result: &RollResult,
    kept: bool,
    dice: &mut Vec<DieOutcome>,
) -> Result<(), DiceError> {
    match result {
        RollResult::Die { sides, value } => dice.push(DieOutcome {
            sides: *sides,
            value: *value,
            kept,
        }),
        RollResult::DicePool { results, .. } => {
            for r in results {
                collect_dice(r, kept, dice)?;
            }
        }
        RollResult::Filtered {
            original,
            kept: retained,
            ..
        } => {
            let mut remaining = std::collections::BTreeMap::new();
            for r in retained {
                let RollResult::Die { sides, value } = r else {
                    return Err(DiceError::ResultShape);
                };
                *remaining.entry((*sides, *value)).or_insert(0usize) += 1;
            }
            let start = dice.len();
            collect_dice(original, kept, dice)?;
            // Keep the original draw order. Equal faces have no identity in the
            // library's retained list; assign tied retained faces left to right.
            for die in &mut dice[start..] {
                let count = remaining.entry((die.sides, die.value)).or_default();
                die.kept = kept && *count > 0;
                *count = count.saturating_sub(1);
            }
            if remaining.values().any(|count| *count != 0) {
                return Err(DiceError::ResultShape);
            }
        }
        _ => return Err(DiceError::ResultShape),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Faces(std::vec::IntoIter<u32>);
    impl diceng::DiceRng for Faces {
        fn roll(&mut self, sides: u32) -> u32 {
            let n = self.0.next().expect("unexpected random draw");
            assert!((1..=sides).contains(&n));
            n
        }
    }
    fn fixed(formula: &str, values: Vec<u32>) -> RollOutcome {
        prepare(formula)
            .unwrap()
            .evaluate(Faces(values.into_iter()))
            .unwrap()
    }
    #[test]
    fn mixed_pools_preserve_sides_and_arithmetic() {
        let r = fixed("(1d8 + 1d6 + 4) * 2", vec![8, 6]);
        assert_eq!(r.total, 36);
        let RollExpression::Binary {
            operator: RollOperator::Multiply,
            left,
            ..
        } = r.expression
        else {
            panic!()
        };
        let RollExpression::Binary { left, .. } = *left else {
            panic!()
        };
        let RollExpression::Binary { left, right, .. } = *left else {
            panic!()
        };
        assert!(
            matches!(*left,RollExpression::Dice{ref dice,..} if dice[0].sides==8 && dice[0].value==8)
        );
        assert!(
            matches!(*right,RollExpression::Dice{ref dice,..} if dice[0].sides==6 && dice[0].value==6)
        );
    }
    #[test]
    fn keep_high_low_and_ties_retain_every_face() {
        for (formula, total) in [("2d20kh1", 19), ("2d20kl1", 3)] {
            let r = fixed(formula, vec![3, 19]);
            assert_eq!(r.total, total);
            let RollExpression::Dice { dice, .. } = r.expression else {
                panic!()
            };
            assert_eq!(dice.len(), 2);
            assert_eq!(dice.iter().filter(|d| d.kept).count(), 1);
            assert_eq!(dice.iter().find(|d| d.kept).unwrap().value, total);
        }
        let r = fixed("2d20kh1", vec![7, 7]);
        let RollExpression::Dice { dice, .. } = r.expression else {
            panic!()
        };
        assert_eq!(dice.iter().filter(|d| d.kept).count(), 1);
        assert_eq!(dice.len(), 2);
    }
    #[test]
    fn filtered_dice_preserve_draw_order_and_mark_the_selected_position() {
        for (formula, values, flags) in [
            ("2d20kh1", vec![3, 19], vec![false, true]),
            ("2d20kh1", vec![19, 3], vec![true, false]),
            ("2d20kl1", vec![3, 19], vec![true, false]),
            ("2d20kl1", vec![19, 3], vec![false, true]),
            ("2d20kh1", vec![7, 7], vec![true, false]),
            ("4d6kh2", vec![4, 6, 1, 4], vec![true, true, false, false]),
        ] {
            let r = fixed(formula, values.clone());
            let RollExpression::Dice { dice, .. } = r.expression else {
                panic!()
            };
            assert_eq!(
                dice.iter().map(|d| d.value).collect::<Vec<_>>(),
                values.iter().map(|v| *v as i32).collect::<Vec<_>>()
            );
            assert_eq!(dice.iter().map(|d| d.kept).collect::<Vec<_>>(), flags);
            assert_eq!(
                dice.iter().filter(|d| d.kept).map(|d| d.value).sum::<i32>(),
                r.total
            );
        }
    }
    #[test]
    fn rejects_unbounded_unsupported_and_unsafe_before_rolling() {
        for formula in [
            "201d6",
            "100d6+101d6",
            "1d0",
            "0d6",
            "1d6!",
            "1d6r1",
            "4d6dh1",
            "2d20kh3",
            "d6/0",
            "d6/(d2-1)",
            "1000000*1000000",
            "@actor.level",
            "d6[fire]",
            "dF",
            "d{1,2}",
            "d6 garbage",
        ] {
            assert!(prepare(formula).is_err(), "{formula}");
        }
    }
    #[test]
    fn signed_modifiers_and_division_follow_library_integer_semantics() {
        assert_eq!(fixed("1d20-5", vec![3]).total, -2);
        assert_eq!(fixed("-(1d6+1)/2", vec![4]).total, -2);
        assert_eq!(fixed("200d2", vec![1; 200]).total, 200);
    }
}
