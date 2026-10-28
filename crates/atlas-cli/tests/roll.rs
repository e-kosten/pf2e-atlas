use std::process::Command;

#[test]
fn roll_needs_no_artifact_and_returns_structured_faces() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["roll", "2d20kh1+12", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    assert!(output.stderr.is_empty());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let data = &json["data"];
    assert_eq!(data["formula"], "2d20kh1+12");
    let dice = data["expression"]["left"]["dice"].as_array().unwrap();
    assert_eq!(dice.len(), 2);
    assert!(
        dice.iter()
            .all(|die| die["sides"] == 20 && (1..=20).contains(&die["value"].as_i64().unwrap()))
    );
    assert_eq!(dice.iter().filter(|die| die["kept"] == true).count(), 1);
    let highest = dice
        .iter()
        .map(|die| die["value"].as_i64().unwrap())
        .max()
        .unwrap();
    assert_eq!(
        dice.iter().find(|die| die["kept"] == true).unwrap()["value"],
        highest
    );
    assert_eq!(
        data["total"].as_i64().unwrap(),
        dice.iter()
            .filter(|die| die["kept"] == true)
            .map(|die| die["value"].as_i64().unwrap())
            .sum::<i64>()
            + 12
    );
}

#[test]
fn signed_formula_after_option_separator_has_readable_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["roll", "--", "-4+10"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "-4+10 = 6\n  -4 + 10\n"
    );
}

#[test]
fn invalid_formula_returns_json_error_and_nonzero_status() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["roll", "1000000d20", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("limits")
    );
}
