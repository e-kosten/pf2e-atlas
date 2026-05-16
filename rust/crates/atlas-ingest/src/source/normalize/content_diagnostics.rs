#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ContentParseDiagnostics {
    pub(crate) dropped_macros: Vec<DroppedContentMacro>,
    pub(crate) unsupported_tags: Vec<String>,
}

impl ContentParseDiagnostics {
    pub(crate) fn record_unsupported_tag(&mut self, name: &str) {
        if !self
            .unsupported_tags
            .iter()
            .any(|existing| existing == name)
        {
            self.unsupported_tags.push(name.to_string());
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DroppedContentMacro {
    pub(crate) name: String,
    pub(crate) raw: String,
}
