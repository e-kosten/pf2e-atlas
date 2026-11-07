use crate::{
    IndexError, PublicationSpool, ReferenceAvailability, ReferenceBacking, ReferenceEntry,
    SqliteIndexReader,
};
use atlas_domain::{DefinitionKey, DefinitionNamespace, OwnedContentLocator, RecordKey};
use atlas_record::source_record::RecordLocalizationOccurrence;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const GENERAL: &str = "bestiary-ability-glossary-srd";
const FAMILY: &str = "bestiary-family-ability-glossary";
const NPC_RULE: &str = "PF2E.NPC.Abilities.Glossary.";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnedAbilityTarget {
    Record(RecordKey),
    Definition(DefinitionKey),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedAbilityEvidence {
    record: RecordKey,
    owners: Vec<OwnedContentLocator>,
    help: Option<OwnedAbilityTarget>,
    has_declaration: bool,
    action_keys: BTreeSet<RecordKey>,
}

#[derive(Default)]
pub struct OwnedAbilityCatalog(BTreeMap<String, Option<OwnedAbilityTarget>>);

impl OwnedAbilityCatalog {
    pub fn add_entry(&mut self, entry: &ReferenceEntry) {
        if entry.availability == ReferenceAvailability::SourceInapplicable
            || (entry.provenance.record_mapping.is_some()
                && matches!(entry.backing, ReferenceBacking::None))
        {
            return;
        }
        let target = match &entry.backing {
            ReferenceBacking::Record { key } => OwnedAbilityTarget::Record(key.clone()),
            _ => OwnedAbilityTarget::Definition(entry.key.clone()),
        };
        self.add(
            [
                &entry.provenance.label_key,
                &entry.provenance.description_key,
            ]
            .into_iter()
            .filter_map(|key| key.as_deref()),
            target,
        );
    }

    pub fn add_trait(
        &mut self,
        key: DefinitionKey,
        label: Option<&str>,
        description: Option<&str>,
    ) {
        self.add(
            [label, description].into_iter().flatten(),
            OwnedAbilityTarget::Definition(key),
        );
    }

    fn add<'a>(&mut self, keys: impl Iterator<Item = &'a str>, target: OwnedAbilityTarget) {
        for key in keys {
            self.0
                .entry(key.into())
                .and_modify(|existing| {
                    if existing.as_ref() != Some(&target) {
                        *existing = None;
                    }
                })
                .or_insert_with(|| Some(target.clone()));
        }
    }

    pub fn project(
        &self,
        occurrences: Vec<RecordLocalizationOccurrence>,
    ) -> Vec<OwnedAbilityEvidence> {
        let mut grouped = BTreeMap::new();
        for occurrence in occurrences {
            let localization = &occurrence.occurrence.localization_key;
            let Some(target) = self.0.get(localization).and_then(Option::as_ref) else {
                continue;
            };
            let locator = occurrence.locator;
            let declaration = locator.owners.len() == 1
                && locator.field == "/system/description/value"
                && localization.starts_with(NPC_RULE);
            let (visible, has_declaration, action_keys): &mut (BTreeSet<_>, bool, BTreeSet<_>) =
                grouped.entry((locator.record, locator.owners)).or_default();
            *has_declaration |= declaration;
            if occurrence.occurrence.visible {
                visible.insert(target.clone());
                if declaration
                    && let OwnedAbilityTarget::Record(key) = target
                    && key.pack().as_str() == GENERAL
                {
                    action_keys.insert(key.clone());
                }
            }
        }
        grouped
            .into_iter()
            .filter_map(
                |((record, owners), (visible, has_declaration, action_keys))| {
                    let help = if visible.len() == 1 {
                        visible.into_iter().next().filter(|target| match target {
                            OwnedAbilityTarget::Record(key) => {
                                matches!(key.pack().as_str(), GENERAL | FAMILY)
                            }
                            OwnedAbilityTarget::Definition(key) => {
                                key.namespace() == DefinitionNamespace::Rule
                                    && key
                                        .identifier()
                                        .strip_prefix("pf2e:")
                                        .is_some_and(|key| key.starts_with(NPC_RULE))
                            }
                        })
                    } else {
                        None
                    };
                    (help.is_some() || has_declaration).then_some(OwnedAbilityEvidence {
                        record,
                        owners,
                        help,
                        has_declaration,
                        action_keys,
                    })
                },
            )
            .collect()
    }
}

