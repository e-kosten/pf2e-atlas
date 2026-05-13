use atlas_domain::RecordFamily;

pub(crate) fn classify_record(document_type: &str, record_type: &str) -> Option<RecordFamily> {
    match (document_type, record_type) {
        ("Actor", "npc") => Some(RecordFamily::Creature),
        ("Actor", "character") => Some(RecordFamily::Character),
        ("Actor", "familiar") => Some(RecordFamily::Companion),
        ("Actor", "army") => Some(RecordFamily::Army),
        ("Actor", "hazard") => Some(RecordFamily::Hazard),
        ("Actor", "vehicle") => Some(RecordFamily::Vehicle),
        (
            "Item",
            "ammo" | "armor" | "backpack" | "consumable" | "equipment" | "kit" | "shield"
            | "treasure" | "weapon",
        ) => Some(RecordFamily::Equipment),
        ("Item", "feat") => Some(RecordFamily::Feat),
        ("Item", "spell") => Some(RecordFamily::Spell),
        ("Item", "affliction" | "affliction-instance") => Some(RecordFamily::Affliction),
        ("Item", "action" | "condition" | "effect") => Some(RecordFamily::Rule),
        ("Item", "ancestry" | "background" | "class" | "heritage") => {
            Some(RecordFamily::CharacterOption)
        }
        ("Item", "deity") | ("JournalEntry", _) | ("JournalEntryPage", _) => {
            Some(RecordFamily::Lore)
        }
        ("Macro", "script") | ("RollTable", _) => Some(RecordFamily::Tooling),
        ("Item", "campaignFeature") => Some(RecordFamily::CampaignFeature),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_complete_foundry_document_type_taxonomy() {
        let cases = [
            ("Actor", "npc", RecordFamily::Creature),
            ("Actor", "character", RecordFamily::Character),
            ("Actor", "familiar", RecordFamily::Companion),
            ("Actor", "army", RecordFamily::Army),
            ("Actor", "hazard", RecordFamily::Hazard),
            ("Actor", "vehicle", RecordFamily::Vehicle),
            ("Item", "ammo", RecordFamily::Equipment),
            ("Item", "armor", RecordFamily::Equipment),
            ("Item", "backpack", RecordFamily::Equipment),
            ("Item", "consumable", RecordFamily::Equipment),
            ("Item", "equipment", RecordFamily::Equipment),
            ("Item", "kit", RecordFamily::Equipment),
            ("Item", "shield", RecordFamily::Equipment),
            ("Item", "treasure", RecordFamily::Equipment),
            ("Item", "weapon", RecordFamily::Equipment),
            ("Item", "feat", RecordFamily::Feat),
            ("Item", "spell", RecordFamily::Spell),
            ("Item", "action", RecordFamily::Rule),
            ("Item", "condition", RecordFamily::Rule),
            ("Item", "effect", RecordFamily::Rule),
            ("Item", "ancestry", RecordFamily::CharacterOption),
            ("Item", "background", RecordFamily::CharacterOption),
            ("Item", "class", RecordFamily::CharacterOption),
            ("Item", "heritage", RecordFamily::CharacterOption),
            ("Item", "deity", RecordFamily::Lore),
            ("JournalEntry", "JournalEntry", RecordFamily::Lore),
            ("Macro", "script", RecordFamily::Tooling),
            ("RollTable", "RollTable", RecordFamily::Tooling),
            ("Item", "campaignFeature", RecordFamily::CampaignFeature),
            ("Item", "affliction", RecordFamily::Affliction),
            ("Item", "affliction-instance", RecordFamily::Affliction),
        ];

        for (document_type, record_type, expected) in cases {
            assert_eq!(
                classify_record(document_type, record_type),
                Some(expected),
                "{document_type}|{record_type}"
            );
        }
    }

    #[test]
    fn leaves_unknown_foundry_taxonomy_for_skip_reporting() {
        assert_eq!(classify_record("Actor", "mystery"), None);
        assert_eq!(classify_record("Item", "mystery"), None);
    }
}
