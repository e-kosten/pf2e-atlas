use super::{
    FormulaContext, FormulaText,
    expression::{Expr, Function, Op, number},
};
use crate::source_content::literal_damage::damage_instances;
use crate::source_content::{LiteralDamageComponent, literal_damage};

fn split(instance: &str) -> Option<(&str, &str)> {
    instance.strip_suffix(']')?.rsplit_once('[')
}

pub fn resolved_contextual_damage(
    formula: &str,
    context: &FormulaContext,
) -> Option<Vec<LiteralDamageComponent>> {
    if let Some(parts) = literal_damage(formula) {
        return Some(parts);
    }
    let mut parts = Vec::new();
    for instance in damage_instances(formula)? {
        let (amount, flavor) = split(instance)?;
        let amount = Expr::parse(amount)?.substitute(context)?.literal()?;
        parts.push(format!("{amount}[{flavor}]"));
    }
    literal_damage(&parts.join(","))
}

pub fn contextual_damage_text(
    formula: &str,
    context: &FormulaContext,
    short: bool,
) -> Option<FormulaText> {
    let mut fragments = Vec::new();
    let mut formula_range = None;
    let mut offset = 0;
    for instance in damage_instances(formula)? {
        let (amount, flavor) = split(instance)?;
        // Type/category interpretation is shared with ordinary literal damage.
        let labels = literal_damage(&format!("1[{flavor}]"))?;
        let label = &labels.first()?.label;
        let expr = Expr::parse(amount).and_then(|expr| expr.substitute(context));
        let normalized = literal_damage(instance).or_else(|| {
            let literal = expr.as_ref()?.literal()?;
            literal_damage(&format!("{literal}[{flavor}]"))
        });
        let text = if let Some(normalized) = normalized {
            let part = normalized.first()?;
            let amount = &part.formula;
            if formula_range.is_none() && part.has_dice {
                formula_range = Some((offset, offset + amount.len()));
            }
            if short {
                amount.clone()
            } else {
                format!("{amount} {label}")
            }
        } else {
            let expr = expr?;
            // Resolved expressions outside the ordinary damage semantics (such
            // as clamped negative damage) remain authored rather than guessed.
            if expr.literal().is_some() {
                return None;
            }
            if short {
                expr.description(context)
            } else {
                describe_amount(&expr, label, context)
            }
        };
        offset += text.len() + 3;
        fragments.push(text);
    }
    Some(FormulaText {
        text: fragments.join(" + "),
        formula_range,
    })
}

fn rank_multiplier(expr: &Expr, context: &FormulaContext) -> Option<f64> {
    match expr {
        Expr::Variable(name)
            if name == "@item.rank" || (name == "@item.level" && context.item_is_spell) =>
        {
            Some(1.0)
        }
        Expr::Binary(Op::Multiply, a, b) => match (&**a, &**b) {
            (Expr::Number(n), value) | (value, Expr::Number(n)) => {
                Some(*n * rank_multiplier(value, context)?)
            }
            _ => None,
        },
        _ => None,
    }
}
fn describe_amount(expr: &Expr, label: &str, context: &FormulaContext) -> String {
    if let Expr::Binary(Op::Dice, count, sides) = expr
        && let Expr::Number(sides) = **sides
    {
        if let Some(multiplier) = rank_multiplier(count, context) {
            return format!(
                "{}d{} {label} per spell rank",
                number(multiplier),
                number(sides)
            );
        }
        if let Expr::Call(Function::Max, args) = &**count
            && let [Expr::Number(minimum), amount] = args.as_slice()
            && let Some(multiplier) = rank_multiplier(amount, context)
        {
            return format!(
                "{}d{} {label} per spell rank, with a minimum of {}d{}",
                number(multiplier),
                number(sides),
                number(*minimum),
                number(sides)
            );
        }
        return format!(
            "a number of d{}s of {label} equal to {}",
            number(sides),
            count.description(context)
        );
    }
    format!("{label} equal to {}", expr.description(context))
}
