//! Capture selected remaster statements without treating names as record identity.
use crate::{IngestError, source_fingerprint::SourceCapture};
use atlas_domain::RecordKey;
use atlas_record::{
    remaster::*,
    source_content::{ContentAudience, ContentVisibilityRule},
    source_record::{
        FieldAvailability, SourceBackedRecord, SourceContentOutcome, SourceContentStatus,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
fn error(e: impl std::fmt::Display) -> IngestError {
    IngestError::SourceSelectionFailed(e.to_string())
}
pub(crate) fn selected_journal(
    record: &SourceBackedRecord,
    content: &[SourceContentOutcome],
    audience: ContentAudience,
) -> Result<(Vec<RemasterDeclaration>, Vec<RemasterSelectedField>), IngestError> {
    let mut declarations = vec![];
    let mut fields = vec![];
    for outcome in content {
        let visible = match outcome.visibility {
            ContentVisibilityRule::All => true,
            ContentVisibilityRule::Gm => audience.include_gm,
            ContentVisibilityRule::Owner => audience.include_owner,
            ContentVisibilityRule::None => false,
        };
        if !visible || !matches!(outcome.visibility_availability, FieldAvailability::Value) {
            continue;
        }
        if let SourceContentStatus::Prepared(prepared) = &outcome.status {
            let section = record
                .node_at(&prepared.locator.owners)
                .and_then(|n| n.name().value().cloned())
                .ok_or_else(|| error("remaster section name unavailable"))?;
            declarations.extend(journal_declarations(&section, prepared).map_err(error)?);
            fields.push(RemasterSelectedField {
                locator: prepared.locator.clone(),
                authored_sha256: prepared.authored_markup_sha256.clone(),
                references: prepared.references.clone(),
            });
        }
    }
    Ok((declarations, fields))
}
pub(crate) fn captured_inputs(
    root: &Path,
    capture: &SourceCapture,
    revision: &str,
) -> Result<Vec<RemasterSourceInput>, IngestError> {
    let directory = root.join("src/module/migration/migrations");
    let mut result = vec![];
    for (name, kind) in [
        (
            "850-flat-footed-to-off-guard.ts",
            RemasterInputKind::Migration850,
        ),
        (
            "887-redirect-spell-links.ts",
            RemasterInputKind::Migration887,
        ),
    ] {
        let path = directory.join(name);
        if !capture.paths(&directory).contains(&path) {
            continue;
        }
        result.push(RemasterSourceInput::new(
            kind,
            RemasterOrigin::Foundry,
            revision.into(),
            crate::source::discovery::relative_source_path(root, &path),
            capture.read(&path)?,
        ));
    }
    result.push(RemasterSourceInput::new(
        RemasterInputKind::AtlasCorrespondence,
        RemasterOrigin::AtlasReviewed,
        "atlas-remaster-correspondence/v2".into(),
        "crates/atlas-record/src/remaster/correspondences.json".into(),
        ATLAS_CORRESPONDENCE_BYTES.to_vec(),
    ));
    Ok(result)
}
pub(crate) fn aliases(
    declarations: &[RemasterDeclaration],
    fields: &[RemasterSelectedField],
    names: &BTreeMap<RecordKey, String>,
    families: &BTreeMap<RecordKey, Option<String>>,
    products: &BTreeSet<RecordKey>,
    sources: &[RemasterSourceInput],
) -> Result<Vec<RemasterAlias>, IngestError> {
    let facts = families
        .iter()
        .filter(|(key, _)| products.contains(*key))
        .map(|(key, family)| {
            (
                key.clone(),
                RemasterRecordFacts {
                    name: names.get(key).cloned(),
                    source_type: family.clone(),
                },
            )
        })
        .collect();
    let bindings = bind_remaster_declarations(declarations, &facts, fields).map_err(error)?;
    Ok(derive_remaster_aliases(
        declarations,
        &bindings,
        &facts,
        sources,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unavailable_excluded_outcomes_and_hidden_fragments_never_become_declarations() {
        use atlas_foundry_model::{SourceContext, admit_document_source};
        let bytes=serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","name":"Remaster Changes","pages":[{"_id":"bbbbbbbbbbbbbbbb","name":"Equipment","type":"text","text":{"format":1,"content":"<table><tr><th>Old</th><th>New</th></tr><tr data-visibility='gm'><td>Secret old name</td><td>@UUID[Compendium.pf2e.items.Item.Secret]</td></tr><tr><td>Public old name</td><td>@UUID[Compendium.pf2e.items.Item.Public]</td></tr></table>"}}]}).to_string();
        let source = admit_document_source(
            "JournalEntry",
            SourceContext::new("fixture", "journal.json", "$"),
            bytes.as_bytes(),
        )
        .unwrap()
        .model
        .unwrap();
        let record = SourceBackedRecord::new("journals", source).unwrap();
        let audience = ContentAudience {
            include_gm: false,
            include_owner: false,
            implicit_check_dc: ContentVisibilityRule::None,
        };
        let mut outcomes =
            atlas_record::source_record::prepare_record_content(&record, audience, None, None);
        let (declarations, fields) = selected_journal(&record, &outcomes, audience).unwrap();
        assert_eq!(declarations.len(), 1);
        let serialized = serde_json::to_string(&declarations).unwrap();
        assert!(serialized.contains("Public old name"));
        assert!(!serialized.contains("Secret"));
        assert_eq!(fields.len(), 1);
        outcomes[0].visibility_availability = FieldAvailability::Null;
        assert!(
            selected_journal(&record, &outcomes, audience)
                .unwrap()
                .0
                .is_empty()
        );
        outcomes[0].visibility_availability = FieldAvailability::Value;
        outcomes[0].visibility = ContentVisibilityRule::None;
        assert!(
            selected_journal(&record, &outcomes, audience)
                .unwrap()
                .0
                .is_empty()
        );
    }
}
