import { parseArgs } from 'node:util';
import { argumentPath, cli, repoRoot } from '../release-data.js';
import { planNightly, prepareNightly, publishNightly } from '../nightly.js';

cli(import.meta.url, async (args) => {
  const { values, positionals } = parseArgs({ args, allowPositionals: true, options: {
    tag: { type: 'string' }, sha: { type: 'string' }, dist: { type: 'string' }, scheduled: { type: 'boolean', default: false },
  } });
  const [command] = positionals;
  if (positionals.length !== 1 || !['plan', 'prepare', 'publish'].includes(command)
    || (command !== 'plan' && !values.tag) || (command === 'publish' && (!values.sha || !values.dist))) {
    console.error('usage: nightly <plan [--scheduled] | prepare --tag TAG | publish --tag TAG --sha SHA --dist DIR>');
    return 2;
  }
  if (command === 'plan') process.stdout.write(planNightly(repoRoot, values.scheduled));
  else if (command === 'prepare') prepareNightly(repoRoot, values.tag!);
  else await publishNightly(repoRoot, values.tag!, values.sha!, argumentPath(values.dist!));
  return 0;
});
