use std::collections::{BTreeMap, BTreeSet};

use atlas_record::FoundryRecordType;
use thiserror::Error;

use crate::{
    AssignmentEvidence, AssignmentFile, OntologySuggestion, OntologySuggestionFile,
    RecordTagAssignments, TagApplicabilityClause, TagCatalogFile, TagDefinition, TagId,
    TagPresentation,
};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TagValidationError {
    #[error("tag catalog must contain at least one tag")]
    EmptyCatalog,
    #[error("tag `{tag_id}` has an empty {field}")]
    EmptyTagField { tag_id: TagId, field: &'static str },
    #[error(
        "tag `{tag_id}` display subgroup `{subgroup:?}` belongs to `{expected_group:?}`, not `{actual_group:?}`"
    )]
    InvalidTagDisplaySubgroup {
        tag_id: TagId,
        actual_group: crate::TagDisplayGroup,
        subgroup: crate::TagDisplaySubgroup,
        expected_group: crate::TagDisplayGroup,
    },
    #[error("duplicate tag id `{0}`")]
    DuplicateTagId(TagId),
    #[error("tag `{0}` has no applicability clauses")]
    EmptyApplicability(TagId),
    #[error("tag `{tag_id}` has applicability clause {clause_index} with no constraints")]
    UnconstrainedApplicabilityClause { tag_id: TagId, clause_index: usize },
    #[error(
        "tag `{tag_id}` applicability clause {clause_index} repeats foundry record type `{foundry_record_type}`"
    )]
    DuplicateFoundryRecordType {
        tag_id: TagId,
        clause_index: usize,
        foundry_record_type: String,
    },
    #[error("assignment file repeats record `{0}`")]
    DuplicateAssignedRecord(String),
    #[error("record `{record_key}` has an empty {field}")]
    EmptyAssignmentRecordField {
        record_key: String,
        field: &'static str,
    },
    #[error("record `{record_key}` repeats assignment for tag `{tag_id}`")]
    DuplicateAssignedTag { record_key: String, tag_id: TagId },
    #[error("record `{record_key}` assignment references unknown tag `{tag_id}`")]
    UnknownAssignedTag { record_key: String, tag_id: TagId },
    #[error("record `{record_key}` assignment for tag `{tag_id}` has invalid evidence: {reason}")]
    InvalidEvidence {
        record_key: String,
        tag_id: TagId,
        reason: &'static str,
    },
    #[error("ontology suggestion `{label}` has an empty {field}")]
    EmptyOntologySuggestionField { label: String, field: &'static str },
    #[error(
        "ontology suggestion `{label}` display subgroup `{subgroup:?}` belongs to `{expected_group:?}`, not `{actual_group:?}`"
    )]
    InvalidOntologySuggestionDisplaySubgroup {
        label: String,
        actual_group: crate::TagDisplayGroup,
        subgroup: crate::TagDisplaySubgroup,
        expected_group: crate::TagDisplayGroup,
    },
    #[error("ontology suggestion `{label}` has no applicability clauses")]
    EmptyOntologySuggestionApplicability { label: String },
    #[error(
        "ontology suggestion `{label}` has applicability clause {clause_index} with no constraints"
    )]
    UnconstrainedOntologySuggestionApplicabilityClause { label: String, clause_index: usize },
    #[error(
        "ontology suggestion `{label}` applicability clause {clause_index} repeats foundry record type `{foundry_record_type}`"
    )]
    DuplicateOntologySuggestionFoundryRecordType {
        label: String,
        clause_index: usize,
        foundry_record_type: String,
    },
}

pub fn validate_catalog_file(file: &TagCatalogFile) -> Result<(), TagValidationError> {
    if file.tags.is_empty() {
        return Err(TagValidationError::EmptyCatalog);
    }
    let mut seen = BTreeSet::new();
    for definition in &file.tags {
        validate_definition(definition)?;
        if !seen.insert(definition.id.clone()) {
            return Err(TagValidationError::DuplicateTagId(definition.id.clone()));
        }
    }
    Ok(())
}

pub fn validate_assignment_file(file: &AssignmentFile) -> Result<(), TagValidationError> {
    validate_assignment_file_against_catalog(file, &BTreeMap::new())
}

pub fn validate_ontology_suggestion_file(
    file: &OntologySuggestionFile,
) -> Result<(), TagValidationError> {
    for suggestion in &file.suggestions {
        validate_ontology_suggestion(suggestion)?;
    }
    Ok(())
}

