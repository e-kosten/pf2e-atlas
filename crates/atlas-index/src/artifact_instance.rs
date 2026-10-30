/// Durable workspace identity and the reference revision visible in one snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactInstance {
    workspace_key: String,
    reference_revision: u64,
}

impl ArtifactInstance {
    pub(crate) fn new(workspace_key: String, reference_revision: u64) -> Self {
        Self {
            workspace_key,
            reference_revision,
        }
    }
    pub fn generation(&self) -> String {
        format!("{}:{}", self.workspace_key, self.reference_revision)
    }
    pub fn workspace_key(&self) -> &str {
        &self.workspace_key
    }
    pub fn reference_revision(&self) -> u64 {
        self.reference_revision
    }
}
