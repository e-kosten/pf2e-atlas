import path from 'node:path';
import { archiveName, argumentPath, cli, desktopAssets, object, readJson, sha256 } from '../release-data.js';

cli(import.meta.url, async (args) => {
  if (args.length !== 3) throw new Error('Expected release tag, source commit and asset directory');
  const [tag, commit, directory] = args;
  const dist = argumentPath(directory);
  for (const { name, target, receiptKey } of desktopAssets(tag)) {
    const receipt = object(readJson(path.join(dist, `${name}.receipt.json`)));
    if (receipt.tag !== tag || receipt.commit !== commit
        || receipt.archive !== await sha256(path.join(dist, archiveName(target, 'full')))
        || receipt[receiptKey] !== await sha256(path.join(dist, name))) {
      throw new Error(`Desktop build receipt for ${name} does not match the release source and packaged assets`);
    }
  }
  return 0;
});
