use crate::source_content::SourceContentLocator;
use atlas_domain::{OwnedContentLocator, RecordKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

mod journal;
pub use journal::journal_context;
pub const ATLAS_CORRESPONDENCE_BYTES: &[u8] = include_bytes!("remaster/correspondences.json");
pub fn remaster_inputs_identity() -> String {
    format!(
        "compact-remaster-context/v1:{:x}",
        Sha256::digest(ATLAS_CORRESPONDENCE_BYTES)
    )
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemasterRelation {
    FormerName,
    Replaces,
    MergedFrom,
    RenamedTo,
    ReplacedBy,
    MergedInto,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemasterTarget {
    pub name: String,
    pub record: Option<RecordKey>,
    pub owners: Vec<OwnedContentLocator>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordRemasterContext {
    pub record: RecordKey,
    pub owners: Vec<OwnedContentLocator>,
    pub relation: RemasterRelation,
    pub targets: Vec<RemasterTarget>,
    pub journal: Option<SourceContentLocator>,
    pub scope: String,
}
#[derive(Deserialize)]
pub struct Correspondences {
    pub entries: Vec<Correspondence>,
}
#[derive(Deserialize)]
pub struct Correspondence {
    pub before: RecordKey,
    pub after: RecordKey,
    pub before_label: String,
    pub source_type: String,
    pub journal_context: Option<JournalContext>,
}
#[derive(Deserialize)]
pub struct JournalContext {
    pub locator: SourceContentLocator,
}

pub fn refine_context(contexts: &mut Vec<RecordRemasterContext>) {
    let cleanse = "spells-srd:SUKaxVZW2TlM8lu0";
    let required = ["Remove Curse", "Remove Disease", "Neutralize Poison"];
    let Some(source) = contexts
        .iter()
        .find(|c| {
            c.record.to_string() == cleanse
                && c.owners.is_empty()
                && c.relation == RemasterRelation::FormerName
                && c.scope == "Spells:"
                && c.journal.is_some()
        })
        .cloned()
    else {
        return;
    };
    let applies = |c: &RecordRemasterContext| {
        c.record == source.record
            && c.owners.is_empty()
            && c.relation == RemasterRelation::FormerName
            && c.scope == source.scope
            && c.journal == source.journal
    };
    let present = contexts
        .iter()
        .filter(|c| applies(c))
        .flat_map(|c| &c.targets)
        .map(|t| t.name.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if required.iter().all(|name| present.contains(name)) {
        contexts.retain(|c| {
            !applies(c)
                || !c
                    .targets
                    .iter()
                    .all(|t| t.record.is_none() && required.contains(&t.name.as_str()))
        });
        contexts.push(RecordRemasterContext {
            relation: RemasterRelation::Replaces,
            targets: required
                .into_iter()
                .map(|name| RemasterTarget {
                    name: name.into(),
                    record: None,
                    owners: vec![],
                })
                .collect(),
            ..source
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cleanse_requires_all_three_predecessors_in_one_selected_journal_scope() {
        let journal = SourceContentLocator {
            record: "journals:aaaaaaaaaaaaaaaa".parse().unwrap(),
            owners: vec![],
            field: "/text/content".into(),
        };
        let mut rows = ["Neutralize Poison", "Remove Curse", "Remove Disease"]
            .into_iter()
            .map(|name| RecordRemasterContext {
                record: "spells-srd:SUKaxVZW2TlM8lu0".parse().unwrap(),
                owners: vec![],
                relation: RemasterRelation::FormerName,
                targets: vec![RemasterTarget {
                    name: name.into(),
                    record: None,
                    owners: vec![],
                }],
                journal: Some(journal.clone()),
                scope: "Spells:".into(),
            })
            .collect::<Vec<_>>();
        let mut partial = rows[..2].to_vec();
        refine_context(&mut partial);
        assert!(
            partial
                .iter()
                .all(|c| c.relation == RemasterRelation::FormerName)
        );
        let mut mixed = rows.clone();
        mixed[2].journal = None;
        refine_context(&mut mixed);
        assert!(
            mixed
                .iter()
                .all(|c| c.relation == RemasterRelation::FormerName)
        );
        refine_context(&mut rows);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].relation, RemasterRelation::Replaces);
        assert_eq!(
            rows[0]
                .targets
                .iter()
                .map(|t| t.name.as_str())
                .collect::<Vec<_>>(),
            ["Remove Curse", "Remove Disease", "Neutralize Poison"]
        );
    }
}
