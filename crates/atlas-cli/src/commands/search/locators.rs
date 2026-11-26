//! Bind addresses in CLI search and Similar responses to the page that produced them.
use crate::commands::record::navigation::Locator;
use atlas_app_model::{
    RecordNavigationView, RecordRelationshipView, RecordSummaryView, ReferenceDestinationView,
    ReferenceHelpView, SearchModeSummary, SearchResultsPage, SimilarityPartsView,
};
use serde_json::{Value, json};

/// At a Search match the containing row already identifies the whole record.
pub(super) fn match_navigation(
    target: &mut Value,
    address: &RecordNavigationView,
    generation: &str,
    containing_record: Option<&str>,
) {
    if address.selects_record_part() || containing_record != Some(address.record_key.as_str()) {
        target["navigation"] = Locator::new(generation, address).compact();
    } else if let Some(object) = target.as_object_mut() {
        object.remove("navigation");
    }
}

fn help(target: &mut Value, value: Option<&ReferenceHelpView>, generation: &str) {
    if let Some(ReferenceHelpView {
        destination: ReferenceDestinationView::Record { navigation },
        ..
    }) = value
    {
        target["help"]["destination"]["navigation"] =
            Locator::new(generation, navigation).compact();
    }
}

fn summary(target: &mut Value, record: &RecordSummaryView, generation: &str) {
    if let Some(rarity) = &record.rarity {
        help(&mut target["rarity"], rarity.help.as_ref(), generation);
    }
    for (i, badge) in record.traits.iter().enumerate() {
        help(&mut target["traits"][i], badge.help.as_ref(), generation);
    }
}

fn relationship(target: &mut Value, edge: &RecordRelationshipView, generation: &str) {
    target["source_navigation"] = Locator::new(generation, &edge.source_navigation).compact();
    if let Some(address) = edge.destination.record_target() {
        target["destination"]["target"] = Locator::new(generation, address).compact();
    }
    if let Some(address) = &edge.contextual_target {
        target["contextual_target"] = Locator::new(generation, address).compact();
    }
}

pub(in crate::commands) fn parts(inventory: &SimilarityPartsView) -> Value {
    let mut value = json!(inventory);
    for (i, part) in inventory.parts.iter().enumerate() {
        if let Some(address) = part.navigation() {
            value["parts"][i]["navigation"] =
                Locator::new(&inventory.artifact_generation, address).compact();
        }
    }
    value
}

pub(super) fn journal_pages(
    target: &mut Value,
    row: &atlas_app_model::SearchResultRow,
    generation: &str,
) {
    if let Some(matches) = &row.exact_journal_pages {
        let pages = matches
            .pages
            .iter()
            .map(|page| {
                json!({
                    "title": page.title,
                    "navigation": Locator::new(generation, &page.navigation).compact(),
                })
            })
            .collect::<Vec<_>>();
        target["exact_journal_pages"] =
            json!({"pages": pages, "additional_count": matches.additional_count});
    }
}

pub(in crate::commands) fn page(result: &SearchResultsPage) -> Value {
    let mut value = json!(result);
    let generation = &result.artifact_generation;
    if let SearchModeSummary::SimilarRecords { seed, .. } = &result.mode {
        summary(&mut value["mode"]["seed"], seed, generation);
    }
    for (i, clause) in result.relationship_clauses.iter().enumerate() {
        summary(
            &mut value["relationship_clauses"][i]["record"],
            &clause.record,
            generation,
        );
    }
    for (i, row) in result.rows.iter().enumerate() {
        let entry = &mut value["rows"][i];
        journal_pages(entry, row, generation);
        summary(&mut entry["record"], &row.record, generation);
        let containing_record = match result.mode {
            SearchModeSummary::SimilarRecords { .. } => None,
            _ => Some(row.record.record_key.as_str()),
        };
        for (j, witness) in row.matches.iter().enumerate() {
            match_navigation(
                &mut entry["matches"][j],
                &witness.navigation,
                generation,
                containing_record,
            );
        }
        for (j, section) in row.matching_sections.iter().flatten().enumerate() {
            match_navigation(
                &mut entry["matching_sections"][j],
                &section.navigation,
                generation,
                containing_record,
            );
        }
        for (j, proof) in row.relationship_evidence.iter().enumerate() {
            for (k, edge) in proof.occurrences.iter().enumerate() {
                relationship(
                    &mut entry["relationship_evidence"][j]["occurrences"][k],
                    edge,
                    generation,
                );
            }
        }
        if let Some(evidence) = &row.similarity {
            let target = &mut entry["similarity"];
            for (j, part) in evidence.parts.iter().enumerate() {
                if let Some(address) = part.part.navigation() {
                    target["parts"][j]["part"]["navigation"] =
                        Locator::new(generation, address).compact();
                }
            }
            for (j, pair) in evidence.pairs.iter().enumerate() {
                let target = &mut target["pairs"][j];
                if let Some(seed) = &pair.seed {
                    target["seed"]["navigation"] =
                        Locator::new(generation, &seed.navigation).compact();
                }
                target["candidate"]["navigation"] =
                    Locator::new(generation, &pair.candidate.navigation).compact();
            }
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_and_rank_addresses_replay_while_root_identity_uses_the_row_key() {
        let mut address = crate::commands::record::navigation::root("spells:one");
        address.field = Some("identity".into());
        address.passage = Some(atlas_domain::SourcePassageAddress::Identity {});
        let mut value = json!({"navigation": address});
        match_navigation(
            &mut value,
            &address,
            "generation",
            Some(&address.record_key),
        );
        assert!(value.get("navigation").is_none());

        address.field = None;
        address.passage = None;
        address.spell_rank = Some(4);
        for field in [None, Some("/system/description/value")] {
            if let Some(field) = field {
                address.spell_rank = None;
                address.field = Some(field.into());
            }
            match_navigation(
                &mut value,
                &address,
                "generation",
                Some(&address.record_key),
            );
            let replay: Locator = serde_json::from_value(value["navigation"].clone()).unwrap();
            assert_eq!(replay.address, address);
            assert_eq!(replay.artifact_generation, "generation");
        }
    }
}
