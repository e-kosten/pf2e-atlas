use std::collections::BTreeMap;

use atlas_record::{
    ContentDiagnostic, ContentDiagnosticKind, ContentExclusion, ContentExclusionReason, ContentId,
    ContentKey, ContentOrigin, ContentOwner, ContentProvenance, ContentRole, CreatureEntityFamily,
    CreatureEntityTarget, CreatureOccurrenceId, CreatureSourceId, DuplicateContentStatus,
    OwnedRichContent, OwnedRichContentDocument, RecordBody,
};

use crate::records::{LoadedSourceRecord, SourceContentFact};

use super::npc_entities::family;

pub(crate) fn finalize_npc_owned_content(records: &mut [LoadedSourceRecord]) {
    for loaded in records {
        let Some(RecordBody::Creature(creature)) = &mut loaded.facts.canonical_body else {
            continue;
        };
        let source_content = loaded.facts.source_facts.content_sources.clone();
        let family_by_source_id = loaded
            .facts
            .npc_embedded_candidates
            .as_ref()
            .and_then(|candidates| candidates.items.as_value())
            .map(|candidates| {
                candidates
                    .iter()
                    .map(|candidate| {
                        (
                            candidate.nested_source_id.clone(),
                            family(&candidate.source),
                        )
                    })
                    .collect::<BTreeMap<_, _>>()
            })
            .unwrap_or_default();
        let embedded = creature.embedded_entities.value.as_value();
        let mut content = OwnedRichContent::default();

        for source in source_content {
            let content_key = match ContentKey::new(source.content_key.clone()) {
                Ok(key) => key,
                Err(_) => continue,
            };
            let content_id = ContentId::new(creature.identity.record_key.clone(), content_key);
            if source.source_kind.is_embedded() {
                attach_embedded_content(
                    &creature.identity.record_key,
                    &loaded.record.provenance.source_path,
                    embedded,
                    &family_by_source_id,
                    source,
                    content_id,
                    &mut content,
                );
            } else {
                let diagnostics = source_diagnostics(&source);
                content.documents.push(OwnedRichContentDocument::new(
                    content_id,
                    source.identity_stability,
                    ContentOwner::Record(creature.identity.record_key.clone()),
                    record_role(source.source_kind),
                    ContentOrigin::RecordField {
                        source_kind: source.source_kind,
                        relative_source_path: source.relative_source_path.clone(),
                    },
                    source.source_kind.default_visibility(),
                    provenance(
                        &creature.identity.record_key,
                        &loaded.record.provenance.source_path,
                        &source,
                    ),
                    source.source_kind,
                    source.authored_order,
                    source.label,
                    source.document,
                    DuplicateContentStatus::Unique,
                    diagnostics,
                ));
            }
        }

        content
            .documents
            .sort_by_key(|document| document.authored_order);
        creature.content = content;
    }
}

fn attach_embedded_content(
    record_key: &atlas_domain::RecordKey,
    source_record_path: &str,
    embedded: Option<&atlas_record::CreatureEmbeddedEntities>,
    family_by_source_id: &BTreeMap<String, CreatureEntityFamily>,
    source: SourceContentFact,
    content_id: ContentId,
    content: &mut OwnedRichContent,
) {
    let source_id = source.nested_source_id.as_deref().unwrap_or_default();
    let Some(family) = family_by_source_id.get(source_id).copied() else {
        content.exclusions.push(ContentExclusion {
            parent_record_key: record_key.clone(),
            content_key: content_id.content_key,
            relative_source_path: source.relative_source_path,
            label: source.label,
            reason: ContentExclusionReason::DeferredEntityFamily,
        });
        return;
    };
    let occurrence_id = CreatureSourceId::new(source_id.to_string())
        .ok()
        .map(|source_id| CreatureOccurrenceId::stable_nested(record_key, family, &source_id));
    let occurrence = occurrence_id.as_ref().and_then(|id| {
        embedded.and_then(|embedded| {
            embedded
                .occurrences
                .iter()
                .find(|occurrence| &occurrence.id == id)
        })
    });
    let Some(occurrence) = occurrence else {
        content.exclusions.push(ContentExclusion {
            parent_record_key: record_key.clone(),
            content_key: content_id.content_key,
            relative_source_path: source.relative_source_path,
            label: source.label,
            reason: ContentExclusionReason::MissingTypedOwner,
        });
        return;
    };
    let (owner, duplicate_status) = match &occurrence.target {
        CreatureEntityTarget::CanonicalRecord(target_record_key) => (
            ContentOwner::CreatureOccurrence(occurrence.id.clone()),
            DuplicateContentStatus::CopiedFromCanonicalTarget {
                target_record_key: target_record_key.clone(),
            },
        ),
        CreatureEntityTarget::ActorOwned(entity_id) => (
            ContentOwner::CreatureEntity(entity_id.clone()),
            DuplicateContentStatus::Unique,
        ),
    };
    let diagnostics = source_diagnostics(&source);
    content.documents.push(OwnedRichContentDocument::new(
        content_id,
        source.identity_stability,
        owner,
        ContentRole::EmbeddedCapability,
        ContentOrigin::EmbeddedEntityField {
            family,
            nested_source_id: source.nested_source_id.clone(),
            relative_source_path: source.relative_source_path.clone(),
        },
        source.source_kind.default_visibility(),
        provenance(record_key, source_record_path, &source),
        source.source_kind,
        source.authored_order,
        source.label,
        source.document,
        duplicate_status,
        diagnostics,
    ));
}

