//! Coherent saved character reads, sharing source decoding and sheet presentation.
pub(crate) mod facts;

use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::*;
use atlas_local_state::{Character, PathbuilderSource};
use atlas_runtime::StateRead;
use atlas_search::AtlasRetrievalService;
use facts::FactSource;
use std::collections::BTreeMap;

struct SavedCharacter {
    character: Character,
    source: Option<PathbuilderSource>,
    titles: Vec<atlas_domain::SourceRecordSummary>,
}

impl AtlasAppService {
    pub fn preview_character_import(
        &self,
        json: &str,
        character_key: Option<&str>,
        request: CharacterContextRequest,
    ) -> AppServiceResult<CharacterImportPreviewView> {
        let selected = selected_sections(&request)?;
        let candidate = crate::pathbuilder::preview(json)?;
        let candidate_reference = candidate.reference.clone();
        let json = json.to_owned();
        let key = character_key.map(str::to_owned);
        self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |session| {
                let preview = crate::pc_reference::prepare_pathbuilder_preview(
                    session,
                    &json,
                    key.as_deref(),
                    candidate,
                )?;
                let parsed = crate::pathbuilder_sheet::read_sheet(&json)?;
                let mut proposed = source_sections(
                    Some(&parsed),
                    Some(&json),
                    Some(&candidate_reference),
                    &selected,
                    Some(session.retrieval()),
                )?;
                if let Some(section) = proposed.get_mut(&CharacterSection::Abilities) {
                    section.abilities = Some(ability_catalogue(
                        preview
                            .sections
                            .iter()
                            .flat_map(|s| s.abilities.iter().cloned())
                            .collect(),
                        &request.abilities,
                    ));
                }
                let current = key
                    .as_deref()
                    .map(|key| {
                        let characters = session.local_state()?.characters();
                        let character = characters.get(key)?;
                        let source = characters.pathbuilder_source(key)?;
                        let titles = if selected.contains(&CharacterSection::Abilities) {
                            let keys = character
                                .reference
                                .as_ref()
                                .map(crate::pc_reference::keys)
                                .unwrap_or_default();
                            session
                                .retrieval()
                                .get_records(atlas_search::GetRecordsRequest {
                                    record_keys: &keys,
                                })?
                                .records
                        } else {
                            Vec::new()
                        };
                        project(
                            SavedCharacter {
                                character,
                                source,
                                titles,
                            },
                            &selected,
                            &request.abilities,
                            Some(session.retrieval()),
                        )
                    })
                    .transpose()?;
                Ok(CharacterImportPreviewView {
                    source_sha256: character_source_sha256(&json),
                    artifact_generation: preview.artifact_generation,
                    name: preview.name,
                    level: preview.level,
                    changes: preview.changes,
                    selected_sections: selected,
                    proposed,
                    current,
                })
            },
        )
    }

    /// Save the exact export reviewed by a caller, retaining existing receipt semantics.
    pub fn save_reviewed_character_import(
        &self,
        request: SavePathbuilderPcRequest,
        expected_source_sha256: &str,
    ) -> AppServiceResult<CharacterMutationView> {
        if character_source_sha256(&request.json) != expected_source_sha256 {
            return Err(AppServiceError::invalid_request(
                "The Pathbuilder export changed after preview. Preview this exact file or input again.",
            ));
        }
        self.save_pathbuilder_pc(request)
    }

    pub fn character_context(
        &self,
        reference: &str,
        request: CharacterContextRequest,
    ) -> AppServiceResult<CharacterContextView> {
        let selected = selected_sections(&request)?;
        let reference = reference.to_owned();
        self.with_saved_detail(
            |runtime, state| runtime.reference_read(state, None),
            move |state| {
                let key = resolve_character(state, &reference)?;
                let saved = load(state, &key, selected.contains(&CharacterSection::Abilities))?;
                let catalog = saved.source.is_some() && needs_catalog(&selected);
                Ok(((saved, selected, request.abilities), catalog))
            },
            |(saved, selected, filter), reference| {
                let retrieval = optional_retrieval(reference)?;
                project(saved, &selected, &filter, retrieval)
            },
        )
    }

    pub fn party_roster(&self, reference: &str) -> AppServiceResult<PartyView> {
        self.with_state(|state| {
            let key = resolve_party(state, reference)?;
            Ok(crate::parties::party_view(
                state.local_state()?.parties().get(&key)?,
            ))
        })
    }

    pub fn party_context(
        &self,
        reference: &str,
        request: CharacterContextRequest,
    ) -> AppServiceResult<PartyContextView> {
        let selected = selected_sections(&request)?;
        let reference = reference.to_owned();
        self.with_saved_detail(
            |runtime, state| runtime.reference_read(state, None),
            move |state| {
                let key = resolve_party(state, &reference)?;
                let party = state.local_state()?.parties().get(&key)?;
                let members = party
                    .members
                    .iter()
                    .map(|m| {
                        load(
                            state,
                            &m.character_key,
                            selected.contains(&CharacterSection::Abilities),
                        )
                    })
                    .collect::<AppServiceResult<Vec<_>>>()?;
                let catalog =
                    members.iter().any(|m| m.source.is_some()) && needs_catalog(&selected);
                Ok(((party, members, selected, request.abilities), catalog))
            },
            |(party, members, selected, filter), reference| {
                let retrieval = optional_retrieval(reference)?;
                let members = members
                    .into_iter()
                    .map(|m| project(m, &selected, &filter, retrieval))
                    .collect::<AppServiceResult<_>>()?;
                Ok(PartyContextView {
                    party_key: party.party_key,
                    name: party.name,
                    revision: party.revision,
                    selected_sections: selected,
                    members,
                })
            },
        )
    }

    pub fn party_characters(
        &self,
        reference: Option<&str>,
    ) -> AppServiceResult<CharacterIndexView> {
        self.with_state(|state| {
            let party = reference.map(|r| resolve_party(state, r)).transpose()?;
            let characters = state
                .local_state()?
                .characters()
                .list()?
                .into_iter()
                .filter(|c| {
                    party
                        .as_ref()
                        .is_none_or(|key| c.parties.iter().any(|p| &p.party_key == key))
                })
                .map(crate::characters::character_summary)
                .collect();
            Ok(CharacterIndexView { characters })
        })
    }
}

