use super::{EncountersRead, EncountersWrite, command_receipts, receipt_codec};
use crate::{LocalStateError, LocalStateResult};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PcAdditionReceipt {
    pub characters: Vec<String>,
    pub participants: Vec<String>,
}

impl PcAdditionReceipt {
    pub(crate) fn decode(encoded: &str) -> LocalStateResult<Self> {
        receipt_codec::decode(encoded)
    }
}

impl EncountersRead<'_> {
    pub fn pc_addition_receipt(
        &self,
        encounter: &str,
        request: &str,
        intent: &serde_json::Value,
    ) -> LocalStateResult<Option<PcAdditionReceipt>> {
        self.get(encounter)?
            .ok_or_else(|| LocalStateError::EncounterNotFound(encounter.into()))?;
        command_receipts::recover(
            self.connection,
            self.owner_id,
            encounter,
            command_receipts::Family::PcAddition,
            request,
            intent,
        )?
        .map(command_receipts::StoredReceipt::pc_addition)
        .transpose()
    }
}

impl EncountersWrite<'_> {
    pub fn record_pc_addition(
        &self,
        encounter: &str,
        request: &str,
        intent: &serde_json::Value,
        receipt: &PcAdditionReceipt,
    ) -> LocalStateResult<()> {
        command_receipts::insert(
            self.connection,
            self.owner_id,
            encounter,
            request,
            intent,
            command_receipts::Outcome::PcAddition(receipt),
        )
    }
}
