import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { cli, isFile, repoRoot, type Json } from '../release-data.js';
import { notices } from '../generate-notices.js';

cli(import.meta.url, (args) => {
  const check = args.length === 1 && args[0] === '--check';
  if (args.length && !check) { console.error('usage: npm --prefix dev-tools/release run notices -- [--check]'); return 2; }
  const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--format-version', '1', '--locked'],
    { cwd: repoRoot, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 })) as Json;
  const file = path.join(repoRoot, 'THIRD-PARTY-NOTICES.md');
  const content = notices(metadata);
  if (check) {
    if (!isFile(file) || readFileSync(file, 'utf8') !== content) {
      console.error(`${file} is out of date; run npm --prefix dev-tools/release run notices`); return 1;
    }
    console.log(`${file} is current`);
  } else { writeFileSync(file, content); console.log(file); }
  return 0;
});
