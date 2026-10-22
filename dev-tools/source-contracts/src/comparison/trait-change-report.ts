import type { ComparedTrait, TraitChangeReport } from './compare-traits.js';

function code(value: string) {
  const text = value.replace(/\r/g, '\\r').replace(/\n/g, '\\n');
  const fence = '`'.repeat(Math.max(0, ...[...text.matchAll(/`+/g)].map(match => match[0].length)) + 1);
  return `${fence} ${text} ${fence}`;
}
function textBlock(value: string | null) {
  if (value === null) return '_No resolved text._';
  if (value === '') return '_Empty string._';
  // Authored HTML, macros and Markdown remain literal, including embedded fences.
  const fence = '`'.repeat(Math.max(3, ...[...value.matchAll(/`+/g)].map(match => match[0].length + 1)));
  return `${fence}text\n${value}\n${fence}`;
}
function metadata(value: string | string[] | null) {
  if (value === null) return '_Unmapped._';
  if (Array.isArray(value)) return value.length ? value.map(code).join(', ') : '_None._';
  return code(value);
}
function wholeTrait(trait: ComparedTrait) {
  return [
    `### ${code(trait.identifier)}`, '', `- Label: ${metadata(trait.label)}`,
    `- Label key: ${metadata(trait.labelKey)}`, `- Description key: ${metadata(trait.descriptionKey)}`,
    `- Catalogs: ${metadata(trait.catalogs)}`, '', textBlock(trait.description), '',
  ];
}
export function renderTraitChangeReport(report: TraitChangeReport): string {
  const lines = [
    '# Trait source changes', '',
    `Before: ${metadata(report.before.source.system_version)} (${code(report.before.source.source_digest)}).`,
    `After: ${metadata(report.after.source.system_version)} (${code(report.after.source.source_digest)}).`, '',
    'Counts describe trait vocabulary and catalog bindings, not record occurrences.', '',
    '| Coverage | Before | After |', '| --- | ---: | ---: |',
    `| Traits | ${report.before.traits} | ${report.after.traits} |`,
    `| Nonempty descriptions | ${report.before.descriptions} | ${report.after.descriptions} |`,
    `| Missing descriptions | ${report.before.missingDescriptions} | ${report.after.missingDescriptions} |`,
    `| Catalog memberships | ${report.before.catalogMemberships} | ${report.after.catalogMemberships} |`, '',
    `Added: ${report.counts.added}. Removed: ${report.counts.removed}. Changed: ${report.counts.changed}.`, '',
    '## Added', '',
  ];
  for (const trait of report.added) lines.push(...wholeTrait(trait));
  if (!report.added.length) lines.push('_None._', '');
  lines.push('## Removed', '');
  for (const trait of report.removed) lines.push(...wholeTrait(trait));
  if (!report.removed.length) lines.push('_None._', '');
  lines.push('## Changed', '');
  for (const change of report.changed) {
    lines.push(`### ${code(change.identifier)}`, '', `Changed fields: ${change.fields.map(code).join(', ')}.`, '');
    for (const field of change.fields) {
      if (field === 'description') {
        lines.push('#### Description before', '', textBlock(change.before.description), '',
          '#### Description after', '', textBlock(change.after.description), '');
      } else {
        lines.push(`- ${code(field)} before: ${metadata(change.before[field])}`,
          `- ${code(field)} after: ${metadata(change.after[field])}`);
      }
    }
    lines.push('');
  }
  if (!report.changed.length) lines.push('_None._', '');
  return `${lines.join('\n').trimEnd()}\n`;
}
