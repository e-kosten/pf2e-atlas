//! Explicit saved-domain/application contract mapping.
use atlas_app_model as view;
use atlas_local_state as saved;
pub(super) fn property_input(value: view::RelationshipPropertyView) -> saved::RelationshipProperty {
    match value {
        view::RelationshipPropertyView::Detection => saved::RelationshipProperty::Detection,
        view::RelationshipPropertyView::Awareness => saved::RelationshipProperty::Awareness,
        view::RelationshipPropertyView::Attitude => saved::RelationshipProperty::Attitude,
        view::RelationshipPropertyView::Control => saved::RelationshipProperty::Control,
    }
}
pub(super) fn property_view(value: saved::RelationshipProperty) -> view::RelationshipPropertyView {
    match value {
        saved::RelationshipProperty::Detection => view::RelationshipPropertyView::Detection,
        saved::RelationshipProperty::Awareness => view::RelationshipPropertyView::Awareness,
        saved::RelationshipProperty::Attitude => view::RelationshipPropertyView::Attitude,
        saved::RelationshipProperty::Control => view::RelationshipPropertyView::Control,
    }
}
pub(super) fn detection_input(value: view::DetectionStateView) -> saved::DetectionState {
    match value {
        view::DetectionStateView::Observed => saved::DetectionState::Observed,
        view::DetectionStateView::Hidden => saved::DetectionState::Hidden,
        view::DetectionStateView::Undetected => saved::DetectionState::Undetected,
    }
}
pub(super) fn detection_view(value: saved::DetectionState) -> view::DetectionStateView {
    match value {
        saved::DetectionState::Observed => view::DetectionStateView::Observed,
        saved::DetectionState::Hidden => view::DetectionStateView::Hidden,
        saved::DetectionState::Undetected => view::DetectionStateView::Undetected,
    }
}
pub(super) fn attitude_input(value: view::AttitudeStateView) -> saved::AttitudeState {
    match value {
        view::AttitudeStateView::Helpful => saved::AttitudeState::Helpful,
        view::AttitudeStateView::Friendly => saved::AttitudeState::Friendly,
        view::AttitudeStateView::Indifferent => saved::AttitudeState::Indifferent,
        view::AttitudeStateView::Unfriendly => saved::AttitudeState::Unfriendly,
        view::AttitudeStateView::Hostile => saved::AttitudeState::Hostile,
    }
}
pub(super) fn attitude_view(value: saved::AttitudeState) -> view::AttitudeStateView {
    match value {
        saved::AttitudeState::Helpful => view::AttitudeStateView::Helpful,
        saved::AttitudeState::Friendly => view::AttitudeStateView::Friendly,
        saved::AttitudeState::Indifferent => view::AttitudeStateView::Indifferent,
        saved::AttitudeState::Unfriendly => view::AttitudeStateView::Unfriendly,
        saved::AttitudeState::Hostile => view::AttitudeStateView::Hostile,
    }
}
fn value_input(value: view::RelationshipValueView) -> saved::RelationshipValue {
    match value {
        view::RelationshipValueView::Detection(value) => {
            saved::RelationshipValue::Detection(detection_input(value))
        }
        view::RelationshipValueView::Attitude(value) => {
            saved::RelationshipValue::Attitude(attitude_input(value))
        }
        view::RelationshipValueView::Awareness(value) => saved::RelationshipValue::Awareness(value),
        view::RelationshipValueView::NoControl => saved::RelationshipValue::NoControl,
        view::RelationshipValueView::Control {
            controller_key,
            instructions,
        } => saved::RelationshipValue::Control {
            controller_key,
            instructions,
        },
    }
}
fn provenance_input(value: view::RelationshipProvenanceView) -> saved::RelationshipProvenance {
    match value {
        view::RelationshipProvenanceView::Table => saved::RelationshipProvenance::Table,
        view::RelationshipProvenanceView::Sources { condition_ids } => {
            saved::RelationshipProvenance::Sources { condition_ids }
        }
        view::RelationshipProvenanceView::SourceDefault => {
            saved::RelationshipProvenance::SourceDefault
        }
        view::RelationshipProvenanceView::Source {
            condition_id,
            record_key,
            application_id,
        } => saved::RelationshipProvenance::Source {
            condition_id,
            record_key,
            application_id,
        },
    }
}
pub(super) fn knowledge_input(
    value: view::RelationshipKnowledgeView,
) -> saved::RelationshipKnowledge {
    match value {
        view::RelationshipKnowledgeView::Known { value, provenance } => {
            saved::RelationshipKnowledge::Known {
                value: value_input(value),
                provenance: provenance_input(provenance),
            }
        }
        view::RelationshipKnowledgeView::Unknown { required_facts } => {
            saved::RelationshipKnowledge::Unknown { required_facts }
        }
        view::RelationshipKnowledgeView::Invalidated { prior_source } => {
            saved::RelationshipKnowledge::Invalidated {
                prior_source: provenance_input(prior_source),
            }
        }
        view::RelationshipKnowledgeView::Conflicting { candidates } => {
            saved::RelationshipKnowledge::Conflicting {
                candidates: candidates
                    .into_iter()
                    .map(|c| saved::RelationshipCandidate {
                        value: value_input(c.value),
                        provenance: provenance_input(c.provenance),
                    })
                    .collect(),
            }
        }
    }
}
fn value_view(value: saved::RelationshipValue) -> view::RelationshipValueView {
    match value {
        saved::RelationshipValue::Detection(value) => {
            view::RelationshipValueView::Detection(detection_view(value))
        }
        saved::RelationshipValue::Attitude(value) => {
            view::RelationshipValueView::Attitude(attitude_view(value))
        }
        saved::RelationshipValue::Awareness(value) => view::RelationshipValueView::Awareness(value),
        saved::RelationshipValue::NoControl => view::RelationshipValueView::NoControl,
        saved::RelationshipValue::Control {
            controller_key,
            instructions,
        } => view::RelationshipValueView::Control {
            controller_key,
            instructions,
        },
    }
}
fn provenance_view(value: saved::RelationshipProvenance) -> view::RelationshipProvenanceView {
    match value {
        saved::RelationshipProvenance::Adjudication { sources } => {
            view::RelationshipProvenanceView::Sources {
                condition_ids: sources
                    .into_iter()
                    .map(|source| source.condition_id)
                    .collect(),
            }
        }
        saved::RelationshipProvenance::Table => view::RelationshipProvenanceView::Table,
        saved::RelationshipProvenance::Sources { condition_ids } => {
            view::RelationshipProvenanceView::Sources { condition_ids }
        }
        saved::RelationshipProvenance::SourceDefault => {
            view::RelationshipProvenanceView::SourceDefault
        }
        saved::RelationshipProvenance::Source {
            condition_id,
            record_key,
            application_id,
        } => view::RelationshipProvenanceView::Source {
            condition_id,
            record_key,
            application_id,
        },
    }
}
pub(super) fn knowledge_view(
    value: saved::RelationshipKnowledge,
) -> view::RelationshipKnowledgeView {
    match value {
        saved::RelationshipKnowledge::Known { value, provenance } => {
            view::RelationshipKnowledgeView::Known {
                value: value_view(value),
                provenance: provenance_view(provenance),
            }
        }
        saved::RelationshipKnowledge::Unknown { required_facts } => {
            view::RelationshipKnowledgeView::Unknown { required_facts }
        }
        saved::RelationshipKnowledge::Invalidated { prior_source } => {
            view::RelationshipKnowledgeView::Invalidated {
                prior_source: provenance_view(prior_source),
            }
        }
        saved::RelationshipKnowledge::Conflicting { candidates } => {
            view::RelationshipKnowledgeView::Conflicting {
                candidates: candidates
                    .into_iter()
                    .map(|c| view::RelationshipCandidateView {
                        value: value_view(c.value),
                        provenance: provenance_view(c.provenance),
                    })
                    .collect(),
            }
        }
    }
}
