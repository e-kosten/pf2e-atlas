//! Product labels for the root Kind field. Embedded kinds keep their scoped vocabulary.
use atlas_app_model::FilterChoiceView;
use atlas_domain::{QueryLiteral, root_kind};

pub(crate) fn record_selector(record: &atlas_domain::SourceRecordSummary) -> Option<&str> {
    if record.document_kind == "JournalEntry" {
        Some("journal")
    } else {
        record.root_kind.as_deref()
    }
}

pub(crate) fn label(id: &str) -> String {
    match id {
        "journal" => "All journals".into(),
        "all_deities" => "Deities".into(),
        "all_spells" => "All spells".into(),
        "all_equipment" => "All equipment".into(),
        "character_advancement" => "Character advancement".into(),
        "character_creation" => "Character creation".into(),
        "kingdom_and_warfare" => "Kingdom and warfare".into(),
        _ => crate::projection::kind_label(id),
    }
}

pub(crate) fn choice(id: &str, packs: &[(String, String)]) -> FilterChoiceView {
    let label = if let Some(pack) = root_kind::journal_pack(id) {
        packs
            .iter()
            .find(|(key, _)| key == pack)
            .map_or_else(|| pack.to_owned(), |(_, label)| label.clone())
    } else {
        label(id)
    };
    FilterChoiceView {
        value: QueryLiteral::String(id.into()),
        label,
        parent_kind: root_kind::parent(id).map(str::to_owned),
        search_aliases: if id == "familiar" {
            vec!["companion".into()]
        } else {
            vec![]
        },
        help: None,
    }
}
