import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { createWriteStream, closeSync, openSync } from 'node:fs';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { Readable } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { sourceIdentity } from '../discovery/source-identity.js';
import { generateInput, preparePortfolio } from '../generation/generate.js';
import { defaultCache } from '../generation/pinned-source.js';
import { sampleDocuments } from './sample-documents.js';
import { sampleRules, type RulePacket } from './sample-rules.js';
import type { ProbeResult } from './source-probe.js';
import { admissionResults, admissionBaselineMatches, type AdmissionProbeResult } from './admission-results.js';

type Context = Omit<RulePacket, 'source'>;
export interface PortfolioBaseline {
  sourceDigest: string; corpusDigest: string; rejectionDigest: string;
  counts: { occurrences: number; accepted: number; rejected: number; fidelityFailures: number };
}
export function portfolioBaselineMatches(actual: PortfolioBaseline, expected: PortfolioBaseline): boolean {
  return actual.sourceDigest === expected.sourceDigest && actual.corpusDigest === expected.corpusDigest
    && actual.rejectionDigest === expected.rejectionDigest
    && (Object.keys(actual.counts) as (keyof PortfolioBaseline['counts'])[]).every(key => actual.counts[key] === expected.counts?.[key]);
}
export function portfolioResults(contexts: Context[], results: ProbeResult[]) {
  if (contexts.length !== results.length) throw new Error('Portfolio probe result count mismatch');
  const counts = { occurrences: results.length, accepted: 0, rejected: 0, fidelityFailures: 0 };
  const byRoot: Record<string, typeof counts> = Object.create(null);
  const failures: { packet: Context; result: ProbeResult }[] = [];
  results.forEach((result, index) => {
    const packet = contexts[index];
    byRoot[packet.key] ??= { occurrences: 0, accepted: 0, rejected: 0, fidelityFailures: 0 };
    byRoot[packet.key].occurrences++;
    const outcome = result.ok ? 'accepted' : 'rejected';
    counts[outcome]++; byRoot[packet.key][outcome]++;
    if (result.ok && result.fidelity !== null) { counts.fidelityFailures++; byRoot[packet.key].fidelityFailures++; }
    if (!result.ok || result.fidelity !== null) failures.push({ packet, result });
  });
  return { counts, byRoot, failures, rejectionDigest: createHash('sha256').update(JSON.stringify(failures)).digest('hex') };
}