pub fn validate_assignments_against_catalog(
    file: &AssignmentFile,
    catalog: &crate::TagCatalog,
) -> Result<(), TagValidationError> {
    let definitions = catalog
        .definitions()
        .iter()
        .cloned()
        .map(|definition| (definition.id.clone(), definition))
        .collect::<BTreeMap<_, _>>();
    validate_assignment_file_against_catalog(file, &definitions)
}

fn validate_assignment_file_against_catalog(
    file: &AssignmentFile,
    catalog: &BTreeMap<TagId, TagDefinition>,
) -> Result<(), TagValidationError> {
    let mut seen_records = BTreeSet::new();
    for record in &file.records {
        let record_key = record.record_key.to_string();
        if !seen_records.insert(record_key.clone()) {
            return Err(TagValidationError::DuplicateAssignedRecord(record_key));
        }
        validate_record_assignments(record, catalog)?;
    }
    Ok(())
}

fn validate_definition(definition: &TagDefinition) -> Result<(), TagValidationError> {
    reject_empty(&definition.id, "label", &definition.label)?;
    reject_empty(&definition.id, "description", &definition.description)?;
    validate_tag_display(&definition.id, &definition.display)?;
    if definition.applicability.any_of.is_empty() {
        return Err(TagValidationError::EmptyApplicability(
            definition.id.clone(),
        ));
    }
    for (index, clause) in definition.applicability.any_of.iter().enumerate() {
        validate_applicability_clause(&definition.id, index, clause)?;
    }
    if definition.guidance.applies_when.is_empty() {
        return Err(TagValidationError::EmptyTagField {
            tag_id: definition.id.clone(),
            field: "guidance.applies_when",
        });
    }
    if definition.guidance.does_not_apply_when.is_empty() {
        return Err(TagValidationError::EmptyTagField {
            tag_id: definition.id.clone(),
            field: "guidance.does_not_apply_when",
        });
    }
    Ok(())
}

fn validate_applicability_clause(
    tag_id: &TagId,
    clause_index: usize,
    clause: &TagApplicabilityClause,
) -> Result<(), TagValidationError> {
    if clause.record_kinds.is_empty()
        && clause.foundry_record_types.is_empty()
        && clause.required_facts.is_empty()
        && clause.excluded_facts.is_empty()
    {
        return Err(TagValidationError::UnconstrainedApplicabilityClause {
            tag_id: tag_id.clone(),
            clause_index,
        });
    }
    let mut foundry_types = BTreeSet::new();
    for foundry_type in &clause.foundry_record_types {
        let foundry_type = foundry_record_type_key(foundry_type);
        if !foundry_types.insert(foundry_type.clone()) {
            return Err(TagValidationError::DuplicateFoundryRecordType {
                tag_id: tag_id.clone(),
                clause_index,
                foundry_record_type: foundry_type,
            });
        }
    }
    Ok(())
}

fn validate_ontology_suggestion(suggestion: &OntologySuggestion) -> Result<(), TagValidationError> {
    let label = suggestion.label.clone();
    reject_empty_ontology(&label, "label", &suggestion.label)?;
    reject_empty_ontology(&label, "rationale", &suggestion.rationale)?;
    validate_ontology_display(&label, &suggestion.display)?;
    if suggestion.applicability.any_of.is_empty() {
        return Err(TagValidationError::EmptyOntologySuggestionApplicability { label });
    }
    for (index, clause) in suggestion.applicability.any_of.iter().enumerate() {
        validate_ontology_applicability_clause(&suggestion.label, index, clause)?;
    }
    Ok(())
}

fn validate_ontology_applicability_clause(
    label: &str,
    clause_index: usize,
    clause: &TagApplicabilityClause,
) -> Result<(), TagValidationError> {
    if clause.record_kinds.is_empty()
        && clause.foundry_record_types.is_empty()
        && clause.required_facts.is_empty()
        && clause.excluded_facts.is_empty()
    {
        return Err(
            TagValidationError::UnconstrainedOntologySuggestionApplicabilityClause {
                label: label.to_string(),
                clause_index,
            },
        );
    }
    let mut foundry_types = BTreeSet::new();
    for foundry_type in &clause.foundry_record_types {
        let foundry_type = foundry_record_type_key(foundry_type);
        if !foundry_types.insert(foundry_type.clone()) {
            return Err(
                TagValidationError::DuplicateOntologySuggestionFoundryRecordType {
                    label: label.to_string(),
                    clause_index,
                    foundry_record_type: foundry_type,
                },
            );
        }
    }
    Ok(())
}

