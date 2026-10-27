//! Bounded transient arithmetic. Only callers supply authoritative runtime values.
mod damage;
mod expression;
mod html;
pub use damage::{contextual_damage_text, resolved_contextual_damage};
use expression::Expr;
pub use html::render_contextual_interactions;

#[derive(Clone, Debug, Default)]
pub struct FormulaContext {
    pub actor_level: Option<i64>,
    pub item_rank: Option<u8>,
    pub item_level: Option<i64>,
    pub badge_value: Option<i64>,
    pub item_is_spell: bool,
    pub condition_name: Option<String>,
}
impl FormulaContext {
    fn value(&self, name: &str) -> Option<f64> {
        match name {
            "@actor.level" => self.actor_level.map(|n| n as f64),
            "@item.rank" => self.item_rank.map(f64::from),
            "@item.level" if self.item_is_spell => self.item_rank.map(f64::from),
            "@item.level" => self.item_level.map(|n| n as f64),
            "@item.badge.value" => self.badge_value.map(|n| n as f64),
            _ => None,
        }
    }
    fn label(&self, name: &str) -> String {
        match name {
            "@actor.level" => "the creature’s level".into(),
            "@item.rank" => "spell rank".into(),
            "@item.level" if self.item_is_spell => "spell rank".into(),
            "@item.level" => "item level".into(),
            "@item.badge.value" => self
                .condition_name
                .as_ref()
                .map(|name| format!("your {} value", name.to_lowercase()))
                .unwrap_or_else(|| "the effect’s counter value".into()),
            "@kingdom.control.dc" => "your kingdom’s Control DC".into(),
            "@self.level" => "a level-based DC".into(),
            _ => name.into(),
        }
    }
}
#[derive(Clone, Debug)]
pub struct FormulaText {
    pub text: String,
    pub formula_range: Option<(usize, usize)>,
}
impl FormulaText {
    pub fn html(&self) -> String {
        let escape = super::escape_content_text;
        match self.formula_range {
            Some((start, end)) => format!(
                "{}<span data-atlas-roll-formula=\"\">{}</span>{}",
                escape(&self.text[..start]),
                escape(&self.text[start..end]),
                escape(&self.text[end..])
            ),
            None => escape(&self.text),
        }
    }
}
pub fn resolve_contextual_formula(input: &str, context: &FormulaContext) -> Option<String> {
    Expr::parse(input)?.substitute(context)?.literal()
}
pub fn describe_contextual_formula(input: &str, context: &FormulaContext) -> Option<String> {
    Some(
        Expr::parse(input)?
            .substitute(context)?
            .description(context),
    )
}

pub(crate) enum DcText {
    Number(String),
    Rule(String),
}
pub(crate) fn contextual_dc(
    input: &str,
    adjustment: &str,
    context: &FormulaContext,
) -> Option<DcText> {
    let expr = Expr::parse(input)?;
    let expr = if adjustment != "0" {
        expression::Expr::Binary(
            expression::Op::Add,
            Box::new(expr),
            Box::new(Expr::parse(adjustment)?),
        )
    } else {
        expr
    };
    let expr = expr.substitute(context)?;
    if let Expr::Number(n) = expr {
        return Some(DcText::Number(expression::number(n)));
    }
    let text = expr.description(context);
    Some(DcText::Rule(
        if input.contains("@self.level") || input.contains("@kingdom.control.dc") {
            format!("against {text}")
        } else {
            format!("with a DC equal to {text}")
        },
    ))
}

/// Resolve only the arithmetic part; a caption never supplies executable input.
pub fn resolved_contextual_roll(
    command: &str,
    arguments: &str,
    context: &FormulaContext,
) -> Option<super::LiteralContentRoll> {
    if let Some(roll) = super::literal_content_roll(command, arguments) {
        return Some(roll);
    }
    let (formula, caption) = arguments.split_once('#').unwrap_or((arguments, ""));
    let formula = formula.trim();
    let formula = formula
        .strip_prefix('{')
        .and_then(|f| f.strip_suffix('}'))
        .unwrap_or(formula);
    let formula = resolve_contextual_formula(formula, context)?;
    super::literal_content_roll(command, &format!("{formula} #{caption}"))
}

pub fn contextual_roll_text(
    command: &str,
    arguments: &str,
    context: &FormulaContext,
) -> Option<FormulaText> {
    if !matches!(command, "r" | "gmr" | "br") {
        return None;
    }
    let (_, caption) = arguments.split_once('#').unwrap_or((arguments, ""));
    let (amount, formula_range) =
        if let Some(roll) = resolved_contextual_roll(command, arguments, context) {
            let length = roll.formula.len();
            (roll.formula, roll.has_dice.then_some((0, length)))
        } else {
            let formula = arguments.split_once('#').map_or(arguments, |p| p.0).trim();
            (describe_contextual_formula(formula, context)?, None)
        };
    Some(FormulaText {
        text: if caption.trim().is_empty() {
            amount
        } else {
            format!("{amount} {}", caption.trim())
        },
        formula_range,
    })
}

#[cfg(test)]
mod tests;
