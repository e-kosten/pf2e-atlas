use crate::artifact::inventory::{REQUIRED_REFERENCES, orphan_reference_sql};
use rusqlite::Connection;

use crate::sql::count_sql;
use crate::{
    ArtifactValidationDiagnostic, ArtifactValidationFamily, IndexValidationError,
    artifact::validation::artifact_validation_diagnostic,
};

pub(super) fn validate_relationships(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    for reference in REQUIRED_REFERENCES {
        let sql = orphan_reference_sql(reference);
        let invalid = count_sql(connection, &sql)?;
        if invalid > 0 {
            diagnostics.push(artifact_validation_diagnostic(
                ArtifactValidationFamily::Data,
                format!("relationship check `{}` failed", reference.key),
                Some(reference.key.to_string()),
                Some("0 invalid rows".to_string()),
                Some(format!("{invalid} invalid rows")),
            ));
        }
    }
    for (key, sql) in RELATIONSHIP_POLICY_CHECKS {
        let invalid = count_sql(connection, sql)?;
        if invalid > 0 {
            diagnostics.push(artifact_validation_diagnostic(
                ArtifactValidationFamily::Data,
                format!("relationship check `{key}` failed"),
                Some(key.to_string()),
                Some("0 invalid rows".to_string()),
                Some(format!("{invalid} invalid rows")),
            ));
        }
    }
    Ok(())
}

const RELATIONSHIP_POLICY_CHECKS: &[(&str, &str)] = &[
    (
        "remaster_links.legacy_visibility",
        "SELECT COUNT(*)
         FROM remaster_links l
         JOIN records legacy ON legacy.record_key = l.legacy_record_key
         WHERE legacy.is_default_visible <> 0",
    ),
    (
        "remaster_links.remaster_visibility",
        "SELECT COUNT(*)
         FROM remaster_links l
         JOIN records remaster ON remaster.record_key = l.remaster_record_key
         WHERE remaster.is_default_visible <> 1",
    ),
];
