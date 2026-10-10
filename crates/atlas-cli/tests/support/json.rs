use serde_json::Value;
use std::process::Output;
pub fn parse_ok_data(output: &Output) -> Result<Value, Box<dyn std::error::Error>> {
    let value: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(value["status"], "ok");
    Ok(value["data"].clone())
}
