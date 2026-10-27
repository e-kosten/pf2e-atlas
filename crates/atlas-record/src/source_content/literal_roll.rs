//! Literal Foundry inline rolls; captions are prose, never part of the expression.
use atlas_dice::RollPreviewExpression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteralContentRoll {
    pub formula: String,
    pub caption: String,
    pub has_dice: bool,
    pub expression: RollPreviewExpression,
}

pub fn literal_content_roll(command: &str, arguments: &str) -> Option<LiteralContentRoll> {
    if !matches!(command, "r" | "gmr" | "br") {
        return None;
    }
    let (formula, caption) = arguments.split_once('#').unwrap_or((arguments, ""));
    let formula = formula.trim();
    // A single-term Foundry pool has the same scalar result. Multiple entries,
    // pool modifiers and nested pools stay outside this literal subset.
    let formula = formula
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
        .unwrap_or(formula)
        .trim();
    // Foundry division/rounding and flavored/runtime expressions have different
    // semantics. Admit only the shared literal integer-arithmetic subset.
    if !formula
        .chars()
        .all(|ch| ch.is_ascii_digit() || ch.is_ascii_whitespace() || "dD+-*()".contains(ch))
    {
        return None;
    }
    let prepared = atlas_dice::prepare(formula).ok()?;
    let expression = prepared.preview_expression().ok()?;
    fn has_dice(expression: &RollPreviewExpression) -> bool {
        match expression {
            RollPreviewExpression::Dice { .. } => true,
            RollPreviewExpression::Binary { left, right, .. } => has_dice(left) || has_dice(right),
            RollPreviewExpression::Negate { inner } => has_dice(inner),
            RollPreviewExpression::Number { .. } => false,
        }
    }
    Some(LiteralContentRoll {
        formula: formula.into(),
        caption: caption.trim().into(),
        has_dice: has_dice(&expression),
        expression,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roll_modes_share_literal_arithmetic_without_interpreting_captions() {
        for command in ["r", "gmr", "br"] {
            let roll = literal_content_roll(command, "2d6+3 #Recharge Breath Weapon").unwrap();
            assert_eq!(roll.formula, "2d6+3");
            assert_eq!(roll.caption, "Recharge Breath Weapon");
            assert!(roll.has_dice);
            assert!(!literal_content_roll(command, "4 #rounds").unwrap().has_dice);
            assert_eq!(
                literal_content_roll(command, "{2d8} #Stamina Points")
                    .unwrap()
                    .formula,
                "2d8"
            );
            for unsupported in [
                "@actor.level+1d6",
                "resolve(1+2d6)",
                "1d6[fire]",
                "1d6kh",
                "1d6/2",
                "201d6",
                "2d1",
                "1d6 trailing prose",
            ] {
                assert!(
                    literal_content_roll(command, unsupported).is_none(),
                    "{unsupported}"
                );
            }
        }
        assert!(literal_content_roll("act", "seek").is_none());
    }
}
