//! Selected authored facts. Renderers own density and layout; source owns meaning.
mod action_cost;
pub(crate) mod actor;
mod facts;
mod spell;
pub(crate) mod spell_context;
mod spell_rank;
use crate::projection::{field_state, navigation_fingerprint};
use action_cost::{casting_cost, item_action_cost};
pub(crate) use actor::compose_participant;
use atlas_app_model::*;
use atlas_domain::QueryFieldState;
use atlas_record::source_content::OwnedContentLocator;
use atlas_record::source_record::{
    ItemSourceView, SourceBackedRecord, SourceFieldView, SourceNodeView, SourceQueryView,
};
use facts::*;

pub(crate) fn apply_spell_rank(
    record: &SourceBackedRecord,
    node: SourceNodeView<'_>,
    requested: Option<u8>,
    presentation: &mut RecordPresentationView,
    retrieval: &atlas_search::AtlasRetrievalService,
) -> crate::AppServiceResult<Option<u8>> {
    let SourceNodeView::Item(ItemSourceView::SpellSource(source)) = node else {
        if requested.is_some() {
            return Err(crate::AppServiceError::invalid_request(
                "Rank preview requires a spell.",
            ));
        }
        return Ok(None);
    };
    let ranks = spell_context::ranks(record, source);
    if let Some(rank) = requested
        && (!(1..=10).contains(&rank)
            || spell_context::base_rank(source).is_some_and(|base| rank < base))
    {
        return Err(crate::AppServiceError::invalid_request(
            "Choose a spell rank from its original rank through 10.",
        ));
    }
    let rank = requested.or_else(|| (ranks.len() == 1).then(|| ranks[0]));
    let Some(rank) = rank else {
        if ranks.len() > 1
            && let RecordBodyView::Spell(view) = &mut presentation.body
        {
            view.rank_note =
                Some("Prepared at multiple ranks. Choose a rank to preview its mechanics.".into());
        }
        return Ok(None);
    };
    let effective = match spell_rank::heighten(source, rank) {
        Ok(effective) => effective,
        Err(reason) => {
            if let RecordBodyView::Spell(view) = &mut presentation.body {
                view.rank_note = Some(format!(
                    "Showing original mechanics; rank {rank} preview is unavailable. {reason}"
                ));
            }
            return Ok(None);
        }
    };
    let mut view = spell::authored_spell(&effective);
    view.rank = NumberFactView {
        state: QueryFieldState::Value,
        value: Some(rank.into()),
        adjustment: None,
    };
    presentation.classification = format!("Spell {rank}");
    presentation.identity.level_label = Some(format!("Rank {rank}"));
    let query = SourceQueryView {
        source: SourceNodeView::Item(ItemSourceView::SpellSource(&effective)),
        pack_id: record.key().pack().as_str(),
        pack_label: "",
    };
    presentation.identity.rarity = identifier(query.rarity()).value;
    presentation.identity.traits = query
        .traits()
        .value()
        .into_iter()
        .flatten()
        .map(|t| RecordBadgeView {
            kind: "trait".into(),
            value: t.clone(),
            label: retrieval.trait_label(t).unwrap_or(t).into(),
        })
        .collect();
    presentation.body = RecordBodyView::Spell(Box::new(view));
    Ok(Some(rank))
}

