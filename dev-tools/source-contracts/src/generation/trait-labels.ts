import type { TraitCatalog } from '../contracts.js';
/** Key-only evidence from the pinned compiler/catalog extraction, never guessed labels. */
export function traitLabelKeys(catalog: TraitCatalog): Record<string, string> {
  if (!catalog.complete) throw new Error('Trait metadata extraction is incomplete');
  const keys = new Map<string, string>();
  for (const group of catalog.catalogs.filter(group => group.namespace === 'trait')) {
    if (!group.complete) throw new Error(`Incomplete trait catalog ${group.name}`);
    for (const entry of group.entries) {
      if (entry.labelKey === null) continue;
      const previous = keys.get(entry.identifier);
      if (previous !== undefined && previous !== entry.labelKey)
        throw new Error(`Conflicting trait label keys for ${entry.identifier}: ${previous} / ${entry.labelKey}`);
      keys.set(entry.identifier, entry.labelKey);
    }
  }
  return Object.fromEntries([...keys].sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0));
}
export function generateTraitLabelModule(keys: Record<string, string>, header: string): string {
  const entries = Object.entries(keys).sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0);
  return `${header}\n/// Verified localization keys from the pinned Foundry trait catalogs.\n/// Unknown valid authored identifiers remain open and have no fabricated label.\npub fn trait_label_key(identifier: &str) -> Option<&'static str> {\n    match identifier {\n${entries.map(([identifier, key]) => `        ${JSON.stringify(identifier)} => Some(${JSON.stringify(key)}),`).join('\n')}\n        _ => None,\n    }\n}\n`;
}
