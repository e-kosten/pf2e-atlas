import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';

import { compareRules } from '../comparison/compare-rules.js';

async function main() {
  const { values } = parseArgs({ options: { source: { type: 'string' }, graph: { type: 'string' }, summary: { type: 'string' }, out: { type: 'string' }, help: { type: 'boolean', short: 'h' } } });
  if (values.help) { console.log('Usage: npm --prefix dev-tools/source-contracts run compare-rules -- --source PATH --graph PATH --summary PATH --out PATH'); return; }
  if (!values.source || !values.graph || !values.summary || !values.out) throw new Error('--source, --graph, --summary and --out are required');
  const cwd = process.env.INIT_CWD ?? process.cwd();
  const { report, exitCode } = await compareRules({ source: path.resolve(cwd, values.source), graph: path.resolve(cwd, values.graph), summary: path.resolve(cwd, values.summary), out: path.resolve(cwd, values.out) });
  console.log(JSON.stringify({ occurrences: report.occurrences, ...report.counts, families: report.families, unmodeled: report.unmodeled.length, runtimeAdmission: report.runtimeAdmission }, null, 2));
  process.exitCode = exitCode;
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { await main(); } catch (error) { console.error(error instanceof Error ? error.message : String(error)); process.exitCode = 1; }
}
