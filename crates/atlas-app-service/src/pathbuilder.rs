use crate::{AppServiceError, AppServiceResult};
use atlas_local_state::{LocalStateError, PathbuilderCharacter, PathbuilderSource};

pub(super) fn preview(source: &str) -> AppServiceResult<PathbuilderCharacter> {
    PathbuilderSource::preview(source).map_err(import_error)
}

pub(super) fn parse(source: &str) -> AppServiceResult<PathbuilderCharacter> {
    PathbuilderSource::receipt_reference(source).map_err(import_error)
}

pub(super) fn source(source: String) -> AppServiceResult<PathbuilderSource> {
    PathbuilderSource::new(source).map_err(import_error)
}

fn import_error(error: LocalStateError) -> AppServiceError {
    match error {
        LocalStateError::InvalidParty(message) => AppServiceError::invalid_request(message),
        error => error.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_local_state::PcAbilityKind;
    use serde_json::json;

    #[test]
    fn preserves_choices_and_independent_focus_spells() {
        let source = json!({ "success":true, "build":{
            "name":"Test wizard", "level":7,
            "feats":[["Assurance","Stealth","Skill Feat",2],["Assurance","Athletics","Skill Feat",4,"Skill Feat 4","standardChoice",null]],
            "specials":["Drain Bonded Item"],
            "spellCasters":[{"spells":[{"spellLevel":1,"list":["Force Barrage"]}],"prepared":[{"spellLevel":1,"list":["Force Barrage"]}]}],
            "focus":{"Unassigned":{"int":{"focusSpells":["Gravity Weapon"]}}}
        }});
        let first = parse(&source.to_string()).unwrap();
        let abilities = first.reference.abilities;
        assert_eq!(abilities.len(), 5);
        assert_ne!(abilities[0].key, abilities[1].key);
        assert_eq!(abilities[0].choice.as_deref(), Some("Stealth"));
        assert_eq!(abilities[4].kind, PcAbilityKind::FocusSpell);
        let mut refreshed = source;
        refreshed["build"]["level"] = json!(8);
        refreshed["build"]["feats"][0][3] = json!(6);
        assert_eq!(
            parse(&refreshed.to_string()).unwrap().reference.abilities,
            abilities
        );
    }

    #[test]
    fn rejects_incomplete_or_unrelated_exports() {
        assert!(parse(r#"{"success":true,"build":{"name":"PC","level":7}}"#).is_err());
        assert!(parse(r#"{"type":"character","items":[]}"#).is_err());
        assert!(parse(r#"{"success":true,"build":{"name":"PC","level":7,"feats":[[42]],"specials":[],"spellCasters":[]}}"#).is_err());
    }
}
