//! Exact pack fallback from the same admitted reference snapshot as the actor.
use std::collections::BTreeMap;

use atlas_domain::RecordKey;
use atlas_record::source_record::foundry_slug;

use crate::{AtlasRetrievalService, SearchError};

const PACK: &str = "bestiary-ability-glossary-srd";
const MAX_ITEMS: usize = 100;

pub(crate) type BestiaryGlossary = BTreeMap<String, Option<(RecordKey, String)>>;

impl AtlasRetrievalService {
    /// A unique glossary Item keyed by Foundry's build-derived slug.
    pub fn bestiary_attack_effect(
        &self,
        token: &str,
    ) -> Result<Option<(&RecordKey, &str)>, SearchError> {
        let glossary = if let Some(glossary) = self.bestiary_glossary.get() {
            glossary
        } else {
            let glossary = self.load_bestiary_glossary()?;
            self.bestiary_glossary.get_or_init(|| glossary)
        };
        Ok(glossary
            .get(token)
            .and_then(Option::as_ref)
            .map(|(key, name)| (key, name.as_str())))
    }

    fn load_bestiary_glossary(&self) -> Result<BestiaryGlossary, SearchError> {
        let mut glossary = BTreeMap::new();
        if atlas_index::reference_pack_applies(
            self.index.context().reference_preparation.as_ref(),
            "pf2e",
            PACK,
        ) && let Some(items) = self.index.item_names_in_pack(PACK, MAX_ITEMS)?
        {
            for (key, name) in items {
                let slug = foundry_slug(&name);
                if !slug.is_empty() {
                    glossary
                        .entry(slug)
                        .and_modify(|target| *target = None)
                        .or_insert(Some((key, name)));
                }
            }
        }
        Ok(glossary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{
        ReferencePreparation, hash, input, open_input_fixture, record, reference_inventory_identity,
    };
    use serde_json::json;

    fn glossary_fixture(
        scope: Option<&str>,
        count: usize,
    ) -> (AtlasRetrievalService, crate::test_support::FixtureArtifact) {
        let records = (0..count)
            .map(|index| {
                record(
                    PACK,
                    "Item",
                    json!({
                        "_id": format!("{index:016}"),
                        "type": "action",
                        "name": if index == 0 { "Swallow Whole".into() } else { format!("Ability {index}") },
                        "system": { "slug": "ignored-build-input" }
                    }),
                )
            })
            .collect();
        let mut data = input(records, false);
        if let Some(scope) = scope {
            let (expected_entry_count, entry_keys_sha256) =
                reference_inventory_identity(std::iter::empty()).unwrap();
            data.context.reference_preparation = Some(ReferencePreparation {
                source_package: Some("pf2e".into()),
                pack_systems: BTreeMap::from([(PACK.into(), Some(scope.into()))]),
                recipe: "test".into(),
                input_hash: hash(b"test"),
                coverage: crate::ReferenceCoverage::Complete,
                expected_entry_count,
                entry_keys_sha256,
            });
        }
        open_input_fixture(data).unwrap()
    }

    #[test]
    fn glossary_lookup_reuses_one_summary_read_and_matches_original_token() {
        let (r, _fixture) = glossary_fixture(Some("pf2e"), 2);
        r.reset_read_metrics();
        assert!(r.bestiary_attack_effect("Swallow Whole").unwrap().is_none());
        assert!(
            r.bestiary_attack_effect("ignored-build-input")
                .unwrap()
                .is_none()
        );
        let (key, name) = r.bestiary_attack_effect("swallow-whole").unwrap().unwrap();
        assert_eq!(key.to_string(), format!("{PACK}:0000000000000000"));
        assert_eq!(name, "Swallow Whole");
        let metrics = r.read_metrics();
        assert_eq!(metrics.sql_statements, 1);
        assert_eq!(metrics.source_body_decodes, 0);
    }

    #[test]
    fn glossary_lookup_requires_verified_scope_and_complete_coverage() {
        for (scope, count) in [(None, 1), (Some("sf2e"), 1), (Some("pf2e"), MAX_ITEMS + 1)] {
            let (r, _fixture) = glossary_fixture(scope, count);
            assert!(r.bestiary_attack_effect("swallow-whole").unwrap().is_none());
        }
    }
}
