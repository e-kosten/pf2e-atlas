// Validate release-prep contents against their branch point, including on retries.
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { lstatSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { cliVersion, lockPath, manifestPath, versionFiles, writeVersionFiles } from '../../dev-tools/release/src/package-version.mjs';

const git = (...args) => execFileSync('git', args, { encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 });
const read = file => readFileSync(file, 'utf8');

// A user-supplied notes path must stay inside the checkout before the shell
// creates a template or launches an editor (including on an interrupted prep).
function notesPath(input) {
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
    const stat = lstatSync(name, { throwIfNoEntry: false });
    if (stat && (i === parts.length - 1 ? !stat.isFile() : !stat.isDirectory())) {
      throw new Error(`Invalid release notes path: ${name}`);
    }
  }
  const file = normalized.join('/');
  if (!file || path.parse(file).root || !parts.at(-1) || ['.', '..'].includes(parts.at(-1)) || file.startsWith(':')
      || [manifestPath, lockPath].includes(file)) throw invalid();
  return file;
}

function checkPaths(base, notes, committed) {
  const allowed = new Set([lockPath, manifestPath, notes]);
  const status = git('status', '--porcelain=v1', '-z', '--untracked-files=all', '--no-renames');
  for (const entry of status.split('\0').filter(Boolean)) {
    const file = entry.slice(3);
    if (!allowed.has(file)) throw new Error(`Unexpected release-prep change: ${file}`);
    if (entry.startsWith('??') && (lstatSync(file).mode & 0o100)
        && git('config', '--type=bool', '--default', 'true', '--get', 'core.filemode').trim() === 'true') {
      throw new Error(`Unexpected executable release-prep file: ${file}`);
    }
    if (committed) throw new Error(`Already committed release prep has a working-tree change: ${file}`);
    if (entry[0] !== ' ' && entry[1] !== ' ' && !entry.startsWith('??')) {
      throw new Error(`Resolve partially staged changes before preparing the release: ${file}`);
    }
  }
  // The entire branch delta includes already-committed edits, mode changes and
  // staged content. Untracked notes are checked separately via status above.
  const delta = git('diff', '--raw', '-z', '--no-renames', base, '--').split('\0');
  for (let i = 0; i + 1 < delta.length; i += 2) {
    const [metadata, file] = delta.slice(i, i + 2);
    if (!allowed.has(file)) throw new Error(`Unexpected release-prep change: ${file}`);
    if (!/^:(?:100644|000000) 100644 /.test(metadata)) throw new Error(`Unexpected file type or mode change: ${file}`);
  }
}

function prepState(mode, version, inputNotes, main) {
  const head = git('rev-parse', 'HEAD').trim();
  const base = git('merge-base', main, head).trim();
  const committed = head !== base;
  const resuming = mode.startsWith('resume');
  if (committed && (resuming || git('show', '-s', '--format=%P', head).trim() !== base)) {
    throw new Error('Expected an uncommitted release prep or one preparation commit directly on its main branch point');
  }
  const notes = notesPath(inputNotes);
  checkPaths(base, notes, committed);
  const fromBase = file => git('cat-file', '--filters', `--path=${file}`, `${base}:${file}`);
  const source = fromBase(manifestPath);
  const originalLock = fromBase(lockPath);
  if (cliVersion(git('show', `${main}:${manifestPath}`)) !== cliVersion(source)) {
    throw new Error('The atlas-cli version on main changed since preparation began');
  }
  const files = versionFiles(source, originalLock, version);
  for (const [file, expected] of Object.entries(files)) {
    if (!lstatSync(file).isFile()) throw new Error(`Expected a regular file at ${file}`);
    const current = read(file);
    if (current !== expected && !(resuming && current === (file === manifestPath ? source : originalLock))) {
      throw new Error(`Unexpected release-prep contents in ${file}. Review the changes against ${base}`);
    }
  }
  if (resuming) {
    if (mode === 'resume') writeVersionFiles('.', files);
    return;
  }
  if (!lstatSync(notes).isFile()) throw new Error(`Expected a regular release notes file at ${notes}`);
  const content = read(notes);
  if (!content.trim() || /Describe the user-facing release|Mention install, update, setup, or compatibility notes/.test(content)) {
    throw new Error(`Edit the release notes in ${notes} before opening the PR`);
  }
  const digest = createHash('sha256');
  for (const value of [...Object.values(files), content]) digest.update(value).update('\0');
  if (committed ? git('rev-parse', `${base}^{tree}`) === git('rev-parse', `${head}^{tree}`) : !git('status', '--porcelain=v1')) {
    throw new Error('No release-preparation changes relative to the base');
  }
  console.log(`${committed ? 'committed' : 'pending'} ${head} ${digest.digest('hex')}`);
}

function matchingPR(branch, head, repository) {
  const prs = JSON.parse(readFileSync(0, 'utf8'));
  if (!Array.isArray(prs)) throw new Error('Expected an array of GitHub pull requests');
  if (!prs.length) return;
  const [pr] = prs;
  const prefix = repository.replace(/\/$/, '') + '/pull/';
  if (prs.length !== 1 || pr.state !== 'OPEN' || pr.isCrossRepository !== false
      || pr.baseRefName !== 'main' || pr.headRefName !== branch || pr.headRefOid !== head
      || typeof pr.url !== 'string' || !pr.url.startsWith(prefix) || !/^\d+$/.test(pr.url.slice(prefix.length))) {
    throw new Error(`Existing pull request does not match the reviewed ${branch} commit and main base`);
  }
  console.log(pr.url);
}

try {
  const [mode, ...args] = process.argv.slice(2);
  if (mode === 'notes-path') {
    console.log(notesPath(args[0]));
  } else if (mode === 'check' || mode === 'write') {
    notesPath(args[1]);
    const files = versionFiles(read(manifestPath), read(lockPath), args[0]);
    if (mode === 'write') writeVersionFiles('.', files);
  } else if (mode === 'resume' || mode === 'resume-dry-run' || mode === 'candidate') {
    prepState(mode, ...args);
  } else if (mode === 'pr') {
    matchingPR(...args);
  } else {
    throw new Error('Unrecognized release preparation step');
  }
} catch (error) {
  console.error(`error: ${error.message}`);
  process.exitCode = 1;
}
