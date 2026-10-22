import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';

import { compareTraits } from '../comparison/compare-traits.js';

async function main() {
  const { values } = parseArgs({ options: {
    before: { type: 'string' }, after: { type: 'string' }, out: { type: 'string' }, help: { type: 'boolean', short: 'h' },
  } });
  if (values.help) {
    console.log('Usage: npm --prefix dev-tools/source-contracts run compare-traits -- --before PATH --after PATH --out PATH');
    console.log('Compare two extractions; write OUT/trait-changes.json and OUT/trait-changes.md with before/after wording.');
    return;
  }
  if (!values.before || !values.after || !values.out) throw new Error('--before, --after and --out are required');
  const cwd = process.env.INIT_CWD ?? process.cwd();
  const out = path.resolve(cwd, values.out);
  const report = await compareTraits({ before: path.resolve(cwd, values.before), after: path.resolve(cwd, values.after), out });
  console.log(JSON.stringify({
    before: { version: report.before.source.system_version, traits: report.before.traits, descriptions: report.before.descriptions },
    after: { version: report.after.source.system_version, traits: report.after.traits, descriptions: report.after.descriptions },
    ...report.counts,
    addedIdentifiers: report.added.map(trait => trait.identifier), removedIdentifiers: report.removed.map(trait => trait.identifier),
    changedTraits: report.changed.map(({ identifier, fields }) => ({ identifier, fields })),
    report: path.join(out, 'trait-changes.json'),
    review: path.join(out, 'trait-changes.md'),
  }, null, 2));
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { await main(); } catch (error) { console.error(error instanceof Error ? error.message : String(error)); process.exitCode = 1; }
}
