import path from 'node:path';
import { archiveName, argumentPath, cli, macosDmg, object, readJson, sha256 } from '../release-data.js';

cli(import.meta.url, async (args) => {
  if (args.length !== 3) throw new Error('Expected release tag, source commit and asset directory');
  const [tag, commit, directory] = args;
  const dist = argumentPath(directory);
  const receipt = object(readJson(path.join(dist, `${macosDmg}.receipt.json`)));
  if (receipt.tag !== tag || receipt.commit !== commit
      || receipt.archive !== await sha256(path.join(dist, archiveName('aarch64-apple-darwin', 'full')))
      || receipt.dmg !== await sha256(path.join(dist, macosDmg))) {
    throw new Error('Desktop build receipt does not match the release source and packaged assets');
  }
  return 0;
});
