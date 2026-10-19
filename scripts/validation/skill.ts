import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

// Content-only validation for the current single-file package. Rust package and
// installer changes still run the Rust suite. If the package grows resources,
// extend this check alongside its explicit Rust file manifest.
export function validateSkill(root: string) {
  const skill = resolve(root, 'skills/pf2e-atlas-cli');
  assert.deepEqual(readdirSync(skill).sort(), ['SKILL.md'],
    'Additional skill resources need explicit bundled-file coverage');
  const contents = readFileSync(resolve(skill, 'SKILL.md'), 'utf8');
  const header = /^---\r?\n([\s\S]*?)\r?\n---(?:\r?\n|$)/.exec(contents);
  assert(header, 'Missing skill frontmatter');
  const frontmatter = header[1];
  // Match the flat metadata accepted by the bundled Rust package parser.
  const values = new Map(frontmatter.split(/\r?\n/).map(line => {
    const colon = line.indexOf(':');
    return [line.slice(0, colon).trim(), line.slice(colon + 1).trim().replace(/^"|"$/g, '')];
  }));
  assert.equal(values.get('name'), 'pf2e-atlas-cli', 'Incorrect skill name');
  assert(values.get('description')?.trim(), 'Missing skill description');
  assert(contents.slice(header[0].length).trim(), 'Missing skill instructions');

  const packagePath = resolve(root, 'crates/atlas-cli/src/agent_skills/package.rs');
  const source = readFileSync(packagePath, 'utf8');
  const includes = [...source.matchAll(/include_str!\("([^"]+)"\)/g)]
    .map(match => resolve(dirname(packagePath), match[1]));
  assert(includes.includes(resolve(skill, 'SKILL.md')), 'CLI must embed the checked skill');
  assert.match(source, /relative_path:\s*"SKILL\.md"/, 'Missing bundled skill file entry');
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  validateSkill(fileURLToPath(new URL('../../', import.meta.url)));
  console.log('Bundled skill metadata and file coverage passed.');
}
