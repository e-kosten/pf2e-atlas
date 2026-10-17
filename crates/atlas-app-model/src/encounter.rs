use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{RecordPresentationView, RecordSummaryView};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterSummaryView {
    pub encounter_key: String,
    pub slug: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    pub status: EncounterStatusView,
    pub round_number: i64,
    pub participant_count: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterIndexView {
    pub encounters: Vec<EncounterSummaryView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterConditionCatalogView {
    pub conditions: Vec<EncounterConditionDefinitionView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ConditionDurationUnit {
    Rounds,
    Minutes,
    Hours,
    Days,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ConditionTurnEvent {
    Start,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct ConditionTimingView {
    pub participant_key: String,
    pub event: ConditionTurnEvent,
}

/// For rounds, amount is the remaining count; zero awaits explicit removal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct ConditionDurationView {
    pub amount: i64,
    pub unit: ConditionDurationUnit,
    pub timing: Option<ConditionTimingView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EncounterTurnReviewView {
    pub encounter_key: String,
    pub expected_revision: i64,
    pub current_participant_key: Option<String>,
    pub next_participant_key: Option<String>,
    pub reminders: Vec<EncounterTurnReminderView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EncounterTurnReminderView {
    PersistentDamage {
        participant_key: String,
        participant_name: String,
        condition_id: i64,
        condition_name: String,
        note: Option<String>,
    },
    DurationEnding {
        participant_key: String,
        participant_name: String,
        condition_id: i64,
        condition_name: String,
        timing: ConditionTimingView,
    },
    DurationEnded {
        participant_key: String,
        participant_name: String,
        condition_id: i64,
        condition_name: String,
    },
    DurationReminder {
        participant_key: String,
        participant_name: String,
        condition_id: i64,
        condition_name: String,
        duration: ConditionDurationView,
    },
    TimingUnavailable {
        participant_key: String,
        participant_name: String,
        condition_id: i64,
        condition_name: String,
        duration: ConditionDurationView,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AdvanceEncounterTurnRequest {
    pub expected_revision: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterConditionDefinitionView {
    pub condition_ref: String,
    pub name: String,
    pub automation_level: EncounterConditionAutomationLevelView,
    pub applies_to: Vec<EncounterConditionApplicabilityView>,
    pub categories: Vec<EncounterConditionCategoryView>,
    pub has_value: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub default_value: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EncounterConditionAutomationLevelView {
    Automated,
    Tracked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EncounterConditionApplicabilityView {
    Creature,
    Hazard,
    Object,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EncounterConditionCategoryView {
    ActionEconomy,
    Attitude,
    DeathAndDying,
    Detection,
    ObjectState,
    RuntimeState,
    StatModifier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EncounterStatusView {
    Draft,
    Running,
    Complete,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EncounterParticipantKindView {
    Creature,
    Hazard,
    Pc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EncounterParticipantSideView {
    Pc,
    Ally,
    Enemy,
    Neutral,
    Hazard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EncounterParticipantVariantView {
    Normal,
    Elite,
    Weak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum StatModifierTypeView {
    Adjustment,
    Status,
    Circumstance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ActionBudgetView {
    pub actions: RuntimeCountView,
    pub reactions: RuntimeCountView,
    pub can_act: RuntimeCapabilityView,
    pub can_react: RuntimeCapabilityView,
    pub notes: Vec<RuntimeEffectNoteView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeCountView {
    pub label: String,
    pub base_value: i64,
    pub adjusted_value: i64,
    pub segments: Vec<RuntimeCountSegmentView>,
    pub adjustments: Vec<RuntimeAdjustmentView>,
    pub suppressed_adjustments: Vec<RuntimeAdjustmentView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeCountSegmentView {
    pub label: String,
    pub value: i64,
    pub restricted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeAdjustmentView {
    pub source: String,
    pub label: String,
    pub value: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeCapabilityView {
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeEffectNoteView {
    pub source: String,
    pub label: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct StatModifierView {
    pub source: String,
    pub label: String,
    pub modifier_type: StatModifierTypeView,
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct UnappliedEffectView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub condition_id: Option<i64>,
    pub source: String,
    pub label: String,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EncounterParticipantStatusView {
    Active,
    Unresolved,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreateEncounterRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterCreateView {
    pub encounter: EncounterSummaryView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterDetailView {
    pub revision: i64,
    pub artifact_generation: Option<String>,
    pub encounter: EncounterSummaryView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub current_turn_participant_key: Option<String>,
    pub participants: Vec<EncounterParticipantView>,
}

/// A complete selected reference in one current participant's runtime context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterReferenceRequest {
    pub artifact_generation: String,
    pub selection: crate::RecordDetailRequest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct UpdateEncounterRequest {
    pub encounter_key: String,
    pub slug: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
    pub status: EncounterStatusView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterMutationView {
    pub encounter_key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterParticipantView {
    pub revision: i64,
    pub hp_origin: String,
    pub variant_origin: String,
    pub participant_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub record_key: Option<String>,
    pub participant_kind: EncounterParticipantKindView,
    pub participant_variant: EncounterParticipantVariantView,
    pub status: EncounterParticipantStatusView,
    pub position: i64,
    pub display_name: String,
    pub side: EncounterParticipantSideView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub initiative: Option<i64>,
    pub initiative_order: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_hp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub current_hp: Option<i64>,
    pub temporary_hp: i64,
    pub defeated: bool,
    pub hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
    pub note_hint: Option<String>,
    pub conditions: Vec<EncounterParticipantConditionView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub presentation: Option<RecordPresentationView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub record: Option<RecordSummaryView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterParticipantConditionView {
    pub condition_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub condition_key: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_participant_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub duration: Option<ConditionDurationView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct AddEncounterRecordParticipantRequest {
    pub encounter_ref: String,
    pub record_ref: String,
    pub quantity: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub initiative: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct AddEncounterManualParticipantRequest {
    pub encounter_ref: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_hp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub current_hp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub initiative: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct UpdateEncounterParticipantRequest {
    pub expected_revision: i64,
    // Explicit manual HP edit even when the entered value is unchanged.
    #[serde(default)]
    pub hp_edit: bool,
    #[serde(default)]
    pub max_hp_edit: bool,
    #[serde(default)]
    pub variant_edit: bool,
    // Discard an explicit capacity override and derive it from authored data.
    #[serde(default)]
    pub use_derived_max: bool,
    pub participant_key: String,
    pub display_name: String,
    pub side: EncounterParticipantSideView,
    pub participant_variant: EncounterParticipantVariantView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub initiative: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_hp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub current_hp: Option<i64>,
    pub temporary_hp: i64,
    pub defeated: bool,
    pub hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum ReorderEncounterParticipantPlacementView {
    Before,
    After,
}

/// A supplied exact value; null explicitly clears current/maximum availability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct EncounterHpValueInput {
    #[serde(deserialize_with = "Option::deserialize")]
    pub input: Option<String>,
}

/// A deliberate HP operation, interpreted against the checked current participant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EncounterHpAction {
    Damage {
        amount: String,
    },
    Heal {
        amount: String,
    },
    SetCurrent {
        #[serde(deserialize_with = "Option::deserialize")]
        input: Option<String>,
    },
    SetMaximum {
        #[serde(deserialize_with = "Option::deserialize")]
        input: Option<String>,
    },
    SetTemporary {
        input: String,
    },
    /// Correct all supplied values atomically; omitted fields retain their intent.
    SetValues {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        current: Option<EncounterHpValueInput>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        maximum: Option<EncounterHpValueInput>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        temporary: Option<String>,
    },
    UseDerivedMaximum,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct ApplyEncounterHpActionRequest {
    pub participant_key: String,
    pub expected_revision: i64,
    pub action: EncounterHpAction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ReorderEncounterParticipantRequest {
    pub participant_key: String,
    pub target_participant_key: String,
    pub placement: ReorderEncounterParticipantPlacementView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct AddEncounterParticipantConditionRequest {
    pub participant_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub condition_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_participant_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub duration: Option<ConditionDurationView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct UpdateEncounterParticipantConditionRequest {
    pub expected_encounter_revision: i64,
    pub condition_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub condition_ref: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_participant_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub duration: Option<ConditionDurationView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SetEncounterTurnRequest {
    pub encounter_ref: String,
    pub participant_key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct DeleteEncounterView {
    pub encounter_key: String,
    pub slug: String,
    pub deleted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn condition_duration_rejects_fractional_missing_and_retired_inputs() {
        let request = json!({"participant_key":"p","name":"Custom","duration":{"amount":2,"unit":"rounds","timing":{"participant_key":"p","event":"end"}}});
        let parsed =
            serde_json::from_value::<AddEncounterParticipantConditionRequest>(request.clone())
                .unwrap();
        assert_eq!(parsed.duration.unwrap().amount, 2);
        for duration in [
            json!({"unit":"rounds"}),
            json!({"amount":1.5,"unit":"minutes"}),
            json!({"amount":null,"unit":"hours"}),
            json!({"amount":2,"unit":"seconds"}),
            json!({"amount":2,"unit":"rounds","timing":{"participant_key":"p","event":"sometime"}}),
        ] {
            let mut bad = request.clone();
            bad["duration"] = duration;
            assert!(
                serde_json::from_value::<AddEncounterParticipantConditionRequest>(bad).is_err()
            );
        }
        assert!(
            serde_json::from_value::<AddEncounterParticipantConditionRequest>(
                json!({"participant_key":"p","name":"Custom","duration_rounds":2})
            )
            .is_err()
        );
    }

    #[test]
    fn hp_action_requires_revision_and_explicit_correction_input() {
        assert!(
            serde_json::from_value::<ApplyEncounterHpActionRequest>(
                json!({"participant_key":"one","action":{"kind":"damage","amount":"1+2"}})
            )
            .is_err()
        );
        assert!(
            serde_json::from_value::<EncounterHpAction>(json!({"kind":"set_current"})).is_err()
        );
        assert!(
            serde_json::from_value::<EncounterHpAction>(json!({"kind":"set_maximum"})).is_err()
        );
        assert_eq!(
            serde_json::from_value::<EncounterHpAction>(json!({"kind":"set_current","input":null}))
                .unwrap(),
            EncounterHpAction::SetCurrent { input: None }
        );
        assert!(
            serde_json::from_value::<EncounterHpAction>(json!({"kind":"damage","amount":12}))
                .is_err()
        );
        assert!(
            serde_json::from_value::<EncounterHpAction>(
                json!({"kind":"heal","amount":"2","defeated":false})
            )
            .is_err()
        );
    }

    #[test]
    fn atomic_hp_values_distinguish_omitted_fields_from_explicit_unknown() {
        let parsed: EncounterHpAction = serde_json::from_value(
            json!({"kind":"set_values","current":{"input":null},"temporary":"0"}),
        )
        .unwrap();
        assert_eq!(
            parsed,
            EncounterHpAction::SetValues {
                current: Some(EncounterHpValueInput { input: None }),
                maximum: None,
                temporary: Some("0".into())
            }
        );
        assert_eq!(
            serde_json::to_value(parsed).unwrap(),
            json!({"kind":"set_values","current":{"input":null},"temporary":"0"})
        );
        for bad in [
            json!({"kind":"set_values","current":{}}),
            json!({"kind":"set_values","maximum":{"input":5}}),
            json!({"kind":"set_values","temporary":0}),
            json!({"kind":"set_values","current":{"input":"4","value":4}}),
        ] {
            assert!(serde_json::from_value::<EncounterHpAction>(bad).is_err());
        }
    }

    #[test]
    fn participant_edit_revision_is_required_and_round_trips_as_a_json_integer() {
        let mut wire = json!({
            "participant_key":"participant_one", "display_name":"One", "side":"enemy",
            "participant_variant":"normal", "temporary_hp":0, "defeated":false, "hidden":false,
        });
        assert!(serde_json::from_value::<UpdateEncounterParticipantRequest>(wire.clone()).is_err());
        wire["expected_revision"] = json!(7);
        let request: UpdateEncounterParticipantRequest = serde_json::from_value(wire).unwrap();
        assert_eq!(request.expected_revision, 7);
        let encoded = serde_json::to_value(request).unwrap();
        assert_eq!(encoded["expected_revision"], json!(7));
        assert!(encoded["expected_revision"].is_i64());
        assert!(UpdateEncounterParticipantRequest::decl().contains("expected_revision: bigint"));
        assert!(!UpdateEncounterParticipantRequest::decl().contains("expected_revision?"));
        assert!(EncounterParticipantView::decl().contains("revision: bigint"));
    }

    #[test]
    fn mutation_receipt_contains_only_durable_encounter_identity() {
        let receipt = EncounterMutationView {
            encounter_key: "encounter_one".into(),
        };
        assert_eq!(
            serde_json::to_value(receipt).unwrap(),
            json!({"encounter_key":"encounter_one"})
        );
    }
}
