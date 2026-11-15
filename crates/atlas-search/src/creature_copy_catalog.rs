use atlas_domain::RecordKey;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreatureCopyRecord {
    pub key: RecordKey,
    pub name: String,
    pub sha256: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreatureCopyDeclaration {
    pub canonical: CreatureCopyRecord,
    pub copies: Vec<CreatureCopyRecord>,
    pub remaster: bool,
    pub source_note: String,
}
