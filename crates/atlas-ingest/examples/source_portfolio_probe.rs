//! Compare the maintained Rust portfolio with raw packets and saved declarations.
//! Contributor-only probe; no normalization or production ingest policy.
use std::collections::BTreeMap;
use std::error::Error;
use std::io::{self, BufRead};
use std::path::Path;

use atlas_ingest::source_model::{self as models, SourceContext, SourceValue};
use serde_json::{Value, json};

// The shared oracle operates on the same public ordered source representation.
mod source_model {
    pub mod parse {
        pub use atlas_ingest::source_model::is_numeric_key;
    }
    pub mod value {
        pub use atlas_ingest::SourceValue;
    }
}
#[path = "support/rule_fidelity.rs"]
mod fidelity;

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_file = std::env::args()
        .nth(1)
        .ok_or("Pass the generation manifest path")?;
    let manifest: Value = serde_json::from_slice(&std::fs::read(&manifest_file)?)?;
    let directory = Path::new(&manifest_file)
        .parent()
        .ok_or("Manifest directory")?;
    let mut nodes = Vec::new();
    let mut roots = BTreeMap::new();
    for module in manifest["modules"].as_array().ok_or("Manifest modules")? {
        let snapshot: Value = serde_json::from_slice(&std::fs::read(
            directory.join(module["file"].as_str().ok_or("Module file")?),
        )?)?;
        nodes.extend(
            snapshot["nodes"]
                .as_array()
                .ok_or("Snapshot nodes")?
                .iter()
                .cloned(),
        );
        for root in snapshot["roots"].as_array().ok_or("Snapshot roots")? {
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
    }
    let graph = fidelity::FidelityGraph::new(&json!({"nodes":nodes}));
    for line in io::stdin().lock().lines() {
        let packet: Value = serde_json::from_str(&line?)?;
        let key = packet["key"].as_str().ok_or("Packet key")?;
        let source = packet["source"].as_str().ok_or("Packet source")?;
        let raw: SourceValue = serde_json::from_str(source)?;
        let context = SourceContext::new(
            packet["context"]["record_key"]
                .as_str()
                .ok_or("Record key")?,
            packet["context"]["source_path"]
                .as_str()
                .ok_or("Source path")?,
            packet["context"]["json_path"].as_str().ok_or("JSON path")?,
        );
        let result = match key {
            "Actor" => models::parse_actor_source_pf2e(context, source.as_bytes())
                .map(serde_json::to_value),
            "Item" => {
                models::parse_item_source_pf2e(context, source.as_bytes()).map(serde_json::to_value)
            }
            "JournalEntry" => models::parse_journal_entry_source(context, source.as_bytes())
                .map(serde_json::to_value),
            "Macro" => {
                models::parse_macro_source(context, source.as_bytes()).map(serde_json::to_value)
            }
            "RollTable" => models::parse_roll_table_source(context, source.as_bytes())
                .map(serde_json::to_value),
            _ if roots.contains_key(key) => {
                models::parse_rule_source(context, source.as_bytes()).map(serde_json::to_value)
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
