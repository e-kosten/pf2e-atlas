use super::*;
use crate::pathbuilder_sheet::ImportedEquipment;
use atlas_domain::RecordKind;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum ItemFamily {
    Any,
    Weapon,
    Armor,
}

struct ItemLinks<'a> {
    retrieval: Option<&'a AtlasRetrievalService>,
    matches: HashMap<(ItemFamily, String), Option<RecordKey>>,
}

impl ItemLinks<'_> {
    fn link(
        &mut self,
        row: &mut SheetRowView,
        family: ItemFamily,
        base: &str,
    ) -> AppServiceResult<()> {
        let Some(retrieval) = self.retrieval else {
            return Ok(());
        };
        if base.trim().is_empty() {
            return Ok(());
        }
        let key = (family, normalize_record_name(base));
        if !self.matches.contains_key(&key) {
            let target = unique_pf2e_record(retrieval, base, |record| {
                record.record_kind.as_value() == Some(&RecordKind::Equipment)
                    && match family {
                        ItemFamily::Any => true,
                        ItemFamily::Weapon => record
                            .source_type
                            .as_value()
                            .is_some_and(|kind| kind == "weapon"),
                        ItemFamily::Armor => record
                            .source_type
                            .as_value()
                            .is_some_and(|kind| matches!(kind.as_str(), "armor" | "shield")),
                    }
            })?;
            self.matches.insert(key.clone(), target);
        }
        row.record_key.clone_from(&self.matches[&key]);
        Ok(())
    }
}

pub(super) fn groups(
    sheet: &PathbuilderSheet,
    retrieval: Option<&AtlasRetrievalService>,
) -> AppServiceResult<Vec<SheetGroupView>> {
    Ok(categorized(sheet, retrieval)?
        .into_iter()
        .map(|(_, group)| group)
        .collect())
}