pub fn character_source_sha256(source: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(source.as_bytes()))
}

fn optional_retrieval(
    reference: Result<Option<&mut atlas_runtime::WorkspaceRead>, atlas_runtime::RuntimeError>,
) -> AppServiceResult<Option<&AtlasRetrievalService>> {
    match reference {
        Ok(reference) => Ok(reference.map(|r| r.retrieval())),
        Err(error) if error.reference_unavailable() => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn needs_catalog(selected: &[CharacterSection]) -> bool {
    selected.iter().any(|s| {
        matches!(
            s,
            CharacterSection::Identity
                | CharacterSection::Equipment
                | CharacterSection::Crafting
                | CharacterSection::Statistics
        )
    })
}

fn load(state: &StateRead, key: &str, abilities: bool) -> AppServiceResult<SavedCharacter> {
    let characters = state.local_state()?.characters();
    let character = characters.get(key)?;
    let source = characters.pathbuilder_source(key)?;
    let titles = if abilities {
        let keys = character
            .reference
            .as_ref()
            .map(crate::pc_reference::keys)
            .unwrap_or_default();
        if keys.is_empty() {
            Vec::new()
        } else {
            crate::pc_reference::read_titles(state.exact_records(), &keys)?
        }
    } else {
        Vec::new()
    };
    Ok(SavedCharacter {
        character,
        source,
        titles,
    })
}

pub(crate) fn selected_sections(
    request: &CharacterContextRequest,
) -> AppServiceResult<Vec<CharacterSection>> {
    let mut selected = if request.sections.is_empty() {
        CharacterSection::INTERPRETED.to_vec()
    } else {
        request.sections.clone()
    };
    selected.sort();
    selected.dedup();
    let filter = &request.abilities;
    if (filter.needs_review || filter.kind.is_some() || filter.query.is_some())
        && selected != [CharacterSection::Abilities]
    {
        return Err(AppServiceError::invalid_request(
            "Ability filters require only --section abilities.",
        ));
    }
    if filter.kind.as_deref().is_some_and(|kind| {
        !["feat", "heritage", "special", "spell", "focus_spell"].contains(&kind)
    }) {
        return Err(AppServiceError::invalid_request(
            "Unknown ability kind. Use feat, heritage, special, spell or focus_spell.",
        ));
    }
    Ok(selected)
}

fn project(
    saved: SavedCharacter,
    selected: &[CharacterSection],
    filter: &CharacterAbilityFilter,
    retrieval: Option<&AtlasRetrievalService>,
) -> AppServiceResult<CharacterContextView> {
    let SavedCharacter {
        character,
        source,
        titles,
    } = saved;
    let parsed = source
        .as_ref()
        .map(|s| crate::pathbuilder_sheet::read_sheet(s.json()))
        .transpose()?;
    let source_state = if source.is_some() {
        CharacterSourceState::Available
    } else if character.reference.is_some() {
        CharacterSourceState::ReferenceOnly
    } else {
        CharacterSourceState::NoImport
    };
    let imported_identity = parsed.as_ref().map(|s| ImportedCharacterIdentity {
        name: s.name.clone(),
        level: s.level,
    });
    let mut sections = source_sections(
        parsed.as_ref(),
        source.as_ref().map(PathbuilderSource::json),
        character.reference.as_ref(),
        selected,
        retrieval,
    )?;
    if let Some(section) = sections.get_mut(&CharacterSection::Abilities) {
        let entries = character
            .reference
            .as_ref()
            .map(|r| crate::pc_reference::ability_sections(r.clone(), titles))
            .unwrap_or_default()
            .into_iter()
            .flat_map(|s| s.abilities)
            .collect();
        section.abilities = Some(ability_catalogue(entries, filter));
    }
    let metadata = CharacterSummaryView {
        character_key: character.character_key,
        name: character.name,
        level: character.level,
        revision: character.revision,
        imported: character.reference.is_some(),
        parties: character
            .parties
            .into_iter()
            .map(|p| CharacterPartyView {
                party_key: p.party_key,
                name: p.name,
            })
            .collect(),
    };
    Ok(CharacterContextView {
        character: metadata,
        source_state,
        imported_identity,
        selected_sections: selected.to_vec(),
        sections,
    })
}

pub(crate) fn source_sections(
    parsed: Option<&crate::pathbuilder_sheet::PathbuilderSheet>,
    source: Option<&str>,
    reference: Option<&atlas_local_state::PcReference>,
    selected: &[CharacterSection],
    retrieval: Option<&AtlasRetrievalService>,
) -> AppServiceResult<BTreeMap<CharacterSection, CharacterSectionView>> {
    let all = parsed.map(FactSource::fact_value);
    let all = match &all {
        Some(ImportedFactValue::Object(fields)) => Some(fields),
        _ => None,
    };
    let mut presentation = parsed
        .map(|s| crate::character_sheet::context_groups(s, reference, retrieval, selected))
        .transpose()?
        .unwrap_or_default();
    selected
        .iter()
        .map(|kind| {
            let fields = section_fields(*kind)
                .iter()
                .map(|name| {
                    let value = all
                        .and_then(|a| a.get(*name))
                        .cloned()
                        .unwrap_or(ImportedFactView::Missing);
                    ((*name).to_owned(), value)
                })
                .collect();
            let presentation = presentation.remove(kind).unwrap_or_default();
            let record_links = presentation
                .iter()
                .flat_map(|group| &group.rows)
                .filter_map(|row| {
                    row.record_key.as_ref().map(|key| RecordTitleLinkView {
                        record_key: key.to_string(),
                        title: row.label.clone(),
                    })
                })
                .collect();
            Ok((
                *kind,
                CharacterSectionView {
                    fields,
                    abilities: None,
                    source_json: if *kind == CharacterSection::Source {
                        source.map(str::to_owned)
                    } else {
                        None
                    },
                    record_links,
                    presentation,
                },
            ))
        })
        .collect()
}

fn section_fields(section: CharacterSection) -> &'static [&'static str] {
    use CharacterSection::*;
    match section {
        Identity => &[
            "name",
            "level",
            "class",
            "xp",
            "ancestry",
            "heritage",
            "background",
            "alignment",
            "gender",
            "age",
            "deity",
            "size",
            "size_name",
            "key_ability",
            "languages",
        ],
        Statistics => &["abilities", "attributes", "resistances", "mods", "ac_total"],
        Proficiencies => &["proficiencies", "specific_proficiencies", "lores"],
        CharacterOptions => &["feats", "specials"],
        Spellcasting => &["spell_casters", "focus_points", "focus"],
        Equipment => &["equipment", "equipment_containers", "weapons", "armor"],
        Money => &["money"],
        Crafting => &["formula"],
        Companions => &["pets"],
        Abilities | Source => &[],
    }
}

fn resolve_character(state: &StateRead, reference: &str) -> AppServiceResult<String> {
    let rows = state.local_state()?.characters().list()?;
    if let Some(row) = rows.iter().find(|r| r.character_key == reference) {
        return Ok(row.character_key.clone());
    }
    let candidates: Vec<_> = rows.iter().filter(|r| r.name == reference.trim()).collect();
    match candidates.as_slice() {
        [row] => Ok(row.character_key.clone()),
        [] => Err(atlas_local_state::LocalStateError::CharacterNotFound(reference.into()).into()),
        _ => Err(ambiguous(
            "character",
            candidates.iter().map(|r| (&r.character_key, &r.name)),
        )),
    }
}

fn resolve_party(state: &StateRead, reference: &str) -> AppServiceResult<String> {
    let rows = state.local_state()?.parties().list()?;
    if let Some(row) = rows.iter().find(|r| r.party_key == reference) {
        return Ok(row.party_key.clone());
    }
    let candidates: Vec<_> = rows.iter().filter(|r| r.name == reference.trim()).collect();
    match candidates.as_slice() {
        [row] => Ok(row.party_key.clone()),
        [] => Err(atlas_local_state::LocalStateError::PartyNotFound(reference.into()).into()),
        _ => Err(ambiguous(
            "party",
            candidates.iter().map(|r| (&r.party_key, &r.name)),
        )),
    }
}

fn ambiguous<'a>(
    kind: &str,
    rows: impl Iterator<Item = (&'a String, &'a String)>,
) -> AppServiceError {
    let candidates: Vec<_> = rows
        .map(|(key, name)| serde_json::json!({"key":key,"name":name}))
        .collect();
    AppError::new(
        AppErrorCode::InvalidRequest,
        format!("More than one {kind} has this name. Use a returned key."),
    )
    .with_details(serde_json::json!({"candidates":candidates}))
    .into()
}

