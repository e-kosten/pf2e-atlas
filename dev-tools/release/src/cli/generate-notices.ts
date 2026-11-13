import { execFileSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { argumentPath, cli, readJson, repoRoot, type Json } from '../release-data.js';
import { notices } from '../generate-notices.js';

cli(import.meta.url, (args) => {
  if (args.length < 2 || args.length > 3 || args[0] !== '--output' || !args[1]
      || (args.length === 3 && args[2] !== '--desktop')) {
    console.error('usage: npm --prefix dev-tools/release run notices -- --output <path> [--desktop]');
    return 2;
  }
  const file = argumentPath(args[1]);
  const packages = (manifest: string) => {
    const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--format-version', '1', '--locked',
      '--manifest-path', manifest], { cwd: repoRoot, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 })) as { packages: (Record<string, Json> & { id: string })[] };
    return metadata.packages;
  };
  const rows = packages('Cargo.toml');
  if (args[2] === '--desktop') rows.push(...packages('desktop/src-tauri/Cargo.toml'));
  const metadata = { packages: [...new Map(rows.map((row) => [row.id, row])).values()] };
  const pf2eNotices = readFileSync(path.join(repoRoot, 'web/atlas-ui/public/third-party-notices.txt'), 'utf8');
  const content = notices(metadata, pf2eNotices, readJson(path.join(repoRoot, 'dev-tools/source-contracts/source-pin.json')));
  mkdirSync(path.dirname(file), { recursive: true });
  writeFileSync(file, content);
  console.log(file);
  return 0;
});
