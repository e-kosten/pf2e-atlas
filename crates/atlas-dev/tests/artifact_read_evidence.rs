//! Local read evidence; fresh connections do not imply an OS-cold page cache.
use atlas_domain::RecordKey;
use atlas_index::SqliteIndexReader;
use serde_json::json;
use std::{fs, path::Path, time::Instant};

fn operation(reader: &SqliteIndexReader, name: &str) -> usize {
    match name {
        "actor_saves_iwr" => {
            let query = atlas_index::parse_where(
                "actor.saves.reflex >= 20 && actor.resistances.exists(r,r.type == 'fire')",
            )
            .unwrap();
            reader
                .eligible_keys(&query, None, false, 0, 100)
                .unwrap()
                .keys
                .len()
        }
        "same_child_spell" => {
            let query=atlas_index::parse_where("actor.items.exists(i,i.source.type == 'spell' && i.spell.rank >= 3 && 'arcane' in i.spell.traditions)").unwrap();
            reader
                .eligible_keys(&query, None, false, 0, 100)
                .unwrap()
                .keys
                .len()
        }
        "lexical_definition" => reader
            .lexical_root_candidates(
                "Ghoul Fever",
                &atlas_index::parse_where("true").unwrap(),
                None,
            )
            .unwrap()
            .len(),
        "exact_name" => reader.lookup_name_or_alias("Fireball").unwrap().len(),
        "selected_detail" => {
            let key: RecordKey = "pathfinder-bestiary:LHHgGSs0ELCR4CYK".parse().unwrap();
            let record = reader.read_source_record(&key).unwrap().unwrap();
            let mut fields = vec![];
            record.visit_content_selections(|owners, selection| {
                if owners.is_empty()
                    && selection.field != "/name"
                    && matches!(
                        selection.text,
                        atlas_record::source_record::SourceFieldView::Value(_)
                    )
                {
                    fields.push(atlas_record::source_content::SourceContentLocator {
                        record: key.clone(),
                        owners: owners.to_vec(),
                        field: selection.field.into(),
                    })
                }
            });
            reader.read_content(&fields).unwrap().fields.len()
        }
        _ => unreachable!(),
    }
}
#[test]
#[ignore = "requires completed full local artifact; run explicitly"]
fn public_reader_warm_and_fresh_connection_evidence() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = root.join("scratch/ingest-validation/full-lexical.sqlite");
    let reader = SqliteIndexReader::open_read_only(&path).unwrap();
    let mut operations = vec![];
    for name in [
        "actor_saves_iwr",
        "same_child_spell",
        "lexical_definition",
        "exact_name",
        "selected_detail",
    ] {
        let count = operation(&reader, name);
        let mut warm = vec![];
        let mut fresh = vec![];
        for _ in 0..20 {
            let time = Instant::now();
            assert_eq!(operation(&reader, name), count);
            warm.push(time.elapsed().as_micros());
        }
        for _ in 0..20 {
            let time = Instant::now();
            let connection = SqliteIndexReader::open_read_only(&path).unwrap();
            assert_eq!(operation(&connection, name), count);
            fresh.push(time.elapsed().as_micros());
        }
        operations.push(json!({"name":name,"result_count":count,"warm_microseconds":warm,"fresh_connection_including_open_microseconds":fresh}));
    }
    fs::write(root.join("scratch/ingest-validation/read-evidence.json"),serde_json::to_vec_pretty(&json!({"basis":"Debug-profile public reader calls; warm same connection and fresh read-only connection including initialization; OS page cache not cleared, no cold-cache claim. Includes shared CEL validation and selected source decode where specified.","context":reader.context(),"statistics":reader.statistics().unwrap(),"operations":operations})).unwrap()).unwrap();
}
