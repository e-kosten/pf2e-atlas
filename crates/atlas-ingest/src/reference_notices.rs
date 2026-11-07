//! Source notice capture is distinct from record bytes and from supplemental data.
use crate::IngestError;
use atlas_index::ReferenceNotice;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};

pub(crate) const SOURCE_POLICY: &[u8] = include_bytes!("../reference-data/source-provenance.json");
pub(crate) const BUNDLED_NOTICES: &[u8] = include_bytes!("../reference-data/notices.json");
pub(crate) fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(crate) struct CapturedReferenceNotices {
    pub(crate) notices: BTreeMap<String, ReferenceNotice>,
    pub(crate) identity: String,
}
impl CapturedReferenceNotices {
    pub(crate) fn capture(
        root: &Path,
        package: &str,
        revision: Option<&str>,
    ) -> Result<Self, IngestError> {
        #[derive(Deserialize)]
        struct Notice {
            id: String,
            text: String,
            source_url: Option<String>,
        }
        #[derive(Deserialize)]
        struct Base {
            schema: u32,
            notices: Vec<Notice>,
        }
        let mut notices = BTreeMap::new();
        let mut identity = Sha256::new();
        identity.update(SOURCE_POLICY);
        identity.update(BUNDLED_NOTICES);
        if package == "pf2e" {
            let base: Base = serde_json::from_slice(BUNDLED_NOTICES)
                .map_err(|error| IngestError::SourceSelectionFailed(error.to_string()))?;
            if base.schema != 1 {
                return Err(IngestError::SourceSelectionFailed(
                    "unsupported reference notice schema".into(),
                ));
            }
            for notice in base.notices {
                if notice.text.trim().is_empty()
                    || notices
                        .insert(
                            notice.id.clone(),
                            ReferenceNotice {
                                id: notice.id,
                                sha256: hash(notice.text.as_bytes()),
                                text: notice.text,
                                source_url: notice.source_url,
                            },
                        )
                        .is_some()
                {
                    return Err(IngestError::SourceSelectionFailed(
                        "empty or duplicate bundled reference notice".into(),
                    ));
                }
            }
            for (name, id, required) in [
                ("ORCLicense.md", "foundry-orc", "TX 9-307-067"),
                ("OpenGameLicense.md", "foundry-ogl", "OPEN GAME LICENSE"),
            ] {
                identity.update(id.as_bytes());
                let path = root.join("static/licenses").join(name);
                // Missing/unreadable optional notice disables its new standalone prose,
                // not ordinary record admission. The exact absent state is still input.
                if let Ok(text) = fs::read_to_string(&path) {
                    let sha256 = hash(text.as_bytes());
                    identity.update(sha256.as_bytes());
                    if text.to_uppercase().contains(&required.to_uppercase()) {
                        notices.insert(id.into(), ReferenceNotice { id: id.into(), sha256, text, source_url: revision.map(|revision| format!("https://github.com/foundryvtt/pf2e/blob/{revision}/static/licenses/{name}")) });
                    }
                } else {
                    identity.update(b"unavailable");
                }
            }
        }
        Ok(Self {
            notices,
            identity: format!("{:x}", identity.finalize()),
        })
    }
}
