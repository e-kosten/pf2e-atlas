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
use atlas_local_state::{Character, CharacterCreationIntent, PcAbilityKind, PcReference};
use atlas_runtime::{ExactRecordRead, RuntimeError, WorkspaceRead};
use atlas_search::{AtlasRetrievalService, GetRecordsRequest, ResolveRecordRequest};
use std::collections::{BTreeMap, HashMap, HashSet};

impl AtlasAppService {
    pub fn preview_pathbuilder_pc(
        &self,
        json: &str,
        character_key: Option<&str>,
    ) -> AppServiceResult<PathbuilderPreviewView> {
        let mut character = pathbuilder::preview(json)?;
        let parsed = crate::pathbuilder_sheet::read_sheet(json)?;
        let json = character_key.map(|_| json.to_owned());
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
            let source = previous
                .as_ref()
                .map(|previous| {
                    session
                        .local_state()?
                        .characters()
                        .pathbuilder_source(&previous.character_key)
                        .map_err(AppServiceError::from)
                })
                .transpose()?
                .flatten();
            let previous_reference = previous.and_then(|character| character.reference);
            preserve_choices(previous_reference.clone(), &mut character.reference);
            resolve(session.retrieval(), &mut character.reference)?;
            let sheet = crate::character_sheet::project_sheet(
                &parsed,
                Some(&character.reference),
                Some(session.retrieval()),
            )?;
            let changes = json
                .as_deref()
                .map(|json| {
                    crate::character_sheet::compare_imports(
                        json,
                        &sheet,
                        &character.reference,
                        source.as_ref().map(|source| source.json()),
                        previous_reference.as_ref(),
                    )
                })
                .transpose()?;
            let records = session.retrieval().get_records(GetRecordsRequest {
                record_keys: &keys(&character.reference),
            })?;
            Ok(PathbuilderPreviewView {
                character_revision,
                previous_name,
                previous_level,
                name: character.name,
                level: character.level,
                artifact_generation: session.generation()?,
                sheet,
                changes,
                sections: ability_sections(character.reference, records.records),
            })
        })
    }
    pub fn save_pathbuilder_pc(
        &self,
        request: SavePathbuilderPcRequest,
    ) -> AppServiceResult<CharacterMutationView> {
        let mut imported = pathbuilder::parse(&request.json)?;
        let legacy = import_intent_digest(
            &(
                imported.name.trim(),
                imported.level,
                &request.party,
                &request.expected_artifact_generation,
                &request.character_key,
                request.expected_character_revision,
                &imported.reference,
            ),
            &request.rule_links,
        )?;
        let full = full_import_digest(&legacy, &request.json)?;
        let intent = CharacterCreationIntent::Pathbuilder {
            legacy: &legacy,
            full: &full,
        };
        if request.character_key.is_none() {
            let recovered = self.with_state(|session| {
                let characters = session.local_state()?.characters();
                characters
                    .creation_receipt(&request.request_id, intent)?
                    .map(|key| crate::characters::creation_view(&characters, &key))
                    .transpose()
            })?;
            if let Some(view) = recovered {
                return Ok(view);
            }
        }
        let source = pathbuilder::source(request.json)?;
        let expected_generation = request.expected_artifact_generation.clone();
        let refresh_key = request.character_key.clone();
        let refresh_revision = request.expected_character_revision;
        let rule_links = request.rule_links;
        let (imported, reference_revision) =
            self.with_workspace(&ReadExecutionContext::uncancellable(), move |session| {
                require_generation(session, &expected_generation)?;
                if let Some(key) = &refresh_key {
                    let expected = refresh_revision.ok_or_else(|| {
                        AppServiceError::invalid_request(
                            "Review the character before importing a new build.",
                        )
                    })?;
                    let previous = session
                        .local_state()?
                        .characters()
                        .get_at_revision(key, expected)?;
                    preserve_choices(previous.reference, &mut imported.reference);
                }
                resolve_import(session.retrieval(), &mut imported.reference, rule_links)?;
                Ok((imported, session.reference_revision()?))
            })?;
        self.with_write(|session| {
            let state = session.local_state()?;
            let characters = state.characters();
            if request.character_key.is_none()
                && let Some(key) = characters.creation_receipt(&request.request_id, intent)?
            {
                return crate::characters::creation_view(&characters, &key);
            }
            session.require_reference_revision(reference_revision)?;
            let saved = if let Some(key) = &request.character_key {
                if request.party.is_some() {
                    return Err(AppServiceError::invalid_request(
                        "Import the new build separately from party membership.",
                    ));
                }
                let expected = request.expected_character_revision.ok_or_else(|| {
                    AppServiceError::invalid_request(
                        "Review the character before importing a new build.",
                    )
                })?;
                characters.update_imported(
                    key,
                    expected,
                    &imported.name,
                    imported.level,
                    &imported.reference,
                    &source,
                )?
            } else {
                if let Some(party) = &request.party {
                    state
                        .parties()
                        .get_at_revision(&party.party_key, party.expected_revision)?;
                }
                let saved = characters.create_imported(
                    &imported.name,
                    imported.level,
                    &imported.reference,
                    &source,
                )?;
                crate::characters::join_party(&state, request.party.as_ref(), &saved)?;
                characters.record_creation(&request.request_id, intent, &saved.character_key)?;
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
                require_available_rules(session.retrieval(), std::slice::from_ref(&record_key))?;
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
            let ability = reference
                .abilities
                .iter_mut()
                .find(|a| a.key == ability_key)
                .ok_or_else(|| {
                    AppServiceError::invalid_request("This ability is no longer on the character.")
                })?;
            if manual.is_none() {
                let unavailable = || {
                    AppServiceError::invalid_request(
                        "The automatic match is no longer available. Your link was kept.",
                    )
                };
                let automatic = ability.automatic.as_ref().ok_or_else(unavailable)?;
                let records =
                    read_titles(session.exact_records(), std::slice::from_ref(automatic))?;
                if !records.iter().any(|record| record.name.value.is_some()) {
                    return Err(unavailable());
                }
            }
            ability.manual = manual;
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

pub(super) fn import_intent_digest(
    import: &impl serde::Serialize,
    rule_links: &BTreeMap<String, RecordKey>,
) -> AppServiceResult<String> {
    // Existing receipts hash the original import tuple for requests without choices.
    if rule_links.is_empty() {
        crate::characters::intent_digest(import)
    } else {
        crate::characters::intent_digest(&(import, rule_links))
    }
}

pub(super) fn full_import_digest(legacy: &str, json: &str) -> AppServiceResult<String> {
    crate::characters::intent_digest(&("pathbuilder-source/v2", legacy, json))
}

pub(super) fn resolve_import(
    retrieval: &AtlasRetrievalService,
    reference: &mut PcReference,
    mut rule_links: BTreeMap<String, RecordKey>,
) -> AppServiceResult<()> {
    if !rule_links.is_empty() {
        let ability_keys: HashSet<_> = reference
            .abilities
            .iter()
            .map(|ability| &ability.key)
            .collect();
        if rule_links.keys().any(|key| !ability_keys.contains(key)) {
            return Err(AppServiceError::invalid_request(
                "An ability is no longer in this export. Review the import and choose its links again.",
            ));
        }
        let record_keys: Vec<_> = rule_links.values().cloned().collect();
        require_available_rules(retrieval, &record_keys)?;
        for ability in &mut reference.abilities {
            if let Some(key) = rule_links.remove(&ability.key) {
                ability.manual = Some(key);
            }
        }
    }
    resolve(retrieval, reference)
}

fn require_available_rules(
    retrieval: &AtlasRetrievalService,
    record_keys: &[RecordKey],
) -> AppServiceResult<()> {
    if !retrieval
        .get_records(GetRecordsRequest { record_keys })?
        .missing
        .is_empty()
    {
        return Err(AppServiceError::invalid_request(
            "A selected rule is no longer available. Search again before linking it.",
        ));
    }
    Ok(())
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
    let sections = if let Some(reference) = character.reference {
        let keys = keys(&reference);
        let records = if keys.is_empty() {
            Vec::new()
        } else {
            titles(&keys)?
        };
        ability_sections(reference, records)
    } else {
        Vec::new()
    };
    Ok(PcReferenceView {
        character_key: character.character_key,
        character_revision: character.revision,
        name: character.name,
        level: character.level,
        imported,
        sections,
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
        .flat_map(|ability| ability.manual.iter().chain(&ability.automatic))
        .filter(|key| seen.insert(*key))
        .cloned()
        .collect()
}

fn ability_sections(
    reference: PcReference,
    records: Vec<SourceRecordSummary>,
) -> Vec<PcAbilitySectionView> {
    let titles: HashMap<_, _> = records
        .into_iter()
        .map(|record| (record.key, record.name.value))
        .collect();
    let mut sections: HashMap<_, Vec<_>> = HashMap::new();
    for ability in reference.abilities {
        let record_key = ability.record_key().cloned();
        let record_title = record_key
            .as_ref()
            .and_then(|key| titles.get(key))
            .and_then(Clone::clone);
        sections
            .entry(ability.kind)
            .or_default()
            .push(PcAbilityView {
                ability_key: ability.key,
                label: ability.label,
                choice: ability.choice,
                kind: ability.kind.as_str().into(),
                manual: ability.manual.is_some(),
                automatic_match_available: ability
                    .automatic
                    .as_ref()
                    .is_some_and(|key| titles.get(key).is_some_and(Option::is_some)),
                ambiguous: ability.ambiguous,
                record_key,
                record_title,
            });
    }
    [
        (PcAbilityKind::Heritage, "Heritage"),
        (PcAbilityKind::Special, "Features"),
        (PcAbilityKind::Feat, "Feats"),
        (PcAbilityKind::FocusSpell, "Focus spells"),
        (PcAbilityKind::Spell, "Spells"),
    ]
    .into_iter()
    .filter_map(|(kind, label)| {
        sections
            .remove(&kind)
            .map(|abilities| PcAbilitySectionView {
                kind: kind.as_str().into(),
                label: label.into(),
                abilities,
            })
    })
    .collect()
}