fn validate_tag_display(
    tag_id: &TagId,
    display: &TagPresentation,
) -> Result<(), TagValidationError> {
    let Some(subgroup) = display.subgroup else {
        return Ok(());
    };
    let expected_group = subgroup.group();
    if display.group != expected_group {
        return Err(TagValidationError::InvalidTagDisplaySubgroup {
            tag_id: tag_id.clone(),
            actual_group: display.group,
            subgroup,
            expected_group,
        });
    }
    Ok(())
}

fn validate_ontology_display(
    label: &str,
    display: &TagPresentation,
) -> Result<(), TagValidationError> {
    let Some(subgroup) = display.subgroup else {
        return Ok(());
    };
    let expected_group = subgroup.group();
    if display.group != expected_group {
        return Err(
            TagValidationError::InvalidOntologySuggestionDisplaySubgroup {
                label: label.to_string(),
                actual_group: display.group,
                subgroup,
                expected_group,
            },
        );
    }
    Ok(())
}

fn validate_record_assignments(
    record: &RecordTagAssignments,
    catalog: &BTreeMap<TagId, TagDefinition>,
) -> Result<(), TagValidationError> {
    reject_empty_assignment_record(record, "name", &record.name)?;
    let mut seen_tags = BTreeSet::new();
    for assignment in &record.tags {
        if !seen_tags.insert(assignment.tag_id.clone()) {
            return Err(TagValidationError::DuplicateAssignedTag {
                record_key: record.record_key.to_string(),
                tag_id: assignment.tag_id.clone(),
            });
        }
        if !catalog.is_empty() && !catalog.contains_key(&assignment.tag_id) {
            return Err(TagValidationError::UnknownAssignedTag {
                record_key: record.record_key.to_string(),
                tag_id: assignment.tag_id.clone(),
            });
        }
        for evidence in &assignment.evidence {
            validate_evidence(record, &assignment.tag_id, evidence)?;
        }
    }
    Ok(())
}

fn reject_empty_assignment_record(
    record: &RecordTagAssignments,
    field: &'static str,
    value: &str,
) -> Result<(), TagValidationError> {
    if value.trim().is_empty() {
        return Err(TagValidationError::EmptyAssignmentRecordField {
            record_key: record.record_key.to_string(),
            field,
        });
    }
    Ok(())
}

fn validate_evidence(
    record: &RecordTagAssignments,
    tag_id: &TagId,
    evidence: &AssignmentEvidence,
) -> Result<(), TagValidationError> {
    let reason = match evidence {
        AssignmentEvidence::ContentExcerpt { path, quote } => {
            empty_reason(path, "content path").or_else(|| empty_reason(quote, "quote"))
        }
        AssignmentEvidence::PresentationSection { section, summary } => {
            empty_reason(section, "presentation section")
                .or_else(|| empty_reason(summary, "summary"))
        }
        AssignmentEvidence::NormalizedFact { value, .. } => empty_reason(value, "fact value"),
        AssignmentEvidence::TagGuidanceMatch {
            signal,
            explanation,
        } => empty_reason(signal, "guidance signal")
            .or_else(|| empty_reason(explanation, "guidance explanation")),
        AssignmentEvidence::SourceReference {
            relationship,
            summary,
            ..
        } => {
            empty_reason(relationship, "relationship").or_else(|| empty_reason(summary, "summary"))
        }
    };
    if let Some(reason) = reason {
        return Err(TagValidationError::InvalidEvidence {
            record_key: record.record_key.to_string(),
            tag_id: tag_id.clone(),
            reason,
        });
    }
    Ok(())
}

fn reject_empty(
    tag_id: &TagId,
    field: &'static str,
    value: &str,
) -> Result<(), TagValidationError> {
    if value.trim().is_empty() {
        return Err(TagValidationError::EmptyTagField {
            tag_id: tag_id.clone(),
            field,
        });
    }
    Ok(())
}

fn reject_empty_ontology(
    label: &str,
    field: &'static str,
    value: &str,
) -> Result<(), TagValidationError> {
    if value.trim().is_empty() {
        return Err(TagValidationError::EmptyOntologySuggestionField {
            label: label.to_string(),
            field,
        });
    }
    Ok(())
}

