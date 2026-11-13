import { parseArgs } from 'node:util';
import { argumentPath, cli, repoRoot } from '../release-data.js';
import { commandRunner, planNightly, prepareNightly, publishNightly } from '../nightly.js';

cli(import.meta.url, async (args) => {
  const { values, positionals } = parseArgs({ args, allowPositionals: true, options: {
    tag: { type: 'string' }, sha: { type: 'string' }, dist: { type: 'string' }, coverage: { type: 'string' }, scheduled: { type: 'boolean', default: false },
  } });
  const [command] = positionals;
  if (positionals.length !== 1 || !['plan', 'prepare', 'publish'].includes(command)
    || (command !== 'plan' && !values.tag) || (command === 'publish' && (!values.sha || !values.dist || !values.coverage))) {
    console.error('usage: nightly <plan [--scheduled] | prepare --tag TAG | publish --tag TAG --sha SHA --dist DIR --coverage FILE>');
    return 2;
  }
  if (command === 'plan') {
    const run = commandRunner(repoRoot);
    const id = process.env.GITHUB_RUN_ID;
    const created = id ? run('gh', ['api', `repos/{owner}/{repo}/actions/runs/${id}`, '--jq', '.created_at']) : undefined;
    const date = created ? new Date(created) : undefined;
    if (id && (!date || !Number.isFinite(date.getTime()))) throw new Error('Missing workflow creation timestamp');
    process.stdout.write(planNightly(repoRoot, values.scheduled, run, date));
  }
  else if (command === 'prepare') prepareNightly(repoRoot, values.tag!);
  else await publishNightly(repoRoot, values.tag!, values.sha!, argumentPath(values.dist!), argumentPath(values.coverage!));
  return 0;
});
