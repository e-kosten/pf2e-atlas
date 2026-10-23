//! Typed authored equipment facts; no wearer, rune or weapon-rule execution.
use super::*;
use atlas_foundry_model::generated::*;

pub(super) fn details(item: ItemSourceView<'_>) -> Option<PhysicalDetailsView> {
    Some(match item {
        ItemSourceView::WeaponSource(source) => {
            let s = SourceFieldView::from(&source.system);
            let damage = s.and_then(|s| (&s.damage).into());
            let runes = s.and_then(|s| (&s.runes).into());
            PhysicalDetailsView::Weapon(Box::new(WeaponReferenceView {
                category: identifier(item.weapon_category()),
                group: identifier(item.weapon_group()),
                damage: fact(damage.map(|d| WeaponDamageReferenceView {
                    dice: number((&d.dice).into()),
                    die: identifier((&d.die).into()),
                    damage_type: identifier((&d.damage_type).into()),
                    modifier: number((&d.modifier).into()),
                    persistent: fact(SourceFieldView::from(&d.persistent).map(|p| {
                        WeaponPersistentReferenceView {
                            number: number((&p.number).into()),
                            faces: number((&p.faces).into()),
                            damage_type: identifier((&p.r#type).into()),
                        }
                    })),
                })),
                range_increment: number(item.weapon_range_increment()),
                maximum_range: number(s.and_then(|s| (&s.max_range).into())),
                reload: identifier(item.weapon_reload()),
                splash_damage: fact(
                    s.and_then(|s| (&s.splash_damage).into())
                        .and_then(|v| (&v.value).into())
                        .map(|v| match v {
                            WeaponSystemSourceSplashDamageValue::Number(v) => v.to_string(),
                            WeaponSystemSourceSplashDamageValue::Alternative2(_) => String::new(),
                        }),
                ),
                ammunition: fact(s.and_then(|s| (&s.ammo).into()).map(|a| {
                    WeaponAmmunitionReferenceView {
                        base_type: identifier((&a.base_type).into()),
                        capacity: number((&a.capacity).into()),
                        built_in: fact(SourceFieldView::from(&a.built_in).map(|v| *v)),
                    }
                })),
                runes: fact(runes.map(|r| WeaponRuneReferenceView {
                    potency: number((&r.potency).into()),
                    striking: rune_name(
                        (&r.striking).into(),
                        &[
                            "",
                            "Striking",
                            "Greater striking",
                            "Major striking",
                            "Mythic striking",
                        ],
                    ),
                    property: identifiers((&r.property).into()),
                })),
                melee_usage: fact(s.and_then(|s| (&s.melee_usage).into()).map(|m| {
                    let d = SourceFieldView::from(&m.damage);
                    WeaponMeleeReferenceView {
                        die: identifier(d.and_then(|d| (&d.die).into())),
                        damage_type: identifier(d.and_then(|d| (&d.r#type).into())),
                        group: identifier((&m.group).into()),
                        traits: identifiers((&m.traits).into()),
                    }
                })),
            }))
        }
        ItemSourceView::ArmorSource(source) => {
            let s = SourceFieldView::from(&source.system);
            let runes = s.and_then(|s| (&s.runes).into());
            PhysicalDetailsView::Armor(Box::new(ArmorReferenceView {
                category: identifier(item.armor_category()),
                group: identifier(s.and_then(|s| (&s.group).into())),
                ac_bonus: number(item.armor_ac_bonus()),
                dex_cap: number(item.armor_dex_cap()),
                check_penalty: number(s.and_then(|s| (&s.check_penalty).into())),
                speed_penalty: number(s.and_then(|s| (&s.speed_penalty).into())),
                strength: number(s.and_then(|s| (&s.strength).into())),
                potency: number(runes.and_then(|r| (&r.potency).into())),
                resilient: rune_name(
                    runes.and_then(|r| (&r.resilient).into()),
                    &[
                        "",
                        "Resilient",
                        "Greater resilient",
                        "Major resilient",
                        "Mythic resilient",
                    ],
                ),
                property_runes: identifiers(runes.and_then(|r| (&r.property).into())),
            }))
        }
        ItemSourceView::ShieldSource(source) => {
            let s = SourceFieldView::from(&source.system);
            PhysicalDetailsView::Shield(Box::new(ShieldReferenceView {
                ac_bonus: number(s.and_then(|s| (&s.ac_bonus).into())),
                speed_penalty: number(s.and_then(|s| (&s.speed_penalty).into())),
                hardness: number(item.shield_hardness()),
                hit_points: number(item.shield_hp_maximum()),
                durability_in_table: item.shield_durability_in_table(),
                reinforcing: number(
                    s.and_then(|s| (&s.runes).into())
                        .and_then(|r| (&r.reinforcing).into()),
                ),
            }))
        }
        ItemSourceView::ContainerSource(source) => {
            let s = SourceFieldView::from(&source.system);
            let bulk = s.and_then(|s| (&s.bulk).into());
            PhysicalDetailsView::Container(Box::new(ContainerReferenceView {
                capacity: number(bulk.and_then(|b| (&b.capacity).into())),
                ignored_bulk: number(bulk.and_then(|b| (&b.ignored).into())),
                held_or_stowed_bulk: number(bulk.and_then(|b| (&b.held_or_stowed).into())),
                stowing: fact(s.and_then(|s| (&s.stowing).into()).map(|v| *v)),
            }))
        }
        ItemSourceView::AmmoSource(source) => {
            let s = SourceFieldView::from(&source.system);
            let uses = s.and_then(|s| (&s.uses).into());
            PhysicalDetailsView::Ammunition(Box::new(AmmunitionReferenceView {
                maximum_uses: number(uses.and_then(|u| (&u.max).into())),
                source_uses: number(uses.and_then(|u| (&u.value).into())),
            }))
        }
        ItemSourceView::KitSource(source) => PhysicalDetailsView::Kit(Box::new(KitReferenceView {
            items: kit_items(SourceFieldView::from(&source.system).and_then(|s| (&s.items).into())),
        })),
        _ => return None,
    })
}

fn kit_items(
    items: SourceFieldView<'_, &KitSystemSourceItems>,
) -> FactView<Vec<KitEntryReferenceView>> {
    fact(items.map(|items| {
        items
            .entries
            .iter()
            .map(|(_, entry)| KitEntryReferenceView {
                name: text((&entry.name).into()),
                quantity: number((&entry.quantity).into()),
                items: kit_items((&entry.items).into()),
            })
            .collect()
    }))
}

pub(super) fn price(item: ItemSourceView<'_>) -> FactView<PhysicalPriceView> {
    let price = match item {
        ItemSourceView::KitSource(source) => {
            SourceFieldView::from(&source.system).and_then(|s| (&s.price).into())
        }
        _ => item.physical_price(),
    };
    fact(
        price
            .and_then(|p| (&p.value).into())
            .map(|c| PhysicalPriceView {
                pp: number((&c.pp).into()),
                gp: number((&c.gp).into()),
                sp: number((&c.sp).into()),
                cp: number((&c.cp).into()),
            }),
    )
}

// Labels follow the pinned PF2e fundamental rune catalog; no bonuses are applied.
fn rune_name(value: SourceFieldView<'_, &serde_json::Number>, labels: &[&str]) -> FactView<String> {
    fact(value.map(|v| {
        v.as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .and_then(|n| labels.get(n))
            .map_or_else(|| v.to_string(), |name| (*name).to_owned())
    }))
}

pub(super) fn material(item: ItemSourceView<'_>) -> FactView<PhysicalMaterialReferenceView> {
    macro_rules! project {
        ($source:expr) => {
            fact(
                SourceFieldView::from(&$source.system)
                    .and_then(|s| (&s.material).into())
                    .map(|m| PhysicalMaterialReferenceView {
                        kind: identifier((&m.r#type).into()),
                        grade: identifier((&m.grade).into()),
                    }),
            )
        };
    }
    match item {
        ItemSourceView::WeaponSource(s) => project!(s),
        ItemSourceView::ArmorSource(s) => project!(s),
        ItemSourceView::ShieldSource(s) => project!(s),
        ItemSourceView::ContainerSource(s) => project!(s),
        ItemSourceView::ConsumableSource(s) => project!(s),
        ItemSourceView::EquipmentSource(s) => project!(s),
        ItemSourceView::AmmoSource(s) => project!(s),
        ItemSourceView::TreasureSource(s) => project!(s),
        _ => unavailable(),
    }
}

pub(super) fn quantity(item: ItemSourceView<'_>) -> NumberFactView {
    macro_rules! project {
        ($source:expr) => {
            number(SourceFieldView::from(&$source.system).and_then(|s| (&s.quantity).into()))
        };
    }
    match item {
        ItemSourceView::WeaponSource(s) => project!(s),
        ItemSourceView::ArmorSource(s) => project!(s),
        ItemSourceView::ShieldSource(s) => project!(s),
        ItemSourceView::ContainerSource(s) => project!(s),
        ItemSourceView::ConsumableSource(s) => project!(s),
        ItemSourceView::EquipmentSource(s) => project!(s),
        ItemSourceView::AmmoSource(s) => project!(s),
        ItemSourceView::TreasureSource(s) => project!(s),
        _ => number(SourceFieldView::NotApplicable),
    }
}
