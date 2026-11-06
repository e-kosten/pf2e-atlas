import type { ReferenceCatalog } from './reference-definitions.js';

/** Emit source evidence only: selected localized prose remains an ingest input. */
export function generateReferenceModule(catalog: ReferenceCatalog, header: string,
  source: { repository: string; commit: string }): string {
  const quote = (value: string) => JSON.stringify(value).replace(/\\u([0-9a-f]{4})/gi, '\\u{$1}');
  const optional = (value: string | null) => value === null ? 'None' : `Some(${quote(value)})`;
  const number = (value: number | null) => value === null ? 'None' : `Some(${value})`;
  return `${header}
// Exact reference vocabulary, record pointers and reviewed template signatures.
// ${source.repository.replace(/\.git$/, '')}/tree/${source.commit}
// Upstream copyright: Copyright 2019 Hooking. License: Apache-2.0.
// Localized game prose is not included in this metadata module.
pub const REFERENCE_CATALOG_SHA256: &str = ${quote(catalog.digest)};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoundryGeneralDefinitionTemplate {
    pub record_key: &'static str,
    pub source_package: &'static str,
    pub description_sha256: &'static str,
    pub rules_sha256: &'static str,
    pub localization_key: &'static str,
    pub localized_start: usize,
    pub localized_end: usize,
    pub parameter_roles: &'static [FoundryDefinitionParameterRole],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoundryDefinitionParameterRole {
    pub role: &'static str,
    pub source_pointer: &'static str,
    pub start: Option<usize>,
    pub end: Option<usize>,
    pub fragment_sha256: Option<&'static str>,
    pub expected_value_json: Option<&'static str>,
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

static GLOSSARY_RECORDS: &[(&str, &str)] = &[
${catalog.records.map(record => `    (${quote(record.key)}, ${quote(record.family)}),`).join('\n')}
];
pub fn foundry_glossary_records() -> &'static [(&'static str, &'static str)] { GLOSSARY_RECORDS }

static TEMPLATES: &[FoundryGeneralDefinitionTemplate] = &[
${catalog.templates.map(template => `    FoundryGeneralDefinitionTemplate {
        record_key: ${quote(template.recordKey)}, source_package: ${quote(template.sourcePackage)},
        description_sha256: ${quote(template.descriptionSha256)}, rules_sha256: ${quote(template.rulesSha256)},
        localization_key: ${quote(template.localizationKey)},
        localized_start: ${template.localizedStart}, localized_end: ${template.localizedEnd},
        parameter_roles: &[
${template.parameterRoles.map(role => `            FoundryDefinitionParameterRole {
                role: ${quote(role.role)}, source_pointer: ${quote(role.sourcePointer)},
                start: ${number(role.start)}, end: ${number(role.end)},
                fragment_sha256: ${optional(role.fragmentSha256)}, expected_value_json: ${optional(role.expectedValueJson)},
            },`).join('\n')}
        ],
    },`).join('\n')}
];
pub fn general_definition_templates() -> &'static [FoundryGeneralDefinitionTemplate] { TEMPLATES }
`;
}
