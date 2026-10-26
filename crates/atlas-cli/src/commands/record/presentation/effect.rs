use super::{line, text};
use atlas_app_model::{SpellEffectFactView, SpellEffectPreviewView};
fn fact(out: &mut String, fact: &SpellEffectFactView) {
    text(out, &fact.label, &fact.value);
    if let Some(qualifier) = &fact.qualifier {
        line(out, format!("  {qualifier}"));
    }
}
pub(super) fn statistics(out: &mut String, effect: &SpellEffectPreviewView) {
    line(out, format!("Rank {} statistics", effect.rank));
    if let Some(reason) = &effect.unavailable {
        line(out, format!("Unavailable: {reason}"));
        return;
    }
    for value in &effect.facts {
        fact(out, value);
    }
    for choice in &effect.choices {
        line(out, format!("{} option", choice.label));
        for value in &choice.facts {
            fact(out, value);
        }
    }
    for strike in &effect.strikes {
        line(
            out,
            format!(
                "{} ({})",
                strike.title,
                strike
                    .traits
                    .iter()
                    .map(|t| t.label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
        fact(out, &strike.attack);
        fact(out, &strike.damage);
    }
}
