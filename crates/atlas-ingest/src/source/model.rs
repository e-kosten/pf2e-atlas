use serde::Deserialize;
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Manifest {
    // Compatibility previously read these as ordinary JSON fields: unknown
    // shapes stay permissive and duplicate unknown keys retain the last value.
    #[serde(flatten)]
    pub(crate) metadata: std::collections::BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub(crate) packs: Vec<ManifestPack>,
    #[serde(default)]
    pub(crate) languages: Vec<ManifestLanguage>,
}
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ManifestLanguage {
    pub(crate) lang: String,
    pub(crate) path: String,
}
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ManifestPack {
    // Keep optional declarations permissive, like root manifest metadata. An
    // unsupported explicit system affects reference applicability, not admission.
    #[serde(flatten)]
    pub(crate) metadata: std::collections::BTreeMap<String, serde_json::Value>,
    pub(crate) name: String,
    pub(crate) label: String,
    #[serde(rename = "type")]
    pub(crate) document_type: String,
    pub(crate) path: String,
}
#[derive(Debug, Clone)]
pub(crate) struct ParsedManifest {
    pub(crate) manifest: Manifest,
    pub(crate) content_hash: String,
}