/// One dispatch policy for selected item bodies and summary link usefulness.
#[derive(Clone, Copy)]
enum SelectedItemBody<'a> {
    Spell(&'a atlas_foundry_model::generated::SpellSource),
    Activity { casting_context: bool },
    Physical,
    Content,
}
fn selected_item_body(item: ItemSourceView<'_>) -> SelectedItemBody<'_> {
    match item {
        ItemSourceView::SpellSource(source) => SelectedItemBody::Spell(source),
        _ if matches!(
            item.family(),
            "melee" | "action" | "effect" | "feat" | "lore" | "spellcastingEntry"
        ) =>
        {
            SelectedItemBody::Activity {
                casting_context: item.family() == "spellcastingEntry",
            }
        }
        _ if !matches!(item.physical_price(), SourceFieldView::NotApplicable) => {
            SelectedItemBody::Physical
        }
        _ => SelectedItemBody::Content,
    }
}

pub(crate) fn record_presentation(
    record: &SourceBackedRecord,
    node: SourceNodeView<'_>,
    summary: &RecordSummaryView,
    owners: &[OwnedContentLocator],
    content: Vec<PreparedContentFieldView>,
    retrieval: &atlas_search::AtlasRetrievalService,
) -> RecordPresentationView {
    let identity = selected_identity(record, node, summary, owners, retrieval);
    let mut owned = owned_links(record, node, owners, retrieval.source_fingerprint());
    let mut body = match node {
        SourceNodeView::Actor(atlas_foundry_model::ActorSourcePF2e::HazardSource(source)) => {
            RecordBodyView::Hazard(Box::new(HazardPresentationView {
                defense_block: hazard_defense_block(node),
                show_saves: hazard_show_saves(node),
                actor: actor::authored_actor(record, node, retrieval.source_fingerprint()),
                complex: fact(
                    SourceFieldView::from(&source.system)
                        .and_then(|s| (&s.details).into())
                        .and_then(|s| (&s.is_complex).into())
                        .map(|b| *b),
                ),
                stealth: number(
                    SourceQueryView {
                        source: node,
                        pack_id: "",
                        pack_label: "",
                    }
                    .actor()
                    .hazard_stealth(),
                ),
                hardness: number(
                    SourceQueryView {
                        source: node,
                        pack_id: "",
                        pack_label: "",
                    }
                    .actor()
                    .hazard_hardness(),
                ),
            }))
        }
        SourceNodeView::Actor(_)
            if node
                .source_type()
                .value()
                .is_some_and(|t| matches!(t, "npc" | "character" | "familiar")) =>
        {
            RecordBodyView::Creature(Box::new(actor::authored_actor(
                record,
                node,
                retrieval.source_fingerprint(),
            )))
        }
        SourceNodeView::Item(item) => match selected_item_body(item) {
            SelectedItemBody::Spell(source) => {
                RecordBodyView::Spell(Box::new(spell::authored_spell(source)))
            }
            SelectedItemBody::Activity { .. } => {
                RecordBodyView::Activity(Box::new(actor::authored_activity(
                    item,
                    RecordNavigationView {
                        spell_rank: None,
                        record_key: record.key().to_string(),
                        owners: owners.to_vec(),
                        field: None,
                        passage: None,
                        source_fingerprint: navigation_fingerprint(
                            owners,
                            retrieval.source_fingerprint(),
                        ),
                    },
                )))
            }
            SelectedItemBody::Physical => {
                RecordBodyView::PhysicalReference(Box::new(physical(item)))
            }
            SelectedItemBody::Content => RecordBodyView::Content,
        },
        SourceNodeView::Table(table) => {
            RecordBodyView::RollTable(Box::new(RollTablePresentationView {
                formula: text((&table.formula).into()),
                results: fact(
                    SourceFieldView::from(&table.results).map(|_| std::mem::take(&mut owned)),
                ),
            }))
        }
        SourceNodeView::Result(result) => {
            RecordBodyView::TableResult(Box::new(TableResultPresentationView {
                range: fact(
                    SourceFieldView::from(&result.range).map(|(lo, hi)| table_range(lo, hi)),
                ),
                weight: number((&result.weight).into()),
                result_type: identifier((&result.r#type).into()),
            }))
        }
        _ => RecordBodyView::Content,
    };
    let label_traits = |activity: &mut ActorActivityView| {
        for badge in &mut activity.traits {
            badge.label = retrieval
                .trait_label(&badge.value)
                .unwrap_or(&badge.value)
                .into();
        }
    };
    match &mut body {
        RecordBodyView::Creature(a) => {
            for activity in a.activities.value.iter_mut().flatten() {
                label_traits(activity);
            }
        }
        RecordBodyView::Hazard(h) => {
            for activity in h.actor.activities.value.iter_mut().flatten() {
                label_traits(activity);
            }
        }
        RecordBodyView::Activity(activity) => {
            label_traits(activity);
            if matches!(node, SourceNodeView::Item(item) if matches!(selected_item_body(item), SelectedItemBody::Activity { casting_context: true }))
                && owners.len() == 1
                && let Some(parent_node @ SourceNodeView::Actor(_)) = record.node_at(&[])
            {
                let activities =
                    actor::authored_activities(record, parent_node, retrieval.source_fingerprint());
                let state = activities.state;
                let spells = activities.value.map(|activities| {
                    activities
                        .into_iter()
                        .filter(|spell| {
                            spell.kind == ActorActivityKindView::Spell
                                && spell.casting_entry.as_ref() == Some(&activity.navigation)
                        })
                        .map(|mut spell| {
                            label_traits(&mut spell);
                            spell
                        })
                        .collect()
                });
                activity.casting_context = Some(CastingEntryContextView {
                    parent: RecordNavigationView {
                        spell_rank: None,
                        record_key: record.key().to_string(),
                        owners: vec![],
                        field: None,
                        passage: None,
                        source_fingerprint: None,
                    },
                    parent_title: summary.title.clone(),
                    spells: FactView {
                        state,
                        value: spells,
                    },
                });
            }
        }
        _ => {}
    }
    RecordPresentationView {
        classification: classification(node, &identity),
        identity,
        content,
        owned,
        body,
    }
}
fn classification(node: SourceNodeView<'_>, identity: &RecordSummaryView) -> String {
    let level = match node {
        SourceNodeView::Item(item) if item.family() == "spell" => item.spell_rank(),
        SourceNodeView::Item(item) => item.item_level(),
        SourceNodeView::Actor(_) => SourceQueryView {
            source: node,
            pack_id: "",
            pack_label: "",
        }
        .actor()
        .level(),
        _ => SourceFieldView::NotApplicable,
    };
    match level {
        SourceFieldView::Value(value) => format!("{} {value}", identity.kind_label),
        SourceFieldView::NotApplicable => identity.kind_label.clone(),
        _ => format!(
            "{} · {} unavailable",
            identity.kind_label,
            if matches!(node, SourceNodeView::Item(item) if item.family() == "spell") {
                "rank"
            } else {
                "level"
            }
        ),
    }
}
/// Upstream's authored hazard defense block uses HP maximum or AC, not saves.
/// Only known numeric zero for both suppresses the primary block.
fn hazard_defense_block(node: SourceNodeView<'_>) -> FactView<bool> {
    let actor = SourceQueryView {
        source: node,
        pack_id: "",
        pack_label: "",
    }
    .actor();
    let hp = actor.hp_maximum();
    let ac = actor.armor_class();
    fact(hp.and_then(|hp| ac.map(|ac| hp.as_f64() != Some(0.0) || ac.as_f64() != Some(0.0))))
}

/// Atlas reading policy, not a declaration that known zero saves are missing.
fn hazard_show_saves(node: SourceNodeView<'_>) -> bool {
    use atlas_record::source_record::ActorSave;
    if hazard_defense_block(node).value != Some(false) {
        return true;
    }
    let actor = SourceQueryView {
        source: node,
        pack_id: "",
        pack_label: "",
    }
    .actor();
    ![ActorSave::Fortitude, ActorSave::Reflex, ActorSave::Will]
        .into_iter()
        .all(|save| {
            actor
                .save(save)
                .value()
                .is_some_and(|v| v.as_f64() == Some(0.0))
                && actor
                    .hazard_save_detail(save)
                    .value()
                    .is_some_and(|v| v.trim().is_empty())
        })
}
fn selected_identity(
    record: &SourceBackedRecord,
    node: SourceNodeView<'_>,
    summary: &RecordSummaryView,
    owners: &[OwnedContentLocator],
    retrieval: &atlas_search::AtlasRetrievalService,
) -> RecordSummaryView {
    if owners.is_empty() {
        return summary.clone();
    }
    let query = SourceQueryView {
        source: node,
        pack_id: record.key().pack().as_str(),
        pack_label: summary.pack.as_deref().unwrap_or(""),
    };
    let level = match node {
        SourceNodeView::Item(i) => i
            .spell_rank()
            .value()
            .map(|v| format!("Rank {v}"))
            .or_else(|| i.item_level().value().map(|v| format!("Level {v}"))),
        _ => query.actor().level().value().map(|v| format!("Level {v}")),
    };
    let family = node.source_type().value().unwrap_or(node.document_kind());
    RecordSummaryView {
        record_key: record.key().to_string(),
        title: node_title(node),
        kind: family.into(),
        kind_label: match family {
            "spellcastingEntry" => "Spellcasting",
            "action" => "Ability",
            _ => family,
        }
        .into(),
        source_type: Some(family.into()),
        level_label: level,
        level_basis: None,
        rarity: identifier(query.rarity()).value,
        traits: query
            .traits()
            .value()
            .into_iter()
            .flatten()
            .map(|t| RecordBadgeView {
                kind: "trait".into(),
                value: t.clone(),
                label: retrieval.trait_label(t).unwrap_or(t).into(),
            })
            .collect(),
        publication: query.publication_title().value().map(str::to_owned),
        pack: summary.pack.clone(),
    }
}
fn table_range(lo: &serde_json::Number, hi: &serde_json::Number) -> String {
    if lo == hi {
        lo.to_string()
    } else {
        format!("{lo}–{hi}")
    }
}
fn node_title(node: SourceNodeView<'_>) -> String {
    node.name().value().cloned().unwrap_or_else(|| match node {
        SourceNodeView::Result(r) => SourceFieldView::from(&r.range)
            .value()
            .map(|(lo, hi)| format!("Result {}", table_range(lo, hi)))
            .unwrap_or_else(|| "Table result".into()),
        _ => node
            .source_type()
            .value()
            .unwrap_or(node.document_kind())
            .into(),
    })
}
fn owned_links(
    record: &SourceBackedRecord,
    node: SourceNodeView<'_>,
    owners: &[OwnedContentLocator],
    fingerprint: &str,
) -> Vec<OwnedRecordLinkView> {
    let mut links = vec![];
    if !owners.is_empty() {
        return links;
    }
    let mut add = |owner: &OwnedContentLocator, child: SourceNodeView<'_>| {
        let owners = vec![owner.clone()];
        links.push(OwnedRecordLinkView {
            navigation: RecordNavigationView {
                spell_rank: None,
                record_key: record.key().to_string(),
                source_fingerprint: navigation_fingerprint(&owners, fingerprint),
                owners,
                field: None,
                passage: None,
            },
            title: node_title(child),
            family: child
                .source_type()
                .value()
                .unwrap_or(child.document_kind())
                .into(),
            usage: match child {
                SourceNodeView::Item(i) => item_usage(i),
                _ => None,
            },
            action_cost: match child {
                SourceNodeView::Item(i) => item_action_cost(i),
                _ => unavailable(),
            },
        });
    };
    match node {
        SourceNodeView::Actor(_)
            if node
                .source_type()
                .value()
                .is_some_and(|t| matches!(t, "npc" | "hazard" | "character" | "familiar")) => {}
        SourceNodeView::Actor(_) => {
            record.visit_immediate_actor_items(|_, o, i| add(o, SourceNodeView::Item(i)));
        }
        SourceNodeView::Journal(_) => {
            record.visit_immediate_journal_pages(&mut add);
        }
        SourceNodeView::Table(_) => {
            record.visit_immediate_table_results(&mut add);
        }
        _ => {}
    }
    links
}
pub(super) fn item_usage(item: ItemSourceView<'_>) -> Option<String> {
    item.physical_usage().value().map(str::to_owned)
}
fn physical(item: ItemSourceView<'_>) -> PhysicalPresentationView {
    let price = item.physical_price();
    PhysicalPresentationView {
        usage: fact(item.physical_usage().map(str::to_owned)),
        bulk: number(item.physical_bulk()),
        price: fact(
            price
                .and_then(|p| (&p.value).into())
                .map(|c| PhysicalPriceView {
                    pp: number((&c.pp).into()),
                    gp: number((&c.gp).into()),
                    sp: number((&c.sp).into()),
                    cp: number((&c.cp).into()),
                }),
        ),
        price_per: number(price.and_then(|p| (&p.per).into())),
    }
}

#[cfg(test)]
mod tests;
