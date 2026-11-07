import path from 'node:path';
import { parseArgs } from 'node:util';
import { prepareReferenceSupplementCandidate } from '../comparison/reference-supplement.js';

try {
  const { values } = parseArgs({ options: {
    languages: { type: 'string' }, groups: { type: 'string' }, 'source-commit': { type: 'string' },
    out: { type: 'string' }, help: { type: 'boolean', short: 'h' },
  } });
  if (values.help) {
    console.log('Usage: npm --prefix dev-tools/source-contracts run prepare-reference-supplement -- --languages PATH --groups PATH --source-commit HASH --out SCRATCH');
    console.log('Read local fixed-source JSON and curated notices; emit a review candidate plus text/mapping/removal/notice diffs. Never fetches or updates approved assets.');
  } else {
    if (!values.languages || !values.groups || !values['source-commit'] || !values.out)
      throw new Error('--languages, --groups, --source-commit and --out are required');
    const cwd = process.env.INIT_CWD ?? process.cwd();
    const resolve = (value: string) => path.resolve(cwd, value);
    console.log(JSON.stringify(await prepareReferenceSupplementCandidate({
      languages: resolve(values.languages), groups: resolve(values.groups), sourceCommit: values['source-commit'],
      out: resolve(values.out),
    }), null, 2));
  }
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error)); process.exitCode = 1;
}
