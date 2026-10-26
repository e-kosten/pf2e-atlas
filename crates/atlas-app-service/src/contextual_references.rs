//! Verified contextual reading is separate from an authored reference target.
use atlas_app_model::{RecordBodyView, RecordDetailView};
use atlas_domain::RecordKey;
use atlas_record::source_content::{ContentReferenceResolution, ContentReferenceTarget};
use atlas_search::{AtlasRetrievalService, SourceRelationshipDirection, SourceRelationshipRequest};

pub(crate) fn attach_consumable_spell(
    view: &mut RecordDetailView,
    retrieval: &AtlasRetrievalService,
) -> crate::AppServiceResult<()> {
    let RecordBodyView::PhysicalReference(physical) = &mut view.presentation.body else {
        return Ok(());
    };
    let Some(consumable) = &mut physical.consumable else {
        return Ok(());
    };
    let Some(spell) = &consumable.spell.value else {
        return Ok(());
    };
    let key = RecordKey::parse(&spell.navigation.record_key)
        .map_err(|e| crate::AppServiceError::invalid_request(e.to_string()))?;
    let provenance = retrieval.read_non_content_relationships(&SourceRelationshipRequest {
        record: key,
        owners: Some(spell.navigation.owners.clone()),
        field: None,
        direction: SourceRelationshipDirection::Outgoing,
        limit: 1024,
    })?;
    if provenance.truncated {
        return Ok(());
    }
    let mut canonical = None;
    for edge in provenance
        .occurrences
        .iter()
        .filter(|edge| edge.kind == "CompendiumProvenance")
    {
        // Conflicting or unresolved explicit provenance must not choose a winner.
        let ContentReferenceResolution::Resolved(ContentReferenceTarget::Record { key }) =
            &edge.resolution
        else {
            return Ok(());
        };
        if canonical.as_ref().is_some_and(|prior| prior != key) {
            return Ok(());
        }
        canonical = Some(key.clone());
    }
    let Some(canonical) = canonical else {
        return Ok(());
    };
    for edge in &mut view.relationships {
        if edge.origin == "content"
            && edge.source.owners == view.selected.owners
            && edge.target.as_ref().is_some_and(|target| {
                target.record_key == canonical.to_string() && target.owners.is_empty()
            })
        {
            edge.contextual_target = Some(spell.navigation.clone());
            // Only selected visible prepared fields may replace the explicit Spell row.
            if view.presentation.content.iter().any(|field| {
                field.locator == edge.source
                    && matches!(
                        field.body,
                        atlas_app_model::PreparedFieldBodyView::Html { .. }
                    )
            }) {
                consumable.spell_presented_inline = true;
            }
        }
    }
    Ok(())
}