fn record_role(source_kind: atlas_record::ContentSourceKind) -> ContentRole {
    match source_kind {
        atlas_record::ContentSourceKind::Description
        | atlas_record::ContentSourceKind::DetailsFieldDescription => {
            ContentRole::PrimaryDescription
        }
        atlas_record::ContentSourceKind::Blurb => ContentRole::Summary,
        atlas_record::ContentSourceKind::GeneratedAffliction => ContentRole::GeneratedNarrative,
        _ => ContentRole::SupplementalRules,
    }
}

fn provenance(
    record_key: &atlas_domain::RecordKey,
    source_path: &str,
    source: &SourceContentFact,
) -> ContentProvenance {
    ContentProvenance {
        source_record_key: record_key.clone(),
        relative_source_path: source_path.to_string(),
        field_or_pointer_family: source.relative_source_path.clone(),
        nested_source_id: source.nested_source_id.clone(),
        authored_ordinal_or_range: source.authored_ordinal_or_range.clone(),
        authored_label: source.label.clone(),
    }
}

fn source_diagnostics(source: &SourceContentFact) -> Vec<ContentDiagnostic> {
    let mut diagnostics = source
        .diagnostics
        .unsupported_tags
        .iter()
        .map(|tag| ContentDiagnostic {
            kind: ContentDiagnosticKind::UnsupportedTag,
            subject: tag.clone(),
            detail: None,
        })
        .chain(
            source
                .diagnostics
                .unsupported_attributes
                .iter()
                .map(|attribute| ContentDiagnostic {
                    kind: ContentDiagnosticKind::UnsupportedAttribute,
                    subject: attribute.name.clone(),
                    detail: Some(attribute.tag.clone()),
                }),
        )
        .chain(
            source
                .diagnostics
                .unknown_macros
                .iter()
                .map(|name| ContentDiagnostic {
                    kind: ContentDiagnosticKind::UnknownFoundryMacro,
                    subject: name.clone(),
                    detail: None,
                }),
        )
        .collect::<Vec<_>>();
    if source.identity_stability == atlas_record::ContentIdentityStability::UnstableAuthoredOrdinal
    {
        diagnostics.push(ContentDiagnostic {
            kind: ContentDiagnosticKind::UnstableIdentity,
            subject: source.content_key.clone(),
            detail: source.authored_ordinal_or_range.clone(),
        });
    }
    diagnostics
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::Path;

    use atlas_domain::PackName;
    use atlas_record::{
        ContentDiagnosticKind, ContentIdentityStability, ContentOrigin, ContentOwner, ContentRole,
        ContentSourceKind, ContentVisibility, CreatureEntityFamily, DuplicateContentStatus,
        RecordBody, render_plain_text,
    };
    use serde_json::{Value, json};

    use super::finalize_npc_owned_content;
    use crate::records::LoadedSourceRecord;
    use crate::records::references::{build_record_reference_index, resolve_content_references};
    use crate::source::ManifestPack;
    use crate::source::normalize::normalize_record;
    use crate::source::npc_entities::finalize_npc_embedded_entities;

    #[test]
    fn canonical_target_prose_is_occurrence_owned_without_mutating_spell_truth() {
        let actor = normalize(
            "Actor",
            "bestiary",
            "actor.json",
            actor_fixture(vec![
                action(
                    "action-1",
                    "Nightmare Touch",
                    None,
                    "<aside data-rule=\"unknown\">@Mystery[kept] @UUID[Compendium.pf2e.spells.Item.missing]{Missing}</aside>",
                ),
                spell(
                    "spell-1",
                    "Fireball",
                    Some("Compendium.pf2e.spells.Item.fireball"),
                    "<p>Actor-local Fireball wording.</p>",
                ),
            ]),
        );
        let spell_record = normalize(
            "Item",
            "spells",
            "fireball.json",
            json!({
                "_id": "fireball",
                "name": "Fireball",
                "type": "spell",
                "system": {
                    "traits": {"rarity": "common", "value": ["fire"]},
                    "description": {"value": "<p>Canonical Fireball wording.</p>"}
                }
            }),
        );
        let mut records = vec![actor, spell_record];
        let index = build_record_reference_index(&records);
        finalize_npc_embedded_entities(&mut records, &index);
        finalize_npc_owned_content(&mut records);
        resolve_content_references(&mut records, &index);

        let RecordBody::Creature(creature) = records[0]
            .facts
            .canonical_body
            .as_ref()
            .expect("creature body");
        let action_content = creature
            .content
            .documents
            .iter()
            .find(|content| content.id.content_key.as_str() == "item:action-1:description")
            .expect("action content");
        assert!(matches!(
            action_content.owner,
            ContentOwner::CreatureEntity(_)
        ));
        assert_eq!(
            action_content.duplicate_status,
            DuplicateContentStatus::Unique
        );
        assert!(
            action_content
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.kind == ContentDiagnosticKind::UnsupportedTag })
        );
        assert!(
            action_content.diagnostics.iter().any(|diagnostic| {
                diagnostic.kind == ContentDiagnosticKind::UnsupportedAttribute
            })
        );
        assert!(
            action_content.diagnostics.iter().any(|diagnostic| {
                diagnostic.kind == ContentDiagnosticKind::UnknownFoundryMacro
            })
        );
        assert!(
            action_content
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.kind == ContentDiagnosticKind::UnresolvedLink })
        );
        assert_eq!(action_content.reference_occurrences.len(), 1);

        let spell_content = creature
            .content
            .documents
            .iter()
            .find(|content| content.id.content_key.as_str() == "item:spell-1:description")
            .expect("spell content");
        assert!(matches!(
            spell_content.owner,
            ContentOwner::CreatureOccurrence(_)
        ));
        assert!(matches!(
            &spell_content.duplicate_status,
            DuplicateContentStatus::CopiedFromCanonicalTarget { target_record_key }
                if target_record_key.to_string() == "spells:fireball"
        ));
        assert_eq!(
            render_plain_text(&spell_content.document),
            "Actor-local Fireball wording."
        );
        assert_eq!(
            render_plain_text(&records[1].record.content.documents[0].document),
            "Canonical Fireball wording."
        );
        assert!(
            records[0]
                .record
                .content
                .documents
                .iter()
                .all(|content| !content.source_kind.is_embedded())
        );
    }

    #[test]
    fn visibility_is_retained_data_and_source_ids_survive_reordering() {
        let items = vec![
            action("action-a", "First", None, "<p>First body.</p>"),
            action("action-b", "Second", None, "<p>Second body.</p>"),
        ];
        let mut reordered = items.clone();
        reordered.swap(0, 1);
        let first = finalized_actor(actor_fixture(items));
        let second = finalized_actor(actor_fixture(reordered));
        let first_ids = content_ids(&first);
        let second_ids = content_ids(&second);

        assert_eq!(first_ids, second_ids);
        assert!(first.content.documents.iter().any(|content| {
            content.visibility == ContentVisibility::GmOnly
                && matches!(content.owner, ContentOwner::Record(_))
        }));
        assert!(first.content.documents.iter().any(|content| {
            content.visibility == ContentVisibility::Private
                && matches!(content.owner, ContentOwner::Record(_))
        }));
    }

    #[test]
    fn pinned_night_hag_has_one_owner_or_exclusion_per_authored_content_source() {
        let Some(root) = std::env::var_os("PF2E_SOURCE_ROOT") else {
            return;
        };
        let relative = "packs/pathfinder-bestiary/night-hag.json";
        let raw: Value = serde_json::from_str(
            &std::fs::read_to_string(Path::new(&root).join(relative)).expect("Night Hag source"),
        )
        .expect("Night Hag JSON");
        let loaded = normalize("Actor", "pathfinder-bestiary", relative, raw);
        let source_count = loaded.facts.source_facts.content_sources.len();
        let embedded_source_count = loaded
            .facts
            .source_facts
            .content_sources
            .iter()
            .filter(|source| source.source_kind.is_embedded())
            .count();
        let mut records = vec![loaded];
        let index = build_record_reference_index(&records);
        finalize_npc_embedded_entities(&mut records, &index);
        finalize_npc_owned_content(&mut records);
        resolve_content_references(&mut records, &index);

        let RecordBody::Creature(creature) = records[0]
            .facts
            .canonical_body
            .as_ref()
            .expect("Night Hag creature body");
        assert_eq!(source_count, 36);
        assert_eq!(embedded_source_count, 35);
        assert_eq!(creature.content.documents.len(), 36);
        assert!(creature.content.exclusions.is_empty());
        assert_eq!(
            creature.content.documents.len() + creature.content.exclusions.len(),
            source_count
        );
        assert_eq!(
            creature
                .content
                .documents
                .iter()
                .map(|content| content.id.content_key.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            creature.content.documents.len()
        );
        assert!(
            records[0]
                .record
                .content
                .documents
                .iter()
                .all(|content| !content.source_kind.is_embedded())
        );
    }

    #[test]
    fn pinned_blackfingers_backpack_gm_description_has_a_typed_owner() {
        let Some(root) = std::env::var_os("PF2E_SOURCE_ROOT") else {
            return;
        };
        let relative = "packs/curtain-call-bestiary/book-3-bring-the-house-down/blackfingers.json";
        let raw: Value = serde_json::from_str(
            &std::fs::read_to_string(Path::new(&root).join(relative)).expect("Blackfingers source"),
        )
        .expect("Blackfingers JSON");
        let mut records = vec![normalize("Actor", "curtain-call-bestiary", relative, raw)];
        let index = build_record_reference_index(&records);
        finalize_npc_embedded_entities(&mut records, &index);
        finalize_npc_owned_content(&mut records);
        resolve_content_references(&mut records, &index);

        let RecordBody::Creature(creature) = records[0]
            .facts
            .canonical_body
            .as_ref()
            .expect("Blackfingers creature body");
        let content = creature
            .content
            .documents
            .iter()
            .find(|content| {
                content.id.content_key.as_str() == "item:dHvX3fZWtSUdBdR6:gm-description"
            })
            .expect("later-family backpack GM description");
        assert_eq!(
            content.identity_stability,
            ContentIdentityStability::StableSourceIdentity
        );
        assert!(matches!(
            content.owner,
            ContentOwner::CreatureEntity(_) | ContentOwner::CreatureOccurrence(_)
        ));
        assert_eq!(content.role, ContentRole::EmbeddedCapability);
        assert_eq!(content.visibility, ContentVisibility::GmOnly);
        assert_eq!(
            content.source_kind,
            ContentSourceKind::EmbeddedGmDescription
        );
        assert!(matches!(
            content.origin,
            ContentOrigin::EmbeddedEntityField {
                family: CreatureEntityFamily::Backpack,
                ..
            }
        ));
        assert_eq!(content.provenance.relative_source_path, relative);
        assert_eq!(
            content.provenance.field_or_pointer_family,
            "$.items[_id=dHvX3fZWtSUdBdR6].system.description.gm"
        );
        assert_eq!(
            content.provenance.nested_source_id.as_deref(),
            Some("dHvX3fZWtSUdBdR6")
        );
        assert!(creature.content.exclusions.iter().all(|exclusion| {
            exclusion.content_key.as_str() != "item:dHvX3fZWtSUdBdR6:gm-description"
        }));
    }

    #[test]
    fn pinned_corpus_retains_all_embedded_gm_descriptions_as_owned_content() {
        let Some(source_root) = std::env::var_os("PF2E_SOURCE_ROOT") else {
            return;
        };
        let mut loaded =
            super::super::loader::load_foundry_source_records(Path::new(&source_root), None)
                .expect("exact pinned source loads");
        assert_eq!(loaded.source_record_count, 25_641);
        assert!(loaded.skipped_records.is_empty());
        assert_eq!(
            loaded.source_signature,
            "foundry-pf2e:sha256:dd78d67f5b6d25bf65e30ca4da66af76e7a31e1e7d990562f139154b1752603a"
        );
        let source_count = loaded
            .records
            .iter()
            .flat_map(|record| &record.facts.source_facts.content_sources)
            .filter(|source| source.source_kind == ContentSourceKind::EmbeddedGmDescription)
            .count();
        assert_eq!(source_count, 70);

        let index = build_record_reference_index(&loaded.records);
        finalize_npc_embedded_entities(&mut loaded.records, &index);
        finalize_npc_owned_content(&mut loaded.records);
        resolve_content_references(&mut loaded.records, &index);

        let documents = loaded
            .records
            .iter()
            .filter_map(|record| record.facts.canonical_body.as_ref())
            .map(|body| match body {
                RecordBody::Creature(creature) => &creature.content,
            })
            .flat_map(|content| &content.documents)
            .filter(|content| content.source_kind == ContentSourceKind::EmbeddedGmDescription)
            .collect::<Vec<_>>();
        assert_eq!(documents.len(), 70);
        assert_eq!(
            documents
                .iter()
                .map(|content| content.id.parent_record_key.to_string())
                .collect::<BTreeSet<_>>()
                .len(),
            61
        );
        assert!(documents.iter().all(|content| {
            content.identity_stability == ContentIdentityStability::StableSourceIdentity
                && content.role == ContentRole::EmbeddedCapability
                && content.visibility == ContentVisibility::GmOnly
                && matches!(
                    content.owner,
                    ContentOwner::CreatureEntity(_) | ContentOwner::CreatureOccurrence(_)
                )
                && matches!(content.origin, ContentOrigin::EmbeddedEntityField { .. })
                && content
                    .provenance
                    .field_or_pointer_family
                    .ends_with(".system.description.gm")
                && content.reference_occurrences.iter().all(|occurrence| {
                    occurrence.owner == content.owner
                        && occurrence.role == content.role
                        && occurrence.origin == content.origin
                        && occurrence.visibility == content.visibility
                        && occurrence.provenance == content.provenance
                })
        }));
        let gm_exclusions = loaded
            .records
            .iter()
            .filter_map(|record| record.facts.canonical_body.as_ref())
            .map(|body| match body {
                RecordBody::Creature(creature) => &creature.content,
            })
            .flat_map(|content| &content.exclusions)
            .filter(|exclusion| {
                exclusion
                    .relative_source_path
                    .ends_with(".system.description.gm")
            })
            .count();
        assert_eq!(gm_exclusions, 0);
    }

    fn finalized_actor(raw: Value) -> atlas_record::CreatureRecord {
        let loaded = normalize("Actor", "bestiary", "actor.json", raw);
        let mut records = vec![loaded];
        let index = build_record_reference_index(&records);
        finalize_npc_embedded_entities(&mut records, &index);
        finalize_npc_owned_content(&mut records);
        let RecordBody::Creature(creature) = records
            .remove(0)
            .facts
            .canonical_body
            .expect("creature body");
        creature
    }

    fn content_ids(creature: &atlas_record::CreatureRecord) -> BTreeSet<String> {
        creature
            .content
            .documents
            .iter()
            .map(|content| format!("{}:{:?}", content.id.content_key.as_str(), content.owner))
            .collect()
    }

    fn normalize(document_type: &str, pack: &str, path: &str, raw: Value) -> LoadedSourceRecord {
        normalize_record(
            &ManifestPack {
                name: pack.to_string(),
                label: pack.to_string(),
                document_type: document_type.to_string(),
                path: format!("packs/{pack}"),
            },
            &PackName::new(pack.to_string()).expect("pack name"),
            Path::new(path),
            Path::new("."),
            raw,
            None,
        )
        .expect("record normalizes")
    }

    fn actor_fixture(items: Vec<Value>) -> Value {
        json!({
            "_id": "actor",
            "name": "Owned Content Actor",
            "type": "npc",
            "items": items,
            "system": {
                "description": {
                    "value": "<h2>Lore</h2><p>Actor lore.</p>",
                    "gm": "<p>GM-only classification remains retained.</p>"
                },
                "details": {
                    "privateNotes": "<p>Private classification remains retained.</p>"
                },
                "traits": {"rarity": "common", "value": []}
            }
        })
    }

    fn action(id: &str, name: &str, source: Option<&str>, description: &str) -> Value {
        json!({
            "_id": id,
            "name": name,
            "type": "action",
            "_stats": {"compendiumSource": source},
            "system": {
                "actionType": {"value": "action"},
                "actions": {"value": 1},
                "description": {"value": description}
            }
        })
    }

    fn spell(id: &str, name: &str, source: Option<&str>, description: &str) -> Value {
        json!({
            "_id": id,
            "name": name,
            "type": "spell",
            "_stats": {"compendiumSource": source},
            "system": {
                "level": {"value": 3},
                "description": {"value": description}
            }
        })
    }
}
