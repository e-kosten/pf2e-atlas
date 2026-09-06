use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use atlas_ingest::{BuildArtifactOptions, build_artifact};
use serde_json::Value;

struct TemporaryArtifact(std::path::PathBuf);

impl TemporaryArtifact {
    fn new() -> Result<Self, std::io::Error> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "pf2e-atlas-h2-d-spell-fidelity-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root)?;
        Ok(Self(root.join("index.sqlite")))
    }
}

struct TemporarySource(std::path::PathBuf);

impl TemporarySource {
    fn copy_from(source: &Path) -> Result<Self, std::io::Error> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "pf2e-atlas-h2-d-spell-source-{}-{nonce}",
            std::process::id()
        ));
        copy_directory(source, &root)?;
        Ok(Self(root))
    }

    fn mutate(
        &self,
        relative_path: &str,
        pointer: &str,
        value: Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.0.join(relative_path);
        let mut document: Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        *document.pointer_mut(pointer).ok_or_else(|| {
            format!("fixture mutation pointer {pointer} missing from {relative_path}")
        })? = value;
        std::fs::write(path, serde_json::to_vec_pretty(&document)?)?;
        Ok(())
    }
}

impl Drop for TemporarySource {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

impl Drop for TemporaryArtifact {
    fn drop(&mut self) {
        if let Some(root) = self.0.parent() {
            let _ = std::fs::remove_dir_all(root);
        }
    }
}

#[test]
fn heal_rime_and_qi_expose_keyed_spell_json_terminal_and_provenance()
-> Result<(), Box<dyn std::error::Error>> {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../atlas-ingest/tests/fixtures/foundry-source/spell-source-contract");
    let source = TemporarySource::copy_from(&source_root)?;
    source.mutate(
        "packs/spells/heal.json",
        "/system/damage/0/materials",
        serde_json::json!({"unsupported": "base damage leaf"}),
    )?;
    source.mutate(
        "packs/spells/rime-slick.json",
        "/system/heightening/levels/5/damage/0/materials",
        serde_json::json!({"unsupported": "patch damage leaf"}),
    )?;
    source.mutate(
        "packs/spells/qi-blast.json",
        "/system/rules/0/suboptions/0/label",
        serde_json::json!({"unsupported": "rule suboption leaf"}),
    )?;
    let artifact = TemporaryArtifact::new()?;
    build_artifact(BuildArtifactOptions {
        source_root: source.0.clone(),
        output_path: artifact.0.clone(),
        manifest_path: None,
        embedding_model_id: BuildArtifactOptions::default_embedding_model_id(),
        embedding_cache_root: None,
        reuse_embeddings: true,
        embedding_batch_size: 8,
    })?;

    let heal = record_json("spells-srd:rfZpqmj0AIIdkVIs", &artifact.0)?;
    assert_eq!(
        heal["data"]["record"]["spell"]["damage"]["value"][0]["key"],
        "0"
    );
    assert_unsupported_provenance(
        "spells-srd:rfZpqmj0AIIdkVIs",
        &artifact.0,
        "system.damage.0.materials",
        "materials",
        0,
        "object",
        r#"{"unsupported":"base damage leaf"}"#,
        "source_field_drift",
    )?;

    let rime = record_json("spells-srd:Popa5umI3H33levx", &artifact.0)?;
    let layers = rime["data"]["record"]["spell"]["heightening"]["value"]["layers"]
        .as_array()
        .expect("fixed heightening layers");
    assert_eq!(layers[0]["key"], "5");
    assert_eq!(
        layers[0]["patch"]["damage"]["value"]["members"][0]["value"]["formula"]["value"],
        "4d4"
    );
    assert_eq!(layers[1]["key"], "8");
    assert_eq!(
        layers[1]["patch"]["damage"]["value"]["members"][0]["value"]["formula"]["value"],
        "6d4"
    );
    let rime_text = record_text("spells-srd:Popa5umI3H33levx", &artifact.0)?;
    assert!(rime_text.contains("Area: 20 burst"));
    assert!(rime_text.contains("Damage 0: 4d4 cold"));
    assert!(rime_text.contains("Damage 0 apply modifier: false"));
    assert!(rime_text.contains("Area: 30 burst"));
    assert!(rime_text.contains("Damage 0: 6d4 cold"));
    assert_unsupported_provenance(
        "spells-srd:Popa5umI3H33levx",
        &artifact.0,
        "system.heightening.levels.5.damage.0.materials",
        "materials",
        0,
        "object",
        r#"{"unsupported":"patch damage leaf"}"#,
        "source_field_drift",
    )?;

    let qi = record_json("spells-srd:oo7YcRC2gcez81PV", &artifact.0)?;
    assert_eq!(
        qi["data"]["record"]["spell"]["rules"]["value"][0]["authored_key"],
        "RollOption"
    );
    assert_eq!(
        qi["data"]["record"]["spell"]["rules"]["value"][0]["value"]["suboptions"]["value"][0]["value"]
            ["value"],
        "electricity"
    );
    assert_eq!(
        qi["data"]["record"]["spell"]["rules"]["value"][0]["value"]["toggleable"]["value"],
        true
    );
    let qi_text = record_text("spells-srd:oo7YcRC2gcez81PV", &artifact.0)?;
    assert!(
        qi_text.contains("electricity"),
        "missing typed Qi suboption from:\n{qi_text}"
    );
    assert!(
        qi_text.contains("Toggleable: true"),
        "missing typed Qi toggleable state from:\n{qi_text}"
    );
    let qi_provenance = command_output(&[
        "record",
        "provenance",
        "spells-srd:oo7YcRC2gcez81PV",
        "--index",
        artifact.0.to_str().expect("UTF-8 artifact path"),
    ])?;
    assert!(qi_provenance.contains("rule RollOption at system.rules.0 (order 0, known state)"));
    assert!(!qi_provenance.contains("authored_object_json"));
    assert_unsupported_provenance(
        "spells-srd:oo7YcRC2gcez81PV",
        &artifact.0,
        "system.rules.0.suboptions.0.label",
        "label",
        0,
        "object",
        r#"{"unsupported":"rule suboption leaf"}"#,
        "source_field_drift",
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn assert_unsupported_provenance(
    key: &str,
    artifact: &Path,
    expected_path: &str,
    expected_key: &str,
    expected_order: u64,
    expected_shape: &str,
    expected_value: &str,
    expected_reason: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = command_output(&[
        "record",
        "provenance",
        key,
        "--json",
        "--index",
        artifact.to_str().expect("UTF-8 artifact path"),
    ])?;
    assert!(!output.contains("authored_object_json"));
    let value: Value = serde_json::from_str(&output)?;
    let unsupported = value["data"]["spell_provenance"]["unsupported"]
        .as_array()
        .expect("spell unsupported provenance");
    let fact = unsupported
        .iter()
        .find(|fact| fact["source_path"] == expected_path)
        .unwrap_or_else(|| panic!("missing unsupported provenance path {expected_path}"));
    assert_eq!(fact["authored_key"], expected_key);
    assert_eq!(fact["order"], expected_order);
    assert_eq!(fact["value"]["shape"], expected_shape);
    assert_eq!(fact["value"]["value"], expected_value);
    assert_eq!(fact["value"]["reason"], expected_reason);
    Ok(())
}

fn copy_directory(source: &Path, destination: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let destination = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_directory(&entry.path(), &destination)?;
        } else {
            std::fs::copy(entry.path(), destination)?;
        }
    }
    Ok(())
}

fn record_json(key: &str, artifact: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = command_output(&[
        "record",
        "get",
        key,
        "--detail",
        "full",
        "--index",
        artifact.to_str().expect("UTF-8 artifact path"),
        "--json",
    ])?;
    Ok(serde_json::from_str(&output)?)
}

fn record_text(key: &str, artifact: &Path) -> Result<String, Box<dyn std::error::Error>> {
    command_output(&[
        "record",
        "get",
        key,
        "--detail",
        "standard",
        "--index",
        artifact.to_str().expect("UTF-8 artifact path"),
    ])
}

fn command_output(args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["--progress", "never"])
        .args(args)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "atlas command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    if !output.stderr.is_empty() {
        return Err(format!(
            "atlas command emitted stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?)
}
