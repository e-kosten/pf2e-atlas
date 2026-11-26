import { execFileSync } from 'node:child_process';
import { lstatSync } from 'node:fs';
import path from 'node:path';
import { cliVersion, lockPath, manifestPath } from '../../dev-tools/release/src/package-version.mjs';

const git = (...args) => execFileSync('git', args, { encoding: 'utf8' });

// A user-supplied notes path must stay inside the checkout before the shell
// creates a template or launches an editor (including on an interrupted prep).
export function notesPath(input, checkWorkingTree = false) {
  const invalid = () => new Error('Choose a separate repository-relative release notes file');
  if (path.parse(input).root || /[\r\n]/.test(input)) throw invalid();
  const parts = input.split(path.sep).join('/').split('/');
  const normalized = [];
  for (const [i, part] of parts.entries()) {
    if (!part || part === '.') continue;
    if (part === '..') {
      if (!normalized.length) throw invalid();
      normalized.pop();
      continue;
    }
    if (part.toLowerCase() === '.git') throw invalid();
    normalized.push(part);
    const name = normalized.join('/');
    const stat = checkWorkingTree && lstatSync(name, { throwIfNoEntry: false });
    if (stat && (i === parts.length - 1 ? !stat.isFile() : !stat.isDirectory())) {
      throw new Error(`Invalid release notes path: ${name}`);
    }
  }
  const file = normalized.join('/');
  if (!file || path.parse(file).root || !parts.at(-1) || ['.', '..'].includes(parts.at(-1)) || file.startsWith(':')
      || [manifestPath, lockPath].includes(file)) throw invalid();
  return file;
}

// Return an immutable blob ID. The caller materializes its exact bytes when
// creating the draft, independently of the current checkout and text filters.
export function candidateNotes(tag, commit, file) {
  const manifest = git('show', `${commit}:${manifestPath}`);
  if (cliVersion(manifest) !== tag.split('/v')[1]) {
    throw new Error(`Committed atlas-cli version does not match ${tag}`);
  }
  const entry = git('--literal-pathspecs', 'ls-tree', '-z', commit, '--', file);
  const match = /^100(?:644|755) blob ([0-9a-f]{40})\t/.exec(entry);
  if (!match || entry.slice(match[0].length) !== file + '\0') {
    throw new Error(`Expected a tracked regular notes file at ${commit}:${file}`);
  }
  const size = Number(git('cat-file', '-s', match[1]));
  const content = execFileSync('git', ['cat-file', 'blob', match[1]], { encoding: 'utf8', maxBuffer: size });
  if (!content.trim()) throw new Error(`Empty release notes: ${file}`);
  return match[1];
}

// Verify the annotation itself as well as its target. A different tag object
// can point to the same commit while selecting an entirely different file.
export function tagNotes(tag, commit, object, override, allowHistorical = false) {
  let selected;
  if (object !== commit) {
    const raw = git('cat-file', 'tag', object);
    const split = raw.indexOf('\n\n');
    const header = raw.slice(0, split).split('\n');
    if (!header.includes(`object ${commit}`) || !header.includes('type commit') || !header.includes(`tag ${tag}`)) {
      throw new Error(`Tag object does not identify ${tag} at ${commit}`);
    }
    const markers = raw.slice(split + 2).split('\n').filter(line => /^Atlas-Release-Notes\b/i.test(line));
    if (markers.length) {
      const prefix = 'Atlas-Release-Notes: ';
      if (markers.length !== 1 || !markers[0].startsWith(prefix)) throw new Error('Invalid release notes annotation');
      selected = markers[0].slice(prefix.length);
      if (notesPath(selected) !== selected) throw new Error('Invalid release notes path in tag annotation');
    }
  }
  if (!selected && !allowHistorical) throw new Error(`Missing release notes annotation on ${tag}`);
  const requested = override ? notesPath(override) : undefined;
  if (selected && requested && selected !== requested) throw new Error('Selected notes differ from the release tag');
  return candidateNotes(tag, commit, selected ?? requested ?? `docs/releases/${tag.split('/')[1]}.md`);
}

export function remoteTagNotes(tag, commit, expectedObject, override) {
  const ref = `refs/tags/${tag}`;
  const refs = new Map(git('ls-remote', '--exit-code', 'origin', ref, ref + '^{}').trimEnd().split('\n').map(line => {
    const [oid, name] = line.split('\t');
    return [name, oid];
  }));
  const object = refs.get(ref);
  if (!object || (refs.get(ref + '^{}') ?? object) !== commit || (expectedObject && object !== expectedObject)) {
    throw new Error(`Remote tag identity does not match the selected release ${tag} at ${commit}`);
  }
  return tagNotes(tag, commit, object, override, !expectedObject);
}
