//! Borrowed authored Item facts shared by root and immediate Actor Item scopes.
use super::{ItemSourceView, SourceFieldView};
use atlas_foundry_model::{SourcePresence, generated::*};
use serde_json::Number;

macro_rules! family_field {
    ($name:ident, $ty:ty, [$($family:ident),+], [$($excluded:ident),+], $source:ident, $body:expr) => {
        pub fn $name(self) -> SourceFieldView<'a, $ty> {
            match self { $(Self::$family($source) => $body,)+ $(Self::$excluded(_) => SourceFieldView::NotApplicable,)+ }
        }
    };
    ($name:ident, $ty:ty, [$($family:ident),+], $source:ident, $body:expr) => {
        pub fn $name(self) -> SourceFieldView<'a, $ty> {
            match self { $(Self::$family($source) => $body,)+ _ => SourceFieldView::NotApplicable }
        }
    };
}
impl<'a> ItemSourceView<'a> {
    family_field!(
        item_level,
        &'a Number,
        [
            AfflictionSource,
            ArmorSource,
            ContainerSource,
            BookSource,
            ConsumableSource,
            EquipmentSource,
            ShieldSource,
            TreasureSource,
            WeaponSource,
            CampaignFeatureSource,
            EffectSource,
            FeatSource
        ],
        [
            AbilitySource,
            AncestrySource,
            BackgroundSource,
            ClassSource,
            ConditionSource,
            DeitySource,
            HeritageSource,
            KitSource,
            LoreSource,
            MeleeSource,
            SpellSource,
            SpellcastingEntrySource
        ],
        s,
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.level).into())
            .and_then(|s| (&s.value).into())
    );
    family_field!(
        physical_price,
        &'a PartialPrice,
        [
            ArmorSource,
            ContainerSource,
            BookSource,
            ConsumableSource,
            EquipmentSource,
            ShieldSource,
            TreasureSource,
            WeaponSource
        ],
        [
            AbilitySource,
            AfflictionSource,
            AncestrySource,
            BackgroundSource,
            CampaignFeatureSource,
            ClassSource,
            ConditionSource,
            DeitySource,
            EffectSource,
            FeatSource,
            HeritageSource,
            KitSource,
            LoreSource,
            MeleeSource,
            SpellSource,
            SpellcastingEntrySource
        ],
        s,
        SourceFieldView::from(&s.system).and_then(|s| (&s.price).into())
    );
    family_field!(
        physical_bulk,
        &'a Number,
        [
            ArmorSource,
            ContainerSource,
            BookSource,
            ConsumableSource,
            EquipmentSource,
            ShieldSource,
            TreasureSource,
            WeaponSource
        ],
        [
            AbilitySource,
            AfflictionSource,
            AncestrySource,
            BackgroundSource,
            CampaignFeatureSource,
            ClassSource,
            ConditionSource,
            DeitySource,
            EffectSource,
            FeatSource,
            HeritageSource,
            KitSource,
            LoreSource,
            MeleeSource,
            SpellSource,
            SpellcastingEntrySource
        ],
        s,
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.bulk).into())
            .and_then(|s| (&s.value).into())
    );
    family_field!(
        physical_size,
        &'a PhysicalSystemSourceSize,
        [
            ArmorSource,
            ContainerSource,
            BookSource,
            ConsumableSource,
            EquipmentSource,
            ShieldSource,
            TreasureSource,
            WeaponSource
        ],
        [
            AbilitySource,
            AfflictionSource,
            AncestrySource,
            BackgroundSource,
            CampaignFeatureSource,
            ClassSource,
            ConditionSource,
            DeitySource,
            EffectSource,
            FeatSource,
            HeritageSource,
            KitSource,
            LoreSource,
            MeleeSource,
            SpellSource,
            SpellcastingEntrySource
        ],
        s,
        SourceFieldView::from(&s.system).and_then(|s| (&s.size).into())
    );
    family_field!(
        ancestry_size,
        &'a PhysicalSystemSourceSize,
        [AncestrySource],
        s,
        SourceFieldView::from(&s.system).and_then(|s| (&s.size).into())
    );
    family_field!(
        weapon_category,
        &'a WeaponSystemSourceCategory,
        [WeaponSource],
        s,
        SourceFieldView::from(&s.system).and_then(|s| (&s.category).into())
    );
    family_field!(
        weapon_group,
        &'a WeaponSystemSourceGroup,
        [WeaponSource],
        s,
        SourceFieldView::from(&s.system).and_then(|s| (&s.group).into())
    );
    family_field!(
        weapon_damage_type,
        &'a ConsumableDamageHealingType,
        [WeaponSource],
        s,
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.damage).into())
            .and_then(|s| (&s.damage_type).into())
    );
    family_field!(
        weapon_range_increment,
        &'a Number,
        [WeaponSource],
        s,
        SourceFieldView::from(&s.system).and_then(|s| (&s.range).into())
    );
    family_field!(
        weapon_reload,
        &'a WeaponSystemSourceReloadValue,
        [WeaponSource],
        s,
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.reload).into())
            .and_then(|s| (&s.value).into())
    );
    family_field!(
        armor_category,
        &'a ArmorSystemSourceCategory,
        [ArmorSource],
        s,
        SourceFieldView::from(&s.system).and_then(|s| (&s.category).into())
    );
    family_field!(
        armor_ac_bonus,
        &'a Number,
        [ArmorSource],
        s,
        SourceFieldView::from(&s.system).and_then(|s| (&s.ac_bonus).into())
    );
    family_field!(
        armor_dex_cap,
        &'a Number,
        [ArmorSource],
        s,
        SourceFieldView::from(&s.system).and_then(|s| (&s.dex_cap).into())
    );
    family_field!(
        shield_hardness,
        &'a Number,
        [ShieldSource],
        s,
        SourceFieldView::from(&s.system).and_then(|s| (&s.hardness).into())
    );
    family_field!(
        shield_hp_maximum,
        &'a Number,
        [ShieldSource],
        s,
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.hp).into())
            .and_then(|s| (&s.max).into())
    );
    family_field!(
        consumable_category,
        &'a ConsumableSystemSourceCategory,
        [ConsumableSource],
        s,
        SourceFieldView::from(&s.system).and_then(|s| (&s.category).into())
    );
    family_field!(
        action_type,
        &'a AbilitySystemSourceActionTypeSourceFromSchemaValue,
        [AbilitySource, FeatSource, CampaignFeatureSource],
        [
            AfflictionSource,
            AncestrySource,
            ArmorSource,
            BackgroundSource,
            BookSource,
            ClassSource,
            ConditionSource,
            ConsumableSource,
            ContainerSource,
            DeitySource,
            EffectSource,
            EquipmentSource,
            HeritageSource,
            KitSource,
            LoreSource,
            MeleeSource,
            ShieldSource,
            SpellSource,
            SpellcastingEntrySource,
            TreasureSource,
            WeaponSource
        ],
        s,
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.action_type).into())
            .and_then(|s| (&s.value).into())
    );
    family_field!(
        action_count,
        &'a Number,
        [AbilitySource, FeatSource, CampaignFeatureSource],
        [
            AfflictionSource,
            AncestrySource,
            ArmorSource,
            BackgroundSource,
            BookSource,
            ClassSource,
            ConditionSource,
            ConsumableSource,
            ContainerSource,
            DeitySource,
            EffectSource,
            EquipmentSource,
            HeritageSource,
            KitSource,
            LoreSource,
            MeleeSource,
            ShieldSource,
            SpellSource,
            SpellcastingEntrySource,
            TreasureSource,
            WeaponSource
        ],
        s,
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.actions).into())
            .and_then(|s| (&s.value).into())
    );
    family_field!(
        action_category,
        &'a AbilitySystemSourceCategory,
        [AbilitySource],
        s,
        SourceFieldView::from(&s.system).and_then(|s| (&s.category).into())
    );
    family_field!(
        feat_category,
        &'a FeatSystemSourceCategory,
        [FeatSource],
        s,
        SourceFieldView::from(&s.system).and_then(|s| (&s.category).into())
    );
    family_field!(
        campaign_feature_category,
        &'a CampaignFeatureSystemSourceCategory,
        [CampaignFeatureSource],
        s,
        SourceFieldView::from(&s.system).and_then(|s| (&s.category).into())
    );
    family_field!(
        heritage_ancestry,
        &'a SourceFromSchemaHeritageAncestrySchema,
        [HeritageSource],
        s,
        SourceFieldView::from(&s.system).and_then(|s| (&s.ancestry).into())
    );
    family_field!(
        heritage_ancestry_slug,
        &'a str,
        [HeritageSource],
        s,
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.ancestry).into())
            .and_then(|s| (&s.slug).into())
            .map(String::as_str)
    );
    family_field!(
        heritage_ancestry_uuid,
        &'a str,
        [HeritageSource],
        s,
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.ancestry).into())
            .and_then(|s| (&s.uuid).into())
            .map(String::as_str)
    );
    family_field!(
        effect_duration_unit,
        &'a DurationDataUnit,
        [EffectSource],
        s,
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.duration).into())
            .and_then(|s| (&s.unit).into())
    );
    family_field!(
        spell_casting_time,
        &'a str,
        [SpellSource],
        s,
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.time).into())
            .and_then(|s| (&s.value).into())
            .map(String::as_str)
    );
    family_field!(
        spell_defense_save,
        &'a SpellDefenseSourceSaveStatistic,
        [SpellSource],
        s,
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.defense).into())
            .and_then(|s| (&s.save).into())
            .and_then(|s| (&s.statistic).into())
    );
    family_field!(
        spell_defense_basic,
        bool,
        [SpellSource],
        s,
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.defense).into())
            .and_then(|s| (&s.save).into())
            .and_then(|s| (&s.basic).into())
            .map(|v| *v)
    );
    family_field!(
        spell_defense_passive,
        &'a SpellDefenseSourcePassiveStatistic,
        [SpellSource],
        s,
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.defense).into())
            .and_then(|s| (&s.passive).into())
            .and_then(|s| (&s.statistic).into())
    );
    family_field!(
        spell_area_type,
        &'a SpellAreaType,
        [SpellSource],
        s,
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.area).into())
            .and_then(|s| (&s.r#type).into())
    );
    family_field!(
        spell_sustained,
        bool,
        [SpellSource],
        s,
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.duration).into())
            .and_then(|s| (&s.sustained).into())
            .map(|v| *v)
    );
    family_field!(
        spell_duration_text,
        &'a str,
        [SpellSource],
        s,
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.duration).into())
            .and_then(|s| (&s.value).into())
            .map(String::as_str)
    );
    family_field!(
        spell_damage,
        &'a SpellSystemSourceDamage,
        [SpellSource],
        s,
        SourceFieldView::from(&s.system).and_then(|s| (&s.damage).into())
    );
    family_field!(
        deity_domains_primary,
        &'a [DeitySystemSourceDomainsAlternateEntry],
        [DeitySource],
        s,
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.domains).into())
            .and_then(|s| (&s.primary).into())
            .map(Vec::as_slice)
    );
    family_field!(
        deity_domains_alternate,
        &'a [DeitySystemSourceDomainsAlternateEntry],
        [DeitySource],
        s,
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.domains).into())
            .and_then(|s| (&s.alternate).into())
            .map(Vec::as_slice)
    );

    /// Physical usage is authored only; Armor, Shield, Treasure and Kit lack it.
    pub fn physical_usage(self) -> SourceFieldView<'a, &'a str> {
        match self {
            Self::WeaponSource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.usage).into())
                .and_then(|s| (&s.value).into())
                .map(|v| match v {
                    WeaponSystemSourceUsageValue::HeldInOneHand => "held-in-one-hand",
                    WeaponSystemSourceUsageValue::HeldInOnePlusHands => "held-in-one-plus-hands",
                    WeaponSystemSourceUsageValue::HeldInTwoHands => "held-in-two-hands",
                    WeaponSystemSourceUsageValue::Worngloves => "worngloves",
                }),
            Self::ContainerSource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.usage).into())
                .and_then(|s| (&s.value).into())
                .map(String::as_str),
            Self::BookSource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.usage).into())
                .and_then(|s| (&s.value).into())
                .map(String::as_str),
            Self::ConsumableSource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.usage).into())
                .and_then(|s| (&s.value).into())
                .map(String::as_str),
            Self::EquipmentSource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.usage).into())
                .and_then(|s| (&s.value).into())
                .map(String::as_str),
            _ => SourceFieldView::NotApplicable,
        }
    }

    pub fn spell_cantrip(self) -> SourceFieldView<'a, bool> {
        if !matches!(self, Self::SpellSource(_)) {
            return SourceFieldView::NotApplicable;
        }
        self.traits().map(|v| v.iter().any(|s| s == "cantrip"))
    }
    /// Pinned Foundry focus semantics, preserving unavailable operands.
    pub fn spell_focus(self) -> SourceFieldView<'a, bool> {
        if !matches!(self, Self::SpellSource(_)) {
            return SourceFieldView::NotApplicable;
        }
        self.traits().and_then(|traits| {
            if traits.iter().any(|s| s == "focus") {
                return SourceFieldView::Value(true);
            }
            if !traits.iter().any(|s| s == "cantrip") {
                return SourceFieldView::Value(false);
            }
            self.spell_traditions().map(|v| v.is_empty())
        })
    }
    pub fn spell_ritual(self) -> SourceFieldView<'a, bool> {
        match self {
            Self::SpellSource(s) => {
                SourceFieldView::from(&s.system).and_then(|s| match &s.ritual {
                    SourcePresence::Null => SourceFieldView::Value(false),
                    p => SourceFieldView::from(p).map(|_| true),
                })
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    /// Exact glyph-map classes; timed/free text is retained separately.
    pub fn spell_casting_form(self) -> SourceFieldView<'a, &'static str> {
        self.spell_casting_time()
            .map(|s| match s.trim().to_lowercase().as_str() {
                "0" | "free" => "free",
                "reaction" => "reaction",
                "1" => "one_action",
                "2" => "two_actions",
                "3" => "three_actions",
                "1 or 2" => "one_or_two",
                "1 to 3" => "one_to_three",
                "2 or 3" => "two_or_three",
                "2 rounds" => "two_rounds",
                _ => "other",
            })
    }
    pub fn spell_area_size(self) -> SourceFieldView<'a, &'a Number> {
        match self {
            Self::SpellSource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.area).into())
                .and_then(|s| (&s.value).into())
                .and_then(|v| match v {
                    StringOrNumber::Number(n) => SourceFieldView::Value(n),
                    StringOrNumber::String(_) => SourceFieldView::ProjectionInvalid {
                        source_path: "/system/area/value",
                        reason: "authored String is unavailable to a numeric feet projection",
                    },
                }),
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn heritage_versatile(self) -> SourceFieldView<'a, bool> {
        match self {
            Self::HeritageSource(s) => {
                SourceFieldView::from(&s.system).and_then(|s| match &s.ancestry {
                    SourcePresence::Null => SourceFieldView::Value(true),
                    p => SourceFieldView::from(p).map(|_| false),
                })
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    /// A unit-gated authored value; no conversion and no unlimited sentinel.
    pub fn effect_duration(self, unit: DurationDataUnit) -> SourceFieldView<'a, &'a Number> {
        match self {
            Self::EffectSource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.duration).into())
                .and_then(|d| {
                    SourceFieldView::from(&d.unit).and_then(|authored| {
                        if matches!(
                            unit,
                            DurationDataUnit::Unlimited | DurationDataUnit::Encounter
                        ) || *authored != unit
                        {
                            SourceFieldView::NotApplicable
                        } else {
                            (&d.value).into()
                        }
                    })
                }),
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn condition_is_valued(self) -> SourceFieldView<'a, bool> {
        match self {
            Self::ConditionSource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.value).into())
                .and_then(|v| match v {
                    ConditionValueData::Alternative1(v) => (&v.is_valued).into(),
                    ConditionValueData::Alternative2(v) => (&v.is_valued).into(),
                })
                .map(|v| *v),
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn deity_fonts(self) -> SourceFieldView<'a, &'static [&'static str]> {
        match self {
            Self::DeitySource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.font).into())
                .map(|v| match v {
                    DivineFonts::Array(_) => &[][..],
                    DivineFonts::Alternative2(_) => &["harm", "heal"][..],
                    DivineFonts::Alternative3(_) => &["harm"][..],
                    DivineFonts::Alternative4(_) => &["heal"][..],
                }),
            _ => SourceFieldView::NotApplicable,
        }
    }

    /// Authored bundle copper / positive declared per, rounded once to binary64.
    /// Only optional denomination/per omission defaults; null never becomes zero.
    pub fn physical_price_per_item_cp(self) -> SourceFieldView<'a, f64> {
        self.physical_price().and_then(|price| {
            let coins = SourceFieldView::from(&price.value);
            coins.and_then(|coins| {
                let mut total = SourceFieldView::Value(0u64);
                for (amount, weight) in [
                    (&coins.cp, 1u64),
                    (&coins.sp, 10),
                    (&coins.gp, 100),
                    (&coins.pp, 1000),
                ] {
                    total = total.and_then(|total| {
                        optional_integer(amount, 0, "/system/price/value").and_then(|n| {
                            match n.checked_mul(weight).and_then(|n| total.checked_add(n)) {
                                Some(sum) if sum <= MAX_EXACT_INTEGER => {
                                    SourceFieldView::Value(sum)
                                }
                                _ => projection_invalid(
                                    "/system/price/value",
                                    "bundle copper exceeds the exact binary64 integer domain",
                                ),
                            }
                        })
                    });
                }
                total.and_then(|total| {
                    optional_integer(&price.per, 1, "/system/price/per").and_then(|per| {
                        if per == 0 {
                            projection_invalid(
                                "/system/price/per",
                                "per must be a positive integer",
                            )
                        } else {
                            SourceFieldView::Value(total as f64 / per as f64)
                        }
                    })
                })
            })
        })
    }
}
const MAX_EXACT_INTEGER: u64 = 1 << 53;
fn projection_invalid<'a, T>(
    source_path: &'static str,
    reason: &'static str,
) -> SourceFieldView<'a, T> {
    SourceFieldView::ProjectionInvalid {
        source_path,
        reason,
    }
}
fn optional_integer<'a>(
    p: &'a SourcePresence<Number>,
    initial: u64,
    path: &'static str,
) -> SourceFieldView<'a, u64> {
    match p {
        SourcePresence::Missing => SourceFieldView::Value(initial),
        _ => SourceFieldView::from(p).and_then(|n| {
            let integer = n.as_u64().or_else(|| {
                n.as_f64()
                    .filter(|n| {
                        n.is_finite()
                            && *n >= 0.0
                            && *n <= MAX_EXACT_INTEGER as f64
                            && n.fract() == 0.0
                    })
                    .map(|n| n as u64)
            });
            match integer {
                Some(n) if n <= MAX_EXACT_INTEGER => SourceFieldView::Value(n),
                _ => projection_invalid(path, "requires a nonnegative integer at most 2^53"),
            }
        }),
    }
}

#[cfg(test)]
#[path = "item_query_tests.rs"]
mod tests;
