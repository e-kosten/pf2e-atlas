//! Exact usage labels from PF2e 8.6.1 config/usage and English localization.
//! Source: 28d9d156f252d084371d994dc359509b49eadf18; unknown tokens stay verbatim.
//! These labels never infer a normalized hand-count axis.
/// Human-readable identifier formatting; the exact authored token remains the key.
pub(crate) fn base_type(value: &str) -> String {
    value
        .split(['-', '_'])
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) const USAGES: &[(&str, &str)] = &[
    ("affixed-or-held-in-one-hand", "Affixed or Held in 1 Hand"),
    ("affixed-to-a-creature", "Affixed to a Creature"),
    ("affixed-to-a-magical-staff", "Affixed to a Magical Staff"),
    ("affixed-to-a-metal-weapon", "Affixed to a Metal Weapon"),
    (
        "affixed-to-a-one-handed-firearm-or-hand-crossbow",
        "Affixed to a One-Handed Firearm or Hand Crossbow",
    ),
    ("affixed-to-a-ranged-weapon", "Affixed to a Ranged Weapon"),
    ("affixed-to-a-shield", "Affixed to a Shield"),
    (
        "affixed-to-a-shield-or-weapon",
        "Affixed to a Shield or Weapon",
    ),
    (
        "affixed-to-a-slashing-weapon",
        "Affixed to a Slashing Weapon",
    ),
    ("affixed-to-a-thrown-weapon", "Affixed to a Thrown Weapon"),
    (
        "affixed-to-a-two-handed-firearm-or-crossbow",
        "Affixed to a Two-Handed Firearm or Crossbow",
    ),
    ("affixed-to-an-innovation", "Affixed to an Innovation"),
    (
        "affixed-to-an-object-or-structure",
        "Affixed to an Object or Structure",
    ),
    ("affixed-to-armor", "Affixed to Armor"),
    (
        "affixed-to-armor-or-a-weapon",
        "Affixed to Armor or a Weapon",
    ),
    (
        "affixed-to-armor-or-travelers-clothing",
        "Affixed to Armor or Travelers Clothing",
    ),
    (
        "affixed-to-armor-shield-or-weapon",
        "Affixed to Armor, Shield, or Weapon",
    ),
    (
        "affixed-to-crossbow-or-firearm",
        "Affixed to Crossbow or Firearm",
    ),
    ("affixed-to-firearm", "Affixed to Firearm"),
    (
        "affixed-to-firearm-with-a-reload-of-1",
        "Affixed to a Firearm With a Reload of 1",
    ),
    (
        "affixed-to-firearm-with-the-kickback-trait",
        "Affixed to a Firearm With the Kickback Trait",
    ),
    (
        "affixed-to-ground-in-10-foot-radius",
        "Affixed to the Ground in Four Spaces Along a Circle with a 10-foot Radius",
    ),
    (
        "affixed-to-ground-in-20-foot-radius",
        "Affixed to the Ground in Four Spaces Along a Circle with a 20-foot Radius",
    ),
    ("affixed-to-harness", "Affixed to Harness"),
    ("affixed-to-headgear", "Affixed to Headgear"),
    ("affixed-to-instrument", "Affixed to Instrument"),
    (
        "affixed-to-load-bearing-wall-or-pillar",
        "Affixed to a Load-Bearing Wall or Pillar",
    ),
    (
        "affixed-to-medium-heavy-armor",
        "Affixed to Medium or Heavy Armor",
    ),
    (
        "affixed-to-medium-heavy-metal-armor",
        "Affixed to Medium or Heavy Metal Armor",
    ),
    ("affixed-to-melee-weapon", "Affixed to a Melee Weapon"),
    (
        "affixed-to-metal-armor-or-a-weapon",
        "Affixed to Metal Armor or a Weapon",
    ),
    ("affixed-to-metal-weapon", "Affixed to a Metal Weapon"),
    (
        "affixed-to-non-metal-armor-or-a-weapon",
        "Affixed to Non-Metal Armor or a Weapon",
    ),
    (
        "affixed-to-object-structure-or-creature",
        "Affixed to Structure, Object, or Creature",
    ),
    ("affixed-to-the-ground", "Affixed to the Ground"),
    (
        "affixed-to-unarmored-defense-item",
        "Affixed to Unarmored Defense Item",
    ),
    ("affixed-to-wall", "Affixed to Wall"),
    ("affixed-to-weapon", "Affixed to a Weapon"),
    (
        "applied-to-a-basket-bag-or-other-container",
        "Applied to a Basket, Bag, or Other Container",
    ),
    (
        "applied-to-a-non-injection-melee-weapon-piercing-damage",
        "Applied to a non-injection melee weapon that deals piercing damage",
    ),
    (
        "applied-to-a-tech-item-of5-bulk-or-less",
        "Applied to a tech item of Bulk 5 or less",
    ),
    ("applied-to-a-weapon", "Applied to a weapon"),
    (
        "applied-to-a-wind-powered-vehicle",
        "Applied to a wind-powered vehicle",
    ),
    (
        "applied-to-any-item-of-light-or-negligible-bulk",
        "Applied to any item of light or negligible Bulk",
    ),
    (
        "applied-to-any-visible-article-of-clothing",
        "Applied to any visible article of clothing",
    ),
    ("applied-to-armor", "Applied to armor"),
    (
        "applied-to-armor-or-unarmored-defense-clothing",
        "Applied to armor or unarmored defense clothing",
    ),
    (
        "applied-to-belt-cape-cloak-or-scarf",
        "Applied to belt, cape, cloak, or scarf",
    ),
    (
        "applied-to-boots-cape-cloak-or-umbrella",
        "Applied to boots, cape, cloak, or umbrella",
    ),
    (
        "applied-to-buckler-shield",
        "Applied to a buckler or a shield",
    ),
    (
        "applied-to-dueling-cape-or-shield",
        "Applied to dueling cape or shield",
    ),
    ("applied-to-footwear", "Applied to Footwear"),
    (
        "applied-to-medium-heavy-armor",
        "Applied to Medium or Heavy Armor",
    ),
    ("applied-to-shield", "Applied to shield"),
    ("attached-to-a-thrown-weapon", "Attached to a Thrown Weapon"),
    (
        "attached-to-crossbow-or-firearm",
        "Attached to Crossbow or Firearm",
    ),
    (
        "attached-to-crossbow-or-firearm-firing-mechanism",
        "Attached to Crossbow or Firearm (Firing Mechanism)",
    ),
    (
        "attached-to-crossbow-or-firearm-scope",
        "Attached to Crossbow or Firearm (Scope)",
    ),
    ("attached-to-firearm", "Attached To Firearm"),
    ("attached-to-firearm-scope", "Attached To Firearm (Scope)"),
    ("attached-to-melee-weapon", "Attached to a Melee Weapon"),
    ("attached-to-ships-bow", "Attached to a Ship's Bow"),
    ("bonded", "Bonded"),
    ("carried", "Carried"),
    (
        "each-rune-applied-to-a-separate-item-that-has-pockets",
        "Each rune applied to a separate item that has pockets",
    ),
    ("etched-onto-a-shield", "Etched onto a Shield"),
    ("etched-onto-a-weapon", "Etched onto a Weapon"),
    ("etched-onto-armor", "Etched onto Armor"),
    (
        "etched-onto-bludgeoning-weapon",
        "Etched on Bludgeoning Weapon",
    ),
    ("etched-onto-clan-dagger", "Etched on Clan Dagger"),
    ("etched-onto-heavy-armor", "Etched onto Heavy Armor"),
    ("etched-onto-light-armor", "Etched onto Light Armor"),
    (
        "etched-onto-lm-nonmetal-armor",
        "Etched on L/M Nonmetal Armor",
    ),
    (
        "etched-onto-med-heavy-armor",
        "Etched onto Medium or Heavy Armor",
    ),
    (
        "etched-onto-medium-heavy-metal-armor",
        "Etched onto Medium or Heavy Metal Armor",
    ),
    ("etched-onto-melee-weapon", "Etched on Melee Weapon"),
    (
        "etched-onto-melee-weapon-monk",
        "Etched onto a melee weapon with the monk trait",
    ),
    ("etched-onto-metal-armor", "Etched onto Metal Armor"),
    (
        "etched-onto-piercing-or-slashing-melee-weapon",
        "Etched on Piercing/Slashing Melee Weapon",
    ),
    (
        "etched-onto-piercing-or-slashing-weapon",
        "Etched on Piercing/Slashing Weapon",
    ),
    (
        "etched-onto-slashing-melee-weapon",
        "Etched on Slashing Melee Weapon",
    ),
    ("etched-onto-thrown-weapon", "Etched on Thrown Weapon"),
    (
        "etched-onto-weapon-wo-anarchic-rune",
        "Etched on Weapon w/o Axiomatic Rune",
    ),
    (
        "etched-onto-weapon-wo-axiomatic-rune",
        "Etched on Weapon w/o Anarchic Rune",
    ),
    (
        "etched-onto-weapon-wo-holy-rune",
        "Etched on Weapon that isn't Unholy",
    ),
    (
        "etched-onto-weapon-wo-unholy-rune",
        "Etched on Weapon that isn't Holy",
    ),
    (
        "held-in-1-hand-hung-on-a-cord-or-attached-to-clothing",
        "Held in 1 hand, hung on a cord, or attached to clothing",
    ),
    ("held-in-one-hand", "Held in 1 hand"),
    // Legacy authored spelling still present in the pinned weapon pack. Preserve
    // the plus qualifier; this is not a normalized one-or-two-hands requirement.
    ("held-in-one-plus-hands", "Held in 1+ hands"),
    (
        "held-in-one-hand-or-free-standing",
        "Held in 1 hand or free-standing",
    ),
    ("held-in-one-or-two-hands", "Held in 1 or 2 hands"),
    ("held-in-two-hands", "Held in 2 hands"),
    ("implanted", "Implanted"),
    ("installed-in-a-comm-unit", "Installed in a comm unit"),
    ("installed-in-a-datapad", "Installed in a datapad"),
    (
        "installed-in-a-grenade-launcher-or-two-handed-weapon-with-an-undermounted-grenade-launcher",
        "Installed in a Grenade Launcher or Two-Handed Weapon with an Undermounded Grenade Launcher",
    ),
    (
        "installed-in-a-ranged-weapon",
        "Installed in a ranged weapon",
    ),
    ("installed-in-a-weapon", "Installed in a Weapon"),
    ("installed-in-a-weapon-sight", "Installed in a Weapon Sight"),
    (
        "installed-in-a-weapon-that-deals-acid-or-fire-damage",
        "Installed in a weapon that deals acid or fire Damage",
    ),
    (
        "installed-in-a-weapon-that-deals-cold-damage",
        "Installed in a weapon that deals cold damage",
    ),
    (
        "installed-in-a-weapon-that-uses-a-battery",
        "Installed in a weapon that uses a battery",
    ),
    (
        "installed-in-a-weapon-that-uses-projectiles",
        "Installed in a weapon that uses projectiles",
    ),
    (
        "installed-in-a-weapon-with-the-kickback-trait",
        "Installed in a Weapon with the Kickback trait",
    ),
    ("installed-in-an-area-weapon", "Installed in an area weapon"),
    (
        "installed-in-an-area-weapon-that-deals-acid-cold-fire-or-void-damage",
        "Installed in an area weapon that deals acid, cold, fire or void damage",
    ),
    ("installed-in-armor", "Installed in Armor"),
    (
        "installed-in-armor-with-the-energy-shielding-upgrade",
        "Installed in Armor with the Energy Shielding upgrade",
    ),
    (
        "installed-in-armor-with-the-exposed-trait",
        "Installed in Armor with the Exposed trait",
    ),
    ("installed-in-atech-vehicle", "Installed in a tech vehicle"),
    (
        "installed-in-light-or-medium-armor",
        "Installed in light or medium armor",
    ),
    (
        "installed-in-one-handed-weapon-grip",
        "Installed in One-Handed Weapon Grip",
    ),
    (
        "installed-in-ranged-weapon-without-a-loudener",
        "Installed in Ranged Weapon without a Loudener",
    ),
    (
        "installed-in-two-handed-weapon",
        "Installed in a Two-Handed Weapon",
    ),
    (
        "installed-on-a-weapon-without-a-silencer",
        "Installed on a Weapon without a Silencer",
    ),
    ("installed-on-atech-lock", "Installed on a tech lock"),
    (
        "mounted-on-a-tripod-or-bracket",
        "Mounted on a tripod or bracket",
    ),
    ("other", "Other"),
    ("sewn-into-clothing", "Sewn Into Clothing"),
    ("tattooed-on-the-body", "Tattooed on the Body"),
    ("touched", "Touched"),
    ("worn", "Worn"),
    (
        "worn-and-attached-to-two-weapons",
        "Worn and Attached to Two Weapons",
    ),
    ("worn-under-armor", "Worn Under Light Armor or Clothes"),
    ("wornamulet", "Worn Amulet"),
    ("wornanklets", "Worn Anklets"),
    ("wornarmbands", "Worn Armbands"),
    ("wornbackpack", "Worn Backpack"),
    ("wornbarding", "Worn Barding"),
    ("wornbelt", "Worn Belt"),
    ("wornbeltpouch", "Worn Belt Pouch"),
    ("wornboots", "Worn Boots"),
    ("wornbracelet", "Worn Bracelet"),
    ("wornbracers", "Worn Bracers"),
    ("worncap", "Worn Cap"),
    ("worncape", "Worn Cape"),
    ("worncirclet", "Worn Circlet"),
    ("worncloak", "Worn Cloak"),
    ("wornclothing", "Worn Clothing"),
    ("worncollar", "Worn Collar"),
    ("worncrown", "Worn Crown"),
    ("wornepaulet", "Worn Epaulets"),
    ("worneyeglasses", "Worn Eyeglasses"),
    ("worneyepiece", "Worn Eyepiece"),
    ("wornfootwear", "Worn Footwear"),
    ("worngarment", "Worn Garment"),
    ("worngloves", "Worn Gloves"),
    ("wornheadwear", "Worn Headwear"),
    ("wornhorseshoes", "Worn Horseshoes"),
    ("wornmask", "Worn Mask"),
    ("wornnecklace", "Worn Necklace"),
    ("wornonbelt", "Worn on Belt"),
    ("wornoronehand", "Worn or Held in One Hand"),
    ("wornring", "Worn Ring"),
    ("wornsaddle", "Worn Saddle"),
    ("wornsandles", "Worn Sandles"),
    ("wornshoes", "Worn Shoes"),
    ("wornwhilesleeping", "Worn while sleeping"),
    ("wornwrist", "Worn on Wrists"),
];

pub(crate) fn usage(value: &str) -> String {
    USAGES
        .iter()
        .find(|(key, _)| *key == value)
        .map_or(value, |(_, label)| label)
        .to_owned()
}

#[cfg(test)]
mod tests {
    #[test]
    fn exact_usage_labels_do_not_collapse_attachment_or_hand_choices() {
        assert_eq!(
            super::usage("held-in-one-or-two-hands"),
            "Held in 1 or 2 hands"
        );
        assert_eq!(super::usage("held-in-one-plus-hands"), "Held in 1+ hands");
        assert_eq!(super::usage("held-in-one-hand"), "Held in 1 hand");
        assert_eq!(super::usage("held-in-two-hands"), "Held in 2 hands");
        assert_eq!(super::usage("attached-to-firearm"), "Attached To Firearm");
        assert_eq!(super::usage("custom-homebrew"), "custom-homebrew");
    }
}
