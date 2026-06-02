#![allow(dead_code)]

use std::process::Output;

use serde_json::Value;

pub fn parse_json(output: &Output) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::from_slice(&output.stdout)?)
}

pub fn ok_data(value: &Value) -> &Value {
    assert_eq!(value["status"], "ok");
    value.get("data").expect("ok envelope should contain data")
}

pub fn parse_ok_data(output: &Output) -> Result<Value, Box<dyn std::error::Error>> {
    let value = parse_json(output)?;
    assert_eq!(value["status"], "ok");
    Ok(value
        .get("data")
        .expect("ok envelope should contain data")
        .clone())
}

pub fn parse_error(
    output: &Output,
    expected_code: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let value = parse_json(output)?;
    assert_eq!(value["status"], "error");
    assert_eq!(value["error"]["code"], expected_code);
    Ok(value)
}

pub fn record_sections(record: &Value) -> Vec<&str> {
    record["sections"]
        .as_array()
        .expect("sections")
        .iter()
        .map(|section| section["kind"].as_str().expect("section kind"))
        .collect()
}
