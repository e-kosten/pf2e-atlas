use atlas_domain::RecordFamily;
use atlas_domain::metadata::{MetadataPredicate, MetadataSetField, MetadataSetMatch};

use crate::VectorQueryError;
use crate::filters::FilterCompileError;
use crate::vector::compile_vector_knn_query;

#[test]
fn composes_vector_knn_query_from_eligible_records() -> Result<(), Box<dyn std::error::Error>> {
    let filter = atlas_domain::SearchFilterNode::all_of(vec![
        atlas_domain::SearchFilterNode::record_family(RecordFamily::Rule),
        atlas_domain::SearchFilterNode::pack("actions"),
    ]);

    let compiled = compile_vector_knn_query(&[0.25, 0.5, 0.75], Some(&filter), 12, true)?;

    assert!(compiled.sql.contains("WITH eligible(record_key) AS"));
    assert!(compiled.sql.contains("FROM record_vector_index v"));
    assert!(
        compiled
            .sql
            .contains("JOIN document_embedding_cache e ON e.rowid = v.rowid")
    );
    assert!(compiled.sql.contains("v.embedding MATCH ?3"));
    assert!(compiled.sql.contains("AND k = ?4"));
    assert!(
        compiled
            .sql
            .contains("v.rowid IN (\n             SELECT candidate.rowid")
    );
    assert_eq!(compiled.parameters.len(), 4);
    assert_eq!(
        compiled.parameters[0],
        rusqlite::types::Value::Text("rule".to_string())
    );
    assert_eq!(
        compiled.parameters[1],
        rusqlite::types::Value::Text("actions".to_string())
    );
    assert_eq!(
        compiled.parameters[2],
        rusqlite::types::Value::Blob(vec![0, 0, 128, 62, 0, 0, 0, 63, 0, 0, 64, 63])
    );
    assert_eq!(compiled.parameters[3], rusqlite::types::Value::Integer(12));
    Ok(())
}

#[test]
fn parent_only_vector_knn_query_restricts_candidate_units() -> Result<(), Box<dyn std::error::Error>>
{
    let compiled = compile_vector_knn_query(&[1.0], None, 5, false)?;

    assert!(compiled.sql.contains("candidate.unit_kind = 'parent'"));
    Ok(())
}

#[test]
fn rejects_invalid_vector_knn_queries() {
    let unsupported = atlas_domain::SearchFilterNode::metadata(MetadataPredicate::Set {
        field: MetadataSetField::DerivedTags,
        r#match: MetadataSetMatch::Includes {
            value: "area-damage".to_string(),
        },
    });

    assert_eq!(
        compile_vector_knn_query(&[1.0], None, 0, true).unwrap_err(),
        VectorQueryError::InvalidLimit
    );
    assert_eq!(
        compile_vector_knn_query(&[], None, 10, true).unwrap_err(),
        VectorQueryError::EmptyQueryVector
    );
    assert!(matches!(
        compile_vector_knn_query(&[1.0], Some(&unsupported), 10, true).unwrap_err(),
        VectorQueryError::Filter(FilterCompileError::Unsupported { .. })
    ));
}
