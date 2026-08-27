use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::error::Error;
use std::path::PathBuf;

use atlas_domain::RecordKey;
use atlas_index::test_support::{
    RecordRoundTripRecordRole, RecordRoundTripRetrievalDisposition,
    record_round_trip_expected_retrieval_projection,
    record_round_trip_persisted_retrieval_projection,
};
use atlas_index::{RecordReadIndex, SqliteIndexReader};
use atlas_record::AtlasRecord;
use serde_json::json;

use crate::index_build_input::index_build_input;
use crate::source_pipeline;

const SOURCE_ROOT_ENV: &str = "PF2E_SOURCE_ROOT";
const ARTIFACT_PATH_ENV: &str = "ATLAS_RECORD_ROUND_TRIP_ARTIFACT";

#[test]
#[ignore]
fn record_key_aligned_round_trip() -> Result<(), Box<dyn Error>> {
    let source_root = required_path(SOURCE_ROOT_ENV)?;
    let artifact_path = required_path(ARTIFACT_PATH_ENV)?;

    let source = source_pipeline::load_foundry_source(&source_root, None)?;
    let input = index_build_input(source);

    let reader = SqliteIndexReader::open_read_only(&artifact_path)?;
    let generation = reader.verified_generation_evidence()?;
    reader.validate_generation_binding()?;

    let expected_projection =
        record_round_trip_expected_retrieval_projection(&input.records, &input.remaster_links)?;
    let persisted_projection = record_round_trip_persisted_retrieval_projection(&reader)?;
    let hydrated = RecordReadIndex::load_record_set(&reader)?;

    reader.validate_generation_binding()?;

    let expected_records = records_by_key(&input.records, "source-expected")?;
    let hydrated_records = records_by_key(&hydrated.records, "public-hydrated")?;
    let expected_keys = expected_records.keys().cloned().collect::<BTreeSet<_>>();
    let persisted_keys = persisted_projection
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let hydrated_keys = hydrated_records.keys().cloned().collect::<BTreeSet<_>>();

    for record_key in expected_keys.difference(&hydrated_keys) {
        println!(
            "{}",
            serde_json::to_string(&json!({
                "kind": "missing_key",
                "record_key": record_key,
                "missing_from": "public_hydration",
            }))?
        );
    }
    for record_key in hydrated_keys.difference(&expected_keys) {
        println!(
            "{}",
            serde_json::to_string(&json!({
                "kind": "extra_key",
                "record_key": record_key,
                "present_in": "public_hydration",
            }))?
        );
    }

    let mut unequal_record_count = 0usize;
    let mut field_difference_count = 0usize;
    for record_key in expected_keys.intersection(&hydrated_keys) {
        let expected = expected_records[record_key];
        let actual = hydrated_records[record_key];
        if expected != actual {
            unequal_record_count += 1;
            field_difference_count += emit_record_field_differences(record_key, expected, actual)?;
        }
    }

    let projection_keys = expected_projection
        .keys()
        .chain(persisted_projection.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut projection_difference_count = 0usize;
    let mut non_ordinary_adjudication_count = 0usize;
    for record_key in projection_keys {
        let expected = expected_projection.get(&record_key);
        let persisted = persisted_projection.get(&record_key);
        let hydrated_record = hydrated_records.get(&record_key).copied();
        let projection_differs = expected != persisted;
        if projection_differs {
            projection_difference_count += 1;
        }
        let requires_adjudication = expected.is_some_and(|row| {
            row.record_role != RecordRoundTripRecordRole::Source
                || row.retrieval_disposition != RecordRoundTripRetrievalDisposition::Ordinary
        });
        if requires_adjudication {
            non_ordinary_adjudication_count += 1;
        }
        if projection_differs || requires_adjudication {
            println!(
                "{}",
                serde_json::to_string(&json!({
                    "kind": if projection_differs {
                        "persisted_projection_difference"
                    } else {
                        "retrieval_projection_adjudication"
                    },
                    "record_key": record_key,
                    "expected": expected,
                    "persisted": persisted,
                    "hydrated": hydrated_record.map(|record| json!({
                        "is_default_visible": record.visibility.visible_by_default(),
                        "visibility_reason": format!("{:?}", record.visibility.reason()),
                    })),
                }))?
            );
        }
    }

    let candidate_owner = if expected_keys == hydrated_keys
        && expected_keys == persisted_keys
        && unequal_record_count == 0
        && projection_difference_count == 0
    {
        "atlas-ingest-validation"
    } else {
        "atlas-index-c1"
    };
    println!(
        "{}",
        serde_json::to_string(&json!({
            "kind": "record_round_trip_summary",
            "generation": generation,
            "expected_record_count": expected_records.len(),
            "persisted_projection_count": persisted_projection.len(),
            "hydrated_record_count": hydrated_records.len(),
            "expected_missing_from_persisted_count": expected_keys
                .difference(&persisted_keys)
                .count(),
            "persisted_extra_from_expected_count": persisted_keys
                .difference(&expected_keys)
                .count(),
            "expected_missing_from_hydrated_count": expected_keys
                .difference(&hydrated_keys)
                .count(),
            "hydrated_extra_from_expected_count": hydrated_keys
                .difference(&expected_keys)
                .count(),
            "aligned_unequal_record_count": unequal_record_count,
            "field_difference_count": field_difference_count,
            "persisted_projection_difference_count": projection_difference_count,
            "non_ordinary_adjudication_count": non_ordinary_adjudication_count,
            "candidate_owner": candidate_owner,
        }))?
    );

    Ok(())
}

fn required_path(name: &'static str) -> Result<PathBuf, Box<dyn Error>> {
    env::var_os(name)
        .map(PathBuf::from)
        .ok_or_else(|| format!("required environment variable `{name}` is not set").into())
}

fn records_by_key<'a>(
    records: &'a [AtlasRecord],
    side: &str,
) -> Result<BTreeMap<RecordKey, &'a AtlasRecord>, Box<dyn Error>> {
    let mut by_key = BTreeMap::new();
    for record in records {
        let record_key = record.identity.key.clone();
        if by_key.insert(record_key.clone(), record).is_some() {
            return Err(format!("{side} records contain duplicate key `{record_key}`").into());
        }
    }
    Ok(by_key)
}

fn emit_record_field_differences(
    record_key: &RecordKey,
    expected: &AtlasRecord,
    hydrated: &AtlasRecord,
) -> Result<usize, serde_json::Error> {
    let mut difference_count = 0usize;
    macro_rules! emit_difference {
        ($field:ident) => {
            if expected.$field != hydrated.$field {
                difference_count += 1;
                println!(
                    "{}",
                    serde_json::to_string(&json!({
                        "kind": "field_difference",
                        "record_key": record_key,
                        "field_path": concat!("AtlasRecord.", stringify!($field)),
                        "expected": format!("{:#?}", expected.$field),
                        "hydrated": format!("{:#?}", hydrated.$field),
                    }))?
                );
            }
        };
    }

    emit_difference!(identity);
    emit_difference!(classification);
    emit_difference!(foundry);
    emit_difference!(provenance);
    emit_difference!(publication);
    emit_difference!(requirements);
    emit_difference!(timing);
    emit_difference!(mechanics);
    emit_difference!(content);
    emit_difference!(variant);
    emit_difference!(visibility);
    Ok(difference_count)
}
