import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { validateSkill } from './skill.ts';

test('skill checks reject broken metadata and unbundled resources', () => {
  const root = mkdtempSync(join(tmpdir(), 'atlas-skill-check-'));
  try {
    const skill = join(root, 'skills/pf2e-atlas-cli');
    const packageDir = join(root, 'crates/atlas-cli/src/agent_skills');
    mkdirSync(skill, { recursive: true });
    mkdirSync(packageDir, { recursive: true });
    const valid = '---\nname: pf2e-atlas-cli\ndescription: Look up PF2e records\n---\nUse atlas.\n';
    const packageSource = 'include_str!("../../../../skills/pf2e-atlas-cli/SKILL.md"); relative_path: "SKILL.md"';
    writeFileSync(join(packageDir, 'package.rs'), packageSource);
    for (const invalid of [valid.replace('name: pf2e-atlas-cli', 'name: wrong'),
      valid.replace('description: Look up PF2e records', 'description:'),
      valid.replace('Use atlas.', ''), 'No metadata']) {
      writeFileSync(join(skill, 'SKILL.md'), invalid);
      assert.throws(() => validateSkill(root));
    }
    writeFileSync(join(skill, 'SKILL.md'), valid);
    assert.doesNotThrow(() => validateSkill(root));
    writeFileSync(join(packageDir, 'package.rs'), '');
    assert.throws(() => validateSkill(root), /embed/);
    writeFileSync(join(packageDir, 'package.rs'), packageSource);
    writeFileSync(join(skill, 'reference.md'), 'Unbundled');
    assert.throws(() => validateSkill(root), /bundled-file/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
