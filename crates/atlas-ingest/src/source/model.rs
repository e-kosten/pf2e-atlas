use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub(crate) struct Manifest {
    #[serde(default)]
    pub(crate) packs: Vec<ManifestPack>,
    #[serde(default)]
    pub(crate) languages: Vec<ManifestLanguage>,
}
#[derive(Debug, Deserialize)]
pub(crate) struct ManifestLanguage {
    pub(crate) lang: String,
    pub(crate) path: String,
}
#[derive(Debug, Deserialize)]
pub(crate) struct ManifestPack {
    pub(crate) name: String,
    pub(crate) label: String,
    #[serde(rename = "type")]
    pub(crate) document_type: String,
    pub(crate) path: String,
}
#[derive(Debug)]
pub(crate) struct ParsedManifest {
    pub(crate) manifest: Manifest,
    pub(crate) content_hash: String,
}
