//! Compare the maintained Rust portfolio with raw packets and freshly extracted declarations.
//! Contributor-only probe; no normalization or production ingest policy.
#[path = "support/model_inspection.rs"]
mod inspection;

use std::collections::BTreeMap;
use std::error::Error;
use std::io::{self, BufRead};

use atlas_foundry_model::{self as models, SourceContext, SourceValue};
use serde_json::{Value, json};

// The shared oracle operates on the same public ordered source representation.
mod source_model {
    pub mod parse {
        pub use atlas_foundry_model::is_numeric_key;
    }
    pub mod value {
        pub use atlas_foundry_model::SourceValue;
    }
}
#[path = "support/rule_fidelity.rs"]
mod fidelity;

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = std::env::args()
        .nth(1)
        .ok_or("Pass the temporary generation input path")?;
    let input: Value = serde_json::from_slice(&std::fs::read(input_file)?)?;
    let mut roots = BTreeMap::new();
    for root in input["selection"].as_array().ok_or("Generation roots")? {
        if let Some(key) = root["documentKind"].as_str().or(root["ruleKey"].as_str()) {
            roots.insert(
                key.to_owned(),
                root["valueRef"]
                    .as_str()
                    .ok_or("Root value reference")?
                    .to_owned(),
            );
        }
    }
    let graph = fidelity::FidelityGraph::new(&input);
    let admission_graph = fidelity::FidelityGraph::for_admission(&input);
    let admission = std::env::args().nth(2).as_deref() == Some("--admission");
    for line in io::stdin().lock().lines() {
        let packet: Value = serde_json::from_str(&line?)?;
        let key = packet["key"].as_str().ok_or("Packet key")?;
        let source = packet["source"].as_str().ok_or("Packet source")?;
        let raw: SourceValue = atlas_foundry_model::parse_source_value(source.as_bytes())?;
        let context = SourceContext::new(
            packet["context"]["record_key"]
                .as_str()
                .ok_or("Record key")?,
            packet["context"]["source_path"]
                .as_str()
                .ok_or("Source path")?,
            packet["context"]["json_path"].as_str().ok_or("JSON path")?,
        );
        let json_path = context.json_path.clone();
        if admission {
            macro_rules! admit {
                ($function:ident) => {
                    models::$function(context, source.as_bytes()).map(|admitted| {
                        let retained = admitted.raw == raw;
                        let modeled = admitted.model.is_some();
                        let fidelity = admitted.model.as_ref().map(|model| {
                            admission_graph.compare_at(roots.get(key).ok_or("Missing root")?, &raw, &inspection::to_value(model)?, &json_path).map_err(|error| error.into())
                        }).transpose();
                        // A fidelity mismatch is reported, rather than silently
                        // claiming that keeping a raw root proves typed parity.
                        let fidelity = fidelity.err().map(|error: Box<dyn Error>| error.to_string());
                        json!({"ok":true,"retained":retained,"modeled":modeled,"fidelity":fidelity,"diagnostics":admitted.diagnostics})
                    })
                };
            }
            let result = match key {
                "Actor" => admit!(admit_actor_source_pf2e),
                "Item" => admit!(admit_item_source_pf2e),
                "JournalEntry" => admit!(admit_journal_entry_source),
                "Macro" => admit!(admit_macro_source),
                "RollTable" => admit!(admit_roll_table_source),
                _ => admit!(admit_rule_source),
            };
            let output = match result {
                Ok(output) => output,
                Err(error) => json!({"ok":false,"error":error}),
            };
            println!("{output}");
            continue;
        }
        let result = match key {
            "Actor" => models::parse_actor_source_pf2e(context, source.as_bytes())
                .map(inspection::to_value),
            "Item" => {
                models::parse_item_source_pf2e(context, source.as_bytes()).map(inspection::to_value)
            }
            "JournalEntry" => models::parse_journal_entry_source(context, source.as_bytes())
                .map(inspection::to_value),
            "Macro" => {
                models::parse_macro_source(context, source.as_bytes()).map(inspection::to_value)
            }
            "RollTable" => models::parse_roll_table_source(context, source.as_bytes())
                .map(inspection::to_value),
            _ if roots.contains_key(key) => {
                models::parse_rule_source(context, source.as_bytes()).map(inspection::to_value)
            }
            _ => return Err(format!("Unmodeled packet key: {key}").into()),
        };
        let output = match result {
            Ok(model) => {
                json!({"ok":true,"fidelity":graph.compare(roots.get(key).ok_or("Missing root")?, &raw, &model?).err()})
            }
            Err(error) => json!({"ok":false,"error":error}),
        };
        println!("{output}");
    }
    Ok(())
}
