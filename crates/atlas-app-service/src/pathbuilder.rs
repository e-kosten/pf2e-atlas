use crate::{AppServiceError, AppServiceResult};
use atlas_domain::normalize_record_name;
use atlas_local_state::{
    MAX_PC_ABILITIES, MAX_PC_ABILITY_LABEL, PcAbility, PcAbilityKind, PcReference,
};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};

const MAX_EXPORT_BYTES: usize = 512 * 1024;

#[derive(Debug)]
pub(super) struct PathbuilderCharacter {
    pub name: String,
    pub level: i64,
    pub reference: PcReference,
}

#[derive(Deserialize)]
struct Export {
    success: bool,
    build: Build,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Build {
    name: String,
    level: i64,
    feats: Vec<Vec<Value>>,
    specials: Vec<String>,
    spell_casters: Vec<Caster>,
    #[serde(default)]
    focus: BTreeMap<String, BTreeMap<String, Focus>>,
}

#[derive(Deserialize)]
struct Caster {
    spells: Vec<SpellRank>,
    #[serde(default)]
    prepared: Vec<SpellRank>,
}

#[derive(Deserialize)]
struct SpellRank {
    list: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Focus {
    #[serde(default)]
    focus_cantrips: Vec<String>,
    #[serde(default)]
    focus_spells: Vec<String>,
}

pub(super) fn parse(source: &str) -> AppServiceResult<PathbuilderCharacter> {
    if source.len() > MAX_EXPORT_BYTES {
        return Err(AppServiceError::invalid_request(
            "The Pathbuilder export is too large.",
        ));
    }
    let Export { success, build } = serde_json::from_str(source).map_err(|_| {
        AppServiceError::invalid_request("Paste a complete Pathbuilder character JSON export.")
    })?;
    let name = build.name.trim();
    if !success || name.is_empty() || name.chars().count() > 255 || !(1..=20).contains(&build.level)
    {
        return Err(AppServiceError::invalid_request(
            "The export must contain a character name and a level from 1 to 20.",
        ));
    }
    let mut abilities = Abilities::default();
    for row in &build.feats {
        let [
            Value::String(label),
            choice,
            Value::String(slot),
            Value::Number(_),
            ..,
        ] = &row[..]
        else {
            return Err(AppServiceError::invalid_request(
                "A feat in the Pathbuilder export is incomplete.",
            ));
        };
        let choice = match choice {
            Value::Null => None,
            Value::String(value) => Some(value.as_str()),
            _ => {
                return Err(AppServiceError::invalid_request(
                    "A feat option in the Pathbuilder export is invalid.",
                ));
            }
        };
        let kind = if slot == "Heritage" {
            PcAbilityKind::Heritage
        } else {
            PcAbilityKind::Feat
        };
        abilities.push(kind, label, choice)?;
    }
    for label in &build.specials {
        abilities.push(PcAbilityKind::Special, label, None)?;
    }
    for caster in &build.spell_casters {
        for rank in caster.spells.iter().chain(&caster.prepared) {
            for spell in &rank.list {
                abilities.push(PcAbilityKind::Spell, spell, None)?;
            }
        }
    }
    for focus in build.focus.values().flat_map(BTreeMap::values) {
        for spell in focus.focus_cantrips.iter().chain(&focus.focus_spells) {
            abilities.push(PcAbilityKind::FocusSpell, spell, None)?;
        }
    }
    Ok(PathbuilderCharacter {
        name: name.into(),
        level: build.level,
        reference: PcReference {
            abilities: abilities.items,
        },
    })
}

#[derive(Default)]
struct Abilities {
    items: Vec<PcAbility>,
    keys: HashSet<String>,
}

impl Abilities {
    fn push(
        &mut self,
        kind: PcAbilityKind,
        label: &str,
        choice: Option<&str>,
    ) -> AppServiceResult<()> {
        let label = label.trim();
        let choice = choice.map(str::trim).filter(|value| !value.is_empty());
        if label.is_empty() || label.eq_ignore_ascii_case("not selected") || label == "undefined" {
            return Ok(());
        }
        if label.chars().count() > MAX_PC_ABILITY_LABEL
            || choice.is_some_and(|value| value.chars().count() > MAX_PC_ABILITY_LABEL)
        {
            return Err(AppServiceError::invalid_request(
                "An ability name or option in the Pathbuilder export is too long.",
            ));
        }
        let parts = [
            kind.as_str().to_owned(),
            normalize_record_name(label),
            choice.map(normalize_record_name).unwrap_or_default(),
        ];
        let mut digest = Sha256::new();
        for part in parts {
            digest.update((part.len() as u64).to_be_bytes());
            digest.update(part);
        }
        let key = format!("{:x}", digest.finalize());
        if self.keys.insert(key.clone()) {
            if self.items.len() == MAX_PC_ABILITIES {
                return Err(AppServiceError::invalid_request(
                    "The Pathbuilder export contains too many abilities.",
                ));
            }
            self.items.push(PcAbility {
                key,
                label: label.into(),
                choice: choice.map(str::to_owned),
                kind,
                automatic: None,
                ambiguous: false,
                manual: None,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
