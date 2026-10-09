//! Contributor proof: admit and roundtrip every root through the shared model
//! without linking ingest, runtime, records, indexing or embedding.
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use atlas_foundry_model::{SourceContext, admit_document_source, decode_snapshot, encode_snapshot};
use serde::Serialize;
use serde_json::Value;

#[derive(Default, Serialize)]
struct Counts {
    packs: usize,
    roots: usize,
    modeled: usize,
    partial: usize,
    diagnostics: usize,
    raw_only: usize,
    snapshot_bytes: usize,
}

fn files(path: &Path, output: &mut Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            files(&entry.path(), output)?;
        } else if entry.path().extension().is_some_and(|ext| ext == "json")
            && entry.file_name() != "_folders.json"
        {
            output.push(entry.path());
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let source = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("Pass the pinned PF2e source root")?,
    );
    let manifest: Value = serde_json::from_slice(&fs::read(source.join("static/system.json"))?)?;
    let mut counts = Counts::default();
    for pack in manifest["packs"].as_array().ok_or("Manifest packs")? {
        let kind = pack["type"].as_str().ok_or("Pack type")?;
        let mut paths = Vec::new();
        files(
            &source.join(pack["path"].as_str().ok_or("Pack path")?),
            &mut paths,
        )?;
        paths.sort();
        counts.packs += 1;
        for path in paths {
            let relative = path.strip_prefix(&source)?.to_string_lossy();
            let admission = admit_document_source(
                kind,
                SourceContext::new(relative.as_ref(), relative.as_ref(), "$"),
                &fs::read(&path)?,
            )?;
            counts.roots += 1;
            counts.diagnostics += admission.diagnostics.len();
            counts.partial +=
                usize::from(!admission.diagnostics.is_empty() && admission.model.is_some());
            if let Some(model) = admission.model {
                let encoded = encode_snapshot(&model)?;
                let decoded = decode_snapshot(&encoded)?;
                if decoded != model {
                    return Err(format!("Typed model changed at {relative}").into());
                }
                counts.snapshot_bytes += encoded.len();
                counts.modeled += 1;
            } else {
                counts.raw_only += 1;
            }
        }
    }
    println!("{}", serde_json::to_string_pretty(&counts)?);
    if (
        counts.packs,
        counts.roots,
        counts.partial,
        counts.diagnostics,
        counts.raw_only,
    ) != (87, 25641, 203, 267, 0)
    {
        return Err("Pinned corpus loading outcomes changed".into());
    }
    Ok(())
}