/// Preserve a selected casting rank on exact effect links without replacing identity.
pub(crate) fn attach_ranked_effects(
    view: &mut RecordDetailView,
    retrieval: &AtlasRetrievalService,
) -> crate::AppServiceResult<()> {
    use atlas_record::source_record::{ItemSourceView, SourceNodeView};
    let Some(rank) = view.selected.spell_rank else {
        return Ok(());
    };
    let from_spell = matches!(view.presentation.body, RecordBodyView::Spell(_));
    let from_effect =
        matches!(&view.presentation.body, RecordBodyView::Activity(a) if a.spell_effect.is_some());
    if !from_spell && !from_effect {
        return Ok(());
    }
    let mut records = std::collections::HashMap::new();
    for edge in &mut view.relationships {
        if edge.origin != "content" || edge.source.owners != view.selected.owners {
            continue;
        }
        let Some(target) = &edge.target else {
            continue;
        };
        if !records.contains_key(&target.record_key) {
            let key = RecordKey::parse(&target.record_key)
                .map_err(|e| crate::AppServiceError::invalid_request(e.to_string()))?;
            records.insert(
                target.record_key.clone(),
                retrieval.get_record(atlas_search::GetRecordRequest {
                    record_key: &key,
                    selected_content: &[],
                })?,
            );
        }
        let Some(detail) = records.get(&target.record_key).and_then(Option::as_ref) else {
            continue;
        };
        let Some(node) = detail.source.node_at(&target.owners) else {
            continue;
        };
        let base_rank = match node {
            SourceNodeView::Item(ItemSourceView::EffectSource(source))
                if from_spell
                    && crate::presentation::spell_effect::supports(
                        ItemSourceView::EffectSource(source),
                    ) =>
            {
                crate::presentation::spell_effect::base_rank(source)
            }
            SourceNodeView::Item(ItemSourceView::SpellSource(source)) if from_effect => {
                crate::presentation::spell_context::base_rank(source)
            }
            _ => None,
        };
        if base_rank.is_some_and(|base| rank >= base) {
            let mut contextual = target.clone();
            contextual.spell_rank = Some(rank);
            edge.contextual_target = Some(contextual);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{executor::RetrievalExecutor, test_support::fixture_worker_with_executor};
    use atlas_search::test_support::{open_source_fixture, record};
    use serde_json::json;

    #[test]
    fn inline_context_preserves_citation_and_requires_unambiguous_provenance() {
        for mode in ["core", "stats", "conflict", "missing", "no_inline"] {
            let mut spell = json!({"_id":"ssssssssssssssss","name":"Local Heal","type":"spell","system":{"level":{"value":1},"location":{"heightenedLevel":2},"description":{"value":"Customized healing"}}});
            let canonical = "Compendium.pf2e.spells.Item.hhhhhhhhhhhhhhhh";
            if mode != "missing" {
                if mode == "stats" {
                    spell["_stats"] = json!({"compendiumSource":canonical});
                } else {
                    spell["flags"] = json!({"core":{"sourceId":canonical}});
                }
            }
            if mode == "conflict" {
                spell["_stats"] =
                    json!({"compendiumSource":"Compendium.pf2e.spells.Item.oooooooooooooooo"});
            }
            let wand = json!({"_id":"wwwwwwwwwwwwwwww","name":"Wand","type":"consumable","system":{"category":"wand","spell":spell,"description":{"value":if mode == "no_inline" {"<p>Cast the spell.</p>"} else {"<p>Cast @UUID[Compendium.pf2e.spells.Item.Heal].</p>"}}}});
            let fixture = fixture_worker_with_executor(
                RetrievalExecutor::from_test_fixture_factory(1, 16, move || {
                    Ok(open_source_fixture(
                        vec![
                            record("equipment", "Item", wand.clone()),
                            record(
                                "spells",
                                "Item",
                                json!({"_id":"hhhhhhhhhhhhhhhh","name":"Heal","type":"spell","system":{"level":{"value":1}}}),
                            ),
                            record(
                                "spells",
                                "Item",
                                json!({"_id":"oooooooooooooooo","name":"Other","type":"spell","system":{"level":{"value":1}}}),
                            ),
                        ],
                        false,
                    )?)
                }),
            );
            let view = fixture
                .worker
                .record_detail_at(atlas_app_model::RecordDetailRequest {
                    record_key: "equipment:wwwwwwwwwwwwwwww".into(),
                    owners: vec![],
                    fields: vec![],
                    spell_rank: None,
                    passage: None,
                    source_fingerprint: None,
                })
                .unwrap();
            let RecordBodyView::PhysicalReference(physical) = &view.presentation.body else {
                panic!("physical")
            };
            let consumable = physical.consumable.as_ref().unwrap();
            assert!(consumable.spell.value.is_some(), "{mode}");
            assert!(
                view.presentation.owned.is_empty(),
                "semantic spell must not leak into Contents"
            );
            let matches = matches!(mode, "core" | "stats");
            assert_eq!(consumable.spell_presented_inline, matches, "{mode}");
            if mode != "no_inline" {
                let edge = view
                    .relationships
                    .iter()
                    .find(|e| e.origin == "content")
                    .unwrap();
                assert_eq!(
                    edge.target.as_ref().unwrap().record_key,
                    "spells:hhhhhhhhhhhhhhhh"
                );
                assert_eq!(edge.contextual_target.is_some(), matches, "{mode}");
                if let Some(target) = &edge.contextual_target {
                    assert_eq!(target.record_key, "equipment:wwwwwwwwwwwwwwww");
                    assert_eq!(target.owners.len(), 1);
                    assert_eq!(target.spell_rank, Some(2));
                }
            }
        }
    }
}
