use crate::{
    AtlasRetrievalService, SearchError,
    creature_copy_catalog::{CreatureCopyDeclaration, CreatureCopyRecord},
};
use atlas_domain::{RecordKind, SourceRecordSummary};
use std::sync::OnceLock;

fn catalog() -> Result<&'static [CreatureCopyDeclaration], SearchError> {
    static CATALOG: OnceLock<Result<Vec<CreatureCopyDeclaration>, serde_json::Error>> =
        OnceLock::new();
    CATALOG
        .get_or_init(|| {
            serde_json::from_str(include_str!(concat!(
                env!("OUT_DIR"),
                "/creature_copy_links.json"
            )))
        })
        .as_deref()
        .map_err(|error| {
            SearchError::artifact_contract_violation(format!(
                "invalid bundled creature copy links: {error}"
            ))
        })
}

fn matches(expected: &CreatureCopyRecord, summary: &SourceRecordSummary, remaster: bool) -> bool {
    summary.key == expected.key
        && summary.name.as_value() == Some(&expected.name)
        && summary.record_kind.as_value() == Some(&RecordKind::Creature)
        && summary
            .source_type
            .as_value()
            .is_some_and(|kind| kind == "npc")
        && summary.publication_remaster.as_value() == Some(&remaster)
        && summary.content_hash == expected.sha256
}

impl AtlasRetrievalService {
    pub fn canonical_creature_copy(
        &self,
        summary: &SourceRecordSummary,
    ) -> Result<Option<SourceRecordSummary>, SearchError> {
        let declaration = catalog()?.iter().find(|declaration| {
            declaration
                .copies
                .iter()
                .any(|copy| matches(copy, summary, declaration.remaster))
        });
        let Some(declaration) = declaration else {
            return Ok(None);
        };
        Ok(self
            .index
            .read_summary(&declaration.canonical.key)?
            .filter(|canonical| matches(&declaration.canonical, canonical, declaration.remaster)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{open_source_fixture, record};
    use serde_json::json;

    #[test]
    fn copy_link_requires_both_exact_reviewed_source_hashes() {
        let declaration = &catalog().unwrap()[0];
        let copy = &declaration.copies[0];
        let canonical = &declaration.canonical;
        let fixture = |entry: &CreatureCopyRecord, drift: bool| {
            let mut input = record(
                entry.key.pack().as_str(),
                "Actor",
                json!({
                    "_id": entry.key.id().as_str(),
                    "name": entry.name,
                    "type": "npc",
                    "system": {"details": {"publication": {
                        "title": "Fixture", "remaster": declaration.remaster, "license": "OGL"
                    }}}
                }),
            );
            if !drift {
                input.content_hash.clone_from(&entry.sha256);
            }
            input
        };
        for (missing_canonical, changed_copy, changed_canonical) in [
            (false, false, false),
            (true, false, false),
            (false, true, false),
            (false, false, true),
        ] {
            let mut records = vec![fixture(copy, changed_copy)];
            if !missing_canonical {
                records.push(fixture(canonical, changed_canonical));
            }
            let (retrieval, _artifact) = open_source_fixture(records, false).unwrap();
            let summary = retrieval.index.read_summary(&copy.key).unwrap().unwrap();
            let link = retrieval.canonical_creature_copy(&summary).unwrap();
            let expected = (!missing_canonical && !changed_copy && !changed_canonical)
                .then_some(&canonical.key);
            assert_eq!(link.as_ref().map(|summary| &summary.key), expected);
            if let Some(target) = link {
                assert!(
                    retrieval
                        .canonical_creature_copy(&target)
                        .unwrap()
                        .is_none()
                );
            }
        }
    }
}
