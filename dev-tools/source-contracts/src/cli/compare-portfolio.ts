import path from 'node:path';
import { parseArgs } from 'node:util';
import { comparePortfolio } from '../comparison/compare-portfolio.js';

try {
  const { values } = parseArgs({ options: { source: { type: 'string' }, manifest: { type: 'string' }, out: { type: 'string' }, baseline: { type: 'string' }, help: { type: 'boolean', short: 'h' } } });
  if (values.help) console.log('Usage: npm --prefix dev-tools/source-contracts run compare-portfolio -- --source PATH --manifest PATH --out PATH [--baseline PATH]');
  else {
    if (!values.source || !values.manifest || !values.out) throw new Error('--source, --manifest and --out are required');
    const cwd = process.env.INIT_CWD ?? process.cwd();
    const { report, exitCode } = await comparePortfolio({ source: path.resolve(cwd, values.source), manifest: path.resolve(cwd, values.manifest), out: path.resolve(cwd, values.out), baseline: values.baseline ? path.resolve(cwd, values.baseline) : undefined });
    console.log(JSON.stringify({ counts: report.counts, baselineMatches: report.baselineMatches, runtimeAdmission: report.runtimeAdmission }, null, 2));
    process.exitCode = exitCode;
  }
} catch (error) { console.error(error instanceof Error ? error.message : String(error)); process.exitCode = 1; }