pub(super) fn categorized(
    sheet: &PathbuilderSheet,
    retrieval: Option<&AtlasRetrievalService>,
) -> AppServiceResult<Vec<(atlas_app_model::CharacterSection, SheetGroupView)>> {
    use atlas_app_model::CharacterSection;
    let mut links = ItemLinks {
        retrieval,
        matches: HashMap::new(),
    };
    let mut weapons = Vec::new();
    for weapon in values(&sheet.weapons) {
        let mut facts = Vec::new();
        add_role_facts(
            &mut facts,
            [
                (SheetFactRole::Quantity, "Quantity", shown(&weapon.quantity)),
                (SheetFactRole::Primary, "Attack", signed(&weapon.attack)),
                (SheetFactRole::Primary, "Damage die", shown(&weapon.die)),
                (
                    SheetFactRole::Primary,
                    "Damage type",
                    source_label(&weapon.damage_type, damage_type),
                ),
                (
                    SheetFactRole::Primary,
                    "Damage bonus",
                    signed(&weapon.damage_bonus),
                ),
            ],
        );
        add_facts(
            &mut facts,
            [
                ("Base weapon", shown(&weapon.name)),
                ("Proficiency", title_case(&weapon.proficiency)),
                ("Potency", signed(&weapon.potency)),
                ("Striking", title_case(&weapon.striking)),
                ("Material", title_case(&weapon.material)),
                ("Grade", title_case(&weapon.grade)),
                ("Runes", text_list(&weapon.runes)),
                ("Increased dice", yes_no(&weapon.increased_dice)),
                ("Inventor weapon", yes_no(&weapon.is_inventor)),
            ],
        );
        let base = name(&weapon.name, "");
        let mut item = row(
            name(
                &weapon.display,
                if base.is_empty() {
                    "Unnamed weapon"
                } else {
                    base
                },
            ),
            facts,
        );
        links.link(&mut item, ItemFamily::Weapon, base)?;
        weapons.push(item);
    }
    let mut armor = Vec::new();
    for worn in values(&sheet.armor) {
        let mut facts = Vec::new();
        add_role_facts(
            &mut facts,
            [
                (SheetFactRole::Quantity, "Quantity", shown(&worn.quantity)),
                (SheetFactRole::Primary, "Worn", yes_no(&worn.worn)),
                (
                    SheetFactRole::Primary,
                    "Proficiency",
                    title_case(&worn.proficiency),
                ),
            ],
        );
        add_facts(
            &mut facts,
            [
                ("Base armor", shown(&worn.name)),
                ("Potency", signed(&worn.potency)),
                ("Resilient", title_case(&worn.resilient)),
                ("Material", title_case(&worn.material)),
                ("Grade", title_case(&worn.grade)),
                ("Runes", text_list(&worn.runes)),
            ],
        );
        let base = name(&worn.name, "");
        let mut item = row(
            name(
                &worn.display,
                if base.is_empty() {
                    "Unnamed armor"
                } else {
                    base
                },
            ),
            facts,
        );
        links.link(&mut item, ItemFamily::Armor, base)?;
        armor.push(item);
    }

    let mut groups = vec![
        typed_group(SheetGroupKind::Weapons, "Weapons", weapons),
        typed_group(SheetGroupKind::Armor, "Armor & shields", armor),
        typed_group(SheetGroupKind::Gear, "Loose gear", Vec::new()),
    ];
    let loose = groups.len() - 1;
    let mut container_indices = HashMap::new();
    let mut counts = HashMap::new();
    let mut ordinals = HashMap::new();
    let containers = value(&sheet.equipment_containers).into_iter().flatten();
    for (_, container) in containers.clone() {
        if let Some(container) = value(container) {
            *counts
                .entry(name(&container.container_name, "Unnamed container"))
                .or_insert(0) += 1;
        }
    }
    for (id, container) in containers {
        if let Some(container) = value(container) {
            let label = name(&container.container_name, "Unnamed container");
            let mut facts = Vec::new();
            add_facts(
                &mut facts,
                [
                    ("Bag of holding", yes_no(&container.bag_of_holding)),
                    ("Backpack", yes_no(&container.backpack)),
                    ("Augmentations", yes_no(&container.augmentations)),
                ],
            );
            container_indices.insert(id.as_str(), groups.len());
            let display = if counts[label] > 1 {
                let ordinal = ordinals.entry(label).or_insert(0);
                *ordinal += 1;
                format!("{label} ({ordinal})")
            } else {
                label.to_owned()
            };
            let mut metadata = row("Container", facts);
            metadata.kind = Some(SheetRowKind::ContainerMetadata);
            let mut container_group =
                typed_group(SheetGroupKind::Container, display, vec![metadata]);
            container_group.container_id = Some(id.clone());
            groups.push(container_group);
        }
    }
    for entry in values(&sheet.equipment) {
        let (label, quantity, container, state) = match entry {
            ImportedEquipment::InContainer(label, quantity, id, state) => {
                (label, quantity, Some(id.as_str()), state)
            }
            ImportedEquipment::Uncontained(label, quantity, state) => {
                (label, quantity, None, state)
            }
        };
        let mut facts = vec![role_fact(SheetFactRole::Quantity, "Quantity", quantity)];
        if !state.is_empty() {
            facts.push(role_fact(SheetFactRole::State, "State", state));
        }
        let index = if let Some(id) = container {
            *container_indices.entry(id).or_insert_with(|| {
                let mut unresolved = typed_group(
                    SheetGroupKind::Container,
                    format!("Unresolved container ({id})"),
                    Vec::new(),
                );
                unresolved.container_id = Some(id.to_owned());
                groups.push(unresolved);
                groups.len() - 1
            })
        } else {
            loose
        };
        let mut item = row(label, facts);
        links.link(&mut item, ItemFamily::Any, label)?;
        groups[index].rows.push(item);
    }

    let mut categorized: Vec<_> = groups
        .into_iter()
        .map(|group| (CharacterSection::Equipment, group))
        .collect();
    if let Some(money) = value(&sheet.money) {
        let mut facts = Vec::new();
        add_facts(
            &mut facts,
            [
                ("pp", shown(&money.pp)),
                ("gp", shown(&money.gp)),
                ("sp", shown(&money.sp)),
                ("cp", shown(&money.cp)),
            ],
        );
        let mut coins = summary("Coins", facts);
        coins.kind = Some(SheetGroupKind::Coins);
        categorized.push((CharacterSection::Money, coins));
    }
    for formulas in values(&sheet.formula) {
        let mut known = Vec::new();
        for label in values(&formulas.known) {
            let mut formula = row(label, Vec::new());
            links.link(&mut formula, ItemFamily::Any, label)?;
            known.push(formula);
        }
        let label = value(&formulas.kind)
            .filter(|kind| !kind.is_empty())
            .map_or_else(
                || "Known formulas".into(),
                |kind| format!("Known formulas · {}", sentence_case(kind)),
            );
        categorized.push((
            CharacterSection::Crafting,
            typed_group(SheetGroupKind::Formulas, label, known),
        ));
    }
    Ok(categorized)
}

fn damage_type(value: &str) -> &str {
    match value {
        "B" => "Bludgeoning",
        "P" => "Piercing",
        "S" => "Slashing",
        "B, P or S" => "Bludgeoning, piercing, or slashing",
        label => label,
    }
}
