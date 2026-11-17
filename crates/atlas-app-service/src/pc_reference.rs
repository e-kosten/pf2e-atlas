use crate::{
    AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext, pathbuilder,
};
use atlas_app_model::{
    AppRecoverableAction, EncounterMutationView, EncounterPcAssociation, EncounterPcReferenceView,
    PathbuilderPreviewView, PcAbilitySelection, PcAbilityView, PcReferenceView,
    SavePathbuilderPcRequest, SavePcAbilityLinkRequest, SetEncounterPcReferenceRequest,
};
use atlas_domain::{RecordKey, SourceRecordSummary};
use atlas_local_state::{PartiesRead, PcAbilityKind, PcReference};
use atlas_runtime::{ExactRecordRead, RuntimeError, WorkspaceRead};
use atlas_search::{AtlasRetrievalService, GetRecordsRequest, ResolveRecordRequest};
use std::collections::{HashMap, HashSet};

impl AtlasAppService {
    pub fn preview_pathbuilder_pc(&self, json: &str) -> AppServiceResult<PathbuilderPreviewView> {
        let mut character = pathbuilder::parse(json)?;
        self.with_workspace(&ReadExecutionContext::uncancellable(), move |session| {
            resolve(session.retrieval(), &mut character.reference)?;
            let keys = keys(&character.reference);
            let records = session
                .retrieval()
                .get_records(GetRecordsRequest { record_keys: &keys })?;
            Ok(PathbuilderPreviewView {
                name: character.name,
                level: character.level,
                artifact_generation: session.generation()?,
                abilities: abilities_view(character.reference, records.records),
            })
        })
    }

    pub fn save_pathbuilder_pc(
        &self,
        party_key: &str,
        request: SavePathbuilderPcRequest,
    ) -> AppServiceResult<PcReferenceView> {
        let mut character = pathbuilder::parse(&request.json)?;
        let party = party_key.to_owned();
        let SavePathbuilderPcRequest {
            member_key,
            expected_party_revision,
            expected_artifact_generation,
            ..
        } = request;
        let refresh_member = member_key.clone();
        let (character, revision) =
            self.with_workspace(&ReadExecutionContext::uncancellable(), move |session| {
                require_generation(session, &expected_artifact_generation)?;
                let state = session.local_state()?;
                let parties = state.parties();
                parties.get_at_revision(&party, expected_party_revision)?;
                let previous = refresh_member
                    .as_deref()
                    .map(|member| parties.pc_reference(&party, member))
                    .transpose()?
                    .flatten();
                resolve(session.retrieval(), &mut character.reference)?;
                if let Some(previous) = previous {
                    let saved: HashMap<_, _> = previous
                        .abilities
                        .into_iter()
                        .filter_map(|ability| ability.manual.map(|record| (ability.key, record)))
                        .collect();
                    for ability in &mut character.reference.abilities {
                        ability.manual = saved.get(&ability.key).cloned();
                    }
                }
                Ok((character, session.reference_revision()?))
            })?;
        self.with_write(|session| {
            session.require_reference_revision(revision)?;
            let state = session.local_state()?;
            let parties = state.parties();
            let member_key = parties.save_imported_pc(
                party_key,
                expected_party_revision,
                member_key.as_deref(),
                &character.name,
                character.level,
                &character.reference,
            )?;
            pc_reference_view(&parties, party_key, &member_key, |keys| {
                read_titles(session.exact_records(), keys)
            })
        })
    }

    pub fn pc_reference(
        &self,
        party_key: &str,
        member_key: &str,
    ) -> AppServiceResult<PcReferenceView> {
        self.with_state(|session| {
            let state = session.local_state()?;
            pc_reference_view(&state.parties(), party_key, member_key, |keys| {
                read_titles(session.exact_records(), keys)
            })
        })
    }

    pub fn save_pc_ability_link(
        &self,
        party_key: &str,
        member_key: &str,
        ability_key: &str,
        request: SavePcAbilityLinkRequest,
    ) -> AppServiceResult<PcReferenceView> {
        let (manual, revision) = match request.selection {
            PcAbilitySelection::Automatic => (None, None),
            PcAbilitySelection::Manual {
                record_key,
                expected_artifact_generation,
            } => self.with_workspace(&ReadExecutionContext::uncancellable(), move |session| {
                require_generation(session, &expected_artifact_generation)?;
                let available = session.retrieval().get_records(GetRecordsRequest {
                    record_keys: std::slice::from_ref(&record_key),
                })?;
                if available.records.is_empty() {
                    return Err(AppServiceError::invalid_request(
                        "This rule is no longer available. Search again before linking it.",
                    ));
                }
                Ok((Some(record_key), Some(session.reference_revision()?)))
            })?,
        };
        self.with_write(|session| {
            if let Some(revision) = revision {
                session.require_reference_revision(revision)?;
            }
            let state = session.local_state()?;
            let parties = state.parties();
            parties.get_at_revision(party_key, request.expected_party_revision)?;
            let mut reference = parties
                .pc_reference(party_key, member_key)?
                .ok_or_else(|| {
                    AppServiceError::invalid_request("This PC has no imported reference.")
                })?;
            let ability = reference
                .abilities
                .iter_mut()
                .find(|ability| ability.key == ability_key)
                .ok_or_else(|| {
                    AppServiceError::invalid_request("This ability is no longer on the PC.")
                })?;
            ability.manual = manual;
            parties.update_pc_reference(
                party_key,
                request.expected_party_revision,
                member_key,
                &reference,
            )?;
            pc_reference_view(&parties, party_key, member_key, |keys| {
                read_titles(session.exact_records(), keys)
            })
        })
    }

