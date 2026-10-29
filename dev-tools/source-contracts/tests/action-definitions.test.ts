import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { actionDefinitions } from '../src/generation/action-definitions.js';

test('registry extraction follows exports, constructor data and variants without executing source', async () => {
  const source = await mkdtemp(path.join(os.tmpdir(), 'atlas-actions-'));
  const folder = path.join(source, 'src/module/system/action-macros');
  await mkdir(folder, { recursive: true });
  try {
    await writeFile(path.join(folder, 'index.ts'), 'import { seek } from "./exports.ts"; const SystemActions: Action[] = [seek, seek]; const SF2eOnlySystemActions = [unrelated];');
    await writeFile(path.join(folder, 'exports.ts'), 'export { action as seek } from "./seek.ts";');
    await writeFile(path.join(folder, 'seek.ts'), 'throw new Error("must not execute"); const PREFIX = "PF2E.Actions.Seek"; class Seek extends Action { constructor() { super({slug:"seek", name:`${PREFIX}.Title`, cost:1, variants:[{slug:"nearby",name:`${PREFIX}.Nearby`}]}); } } const action = new Seek(); export { action };');
    const actions = await actionDefinitions(source);
    assert.deepEqual(actions, [{
      slug: 'seek', name: 'PF2E.Actions.Seek.Title', cost: '1',
      variants: [{slug:'nearby', name:'PF2E.Actions.Seek.Nearby',cost:null}],
      file: 'src/module/system/action-macros/seek.ts', line: 1,
    }]);
    await writeFile(path.join(folder, 'seek.ts'), 'const action = new Action({slug:"seek",name:runtimeLookup(),cost:1}); export { action };');
    await assert.rejects(actionDefinitions(source), /Nonliteral action metadata/);
  } finally { await rm(source, { recursive: true, force: true }); }
});
