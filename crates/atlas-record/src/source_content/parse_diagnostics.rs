#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContentParseDiagnostics {
    pub dropped_macros: Vec<DroppedContentMacro>,
    pub unsupported_tags: Vec<String>,
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
pub struct DroppedContentMacro {
    pub name: String,
    pub raw: String,
}