/** Check the maintained crate, rather than generating another scratch parser. */
export async function comparePortfolio(args: { source: string; out: string; baseline?: string; cacheDir?: string; admission?: boolean }) {
  const repo = fileURLToPath(new URL('../../../../..', import.meta.url));
  const overlaps = (a: string, b: string) => {
    const relative = path.relative(path.resolve(a), path.resolve(b));
    return !relative || !relative.startsWith('..' + path.sep) && relative !== '..' && !path.isAbsolute(relative);
  };
  if ([args.source, args.cacheDir ?? defaultCache, ...(args.baseline ? [args.baseline] : []), path.join(repo, 'crates'), path.join(repo, 'dev-tools')]
    .some(input => overlaps(input, args.out) || overlaps(args.out, input))) throw new Error('Comparison output must be separate from source/code inputs');
  await mkdir(args.out, { recursive: true });
  const reportFile = path.join(args.out, 'comparison.json');
  await writeFile(reportFile, JSON.stringify({ status: 'incomplete', runtimeAdmission: 'not-executed' }) + '\n');
  const { input } = await preparePortfolio({ source: args.source, cacheDir: args.cacheDir }), source = await sourceIdentity(args.source);
  if (!input.portfolio || input.source.source_digest !== source.source_digest) throw new Error('Maintained portfolio must describe the same source bytes');
  await generateInput(input, path.join(repo, 'crates/atlas-foundry-model/src/source_model/generated'), true);
  const inputFile = path.join(args.out, 'generation-input.json');
  await writeFile(inputFile, JSON.stringify(input) + '\n');
  const contexts: Context[] = [], digest = createHash('sha256'), packets = path.join(args.out, 'packets.ndjson');
  await pipeline(Readable.from((async function* () {
    for (const stream of [sampleDocuments(args.source, true), sampleRules(args.source)]) {
      for await (const { source: raw, ...packet } of stream) {
        const context: Context = { key: packet.key, context: packet.context };
        const text = JSON.stringify({ ...context, source: raw }) + '\n';
        contexts.push(context); digest.update(text); yield text;
      }
    }
  })()), createWriteStream(packets));
  const target = process.env.CARGO_TARGET_DIR ? path.resolve(process.env.CARGO_TARGET_DIR) : path.join(repo, 'target');
  const build = spawnSync('cargo', ['build', '--offline', '--locked', '-p', 'atlas-foundry-model', '--example', 'source_portfolio_probe', '--target-dir', target], { cwd: repo, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 });
  if (build.error || build.status !== 0) throw new Error(`Maintained probe build failed: ${build.error?.message ?? build.stderr}`);
  const resultFile = path.join(args.out, 'results.ndjson'), stdin = openSync(packets, 'r');
  try {
    const stdout = openSync(resultFile, 'w');
    try {
      const run = spawnSync(path.join(target, 'debug/examples', `source_portfolio_probe${process.platform === 'win32' ? '.exe' : ''}`), [inputFile, ...(args.admission ? ['--admission'] : [])], { stdio: [stdin, stdout, 'pipe'], encoding: 'utf8' });
      if (run.error || run.status !== 0) throw new Error(`Maintained probe failed: ${run.error?.message ?? run.stderr}`);
    } finally { closeSync(stdout); }
  } finally { closeSync(stdin); }
  const results = (await readFile(resultFile, 'utf8')).trim().split('\n').filter(Boolean).map(line => JSON.parse(line));
  if (args.admission) {
    const comparison = admissionResults(contexts, results as AdmissionProbeResult[]);
    const rootIndices = contexts.flatMap((packet,index) => input.portfolio!.schemaRoots.some(root => root.documentKind === packet.key)
      && packet.context.json_path === '$' ? [index] : []);
    const rootDocuments = admissionResults(rootIndices.map(index=>contexts[index]),
      rootIndices.map(index=>results[index] as AdmissionProbeResult)).counts;
    const baseline = { sourceDigest: source.source_digest, corpusDigest: digest.digest('hex'),
      counts: comparison.counts, outcomeDigest: comparison.outcomeDigest };
    const expected = args.baseline ? JSON.parse(await readFile(args.baseline, 'utf8')) as typeof baseline : undefined;
    const baselineMatches = expected ? admissionBaselineMatches(baseline, expected) : null;
    const report = { status: 'complete', profile: 'retained-authored-source', source,
      generatedRoots: input.selection.filter(root => root.documentKind || root.ruleKey).length,
      runtimeAdmission: 'not-executed', ...comparison, rootDocuments, baseline, baselineMatches };
    await writeFile(reportFile, JSON.stringify(report, null, 2) + '\n');
    return { report, exitCode: comparison.counts.rejected || comparison.counts.fidelityFailures || baselineMatches === false ? 1 : 0 };
  }
  const comparison = portfolioResults(contexts, results);
  const baseline = { sourceDigest: source.source_digest, corpusDigest: digest.digest('hex'), counts: comparison.counts, rejectionDigest: comparison.rejectionDigest };
  const expected = args.baseline ? JSON.parse(await readFile(args.baseline, 'utf8')) as PortfolioBaseline : undefined;
  const baselineMatches = expected ? portfolioBaselineMatches(baseline, expected) : null;
  const report = { status: 'complete', profile: 'maintained-authored-input-before-defaults', source,
    generatedRoots: input.selection.filter(root => root.documentKind || root.ruleKey).length,
    runtimeAdmission: 'not-executed', ...comparison, baseline, baselineMatches };
  await writeFile(reportFile, JSON.stringify(report, null, 2) + '\n');
  // Matching known rejections never turns parser failures into accepted input.
  return { report, exitCode: comparison.counts.rejected || comparison.counts.fidelityFailures || baselineMatches === false ? 1 : 0 };
}
