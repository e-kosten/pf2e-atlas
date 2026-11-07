import type { ReferenceCatalog } from './reference-definitions.js';

/** Emit source evidence only: selected localized prose remains an ingest input. */
export function generateReferenceModule(catalog: ReferenceCatalog, header: string,
  source: { repository: string; commit: string }): string {
  const quote = (value: string) => JSON.stringify(value).replace(/\\u([0-9a-f]{4})/gi, '\\u{$1}');
  const optional = (value: string | null) => value === null ? 'None' : `Some(${quote(value)})`;
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
    pub filter_fields: &'static [&'static str],
}
static DEFINITIONS: &[FoundryReferenceDefinition] = &[
${catalog.definitions.map(definition => `    FoundryReferenceDefinition {
        namespace: ${quote(definition.namespace)},
        identifier: ${quote(definition.identifier)},
        label_key: ${optional(definition.labelKey)},
        description_key: ${optional(definition.descriptionKey)},
        display_label: ${optional(definition.displayLabel)},
        record_key: ${optional(definition.recordKey)},
        expected_family: ${optional(definition.expectedFamily)},
        required_localization_key: ${optional(definition.requiredLocalizationKey)},
        source_file: ${quote(definition.sourceFile)},
        source_line: ${definition.sourceLine},
        excluded: ${definition.excluded},
        filter_fields: &[${definition.filterFields.map(quote).join(', ')}],
    },`).join('\n')}
];
pub fn reference_definitions() -> &'static [FoundryReferenceDefinition] { DEFINITIONS }

`;
}
