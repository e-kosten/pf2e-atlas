//! Emit one equipment variant outcome per input key from a private reference snapshot.

use atlas_domain::{RecordKey, SearchProfileId};
use atlas_index::SqliteIndexReader;
use atlas_search::{AtlasRetrievalService, VariantGroupOutcome, VariantGroupRequest};
use atlas_storage::Workspace;
use serde_json::json;
use std::io::{BufRead, BufWriter, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let [path, schema]: [String; 2] = std::env::args()
        .skip(1)
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| "usage: equipment_variant_audit <workspace.sqlite> <workspace schema>")?;
    let session = Workspace::at(path)?.read(schema.parse()?)?;
    let reader = SqliteIndexReader::from_session(session)?;
    let retrieval =
        AtlasRetrievalService::from_index(reader, SearchProfileId::new("bge-base"), None)?;
    let mut output = BufWriter::new(std::io::stdout().lock());
    for key in std::io::stdin().lock().lines() {
        let key = RecordKey::parse(&key?)?;
        let outcome = match retrieval.variant_group(VariantGroupRequest { record_key: &key })? {
            VariantGroupOutcome::Content(group) => json!({
                "kind": "content",
                "family": group.base_name,
                "members": group.variants.iter().map(|member| &member.key).collect::<Vec<_>>(),
                "verified": group.evidence.first().is_some_and(|e| e.naming_convention == "reviewed_membership"),
                "qualifiers": group.evidence.iter().map(|e| &e.qualifier).collect::<Vec<_>>(),
                "ambiguous": group.ambiguous,
                "truncated": group.truncated,
            }),
            VariantGroupOutcome::Missing => json!({ "kind": "missing" }),
            VariantGroupOutcome::Empty => json!({ "kind": "empty" }),
            VariantGroupOutcome::Unavailable { reason } => {
                json!({ "kind": "unavailable", "reason": reason })
            }
        };
        serde_json::to_writer(&mut output, &json!({ "key": key, "outcome": outcome }))?;
        writeln!(output)?;
    }
    output.flush()?;
    Ok(())
}
