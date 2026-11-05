import { execFileSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { cli, readJson, repoRoot, type Json } from '../release-data.js';
import { notices } from '../generate-notices.js';

cli(import.meta.url, (args) => {
  if (args.length !== 2 || args[0] !== '--output' || !args[1]) {
    console.error('usage: npm --prefix dev-tools/release run notices -- --output <path>');
    return 2;
  }
  const file = path.resolve(args[1]);
  const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--format-version', '1', '--locked'],
    { cwd: repoRoot, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 })) as Json;
  const pf2eNotices = readFileSync(path.join(repoRoot, 'web/atlas-ui/public/third-party-notices.txt'), 'utf8');
  const content = notices(metadata, pf2eNotices, readJson(path.join(repoRoot, 'dev-tools/source-contracts/source-pin.json')));
  mkdirSync(path.dirname(file), { recursive: true });
  writeFileSync(file, content);
  console.log(file);
  return 0;
});
