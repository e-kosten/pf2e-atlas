use std::process::ExitCode;

use atlas_cli_support::write_json_data;
use atlas_dice::{RollExpression, RollOperator, RollOutcome};
use clap::Args;

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas roll '1d20+12'\n  atlas roll '2d20kh1+12'\n  atlas roll '(2d6+4)*2' --json\n  atlas roll -- '-1d6+10'\n\nSupports integer arithmetic, parentheses, dice, kh (keep highest) and kl (keep lowest).\nRolls do not change encounters or apply damage."
)]
pub(crate) struct RollArgs {
    #[arg(
        value_name = "FORMULA",
        help = "Dice expression; quote spaces and shell operators"
    )]
    pub(crate) formula: String,
    #[arg(
        long,
        help = "Return the structured result, including every die and kept/dropped state"
    )]
    pub(crate) json: bool,
}

pub(crate) fn run_roll(args: RollArgs) -> Result<ExitCode, String> {
    let outcome = atlas_dice::roll(&args.formula).map_err(|error| error.to_string())?;
    if args.json {
        write_json_data(outcome)?;
    } else {
        println!("{}", render_roll(&outcome));
    }
    Ok(ExitCode::SUCCESS)
}

fn render_roll(outcome: &RollOutcome) -> String {
    format!(
        "{} = {}\n  {}",
        outcome.formula,
        outcome.total,
        render_expression(&outcome.expression, 0)
    )
}

fn render_expression(expression: &RollExpression, minimum_precedence: u8) -> String {
    let (text, precedence) = match expression {
        RollExpression::Number { value } => (value.to_string(), 4),
        RollExpression::Dice { dice, .. } => {
            let faces = dice
                .iter()
                .map(|die| {
                    let dropped = if die.kept { "" } else { " dropped" };
                    format!("d{}: {}{dropped}", die.sides, die.value)
                })
                .collect::<Vec<_>>()
                .join(", ");
            (format!("[{faces}]"), 4)
        }
        RollExpression::Binary {
            operator,
            left,
            right,
            ..
        } => {
            let (symbol, precedence, right_precedence) = match operator {
                RollOperator::Add => ("+", 1, 1),
                RollOperator::Subtract => ("-", 1, 2),
                RollOperator::Multiply => ("*", 2, 3),
                RollOperator::Divide => ("/", 2, 3),
            };
            (
                format!(
                    "{} {symbol} {}",
                    render_expression(left, precedence),
                    render_expression(right, right_precedence)
                ),
                precedence,
            )
        }
        RollExpression::Negate { inner, .. } => (format!("-{}", render_expression(inner, 4)), 3),
    };
    if precedence < minimum_precedence {
        format!("({text})")
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_dice::DieOutcome;

    #[test]
    fn breakdown_preserves_draw_order_discarded_faces_and_arithmetic() {
        let outcome = RollOutcome {
            formula: "2d20kh1+12".into(),
            total: 29,
            expression: RollExpression::Binary {
                operator: RollOperator::Add,
                left: Box::new(RollExpression::Dice {
                    dice: vec![
                        DieOutcome {
                            sides: 20,
                            value: 3,
                            kept: false,
                        },
                        DieOutcome {
                            sides: 20,
                            value: 17,
                            kept: true,
                        },
                    ],
                    total: 17,
                }),
                right: Box::new(RollExpression::Number { value: 12 }),
                total: 29,
            },
        };
        assert_eq!(
            render_roll(&outcome),
            "2d20kh1+12 = 29\n  [d20: 3 dropped, d20: 17] + 12"
        );
    }

    #[test]
    fn breakdown_preserves_grouping_without_nested_subtotals() {
        for (formula, expected) in [
            ("(3+5+4)*2+4", "(3 + 5 + 4) * 2 + 4"),
            ("10-(3-1)", "10 - (3 - 1)"),
            ("12/(6/2)", "12 / (6 / 2)"),
            ("3*(5/2)", "3 * (5 / 2)"),
            ("-(3+4)", "-(3 + 4)"),
            ("-(-4)", "-(-4)"),
        ] {
            let outcome = atlas_dice::roll(formula).unwrap();
            assert_eq!(
                render_expression(&outcome.expression, 0),
                expected,
                "{formula}"
            );
        }
    }
}
