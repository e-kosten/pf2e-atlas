import { argumentPath, cli, repoRoot } from '../release-data.js';
import { commandRunner } from '../nightly.js';
import { publishOfficial } from '../official.js';
import { validateAssets } from '../validate-release-assets.js';

cli(import.meta.url, async (args) => {
  const [tag, sha, directory] = args;
  if (args.length !== 3 || !/^v\d+\.\d+\.\d+(?:-[\da-zA-Z][\da-zA-Z.-]*)?$/.test(tag)
      || !/^[a-f0-9]{40}$/.test(sha)) throw new Error('Expected release tag, source commit and asset directory');
  const dist = argumentPath(directory);
  await validateAssets(dist);
  await publishOfficial(tag, sha, dist, commandRunner(repoRoot));
  return 0;
});
