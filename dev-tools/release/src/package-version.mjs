// @ts-check
import { mkdirSync, mkdtempSync, readFileSync, renameSync, rmSync, statSync, writeFileSync } from 'node:fs';
import path from 'node:path';

export const manifestPath = 'crates/atlas-cli/Cargo.toml';
export const lockPath = 'Cargo.lock';

/** @param {string} contents @param {string} header @param {string} description */
function packageSection(contents, header, description) {
  const sections = contents.split(/(?=^\[)/m);
  const matches = sections.filter(section => section.split(/\r?\n/, 1)[0] === header
    && /^name = "atlas-cli"\r?$/m.test(section));
  if (matches.length !== 1) throw new Error(`Expected one CLI ${description}`);
  const versions = [...matches[0].matchAll(/^version = "([^"]+)"\r?$/gm)];
  if (versions.length !== 1) throw new Error(`Expected one explicit CLI version in ${description}`);
  return { sections, index: sections.indexOf(matches[0]), version: versions[0][1] };
}

/** @param {string} manifest */
export function cliVersion(manifest) {
  return packageSection(manifest, '[package]', 'package manifest').version;
}

/** @param {string} manifest @param {string} lock @param {string} version */
export function versionFiles(manifest, lock, version) {
  const crate = packageSection(manifest, '[package]', 'package manifest');
  const locked = packageSection(lock, '[[package]]', 'lockfile entry');
  if (crate.version !== locked.version) throw new Error('CLI manifest and lockfile disagree');
  /** @param {ReturnType<typeof packageSection>} section */
  function replace({ sections, index }) {
    sections[index] = sections[index].replace(/^(version = ")[^"]+("\r?)$/m, (_, start, end) => start + version + end);
    return sections.join('');
  }
  return { [manifestPath]: replace(crate), [lockPath]: replace(locked) };
}

/** Both proposed files have been validated before either is replaced.
 * @param {string} root @param {Record<string, string>} files */
export function writeVersionFiles(root, files) {
  // Git ignores .cache in release checkouts. An interrupted pair of renames
  // can therefore resume from its branch point with no untracked prep files.
  const cache = path.join(root, '.cache');
  mkdirSync(cache, { recursive: true });
  const staging = mkdtempSync(path.join(cache, 'atlas-package-version-'));
  try {
    for (const [relative, contents] of Object.entries(files)) {
      const file = path.join(root, relative);
      if (readFileSync(file, 'utf8') === contents) continue;
      const temporary = path.join(staging, path.basename(file));
      writeFileSync(temporary, contents, { flag: 'wx', mode: statSync(file).mode });
      renameSync(temporary, file);
    }
  } finally {
    rmSync(staging, { recursive: true, force: true });
  }
}
