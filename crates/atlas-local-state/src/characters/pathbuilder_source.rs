use super::reference::{
    MAX_PC_ABILITIES, MAX_PC_ABILITY_LABEL, PcAbility, PcAbilityKind, PcReference,
};
use crate::{LocalStateError, LocalStateResult};
use atlas_domain::normalize_record_name;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::value::RawValue;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};

const MAX_PATHBUILDER_EXPORT_BYTES: usize = 512 * 1024;

/// The original accepted JSON. Its fields may include private player data.
#[derive(Serialize)]
#[serde(transparent)]
pub struct PathbuilderSource(String);

#[derive(Deserialize)]
struct Envelope<'a> {
    #[serde(borrow)]
    build: &'a RawValue,
}

impl PathbuilderSource {
    pub fn new(json: String) -> LocalStateResult<Self> {
        Self::preview(&json)?;
        Ok(Self(json))
    }

    pub fn json(&self) -> &str {
        &self.0
    }

    /// Read the authored reference from a current, object-shaped export.
    pub fn preview(json: &str) -> LocalStateResult<PathbuilderCharacter> {
        check_size(json)?;
        let invalid =
            || LocalStateError::InvalidParty("Paste a complete Pathbuilder character JSON export.");
        let envelope: Envelope<'_> = serde_json::from_str(json).map_err(|_| invalid())?;
        if !json.trim_start().starts_with('{') || !envelope.build.get().starts_with('{') {
            return Err(invalid());
        }
        parse_reference(json)
    }

    /// Reconstruct the original import intent for receipts written before full
    /// exports were stored, including the shapes that their decoder accepted.
    pub fn receipt_reference(json: &str) -> LocalStateResult<PathbuilderCharacter> {
        parse_reference(json)
    }

    pub(crate) fn verify_reference(
        &self,
        reference: &PcReference,
    ) -> LocalStateResult<PathbuilderCharacter> {
        let authored = parse_reference(self.json())?;
        let actual = &reference.abilities;
        let expected = &authored.reference.abilities;
        if actual.len() != expected.len()
            || actual.iter().zip(expected).any(|(a, e)| {
                a.key != e.key || a.kind != e.kind || a.label != e.label || a.choice != e.choice
            })
        {
            return Err(LocalStateError::InvalidParty(
                "Saved Pathbuilder export and rule reference do not match.",
            ));
        }
        Ok(authored)
    }
}

impl std::fmt::Debug for PathbuilderSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PathbuilderSource")
            .finish_non_exhaustive()
    }
}

impl<'de> Deserialize<'de> for PathbuilderSource {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug)]
pub struct PathbuilderCharacter {
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
    feats: Vec<Vec<Box<RawValue>>>,
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

fn check_size(source: &str) -> LocalStateResult<()> {
    if source.len() > MAX_PATHBUILDER_EXPORT_BYTES {
        return Err(LocalStateError::InvalidParty(
            "The Pathbuilder export is too large.",
        ));
    }
    Ok(())
}

fn parse_reference(source: &str) -> LocalStateResult<PathbuilderCharacter> {
    check_size(source)?;
    let Export { success, build } = serde_json::from_str(source).map_err(|_| {
        LocalStateError::InvalidParty("Paste a complete Pathbuilder character JSON export.")
    })?;
    let name = build.name.trim();
    if !success || name.is_empty() || name.chars().count() > 255 || !(1..=20).contains(&build.level)
    {
        return Err(LocalStateError::InvalidParty(
            "The export must contain a character name and a level from 1 to 20.",
        ));
    }
    let mut abilities = Abilities::default();
    for row in &build.feats {
        let malformed =
            || LocalStateError::InvalidParty("A feat in the Pathbuilder export is incomplete.");
        let [label, choice, slot, level, ..] = row.as_slice() else {
            return Err(malformed());
        };
        let label: String = serde_json::from_str(label.get()).map_err(|_| malformed())?;
        let slot: String = serde_json::from_str(slot.get()).map_err(|_| malformed())?;
        serde_json::from_str::<serde_json::Number>(level.get()).map_err(|_| malformed())?;
        let choice: Option<String> = serde_json::from_str(choice.get()).map_err(|_| {
            LocalStateError::InvalidParty("A feat option in the Pathbuilder export is invalid.")
        })?;
        let kind = if slot == "Heritage" {
            PcAbilityKind::Heritage
        } else {
            PcAbilityKind::Feat
        };
        abilities.push(kind, &label, choice.as_deref())?;
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
    ) -> LocalStateResult<()> {
        let label = label.trim();
        let choice = choice.map(str::trim).filter(|value| !value.is_empty());
        if label.is_empty() || label.eq_ignore_ascii_case("not selected") || label == "undefined" {
            return Ok(());
        }
        if label.chars().count() > MAX_PC_ABILITY_LABEL
            || choice.is_some_and(|value| value.chars().count() > MAX_PC_ABILITY_LABEL)
        {
            return Err(LocalStateError::InvalidParty(
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
                return Err(LocalStateError::InvalidParty(
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