fn empty_reason(value: &str, reason: &'static str) -> Option<&'static str> {
    value.trim().is_empty().then_some(reason)
}

fn foundry_record_type_key(value: &FoundryRecordType) -> String {
    value.as_str().to_string()
}

#[cfg(test)]
mod tests {
    use crate::{
        AssignmentFile, TagCatalog, TagValidationError, assignment_file_from_str,
        catalog_file_from_str,
    };

    use super::validate_assignments_against_catalog;

    const CATALOG: &str = r#"
tags:
  - id: problem.counteract_magic
    label: Counteract Magic
    description: Helps suppress, dispel, counteract, or remove active magical effects.
    display:
      group: ProblemSolving
      subgroup: Countermeasure
    applicability:
      any_of:
        - record_kinds: [spell, equipment]
    guidance:
      applies_when:
        - The record explicitly counteracts or dispels active magic.
      does_not_apply_when:
        - The record only deals magical damage.

  - id: equipment.party_support.defense
    label: Defensive Party Support
    description: Equipment that protects allies or improves group defense.
    display:
      group: BuildAndEquipmentSupport
      subgroup: PartyRole
    applicability:
      any_of:
        - record_kinds: [equipment]
          foundry_record_types: [armor, shield, consumable, equipment]
    guidance:
      applies_when:
        - The item improves durability, mitigation, or ally protection.
      does_not_apply_when:
        - The item is only personally offensive.
"#;

    const ASSIGNMENTS: &str = r#"
records:
  - record_key: spells-srd:dispel-magic
    name: Dispel Magic
    tags:
      - tag_id: problem.counteract_magic
        evidence:
          - kind: content_excerpt
            path: description
            quote: You attempt to counteract a spell or magical effect.
          - kind: normalized_fact
            field: record_kind
            value: spell
          - kind: tag_guidance_match
            signal: Counteracts or dispels an active magical effect.
            explanation: The record explicitly uses counteract language.
        note: High-confidence assignment.

  - record_key: equipment-srd:plain-clothing
    name: Plain Clothing
    tags: []
    note: Reviewed; no high-signal tags justified.
"#;

    #[test]
    fn parses_catalog_yaml_into_typed_definitions() {
        let catalog = catalog_file_from_str(CATALOG).expect("catalog parses");

        assert_eq!(catalog.tags.len(), 2);
        assert_eq!(catalog.tags[0].id.as_str(), "problem.counteract_magic");
        assert_eq!(
            catalog.tags[1].applicability.any_of[0].foundry_record_types[1].as_str(),
            "shield"
        );
    }

    #[test]
    fn parses_record_centered_assignments_and_reviewed_empty_entries() {
        let assignments = assignment_file_from_str(ASSIGNMENTS).expect("assignments parse");

        assert_eq!(assignments.records.len(), 2);
        assert_eq!(assignments.records[0].tags.len(), 1);
        assert!(assignments.records[1].tags.is_empty());
    }

    #[test]
    fn validates_assignments_against_catalog() {
        let catalog = TagCatalog::new(catalog_file_from_str(CATALOG).expect("catalog parses"))
            .expect("catalog validates");
        let assignments = assignment_file_from_str(ASSIGNMENTS).expect("assignments parse");

        validate_assignments_against_catalog(&assignments, &catalog)
            .expect("assignments reference known tags");
    }

    #[test]
    fn catalog_merges_valid_ids_from_multiple_files() {
        let second_catalog = CATALOG
            .replace("problem.counteract_magic", "problem.remove_affliction")
            .replace("Counteract Magic", "Remove Affliction")
            .replace("counteract_magic", "remove_affliction")
            .replace(
                "equipment.party_support.defense",
                "equipment.party_support.mobility",
            )
            .replace("Defensive Party Support", "Mobility Party Support")
            .replace("defense_support", "mobility_support");
        let catalog = TagCatalog::from_files([
            catalog_file_from_str(CATALOG).expect("first catalog parses"),
            catalog_file_from_str(&second_catalog).expect("second catalog parses"),
        ])
        .expect("catalog files merge");

        let ids = catalog.ids().map(|id| id.as_str()).collect::<Vec<_>>();
        assert_eq!(ids.len(), 4);
        assert!(ids.contains(&"problem.counteract_magic"));
        assert!(ids.contains(&"equipment.party_support.defense"));
        assert!(catalog.contains_id(&"problem.remove_affliction".parse().expect("id parses")));
        assert!(
            catalog
                .get(
                    &"equipment.party_support.mobility"
                        .parse()
                        .expect("id parses")
                )
                .is_some()
        );
    }

    #[test]
    fn catalog_rejects_duplicate_ids_across_files() {
        let error = TagCatalog::from_files([
            catalog_file_from_str(CATALOG).expect("first catalog parses"),
            catalog_file_from_str(CATALOG).expect("second catalog parses"),
        ])
        .expect_err("duplicate ids across files are rejected");

        assert!(matches!(error, TagValidationError::DuplicateTagId(_)));
    }

    #[test]
    fn rejects_duplicate_catalog_ids() {
        let duplicate = CATALOG.replace(
            "equipment.party_support.defense",
            "problem.counteract_magic",
        );

        let error = catalog_file_from_str(&duplicate).expect_err("duplicate ids are rejected");
        assert!(matches!(
            error,
            crate::TagYamlError::Validation(TagValidationError::DuplicateTagId(_))
        ));
    }

    #[test]
    fn rejects_catalog_display_subgroups_under_wrong_group() {
        let invalid = CATALOG.replace("group: ProblemSolving", "group: HazardAndObstacle");

        let error = catalog_file_from_str(&invalid).expect_err("display pair is rejected");
        assert!(matches!(
            error,
            crate::TagYamlError::Validation(TagValidationError::InvalidTagDisplaySubgroup { .. })
        ));
    }

    #[test]
    fn rejects_ontology_suggestion_display_subgroups_under_wrong_group() {
        let invalid = r#"
suggestions:
  - proposed_id: hazard.countermeasure.noise
    label: Noise Countermeasure
    display:
      group: HazardAndObstacle
      subgroup: Countermeasure
    applicability:
      any_of:
        - record_kinds: [hazard, spell, equipment]
    rationale: This needs a valid presentation pair.
    triggering_record_key: hazards:resonant-alarm
    follow_up_research_needed: true
"#;

        let error = crate::ontology_suggestion_file_from_str(invalid)
            .expect_err("display pair is rejected");
        assert!(matches!(
            error,
            crate::TagYamlError::Validation(
                TagValidationError::InvalidOntologySuggestionDisplaySubgroup { .. }
            )
        ));
    }

    #[test]
    fn rejects_unknown_assigned_tags_when_catalog_is_supplied() {
        let catalog = TagCatalog::new(catalog_file_from_str(CATALOG).expect("catalog parses"))
            .expect("catalog validates");
        let assignments = AssignmentFile {
            records: vec![crate::RecordTagAssignments {
                record_key: "spells-srd:mystery".parse().expect("record key parses"),
                name: "Mystery".to_string(),
                tags: vec![crate::TagAssignment {
                    tag_id: "problem.unknown".parse().expect("tag id parses"),
                    evidence: Vec::new(),
                    note: None,
                }],
                note: None,
            }],
        };

        let error = validate_assignments_against_catalog(&assignments, &catalog)
            .expect_err("unknown tag is rejected");
        assert!(matches!(
            error,
            TagValidationError::UnknownAssignedTag { .. }
        ));
    }

    #[test]
    fn rejects_duplicate_record_entries() {
        let duplicate = r#"
records:
  - record_key: spells-srd:one
    name: One
    tags: []
  - record_key: spells-srd:one
    name: One Again
    tags: []
"#;

        let error = assignment_file_from_str(duplicate).expect_err("duplicate record is rejected");
        assert!(matches!(
            error,
            crate::TagYamlError::Validation(TagValidationError::DuplicateAssignedRecord(_))
        ));
    }

    #[test]
    fn rejects_assignment_records_without_names() {
        let missing_name = r#"
records:
  - record_key: spells-srd:one
    tags: []
"#;

        let error = assignment_file_from_str(missing_name).expect_err("name is required");
        assert!(matches!(error, crate::TagYamlError::Parse(_)));
    }

    #[test]
    fn rejects_unknown_assignment_fields() {
        let unknown_field = r#"
records:
  - record_key: spells-srd:one
    name: One
    unexpected: true
    tags: []
"#;

        let error = assignment_file_from_str(unknown_field).expect_err("unknown field is rejected");
        assert!(matches!(error, crate::TagYamlError::Parse(_)));
    }
}
