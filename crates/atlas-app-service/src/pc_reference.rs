use crate::{
    AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext, pathbuilder,
};
use atlas_app_model::{
    AppRecoverableAction, CharacterMutationView, CharacterPartyView, EncounterMutationView,
    EncounterPcAssociation, EncounterPcReferenceView, PathbuilderPreviewView, PcAbilitySectionView,
    PcAbilitySelection, PcAbilityView, PcReferenceView, SavePathbuilderPcRequest,
    SavePcAbilityLinkRequest, SetEncounterPcReferenceRequest,
};
use atlas_domain::{RecordKey, SourceRecordSummary};
use atlas_local_state::{Character, PcAbilityKind, PcReference};
use atlas_runtime::{ExactRecordRead, RuntimeError, WorkspaceRead};
use atlas_search::{AtlasRetrievalService, GetRecordsRequest, ResolveRecordRequest};
use std::collections::{HashMap, HashSet};

impl AtlasAppService {
    pub fn preview_pathbuilder_pc(
        &self,
        json: &str,
        character_key: Option<&str>,
    ) -> AppServiceResult<PathbuilderPreviewView> {
        let mut character = pathbuilder::parse(json)?;
        let character_key = character_key.map(str::to_owned);
        self.with_workspace(&ReadExecutionContext::uncancellable(), move |session| {
            let previous = character_key
                .as_deref()
                .map(|key| -> AppServiceResult<Character> {
                    Ok(session.local_state()?.characters().get(key)?)
                })
                .transpose()?;
            let character_revision = previous.as_ref().map(|pc: &Character| pc.revision);
            let previous_name = previous.as_ref().map(|pc| pc.name.clone());
            let previous_level = previous.as_ref().and_then(|pc| pc.level);
            if let Some(previous) = previous {
                preserve_choices(previous.reference, &mut character.reference);
            }
            resolve(session.retrieval(), &mut character.reference)?;
            let retained_manual_count = character
                .reference
                .abilities
                .iter()
                .filter(|a| a.manual.is_some())
                .count();
            let records = session.retrieval().get_records(GetRecordsRequest {
                record_keys: &keys(&character.reference),
            })?;
            let abilities = abilities_view(character.reference, records.records);
            let (sections, resolved_count, unresolved_count) = sections(&abilities);
            Ok(PathbuilderPreviewView {
                character_revision,
                previous_name,
                previous_level,
                retained_manual_count,
                name: character.name,
                level: character.level,
                artifact_generation: session.generation()?,
                abilities,
                sections,
                resolved_count,
                unresolved_count,
            })
        })
    }
    pub fn save_pathbuilder_pc(
        &self,
        request: SavePathbuilderPcRequest,
    ) -> AppServiceResult<CharacterMutationView> {
        let mut imported = pathbuilder::parse(&request.json)?;
        let digest = crate::characters::intent_digest(&(
            imported.name.trim(),
            imported.level,
            &request.party,
            &request.expected_artifact_generation,
            &request.character_key,
            request.expected_character_revision,
            &imported.reference,
        ))?;
        if request.character_key.is_none() {
            let recovered = self.with_state(|session| {
                let characters = session.local_state()?.characters();
                characters
                    .creation_receipt(&request.request_id, &digest)?
                    .map(|key| crate::characters::creation_view(&characters, &key))
                    .transpose()
            })?;
            if let Some(view) = recovered {
                return Ok(view);
            }
        }
        let expected_generation = request.expected_artifact_generation.clone();
        let refresh_key = request.character_key.clone();
        let refresh_revision = request.expected_character_revision;
        let (imported, reference_revision) =
            self.with_workspace(&ReadExecutionContext::uncancellable(), move |session| {
                require_generation(session, &expected_generation)?;
                if let Some(key) = &refresh_key {
                    let expected = refresh_revision.ok_or_else(|| {
                        AppServiceError::invalid_request(
                            "Review the character before refreshing it.",
                        )
                    })?;
                    let previous = session
                        .local_state()?
                        .characters()
                        .get_at_revision(key, expected)?;
                    preserve_choices(previous.reference, &mut imported.reference);
                }
                resolve(session.retrieval(), &mut imported.reference)?;
                Ok((imported, session.reference_revision()?))
            })?;
        self.with_write(|session| {
            let state = session.local_state()?;
            let characters = state.characters();
            if request.character_key.is_none()
                && let Some(key) = characters.creation_receipt(&request.request_id, &digest)?
            {
                return crate::characters::creation_view(&characters, &key);
            }
            session.require_reference_revision(reference_revision)?;
            let saved = if let Some(key) = &request.character_key {
                if request.party.is_some() {
                    return Err(AppServiceError::invalid_request(
                        "Refresh the character separately from party membership.",
                    ));
                }
                let expected = request.expected_character_revision.ok_or_else(|| {
                    AppServiceError::invalid_request("Review the character before refreshing it.")
                })?;
                characters.update(
                    key,
                    expected,
                    &imported.name,
                    Some(imported.level),
                    Some(&imported.reference),
                )?
            } else {
                if let Some(party) = &request.party {
                    state
                        .parties()
                        .get_at_revision(&party.party_key, party.expected_revision)?;
                }
                let saved = characters.create(
                    &imported.name,
                    Some(imported.level),
                    Some(&imported.reference),
                )?;
                crate::characters::join_party(&state, request.party.as_ref(), &saved)?;
                characters.record_creation(&request.request_id, &digest, &saved.character_key)?;
                saved
            };
            crate::characters::creation_view(&characters, &saved.character_key)
        })
    }
    pub fn character(&self, key: &str) -> AppServiceResult<PcReferenceView> {
        self.with_state(|session| {
            let character = session.local_state()?.characters().get(key)?;
            character_view(character, |keys| read_titles(session.exact_records(), keys))
        })
    }
    pub fn save_pc_ability_link(
        &self,
        key: &str,
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
                if session
                    .retrieval()
                    .get_records(GetRecordsRequest {
                        record_keys: std::slice::from_ref(&record_key),
                    })?
                    .records
                    .is_empty()
                {
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
            let characters = session.local_state()?.characters();
            let mut character =
                characters.get_at_revision(key, request.expected_character_revision)?;
            let reference = character.reference.as_mut().ok_or_else(|| {
                AppServiceError::invalid_request("This character has no imported reference.")
            })?;
            reference
                .abilities
                .iter_mut()
                .find(|a| a.key == ability_key)
                .ok_or_else(|| {
                    AppServiceError::invalid_request("This ability is no longer on the character.")
                })?
                .manual = manual;
            let saved = characters.update(
                key,
                request.expected_character_revision,
                &character.name,
                character.level,
                character.reference.as_ref(),
            )?;
            character_view(saved, |keys| read_titles(session.exact_records(), keys))
        })
    }
    pub fn encounter_pc_reference(
        &self,
        encounter: &str,
        participant: &str,
    ) -> AppServiceResult<EncounterPcReferenceView> {
        self.with_state(|session| {
            let characters = session.local_state()?.characters();
            let reference = characters
                .encounter_character(encounter, participant)?
                .map(|key| {
                    character_view(characters.get(&key)?, |keys| {
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
            let key = match &request.association {
                EncounterPcAssociation::Attach { character_key } => Some(character_key.as_str()),
                EncounterPcAssociation::Detach => None,
            };
            session
                .local_state()?
                .characters()
                .set_encounter_character(
                    encounter,
                    participant,
                    request.expected_encounter_revision,
                    key,
                )?;
            Ok(EncounterMutationView {
                encounter_key: encounter.into(),
            })
        })
    }
}
fn preserve_choices(previous: Option<PcReference>, reference: &mut PcReference) {
    if let Some(previous) = previous {
        let manual: HashMap<_, _> = previous
            .abilities
            .into_iter()
            .filter_map(|a| a.manual.map(|target| (a.key, target)))
            .collect();
        for ability in &mut reference.abilities {
            ability.manual = manual.get(&ability.key).cloned();
        }
    }
}
pub(super) fn character_view(
    character: Character,
    titles: impl FnOnce(&[RecordKey]) -> AppServiceResult<Vec<SourceRecordSummary>>,
) -> AppServiceResult<PcReferenceView> {
    let imported = character.reference.is_some();
    let abilities = if let Some(reference) = character.reference {
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
    let (sections, resolved_count, unresolved_count) = sections(&abilities);
    Ok(PcReferenceView {
        character_key: character.character_key,
        character_revision: character.revision,
        name: character.name,
        level: character.level,
        imported,
        abilities,
        sections,
        resolved_count,
        unresolved_count,
        parties: character
            .parties
            .into_iter()
            .map(|p| CharacterPartyView {
                party_key: p.party_key,
                name: p.name,
            })
            .collect(),
    })
}
fn sections(abilities: &[PcAbilityView]) -> (Vec<PcAbilitySectionView>, usize, usize) {
    let sections = [
        ("heritage", "Heritage"),
        ("special", "Features"),
        ("feat", "Feats"),
        ("focus_spell", "Focus spells"),
        ("spell", "Spells"),
    ]
    .into_iter()
    .filter_map(|(kind, label)| {
        let items: Vec<_> = abilities
            .iter()
            .filter(|a| a.kind == kind)
            .cloned()
            .collect();
        (!items.is_empty()).then_some(PcAbilitySectionView {
            kind: kind.into(),
            label: label.into(),
            abilities: items,
        })
    })
    .collect();
    let resolved = abilities
        .iter()
        .filter(|a| a.record_key.is_some() && a.record_title.is_some())
        .count();
    (sections, resolved, abilities.len() - resolved)
}

pub(super) fn require_generation(session: &WorkspaceRead, expected: &str) -> AppServiceResult<()> {
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

pub(super) fn resolve(
    retrieval: &AtlasRetrievalService,
    reference: &mut PcReference,
) -> AppServiceResult<()> {
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

pub(super) fn read_titles(
    records: Result<ExactRecordRead<'_>, RuntimeError>,
    keys: &[RecordKey],
) -> AppServiceResult<Vec<SourceRecordSummary>> {
    match records {
        Ok(records) => Ok(records.summaries(keys)?),
        Err(error) if error.reference_unavailable() => Ok(Vec::new()),
        Err(error) => Err(error.into()),
    }
}

pub(super) fn keys(reference: &PcReference) -> Vec<RecordKey> {
    let mut seen = HashSet::new();
    reference
        .abilities
        .iter()
        .filter_map(|ability| ability.record_key())
        .filter(|key| seen.insert(*key))
        .cloned()
        .collect()
}

pub(super) fn abilities_view(
    reference: PcReference,
    records: Vec<SourceRecordSummary>,
) -> Vec<PcAbilityView> {
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
