import type { Location, TraitCatalog } from '../contracts.js';

export interface TraitDefinition {
  identifier: string;
  labelKey: string | null;
  descriptionKey: string | null;
  catalogs: string[];
  sourceLocations: Location[];
}
const compare = (a: string, b: string) => a < b ? -1 : a > b ? 1 : 0;
/** Exact catalog identity and authored mappings; resolved English is deliberately excluded. */
export function traitDefinitions(catalog: TraitCatalog): TraitDefinition[] {
  if (!catalog.complete) throw new Error('Trait metadata extraction is incomplete');
  const definitions = new Map<string, TraitDefinition>();
  for (const group of catalog.catalogs.filter(group => group.namespace === 'trait')) {
    if (!group.complete) throw new Error(`Incomplete trait catalog ${group.name}`);
    for (const entry of group.entries) {
      let definition = definitions.get(entry.identifier);
      if (!definition) {
        definition = { identifier: entry.identifier, labelKey: entry.labelKey,
          descriptionKey: entry.descriptionKey, catalogs: [], sourceLocations: [] };
        definitions.set(entry.identifier, definition);
      } else {
        for (const field of ['labelKey', 'descriptionKey'] as const) {
          if (definition[field] !== entry[field])
            throw new Error(`Conflicting trait ${field} for ${entry.identifier}`);
        }
      }
      if (!definition.catalogs.includes(group.name)) definition.catalogs.push(group.name);
      if (!definition.sourceLocations.some(source => source.file === entry.source.file
        && source.line === entry.source.line && source.column === entry.source.column))
        definition.sourceLocations.push(entry.source);
    }
  }
  return [...definitions.values()].sort((a, b) => compare(a.identifier, b.identifier)).map(definition => ({
    ...definition, catalogs: definition.catalogs.sort(compare),
    sourceLocations: definition.sourceLocations.sort((a, b) => compare(a.file, b.file) || a.line - b.line || a.column - b.column),
  }));
}
export function generateTraitDefinitionModule(definitions: TraitDefinition[], header: string,
  source: { repository: string; commit: string }): string {
  const quote = JSON.stringify;
  const optional = (value: string | null) => value === null ? 'None' : `Some(${quote(value)})`;
  const repository = source.repository.replace(/\.git$/, '');
  return `${header}
// Derived from PF2e system configuration:
// ${repository}/blob/${source.commit}/src/scripts/config/traits.ts
// Upstream copyright: Copyright 2019 Hooking.
// Upstream license: Apache-2.0 (${repository}/blob/${source.commit}/LICENSE).
// Generated transformation: exact trait identifiers, localization-key mappings,
// catalog memberships and source locations. Localized prose is not included.
/// Exact authored vocabulary metadata from the pinned Foundry source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoundryTraitDefinition {
    pub identifier: &'static str,
    pub label_key: Option<&'static str>,
    pub description_key: Option<&'static str>,
    pub catalogs: &'static [&'static str],
    pub source_locations: &'static [FoundryTraitSourceLocation],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoundryTraitSourceLocation {
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}
static DEFINITIONS: &[FoundryTraitDefinition] = &[
${definitions.map(definition => `    FoundryTraitDefinition {
        identifier: ${quote(definition.identifier)},
        label_key: ${optional(definition.labelKey)},
        description_key: ${optional(definition.descriptionKey)},
        catalogs: &[${definition.catalogs.map(value => quote(value)).join(', ')}],
        source_locations: &[${definition.sourceLocations.map(source => `FoundryTraitSourceLocation { file: ${quote(source.file)}, line: ${source.line}, column: ${source.column} }`).join(', ')}],
    },`).join('\n')}
];
pub fn trait_definitions() -> &'static [FoundryTraitDefinition] {
    DEFINITIONS
}
pub fn trait_definition(identifier: &str) -> Option<&'static FoundryTraitDefinition> {
    DEFINITIONS.binary_search_by_key(&identifier, |definition| definition.identifier)
        .ok().map(|index| &DEFINITIONS[index])
}
`;
}
