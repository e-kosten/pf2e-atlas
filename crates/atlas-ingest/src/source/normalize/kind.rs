use atlas_domain::RecordKind;

pub(crate) fn classify_record(document_type: &str, record_type: &str) -> Option<RecordKind> {
    match (document_type, record_type) {
        ("Actor", "npc") => Some(RecordKind::Creature),
        ("Actor", "character") => Some(RecordKind::Character),
        ("Actor", "familiar") => Some(RecordKind::Companion),
        ("Actor", "army") => Some(RecordKind::Army),
        ("Actor", "hazard") => Some(RecordKind::Hazard),
        ("Actor", "vehicle") => Some(RecordKind::Vehicle),
        (
            "Item",
            "ammo" | "armor" | "backpack" | "consumable" | "equipment" | "kit" | "shield"
            | "treasure" | "weapon",
        ) => Some(RecordKind::Equipment),
        ("Item", "feat") => Some(RecordKind::Feat),
        ("Item", "spell") => Some(RecordKind::Spell),
        ("Item", "affliction" | "affliction-instance") => Some(RecordKind::Affliction),
        ("Item", "action" | "condition" | "effect") => Some(RecordKind::Rule),
        ("Item", "ancestry" | "background" | "class" | "heritage") => {
            Some(RecordKind::CharacterOption)
        }
        ("Item", "deity") | ("JournalEntry", _) | ("JournalEntryPage", _) => Some(RecordKind::Lore),
        ("Macro", "script") | ("RollTable", _) => Some(RecordKind::Tooling),
        ("Item", "campaignFeature") => Some(RecordKind::CampaignFeature),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_complete_foundry_document_type_taxonomy() {
        let cases = [
            ("Actor", "npc", RecordKind::Creature),
            ("Actor", "character", RecordKind::Character),
            ("Actor", "familiar", RecordKind::Companion),
            ("Actor", "army", RecordKind::Army),
            ("Actor", "hazard", RecordKind::Hazard),
            ("Actor", "vehicle", RecordKind::Vehicle),
            ("Item", "ammo", RecordKind::Equipment),
            ("Item", "armor", RecordKind::Equipment),
            ("Item", "backpack", RecordKind::Equipment),
            ("Item", "consumable", RecordKind::Equipment),
            ("Item", "equipment", RecordKind::Equipment),
            ("Item", "kit", RecordKind::Equipment),
            ("Item", "shield", RecordKind::Equipment),
            ("Item", "treasure", RecordKind::Equipment),
            ("Item", "weapon", RecordKind::Equipment),
            ("Item", "feat", RecordKind::Feat),
            ("Item", "spell", RecordKind::Spell),
            ("Item", "action", RecordKind::Rule),
            ("Item", "condition", RecordKind::Rule),
            ("Item", "effect", RecordKind::Rule),
            ("Item", "ancestry", RecordKind::CharacterOption),
            ("Item", "background", RecordKind::CharacterOption),
            ("Item", "class", RecordKind::CharacterOption),
            ("Item", "heritage", RecordKind::CharacterOption),
            ("Item", "deity", RecordKind::Lore),
            ("JournalEntry", "JournalEntry", RecordKind::Lore),
            ("Macro", "script", RecordKind::Tooling),
            ("RollTable", "RollTable", RecordKind::Tooling),
            ("Item", "campaignFeature", RecordKind::CampaignFeature),
            ("Item", "affliction", RecordKind::Affliction),
            ("Item", "affliction-instance", RecordKind::Affliction),
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
