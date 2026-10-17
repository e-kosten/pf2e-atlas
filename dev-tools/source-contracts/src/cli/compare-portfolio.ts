import path from 'node:path';
import { parseArgs } from 'node:util';
import { comparePortfolio } from '../comparison/compare-portfolio.js';

try {
  const { values } = parseArgs({ options: { source: { type: 'string' }, 'cache-dir': { type: 'string' }, out: { type: 'string' }, baseline: { type: 'string' }, admission: { type: 'boolean' }, help: { type: 'boolean', short: 'h' } } });
  if (values.help) console.log('Usage: npm --prefix dev-tools/source-contracts run compare-portfolio -- --source PATH --out PATH [--admission] [--baseline PATH] [--cache-dir PATH]');
  else {
    if (!values.source || !values.out) throw new Error('--source and --out are required');
    const cwd = process.env.INIT_CWD ?? process.cwd();
    const { report, exitCode } = await comparePortfolio({ source: path.resolve(cwd, values.source), cacheDir: values['cache-dir'] ? path.resolve(cwd, values['cache-dir']) : undefined, out: path.resolve(cwd, values.out), baseline: values.baseline ? path.resolve(cwd, values.baseline) : undefined, admission: values.admission });
    console.log(JSON.stringify({ counts: report.counts, rootDocuments: 'rootDocuments' in report ? report.rootDocuments : undefined, baselineMatches: report.baselineMatches, runtimeAdmission: report.runtimeAdmission }, null, 2));
    process.exitCode = exitCode;
  }
} catch (error) { console.error(error instanceof Error ? error.message : String(error)); process.exitCode = 1; }