    pub fn encounter_pc_reference(
        &self,
        encounter: &str,
        participant: &str,
    ) -> AppServiceResult<EncounterPcReferenceView> {
        self.with_state(|session| {
            let state = session.local_state()?;
            let parties = state.parties();
            let reference = parties
                .encounter_pc_member(encounter, participant)?
                .map(|member| {
                    pc_reference_view(&parties, &member.party_key, &member.member_key, |keys| {
                        read_titles(session.exact_records(), keys)
                    })
                })
                .transpose()?;
            Ok(EncounterPcReferenceView { reference })
        })
    }

    pub fn set_encounter_pc_reference(
        &self,
        encounter: &str,
        participant: &str,
        request: SetEncounterPcReferenceRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        self.with_write(|session| {
            let member = match &request.association {
                EncounterPcAssociation::Attach { member_key } => Some(member_key.as_str()),
                EncounterPcAssociation::Detach => None,
            };
            session.local_state()?.parties().set_encounter_pc_member(
                encounter,
                participant,
                request.expected_encounter_revision,
                member,
            )?;
            Ok(EncounterMutationView {
                encounter_key: encounter.into(),
            })
        })
    }
}

fn require_generation(session: &WorkspaceRead, expected: &str) -> AppServiceResult<()> {
    let actual = session.generation()?;
    if actual != expected {
        return Err(AppServiceError::artifact_generation_changed(
            expected,
            &actual,
            AppRecoverableAction::RefreshResults,
        ));
    }
    Ok(())
}

fn resolve(retrieval: &AtlasRetrievalService, reference: &mut PcReference) -> AppServiceResult<()> {
    let mut resolved = HashMap::new();
    for ability in &mut reference.abilities {
        let cache_key = (ability.kind, ability.label.clone());
        let (automatic, ambiguous) = if let Some(cached) = resolved.get(&cache_key) {
            cached
        } else {
            let source_types: &[_] = match ability.kind {
                PcAbilityKind::Feat => &["feat", "action"],
                PcAbilityKind::Heritage => &["heritage"],
                PcAbilityKind::Special => &["feat", "action", "spell", "heritage"],
                PcAbilityKind::Spell | PcAbilityKind::FocusSpell => &["spell"],
            };
            let matches = retrieval.resolve_record(ResolveRecordRequest {
                query: &ability.label,
                filter: None,
            })?;
            let mut matches = matches.into_iter().filter(|item| {
                retrieval.is_pf2e_record(&item.record.key)
                    && item
                        .record
                        .source_type
                        .as_value()
                        .is_some_and(|kind| source_types.contains(&kind.as_str()))
            });
            let first = matches.next();
            let ambiguous = matches.next().is_some();
            let target = if ambiguous {
                None
            } else {
                first.map(|result| result.record.key)
            };
            resolved.entry(cache_key).or_insert((target, ambiguous))
        };
        ability.automatic = automatic.clone();
        ability.ambiguous = *ambiguous;
    }
    Ok(())
}

fn pc_reference_view(
    parties: &PartiesRead<'_>,
    party_key: &str,
    member_key: &str,
    titles: impl FnOnce(&[RecordKey]) -> AppServiceResult<Vec<SourceRecordSummary>>,
) -> AppServiceResult<PcReferenceView> {
    let party = parties.get(party_key)?;
    let reference = parties.pc_reference(party_key, member_key)?;
    let member = party
        .members
        .into_iter()
        .find(|member| member.member_key == member_key)
        .ok_or_else(|| {
            AppServiceError::invalid_request("The selected PC is no longer in this party.")
        })?;
    let imported = reference.is_some();
    let abilities = if let Some(reference) = reference {
        let keys = keys(&reference);
        let records = if keys.is_empty() {
            Vec::new()
        } else {
            titles(&keys)?
        };
        abilities_view(reference, records)
    } else {
        Vec::new()
    };
    Ok(PcReferenceView {
        party_key: party.party_key,
        party_revision: party.revision,
        member_key: member.member_key,
        name: member.name,
        level: member.level,
        imported,
        abilities,
    })
}

fn read_titles(
    records: Result<ExactRecordRead<'_>, RuntimeError>,
    keys: &[RecordKey],
) -> AppServiceResult<Vec<SourceRecordSummary>> {
    match records {
        Ok(records) => Ok(records.summaries(keys)?),
        Err(error) if error.reference_unavailable() => Ok(Vec::new()),
        Err(error) => Err(error.into()),
    }
}

fn keys(reference: &PcReference) -> Vec<RecordKey> {
    let mut seen = HashSet::new();
    reference
        .abilities
        .iter()
        .filter_map(|ability| ability.record_key())
        .filter(|key| seen.insert(*key))
        .cloned()
        .collect()
}

fn abilities_view(reference: PcReference, records: Vec<SourceRecordSummary>) -> Vec<PcAbilityView> {
    let titles: HashMap<_, _> = records
        .into_iter()
        .map(|record| (record.key, record.name.value))
        .collect();
    reference
        .abilities
        .into_iter()
        .map(|ability| {
            let record_key = ability.record_key().cloned();
            let record_title = record_key
                .as_ref()
                .and_then(|key| titles.get(key))
                .and_then(Clone::clone);
            PcAbilityView {
                ability_key: ability.key,
                label: ability.label,
                choice: ability.choice,
                kind: ability.kind.as_str().into(),
                manual: ability.manual.is_some(),
                ambiguous: ability.ambiguous,
                record_key,
                record_title,
            }
        })
        .collect()
}
