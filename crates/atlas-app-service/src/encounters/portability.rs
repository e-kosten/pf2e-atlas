use crate::{AppServiceResult, AtlasAppService};
use atlas_domain::RecordKey;
use atlas_local_state::{EncounterDocument, EncounterRecordSource};
use serde::Serialize;
use std::{collections::BTreeMap, io::Read};

#[derive(Debug, Serialize)]
pub struct EncounterImportReport {
    pub encounter_key: String,
    pub slug: String,
    pub missing_records: Vec<RecordKey>,
    pub changed_records: Vec<RecordKey>,
    pub reference_checked: bool,
    pub source_changed: Option<bool>,
}

impl AtlasAppService {
    pub fn export_encounter(&self, encounter_ref: &str) -> AppServiceResult<Vec<u8>> {
        self.with_state(|session| {
            let mut document = session
                .local_state()?
                .encounters()
                .export_document(encounter_ref)?;
            match session.exact_records() {
                Ok(reader) => {
                    document.source_fingerprint = Some(reader.source_fingerprint().into());
                    document.record_sources = reader
                        .summaries(&document.record_keys()?)?
                        .into_iter()
                        .map(|record| EncounterRecordSource {
                            record_key: record.key,
                            content_hash: record.content_hash,
                        })
                        .collect();
                }
                Err(error) if error.reference_unavailable() => {}
                Err(error) => return Err(error.into()),
            }
            Ok(document.encode()?)
        })
    }

    pub fn import_encounter(
        &self,
        input: impl Read,
        slug: Option<String>,
    ) -> AppServiceResult<EncounterImportReport> {
        let document = EncounterDocument::read(input)?;
        let keys = document.record_keys()?;
        self.with_write(move |session| {
            let mut missing_records = Vec::new();
            let mut changed_records = Vec::new();
            let mut source_changed = None;
            let reference_checked = match session.exact_records() {
                Ok(reader) => {
                    source_changed = document
                        .source_fingerprint
                        .as_deref()
                        .map(|original| original != reader.source_fingerprint());
                    let target: BTreeMap<_, _> = reader
                        .summaries(&keys)?
                        .into_iter()
                        .map(|record| (record.key, record.content_hash))
                        .collect();
                    let source: BTreeMap<_, _> = document
                        .record_sources
                        .iter()
                        .map(|record| (&record.record_key, &record.content_hash))
                        .collect();
                    for key in keys {
                        match target.get(&key) {
                            None => missing_records.push(key),
                            Some(hash)
                                if source.get(&key).is_some_and(|original| *original != hash) =>
                            {
                                changed_records.push(key);
                            }
                            Some(_) => {}
                        }
                    }
                    true
                }
                Err(error) if error.reference_unavailable() => {
                    missing_records = keys;
                    false
                }
                Err(error) => return Err(error.into()),
            };
            let encounter = session
                .local_state()?
                .encounters()
                .import_document(document, slug)?;
            Ok(EncounterImportReport {
                encounter_key: encounter.encounter_key,
                slug: encounter.slug,
                missing_records,
                changed_records,
                reference_checked,
                source_changed,
            })
        })
    }
}