impl SqliteIndexReader {
    pub fn owned_ability_help(
        &self,
        record: &RecordKey,
        owners: &[OwnedContentLocator],
    ) -> Result<Option<OwnedAbilityTarget>, IndexError> {
        let json = self.native()?.query_row(
            "SELECT help_json FROM owned_ability_evidence WHERE record_key=? AND owners_json=? AND help_json IS NOT NULL",
            params![record.to_string(), serde_json::to_string(owners)?],
            |row| row.get::<_, String>(0),
        ).optional()?;
        json.map(|json| serde_json::from_str(&json).map_err(IndexError::from))
            .transpose()
    }
}

type StoredEvidence = (String, Option<String>, bool, String);

fn stored(row: &OwnedAbilityEvidence) -> Result<StoredEvidence, IndexError> {
    Ok((
        serde_json::to_string(&row.owners)?,
        row.help.as_ref().map(serde_json::to_string).transpose()?,
        row.has_declaration,
        serde_json::to_string(&row.action_keys)?,
    ))
}

pub(crate) fn write_evidence(c: &Connection, input: &PublicationSpool) -> Result<(), IndexError> {
    let mut stale = c
        .prepare("SELECT record_key,owners_json FROM owned_ability_evidence")?
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<BTreeSet<_>, _>>()?;
    input.owned_ability_evidence.visit(&mut |row| {
        let record = row.record.to_string();
        let (owners, help, has_declaration, action_keys) = stored(&row)?;
        stale.remove(&(record.clone(), owners.clone()));
        c.execute(
            "INSERT INTO owned_ability_evidence VALUES(?1,?2,?3,?4,?5)
             ON CONFLICT(record_key,owners_json) DO UPDATE SET
                 help_json=excluded.help_json, has_declaration=excluded.has_declaration,
                 action_keys_json=excluded.action_keys_json
             WHERE help_json IS NOT excluded.help_json OR has_declaration IS NOT excluded.has_declaration
                 OR action_keys_json IS NOT excluded.action_keys_json",
            params![record, owners, help, has_declaration, action_keys],
        )?;
        Ok(())
    })?;
    for (record, owners) in stale {
        c.execute(
            "DELETE FROM owned_ability_evidence WHERE record_key=? AND owners_json=?",
            params![record, owners],
        )?;
    }
    Ok(())
}

pub(crate) fn validate_record(
    c: &Connection,
    record: &RecordKey,
    expected: Vec<OwnedAbilityEvidence>,
) -> Result<(), IndexError> {
    let mut statement = c.prepare_cached(
        "SELECT owners_json,help_json,has_declaration,action_keys_json
         FROM owned_ability_evidence WHERE record_key=? ORDER BY owners_json",
    )?;
    let rows = statement.query_map([record.to_string()], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, bool>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    let actual = rows.collect::<Result<BTreeSet<_>, _>>()?;
    let expected = expected
        .iter()
        .map(stored)
        .collect::<Result<BTreeSet<_>, _>>()?;
    if actual != expected {
        return Err(IndexError::Invalid(format!(
            "owned ability evidence differs from checked source for {record}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_domain::OwnedContentIdentity;

    #[test]
    fn audit_rejects_noncanonical_owned_ability_addresses() {
        let input = crate::foundation_tests::fixture(false);
        let (_directory, path) = crate::foundation_tests::build(&input);
        let c = Connection::open(path).unwrap();
        let record = input.records[0].record.key().clone();
        let evidence = OwnedAbilityEvidence {
            record: record.clone(),
            owners: vec![OwnedContentLocator {
                collection: "/items".into(),
                identity: OwnedContentIdentity::Stable("aaaaaaaaaaaaaaaa".into()),
            }],
            help: None,
            has_declaration: true,
            action_keys: BTreeSet::new(),
        };
        let (owners, help, declaration, actions) = stored(&evidence).unwrap();
        c.execute(
            "INSERT INTO owned_ability_evidence VALUES(?,?,?,?,?)",
            params![record.to_string(), owners, help, declaration, actions],
        )
        .unwrap();
        validate_record(&c, &record, vec![evidence.clone()]).unwrap();
        c.execute(
            "UPDATE owned_ability_evidence SET owners_json=' ' || owners_json",
            [],
        )
        .unwrap();
        assert!(validate_record(&c, &record, vec![evidence]).is_err());
    }
}
