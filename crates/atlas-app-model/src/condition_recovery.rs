//! Explicit condition recovery and qualifying table events.
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CheckDegreeView {
    CriticalFailure,
    Failure,
    Success,
    CriticalSuccess,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LifecycleCheckInputView {
    Roll,
    TableDegree { degree: CheckDegreeView },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConditionLifecycleActionView {
    DyingRecovery {
        check: LifecycleCheckInputView,
    },
    Retch {
        condition_id: i64,
        check: LifecycleCheckInputView,
    },
    FrightenedEndTurn,
    StunnedActionsLost {
        condition_id: i64,
        actions_lost: i64,
    },
    ConfusedDamageRecovery {
        consequence_key: String,
        check: Option<LifecycleCheckInputView>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ApplyConditionLifecycleRequest {
    pub boundary_key: Option<String>,
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub participant_key: String,
    pub action: ConditionLifecycleActionView,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct LifecycleCheckResultView {
    pub dc: Option<i64>,
    pub modifier: Option<i64>,
    pub roll: Option<atlas_dice::RollOutcome>,
    pub table_result: Option<i64>,
    pub degree: CheckDegreeView,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ConditionLifecycleResultView {
    pub encounter_key: String,
    pub request_id: String,
    pub check: Option<LifecycleCheckResultView>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PendingConditionRecoveryView {
    ConfusedDamage {
        consequence_key: String,
        participant_key: String,
        condition_ids: Vec<i64>,
        qualifying_damage: Option<bool>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FrightenedRecoveryView {
    EndTurn { minimum: i64 },
    Held,
    SourceDependent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct LifecycleCheckFactsView {
    pub dc: Option<i64>,
    pub modifier: Option<i64>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConditionRecoveryOfferView {
    DyingRecovery {
        check: LifecycleCheckFactsView,
    },
    Retch {
        condition_id: i64,
        check: LifecycleCheckFactsView,
        issues: Vec<crate::EncounterActionIssueView>,
    },
    FrightenedEndTurn,
    Reminder {
        condition_id: i64,
        text: String,
    },
    StunnedActionsLost {
        condition_id: i64,
        actions_lost: Option<i64>,
    },
    ConfusedDamageRecovery {
        consequence_key: String,
        source_condition_ids: Vec<i64>,
        qualifying_damage: Option<bool>,
        check: LifecycleCheckFactsView,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ConditionLifecycleOptionView {
    #[serde(flatten)]
    pub action: ConditionRecoveryOfferView,
    /// Saved sources whose row can expose this recovery action.
    pub condition_ids: Vec<i64>,
    pub label: String,
    pub available: bool,
    pub unavailable_reason: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ConditionLifecycleActionsView {
    pub encounter_key: String,
    pub encounter_revision: i64,
    pub participant_key: String,
    pub actions: Vec<ConditionLifecycleOptionView>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_inputs_accept_roll_or_table_degree_and_reject_raw_totals() {
        assert_eq!(
            serde_json::from_str::<LifecycleCheckInputView>(r#"{"kind":"roll"}"#).unwrap(),
            LifecycleCheckInputView::Roll
        );
        assert_eq!(
            serde_json::from_str::<LifecycleCheckInputView>(
                r#"{"kind":"table_degree","degree":"success"}"#
            )
            .unwrap(),
            LifecycleCheckInputView::TableDegree {
                degree: CheckDegreeView::Success
            }
        );
        assert!(
            serde_json::from_str::<LifecycleCheckInputView>(
                r#"{"kind":"table_result","total":18,"natural":12}"#
            )
            .is_err()
        );
        let historical: LifecycleCheckResultView = serde_json::from_str(
            r#"{"dc":15,"modifier":6,"roll":null,"table_result":18,"degree":"success"}"#,
        )
        .unwrap();
        assert_eq!(historical.table_result, Some(18));
    }
}
