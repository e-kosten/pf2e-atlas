use crate::creature_catalog::{CreatureCatalog, CreatureFamily, Member, ScenarioUse};
use crate::{AtlasRetrievalService, GetRecordsRequest, SearchError};
use atlas_domain::{RecordKey, RecordKind, SourceRecordSummary};
use std::sync::OnceLock;

pub use crate::creature_catalog::CreatureAdjustment;

#[derive(Debug)]
pub struct CreatureFamilyRecords {
    pub id: &'static str,
    pub name: &'static str,
    pub members: Vec<CreatureFamilyRecord>,
}

#[derive(Debug)]
pub struct CreatureFamilyRecord {
    pub summary: SourceRecordSummary,
    pub named_base: Option<&'static RecordKey>,
    pub qualifier: Option<&'static str>,
}

pub struct CreatureDisplay {
    pub label: Option<&'static str>,
    pub qualifier: Option<&'static str>,
    pub scenario_uses: Vec<CreatureScenarioUse>,
}

pub struct CreatureScenarioUse {
    pub id: &'static str,
    pub publication: String,
    pub party_levels: Option<[u8; 2]>,
    pub minimum_challenge_points: Option<u16>,
    pub qualifier: Option<&'static str>,
}

fn catalog() -> Option<&'static CreatureCatalog> {
    static CATALOG: OnceLock<Option<CreatureCatalog>> = OnceLock::new();
    CATALOG
        .get_or_init(|| {
            match serde_json::from_str(include_str!(concat!(
                env!("OUT_DIR"),
                "/creature_families.json"
            ))) {
                Ok(catalog) => Some(catalog),
                Err(error) => {
                    eprintln!("invalid bundled creature families: {error}");
                    None
                }
            }
        })
        .as_ref()
}

fn member_for(key: &RecordKey) -> Option<(&'static str, &'static CreatureFamily, &'static Member)> {
    catalog()?.iter().find_map(|(id, family)| {
        let member = family.members.iter().find(|member| member.key == *key)?;
        Some((id.as_str(), family, member))
    })
}

fn matches(member: &Member, summary: &SourceRecordSummary) -> bool {
    member.key == summary.key
        && summary.record_kind.as_value() == Some(&RecordKind::Creature)
        && summary.name.as_value() == Some(&member.name)
        && member
            .publication
            .as_ref()
            .is_none_or(|title| summary.publication_title.as_value() == Some(title))
}

pub fn creature_display(summary: &SourceRecordSummary) -> Option<CreatureDisplay> {
    let (_, _, member) = member_for(&summary.key)?;
    if !matches(member, summary) {
        return None;
    }
    let scenario_uses = member
        .uses
        .iter()
        .filter_map(|use_| {
            let title = summary.publication_title.as_value()?;
            let short_title = title
                .split_once(": ")
                .map_or(title.as_str(), |(_, title)| title);
            Some(CreatureScenarioUse {
                id: &use_.id,
                publication: short_title.into(),
                party_levels: use_.party_levels,
                minimum_challenge_points: use_.minimum_challenge_points,
                qualifier: use_.qualifier.as_deref(),
            })
        })
        .collect();
    Some(CreatureDisplay {
        label: member.label.as_deref(),
        qualifier: member.qualifier.as_deref(),
        scenario_uses,
    })
}

fn select_use<'a>(member: &'a Member, selected: Option<&str>) -> Option<&'a ScenarioUse> {
    match selected {
        Some(id) => member.uses.iter().find(|use_| use_.id == id),
        None => match member.uses.as_slice() {
            [use_] if use_.minimum_challenge_points.is_none() => Some(use_),
            _ => None,
        },
    }
}

impl AtlasRetrievalService {
    pub fn creature_family_for(
        &self,
        summary: &SourceRecordSummary,
    ) -> Result<Option<CreatureFamilyRecords>, SearchError> {
        let Some((id, family, member)) = member_for(&summary.key) else {
            return Ok(None);
        };
        if !matches(member, summary) {
            return Ok(None);
        }
        Ok(self.read_creature_family(id, family)?.filter(|family| {
            family
                .members
                .iter()
                .any(|member| member.summary.key == summary.key)
        }))
    }

    pub fn creature_family(&self, id: &str) -> Result<Option<CreatureFamilyRecords>, SearchError> {
        let Some((id, family)) = catalog().and_then(|catalog| catalog.get_key_value(id)) else {
            return Ok(None);
        };
        self.read_creature_family(id, family)
    }

    fn read_creature_family(
        &self,
        id: &'static str,
        family: &'static CreatureFamily,
    ) -> Result<Option<CreatureFamilyRecords>, SearchError> {
        let keys: Vec<_> = family
            .members
            .iter()
            .map(|member| member.key.clone())
            .collect();
        let batch = self.get_records(GetRecordsRequest { record_keys: &keys })?;
        let mut members: Vec<_> = batch
            .records
            .into_iter()
            .filter_map(|summary| {
                let member = family
                    .members
                    .iter()
                    .find(|member| matches(member, &summary))?;
                Some(CreatureFamilyRecord {
                    summary,
                    named_base: member.named_base.as_ref(),
                    qualifier: member.qualifier.as_deref(),
                })
            })
            .collect();
        let verified_bases: std::collections::BTreeSet<_> = members
            .iter()
            .filter(|member| member.named_base.is_none())
            .map(|member| member.summary.key.clone())
            .collect();
        members.retain(|member| {
            member
                .named_base
                .is_none_or(|base| verified_bases.contains(base))
        });
        if members.len() < 2 || members.iter().all(|member| member.named_base.is_some()) {
            return Ok(None);
        }
        Ok(Some(CreatureFamilyRecords {
            id,
            name: &family.name,
            members,
        }))
    }

    pub fn creature_adjustment_override(
        &self,
        base: &SourceRecordSummary,
        adjustment: CreatureAdjustment,
        scenario_use: Option<&str>,
    ) -> Result<Option<RecordKey>, SearchError> {
        let Some((_, family, member)) = member_for(&base.key) else {
            return Ok(None);
        };
        if !matches(member, base) {
            return Ok(None);
        }
        let Some(scope) = select_use(member, scenario_use) else {
            return Ok(None);
        };
        let Some(entry) = family.overrides.iter().find(|entry| {
            entry.base == base.key
                && entry.adjustment == adjustment
                && entry.scenario_use == scope.id
        }) else {
            return Ok(None);
        };
        let batch = self.get_records(GetRecordsRequest {
            record_keys: std::slice::from_ref(&entry.target),
        })?;
        let Some((_, _, target)) = member_for(&entry.target) else {
            return Ok(None);
        };
        Ok(batch
            .records
            .iter()
            .any(|summary| matches(target, summary))
            .then(|| entry.target.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sole_challenge_point_use_requires_explicit_selection() {
        let (_, _, actor) =
            member_for(&RecordKey::parse("pfs-season-3-bestiary:pYR3Ovkd9eKQQ5qE").unwrap())
                .unwrap();

        assert!(select_use(actor, None).is_none());
        assert_eq!(
            select_use(actor, Some("party-11-12-cp-28"))
                .unwrap()
                .minimum_challenge_points,
            Some(28),
        );
    }
}
