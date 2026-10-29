//! Durable receipt framing belongs to storage, not the application response DTO.
use crate::{LocalStateError, LocalStateResult};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
const VERSION: u32 = 1;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope<T> {
    version: u32,
    payload: T,
}
pub(super) fn encode<T: Serialize>(payload: &T) -> LocalStateResult<String> {
    Ok(serde_json::to_string(&Envelope {
        version: VERSION,
        payload,
    })?)
}
pub(super) fn decode<T: DeserializeOwned>(encoded: &str) -> LocalStateResult<T> {
    let envelope: Envelope<serde_json::Value> = serde_json::from_str(encoded)?;
    if envelope.version != VERSION {
        return Err(LocalStateError::UnsupportedReceiptVersion(envelope.version));
    }
    Ok(serde_json::from_value(envelope.payload)?)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_version_is_rejected_before_payload_interpretation() {
        assert!(matches!(
            decode::<u32>(r#"{"version":2,"payload":"new shape"}"#),
            Err(LocalStateError::UnsupportedReceiptVersion(2))
        ));
        let encoded = encode(&7u32).unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&encoded).unwrap(),
            serde_json::json!({"version":1,"payload":7})
        );
        assert_eq!(decode::<u32>(&encoded).unwrap(), 7);
        assert!(decode::<u32>("7").is_err());
    }
}
