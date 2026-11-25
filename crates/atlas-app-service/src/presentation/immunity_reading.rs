//! Canonical reading targets for immunity labels on an admitted PF2e stat block.
use crate::definitions::{HelpContext, record_navigation};
use atlas_app_model::{ReferenceDestinationView, ReferenceHelpView};
use atlas_domain::{DefinitionKey, DefinitionNamespace, RecordKey};
use atlas_foundry_model::generated::immunity_reading::{
    self, SWARM_MIND_RECORD_KEY, SWARM_MIND_RULE_KEY, Target,
};
use atlas_search::{AtlasRetrievalService, GetRecordsRequest};
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct ImmunityTargets {
    conditions: BTreeMap<RecordKey, String>,
    definitions: BTreeMap<String, DefinitionKey>,
}

impl ImmunityTargets {
    pub(super) fn collect(&mut self, token: &str) {
        let key = match immunity_reading::target(token) {
            Some(Target::Condition(key)) => {
                self.conditions.insert(
                    RecordKey::parse(key).expect("generated condition record key"),
                    token.to_owned(),
                );
                return;
            }
            Some(Target::Trait(slug)) => {
                DefinitionKey::new(DefinitionNamespace::Trait, slug).expect("generated trait slug")
            }
            Some(Target::SwarmMind) => SWARM_MIND_RULE_KEY.parse().expect("generated rule key"),
            None => return,
        };
        self.definitions.insert(token.to_owned(), key);
    }

    pub(super) fn definition_keys(&self) -> impl Iterator<Item = DefinitionKey> + '_ {
        self.definitions.values().cloned()
    }

    pub(super) fn resolve(
        mut self,
        retrieval: &AtlasRetrievalService,
        context: &HelpContext<'_>,
    ) -> crate::AppServiceResult<BTreeMap<String, ReferenceHelpView>> {
        let mut links = BTreeMap::new();
        let keys: Vec<_> = self
            .conditions
            .keys()
            .filter(|key| retrieval.is_pf2e_record(key))
            .cloned()
            .collect();
        for record in retrieval
            .get_records(GetRecordsRequest { record_keys: &keys })?
            .records
        {
            if record.document_kind == "Item"
                && record
                    .source_type
                    .as_value()
                    .is_some_and(|family| family == "condition")
                && let Some(token) = self.conditions.remove(&record.key)
            {
                links.insert(
                    token,
                    ReferenceHelpView {
                        definition_key: None,
                        destination: ReferenceDestinationView::Record {
                            navigation: record_navigation(
                                record.key,
                                vec![],
                                retrieval.source_fingerprint(),
                            )
                            .into(),
                        },
                    },
                );
            }
        }
        for (token, key) in self.definitions {
            let Some(mut help) = context.help(&key) else {
                continue;
            };
            match &help.destination {
                ReferenceDestinationView::Definition { .. }
                    if key.namespace() == DefinitionNamespace::Trait => {}
                ReferenceDestinationView::Record { navigation }
                    if key.namespace() == DefinitionNamespace::Rule
                        && navigation.record_key == SWARM_MIND_RECORD_KEY =>
                {
                    help.definition_key = None;
                }
                _ => continue,
            }
            links.insert(token, help);
        }
        links.retain(|token, _| {
            context
                .label(DefinitionNamespace::ImmunityType, token)
                .is_some()
        });
        Ok(links)
    }
}