fn ability_catalogue(
    entries: Vec<PcAbilityView>,
    filter: &CharacterAbilityFilter,
) -> CharacterAbilitiesView {
    let total = entries.len();
    let query = filter.query.as_deref().unwrap_or("").trim().to_lowercase();
    let entries: Vec<_> = entries
        .into_iter()
        .map(|ability| {
            let link_state = if ability.record_key.is_some() {
                if ability.record_title.is_none() {
                    AbilityLinkState::Unavailable
                } else if ability.manual {
                    AbilityLinkState::Manual
                } else {
                    AbilityLinkState::Automatic
                }
            } else if ability.ambiguous {
                AbilityLinkState::Ambiguous
            } else {
                AbilityLinkState::Unlinked
            };
            CharacterAbilityView {
                ability,
                link_state,
            }
        })
        .filter(|entry| {
            let a = &entry.ability;
            (!filter.needs_review
                || matches!(
                    entry.link_state,
                    AbilityLinkState::Unlinked
                        | AbilityLinkState::Ambiguous
                        | AbilityLinkState::Unavailable
                ))
                && filter.kind.as_ref().is_none_or(|kind| kind == &a.kind)
                && (query.is_empty()
                    || [
                        a.label.as_str(),
                        a.choice.as_deref().unwrap_or(""),
                        a.record_title.as_deref().unwrap_or(""),
                    ]
                    .iter()
                    .any(|text| text.to_lowercase().contains(&query)))
        })
        .collect();
    CharacterAbilitiesView {
        total,
        shown: entries.len(),
        filter: filter.clone(),
        entries,
    }
}
