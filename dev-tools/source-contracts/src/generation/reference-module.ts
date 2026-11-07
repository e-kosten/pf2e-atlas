import type { ReferenceCatalog } from './reference-definitions.js';

/** Emit source evidence only: selected localized prose remains an ingest input. */
export function generateReferenceModule(catalog: ReferenceCatalog, header: string,
  source: { repository: string; commit: string }): string {
  const quote = (value: string) => JSON.stringify(value).replace(/\\u([0-9a-f]{4})/gi, '\\u{$1}');
  const optional = (value: string | null) => value === null ? 'None' : `Some(${quote(value)})`;
  const row = (entry: ReferenceCatalog['definitions'][number]) => {
    const { namespace, identifier, labelKey, sourceFile, sourceLine } = entry;
    let value = `    FoundryReferenceDefinition::new(${quote(namespace)}, ${quote(identifier)}, ${optional(labelKey)}, ${quote(sourceFile)}, ${sourceLine})`;
    if (entry.descriptionKey !== null) value += `.described_by(${quote(entry.descriptionKey)})`;
    if (entry.displayLabel !== null) value += `.labeled(${quote(entry.displayLabel)})`;
    if (entry.recordKey !== null) {
      if (entry.expectedFamily === null) throw new Error('Mapped reference has no expected family: ' + identifier);
      value += `.mapped_to(${quote(entry.recordKey)}, ${quote(entry.expectedFamily)}, ${optional(entry.requiredLocalizationKey)})`;
    }
    if (entry.excluded) value += '.excluded()';
    return value + ',';
  };
  return `${header}
// Exact reference vocabulary and record pointers.
// ${source.repository.replace(/\.git$/, '')}/tree/${source.commit}
// Upstream copyright: Copyright 2019 Hooking. License: Apache-2.0.
// Localized game prose is not included in this metadata module.
pub const REFERENCE_CATALOG_SHA256: &str = ${quote(catalog.digest)};

/// Sense type, fixed acuity, and whether Foundry defines unlimited range.
pub const SENSE_RULES: &[(&str, Option<&str>, bool)] = &[
${catalog.senseRules.map(rule => `    (${quote(rule.sense)}, ${optional(rule.fixedAcuity)}, ${rule.unlimitedRange}),`).join('\n')}
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoundryReferenceDefinition {
    pub namespace: &'static str,
    pub identifier: &'static str,
    pub label_key: Option<&'static str>,
    pub description_key: Option<&'static str>,
    pub display_label: Option<&'static str>,
    pub record_key: Option<&'static str>,
    pub expected_family: Option<&'static str>,
    pub required_localization_key: Option<&'static str>,
    pub source_file: &'static str,
    pub source_line: u32,
    pub excluded: bool,
}
impl FoundryReferenceDefinition {
    const fn new(
        namespace: &'static str,
        identifier: &'static str,
        label_key: Option<&'static str>,
        source_file: &'static str,
        source_line: u32,
    ) -> Self {
        Self {
            namespace, identifier, label_key, source_file, source_line,
            description_key: None, display_label: None, record_key: None,
            expected_family: None, required_localization_key: None, excluded: false,
        }
    }
    const fn described_by(mut self, key: &'static str) -> Self {
        self.description_key = Some(key);
        self
    }
    const fn labeled(mut self, label: &'static str) -> Self {
        self.display_label = Some(label);
        self
    }
    const fn mapped_to(mut self, key: &'static str, family: &'static str, required: Option<&'static str>) -> Self {
        self.record_key = Some(key);
        self.expected_family = Some(family);
        self.required_localization_key = required;
        self
    }
    const fn excluded(mut self) -> Self {
        self.excluded = true;
        self
    }
}
#[rustfmt::skip]
static DEFINITIONS: &[FoundryReferenceDefinition] = &[
${catalog.definitions.map(row).join('\n')}
];
pub fn reference_definitions() -> &'static [FoundryReferenceDefinition] { DEFINITIONS }

`;
}
