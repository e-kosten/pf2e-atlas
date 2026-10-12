use std::sync::Arc;

/// Process-local identity of a successfully opened immutable artifact.
///
/// Equality requires both the same open file and complete preparation context.
/// Retaining the handle keeps its filesystem identity alive; it is not a
/// transport identifier or a stronger guarantee than `same_file` provides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactInstance {
    file: Arc<same_file::Handle>,
    context_hash: String,
}

impl ArtifactInstance {
    pub(crate) fn new(file: same_file::Handle, context_hash: String) -> Self {
        Self {
            file: Arc::new(file),
            context_hash,
        }
    }
}
