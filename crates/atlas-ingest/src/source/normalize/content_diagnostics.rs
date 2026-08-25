#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ContentParseDiagnostics {
    pub(crate) dropped_macros: Vec<DroppedContentMacro>,
    pub(crate) unsupported_tags: Vec<String>,
    pub(crate) unsupported_attributes: Vec<UnsupportedContentAttribute>,
    pub(crate) unknown_macros: Vec<String>,
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

    pub(crate) fn record_unsupported_attribute(&mut self, tag: &str, name: &str) {
        let attribute = UnsupportedContentAttribute {
            tag: tag.to_string(),
            name: name.to_string(),
        };
        if !self.unsupported_attributes.contains(&attribute) {
            self.unsupported_attributes.push(attribute);
        }
    }

    pub(crate) fn record_unknown_macro(&mut self, name: &str) {
        if !self.unknown_macros.iter().any(|existing| existing == name) {
            self.unknown_macros.push(name.to_string());
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct UnsupportedContentAttribute {
    pub(crate) tag: String,
    pub(crate) name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DroppedContentMacro {
    pub(crate) name: String,
    pub(crate) raw: String,
}
