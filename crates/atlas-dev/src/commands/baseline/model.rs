use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub(super) struct Pointer {
    pub version: u32,
    pub baseline: String,
}
#[derive(Serialize, Deserialize)]
pub(super) struct Manifest {
    pub version: u32,
    pub id: String,
    pub created_at: String,
    pub build: Value,
    pub database: Value,
    pub assets: Assets,
    pub captured_from: PathBuf,
    pub validation: Value,
}
#[derive(Default, Serialize, Deserialize)]
pub(super) struct Assets {
    pub source: Option<PathBuf>,
    pub embedding_cache: Option<PathBuf>,
}
#[derive(Serialize, Deserialize)]
pub(super) struct Receipt {
    pub version: u32,
    pub baseline: String,
    pub store: PathBuf,
    pub seed_sha256: String,
    pub copy_method: String,
}
