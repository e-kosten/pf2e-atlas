//! Selected authored facts. Renderers own density and layout; source owns meaning.
pub(crate) mod actor;
mod facts;
mod spell;
use crate::projection::{field_state, navigation_fingerprint};
pub(crate) use actor::compose_participant;
use atlas_app_model::*;
use atlas_domain::QueryFieldState;
use atlas_record::source_content::OwnedContentLocator;
use atlas_record::source_record::{
    ItemSourceView, SourceBackedRecord, SourceFieldView, SourceNodeView, SourceQueryView,
};
use facts::*;

pub(crate) fn record_presentation(
    record: &SourceBackedRecord,
    node: SourceNodeView<'_>,
    summary: &RecordSummaryView,
    owners: &[OwnedContentLocator],
    content: Vec<PreparedContentFieldView>,
    retrieval: &atlas_search::AtlasRetrievalService,
) -> RecordPresentationView {
    let identity = selected_identity(record, node, summary, owners, retrieval);
    let owned = owned_links(record, node, owners, retrieval.source_fingerprint());
    let mut body = match node {
        SourceNodeView::Actor(atlas_foundry_model::ActorSourcePF2e::HazardSource(source)) => {
            RecordBodyView::Hazard(Box::new(HazardPresentationView {
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
        SourceNodeView::Item(ItemSourceView::SpellSource(source)) => {
            RecordBodyView::Spell(Box::new(spell::authored_spell(source)))
        }
        SourceNodeView::Item(item)
            if matches!(
                item.family(),
                "melee" | "action" | "effect" | "feat" | "lore" | "spellcastingEntry"
            ) =>
        {
            RecordBodyView::Activity(Box::new(actor::authored_activity(
                item,
                RecordNavigationView {
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
        SourceNodeView::Item(item)
            if !matches!(item.physical_price(), SourceFieldView::NotApplicable) =>
        {
            RecordBodyView::PhysicalReference(Box::new(physical(item)))
        }
        SourceNodeView::Table(table) => {
            RecordBodyView::RollTable(Box::new(RollTablePresentationView {
                formula: text((&table.formula).into()),
            }))
        }
        SourceNodeView::Result(result) => {
            RecordBodyView::TableResult(Box::new(TableResultPresentationView {
                range: fact(
                    SourceFieldView::from(&result.range).map(|(lo, hi)| format!("{lo}–{hi}")),
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
        RecordBodyView::Activity(activity) => label_traits(activity),
        _ => {}
    }
    RecordPresentationView {
        identity,
        content,
        owned,
        body,
    }
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
        kind_label: family.into(),
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
fn node_title(node: SourceNodeView<'_>) -> String {
    node.name().value().cloned().unwrap_or_else(|| match node {
        SourceNodeView::Result(r) => SourceFieldView::from(&r.range)
            .value()
            .map(|(l, h)| format!("Result {l}–{h}"))
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
    item.spell_casting_time()
        .value()
        .map(cast_label)
        .or_else(|| item.physical_usage().value().map(str::to_owned))
        .or_else(|| {
            identifier(item.action_type()).value.map(|kind| {
                if kind == "action" {
                    item.action_count()
                        .value()
                        .map(|n| {
                            if n.as_i64() == Some(1) {
                                "1 action".into()
                            } else {
                                format!("{n} actions")
                            }
                        })
                        .unwrap_or(kind)
                } else {
                    kind
                }
            })
        })
}
fn physical(item: ItemSourceView<'_>) -> PhysicalPresentationView {
    let price = item.physical_price();
    PhysicalPresentationView {
        usage: fact(item.physical_usage().map(str::to_owned)),
        bulk: number(item.physical_bulk()),
        price: fact(price.and_then(|p| (&p.value).into()).map(|c| {
            [("pp", &c.pp), ("gp", &c.gp), ("sp", &c.sp), ("cp", &c.cp)]
                .into_iter()
                .filter_map(|(u, n)| SourceFieldView::from(n).value().map(|n| format!("{n} {u}")))
                .collect::<Vec<_>>()
                .join(", ")
        })),
        price_per: number(price.and_then(|p| (&p.per).into())),
    }
}

pub(super) fn cast_label(value: &str) -> String {
    match value {
        "1" => "1 action".into(),
        "2" => "2 actions".into(),
        "3" => "3 actions".into(),
        "1 to 3" => "1 to 3 actions".into(),
        _ => value.into(),
    }
}

#[cfg(test)]
mod tests;
