//! Bounded interpretation of PF2e damage-instance annotations, without rolling.
//! The pinned roll-grammar.peggy uses commas for instances and a trailing flavor
//! on each instance. Arithmetic validation belongs to the existing dice library.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteralDamageComponent {
    pub formula: String,
    pub label: String,
    pub has_dice: bool,
    pub expression: atlas_dice::RollPreviewExpression,
    pub kind: LiteralDamageKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralDamageKind {
    Damage,
    Healing,
    DamageOrHealing,
}

pub fn literal_damage(formula: &str) -> Option<Vec<LiteralDamageComponent>> {
    let formula = formula.trim();
    if formula.len() > 4096 || formula.is_empty() {
        return None;
    }
    let formula = if formula.starts_with('{') {
        formula.strip_prefix('{')?.strip_suffix('}')?
    } else {
        formula
    };
    let mut parts = Vec::new();
    let (mut start, mut round, mut square) = (0, 0u32, false);
    for (index, ch) in formula.char_indices() {
        match ch {
            '(' if !square => round = round.checked_add(1).filter(|n| *n <= 32)?,
            ')' if !square => round = round.checked_sub(1)?,
            '[' if !square => square = true,
            ']' if square => square = false,
            '[' | ']' | '{' | '}' => return None,
            ',' if !square && round == 0 => {
                parts.push(component(&formula[start..index])?);
                if parts.len() >= 8 {
                    return None;
                }
                start = index + 1;
            }
            _ => {}
        }
    }
    if square || round != 0 {
        return None;
    }
    parts.push(component(&formula[start..])?);
    Some(parts)
}

fn component(value: &str) -> Option<LiteralDamageComponent> {
    let (formula, flavor) = value.trim().strip_suffix(']')?.rsplit_once('[')?;
    let mut formula = formula.trim();
    // Remove only parentheses enclosing the whole arithmetic expression.
    while formula.starts_with('(') && formula.ends_with(')') {
        let mut depth = 0;
        let encloses = formula.char_indices().all(|(i, ch)| {
            if ch == '(' {
                depth += 1;
            }
            if ch == ')' {
                depth -= 1;
            }
            depth != 0 || i == formula.len() - 1
        });
        if !encloses {
            break;
        }
        formula = formula[1..formula.len() - 1].trim();
    }
    if !formula
        .chars()
        .all(|c| c.is_ascii_digit() || c.is_ascii_whitespace() || "dD+-*()".contains(c))
    {
        return None;
    }
    let prepared = atlas_dice::prepare(formula).ok()?;
    if prepared.bounds().0 < 0 {
        return None;
    }
    let expression = prepared.preview_expression().ok()?;
    let has_dice = has_dice(&expression);
    // PF2e clamps non-persistent damage totals to at least one. Only expose
    // ordinary reference dice when that runtime rule cannot change the result.
    if has_dice && prepared.bounds().0 < 1 {
        return None;
    }
    let tags: Vec<_> = flavor.split(',').map(str::trim).collect();
    let types = [
        "acid",
        "bleed",
        "bludgeoning",
        "cold",
        "electricity",
        "fire",
        "force",
        "mental",
        "piercing",
        "poison",
        "slashing",
        "sonic",
        "spirit",
        "untyped",
        "vitality",
        "void",
    ];
    let materials = [
        "adamantine",
        "cold-iron",
        "dawnsilver",
        "duskwood",
        "orichalcum",
        "silver",
        "sisterstone-dusk",
        "sisterstone-scarlet",
        "warpglass",
    ];
    if tags.is_empty()
        || tags.iter().any(|t| {
            !types.contains(t)
                && !materials.contains(t)
                && !["persistent", "precision", "splash", "healing", "damage"].contains(t)
        })
    {
        return None;
    }
    if tags
        .iter()
        .filter(|t| ["persistent", "precision", "splash"].contains(t))
        .count()
        > 1
    {
        return None;
    }
    let damage_types: Vec<_> = tags.iter().copied().filter(|t| types.contains(t)).collect();
    if damage_types.len() > 1 {
        return None;
    }
    let damage_type = damage_types.first().copied().unwrap_or("untyped");
    let healing = tags.contains(&"healing");
    if healing
        && (!matches!(damage_type, "untyped" | "void" | "vitality")
            || tags.contains(&"persistent")
            || tags.iter().any(|t| materials.contains(t)))
    {
        return None;
    }
    let mut label: Vec<String> = tags
        .iter()
        .filter(|t| ["persistent", "precision", "splash"].contains(t))
        .map(|t| (*t).to_owned())
        .collect();
    label.extend(
        tags.iter()
            .filter(|t| materials.contains(t))
            .map(|t| t.replace('-', " ")),
    );
    if damage_type != "untyped" || tags.contains(&"untyped") {
        label.push(damage_type.into());
    }
    label.push(
        if healing && tags.contains(&"damage") {
            "damage or healing"
        } else if healing {
            "healing"
        } else {
            "damage"
        }
        .into(),
    );
    Some(LiteralDamageComponent {
        formula: formula.to_owned(),
        label: label.join(" "),
        has_dice,
        expression,
        kind: if healing && tags.contains(&"damage") {
            LiteralDamageKind::DamageOrHealing
        } else if healing {
            LiteralDamageKind::Healing
        } else {
            LiteralDamageKind::Damage
        },
    })
}

fn has_dice(expression: &atlas_dice::RollPreviewExpression) -> bool {
    use atlas_dice::RollPreviewExpression as E;
    match expression {
        E::Dice { .. } => true,
        E::Binary { left, right, .. } => has_dice(left) || has_dice(right),
        E::Negate { inner } => has_dice(inner),
        E::Number { .. } => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literals_keep_typed_instances_and_never_roll_during_interpretation() {
        let flat = literal_damage("1[fire]").unwrap();
        assert_eq!(flat[0].formula, "1");
        assert_eq!(flat[0].label, "fire damage");
        assert!(!flat[0].has_dice);
        let parts =
            literal_damage("{(4d8+10)[bludgeoning],1d6[persistent,fire],10[healing]}").unwrap();
        assert_eq!(parts[0].formula, "4d8+10");
        assert_eq!(parts[0].label, "bludgeoning damage");
        assert!(parts[0].has_dice);
        assert_eq!(parts[1].label, "persistent fire damage");
        assert_eq!(parts[2].label, "healing");
        assert!(!parts[2].has_dice);
    }

    #[test]
    fn unsupported_or_partial_meanings_are_not_made_executable() {
        for value in [
            "",
            "1d6",
            "@actor.level[fire]",
            "resolve(2+@item.badge.value)[fire]",
            "1d6[fire]+1d6[cold]",
            "1d6[fire,unknown]",
            "1d6[fire,cold]",
            "1d6[fire,healing]",
            "201d6[fire]",
            "1d6/2[fire]",
            "1d6[fire],broken",
            "(1d6[precision]+3)[piercing]",
            "-10[fire]",
            "(1d6-1)[fire]",
            "1d6[persistent,precision,fire]",
            "1d6[fire",
            "{1d6[fire]}suffix",
        ] {
            assert!(literal_damage(value).is_none(), "{value}");
        }
        let parts = literal_damage("(1d6)+(2d6)[fire]").unwrap();
        assert_eq!(parts[0].formula, "(1d6)+(2d6)");
    }
}
