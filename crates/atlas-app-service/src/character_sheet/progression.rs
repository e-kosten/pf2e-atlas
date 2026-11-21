use super::*;
use std::collections::BTreeMap;

pub(super) fn groups(sheet: &PathbuilderSheet, keys: &AbilityKeys<'_>) -> Vec<SheetGroupView> {
    let mut levels = BTreeMap::<i64, Vec<SheetRowView>>::new();
    for feat in values(&sheet.feats) {
        let mut facts = vec![
            role_fact(SheetFactRole::Category, "Type", &feat.category),
            role_fact(
                SheetFactRole::AcquiredLevel,
                "Acquired at level",
                feat.level,
            ),
        ];
        add_role_facts(
            &mut facts,
            [(SheetFactRole::Choice, "Choice", feat.choice.clone())],
        );
        add_facts(
            &mut facts,
            [
                ("Slot", shown(&feat.slot)),
                (
                    "Selection",
                    source_label(&feat.selection_kind, feat_selection),
                ),
                ("Selection detail", shown(&feat.selection_detail)),
            ],
        );
        let kind = if feat.category == "Heritage" {
            PcAbilityKind::Heritage
        } else {
            PcAbilityKind::Feat
        };
        levels.entry(feat.level).or_default().push(ability_row(
            keys,
            kind,
            &feat.name,
            feat.choice.as_deref(),
            facts,
        ));
    }
    let specials = values(&sheet.specials)
        .map(|name| ability_row(keys, PcAbilityKind::Special, name, None, Vec::new()));
    let mut groups: Vec<_> = levels
        .into_iter()
        .map(|(level, rows)| {
            typed_group(SheetGroupKind::Advancement, format!("Level {level}"), rows)
        })
        .collect();
    groups.push(typed_group(
        SheetGroupKind::SpecialAbilities,
        "Special abilities",
        specials.collect(),
    ));
    if let Some(boosts) = value(&sheet.abilities).and_then(|abilities| value(&abilities.breakdown))
    {
        let mut facts = Vec::new();
        add_facts(
            &mut facts,
            [
                ("Free ancestry boosts", text_list(&boosts.ancestry_free)),
                ("Ancestry boosts", text_list(&boosts.ancestry_boosts)),
                ("Ancestry flaws", text_list(&boosts.ancestry_flaws)),
                ("Background boosts", text_list(&boosts.background_boosts)),
                ("Class boosts", text_list(&boosts.class_boosts)),
            ],
        );
        for (level, boosts) in value(&boosts.map_levelled_boosts).into_iter().flatten() {
            if let Some(boosts) = text_list(boosts) {
                facts.push(fact_value(format!("Level {level} boosts"), boosts));
            }
        }
        let mut boosts = summary("Ability boosts", facts);
        boosts.kind = Some(SheetGroupKind::AbilityBoosts);
        groups.push(boosts);
    }
    groups
}
