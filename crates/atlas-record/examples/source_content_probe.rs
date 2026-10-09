//! Developer proof over explicitly inventoried HTML fields. No product CLI
//! command, database or ingest dependency. Probe owners are snapshot-local:
//! input packets cannot establish sibling-ID uniqueness across source rebuilds.
use std::{
    collections::BTreeMap,
    error::Error,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};

use atlas_domain::{PackName, RecordId, RecordKey};
use atlas_record::source_content::{
    ContentAudience, ContentReferenceResolver, ContentReferenceTarget, ContentVisibilityRule,
    LocalizationResolver, OwnedContentIdentity, OwnedContentLocator, ResolvedContentReference,
    SourceContentLocator, prepare_source_content,
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct Packet {
    owner: String,
    field: String,
    markup: String,
}

#[derive(Default)]
struct Context {
    strings: BTreeMap<String, String>,
    records: BTreeMap<String, Option<(RecordKey, String)>>,
}

impl LocalizationResolver for Context {
    fn localized_value(&self, key: &str) -> Option<&str> {
        self.strings.get(key).map(String::as_str)
    }
}

impl ContentReferenceResolver for Context {
    fn resolve_reference(
        &self,
        _source: &SourceContentLocator,
        target: &str,
    ) -> Option<ResolvedContentReference> {
        let rest = target.strip_prefix("Compendium.pf2e.")?;
        let (pack, target) = rest.split_once('.')?;
        let target = ["Item.", "Actor.", "JournalEntry.", "RollTable."]
            .iter()
            .find_map(|prefix| target.strip_prefix(prefix))
            .unwrap_or(target);
        let (key, name) = self.records.get(&format!("{pack}.{target}"))?.as_ref()?;
        Some(ResolvedContentReference {
            target: ContentReferenceTarget::Record { key: key.clone() },
            display_name: Some(name.clone()),
        })
    }
}

fn flatten(value: &Value, prefix: &str, output: &mut BTreeMap<String, String>) {
    match value {
        Value::String(text) => {
            output.insert(prefix.into(), text.clone());
        }
        Value::Object(fields) => {
            for (key, value) in fields {
                let key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                flatten(value, &key, output);
            }
        }
        _ => {}
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 4 {
        return Err(
            "usage: source_content_probe HTML-fields.jsonl locale.json records.jsonl output.jsonl"
                .into(),
        );
    }
    let mut context = Context::default();
    flatten(
        &serde_json::from_reader::<_, Value>(File::open(&args[1])?)?,
        "",
        &mut context.strings,
    );
    for line in BufReader::new(File::open(&args[2])?).lines() {
        let row: Value = serde_json::from_str(&line?)?;
        let key = packet_key(row["id"].as_str().ok_or("record id")?)?;
        let name = row["name"].as_str().ok_or("record name")?.to_string();
        for lookup in [
            format!("{}.{}", key.pack(), key.id()),
            format!("{}.{}", key.pack(), name),
        ] {
            context
                .records
                .entry(lookup)
                .and_modify(|value| *value = None)
                .or_insert_with(|| Some((key.clone(), name.clone())));
        }
    }
    let mut output = BufWriter::new(File::create(&args[3])?);
    let mut fields = 0;
    let mut references = 0;
    let mut interactions = 0;
    let mut diagnostics = BTreeMap::<String, usize>::new();
    for (index, line) in BufReader::new(File::open(&args[0])?).lines().enumerate() {
        let packet: Packet = serde_json::from_str(&line?)?;
        let (root, owner) = packet.owner.split_once('/').unwrap_or((&packet.owner, ""));
        let owners = if owner.is_empty() {
            Vec::new()
        } else {
            vec![OwnedContentLocator {
                collection: format!("/{}", owner.split('/').next().ok_or("owner collection")?),
                identity: OwnedContentIdentity::SnapshotLocal { index },
            }]
        };
        let locator = SourceContentLocator {
            record: packet_key(root)?,
            owners,
            field: format!("/{}", packet.field.replace('.', "/")),
        };
        // Explicit source-inspection inputs, not a chosen product default.
        let audience = ContentAudience {
            include_gm: true,
            include_owner: true,
            implicit_check_dc: ContentVisibilityRule::All,
        };
        let prepared = prepare_source_content(
            locator,
            &packet.markup,
            audience,
            Some(&context),
            Some(&context),
        )?;
        fields += 1;
        references += prepared.references.len();
        interactions += prepared.interactions.len();
        for diagnostic in &prepared.diagnostics {
            *diagnostics
                .entry(format!("{:?}", diagnostic.code))
                .or_default() += 1;
        }
        serde_json::to_writer(&mut output, &prepared)?;
        writeln!(&mut output)?;
    }
    output.flush()?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "fields": fields, "references": references, "interactions": interactions,
            "diagnostics": diagnostics,
            "scope": "inventoried HTML field projection; not Foundry/browser or product-ingest acceptance"
        }))?
    );
    Ok(())
}

fn packet_key(value: &str) -> Result<RecordKey, Box<dyn Error>> {
    let (pack, id) = value.split_once('.').ok_or("packet key must be pack.id")?;
    Ok(RecordKey::new(PackName::new(pack)?, RecordId::new(id)?))
}
