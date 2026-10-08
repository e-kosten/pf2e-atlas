import { createHash } from 'node:crypto';
import { createWriteStream } from 'node:fs';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { Readable } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import type { ExtractionSummary, TypeGraph } from '../contracts.js';
import { loadGenerationInput } from '../generation/generation-input.js';
import { sourceIdentity } from '../discovery/source-identity.js';
import { sampleDocuments, type DocumentPacket } from './sample-documents.js';
import { sourceProbe, type ProbeResult } from './source-probe.js';
import { documentTraitArrays } from '../generation/document-traits.js';

type Context = Omit<DocumentPacket, 'source'>;
export function compareDocumentResults(packets: Context[], results: ProbeResult[]) {
  if (packets.length !== results.length) throw new Error('Document probe result count mismatch');
  const counts = { occurrences: packets.length, accepted: 0, rejected: 0, fidelityFailures: 0 };
  const families: Record<string, typeof counts> = Object.create(null);
  const failures: { packet: Context; result: ProbeResult }[] = [];
  const grouped = new Map<string, { family: string; pathPattern: string; expected: string; occurrences: number;
    examples: { context: Context['context']; actual: string }[] }>();
  for (const [index, packet] of packets.entries()) {
    const result = results[index], name = `${packet.key}/${packet.family}`;
    families[name] ??= { occurrences: 0, accepted: 0, rejected: 0, fidelityFailures: 0 };
    families[name].occurrences++;
    if (result.ok) { counts.accepted++; families[name].accepted++; }
    else { counts.rejected++; families[name].rejected++; }
    if (result.ok && result.fidelity !== null) { counts.fidelityFailures++; families[name].fidelityFailures++; }
    if (!result.ok || result.fidelity !== null) failures.push({ packet, result });
    if (!result.ok) {
      const pathPattern = result.error.json_path.replace(/\[\d+\]/g, '[]').replace(/\["[^"]+"\]/g, '[*]').replace(/\.(\d+)(?=\.|$)/g, '.*');
      const key = JSON.stringify([name,pathPattern,result.error.expected]);
      if (!grouped.has(key)) grouped.set(key, { family: name, pathPattern, expected: result.error.expected, occurrences: 0, examples: [] });
      const group = grouped.get(key)!; group.occurrences++;
      if (group.examples.length < 3) group.examples.push({ context: packet.context, actual: result.error.actual });
    }
  }
  return { counts, families, failures, rejectionGroups: [...grouped.values()].sort((a,b) => b.occurrences-a.occurrences || a.family.localeCompare(b.family) || a.pathPattern.localeCompare(b.pathPattern)) };
}
const contains = (parent: string, child: string) => {
  const relative = path.relative(parent, child);
  return !relative || relative !== '..' && !relative.startsWith('..' + path.sep) && !path.isAbsolute(relative);
};
export async function compareDocuments(args: { source: string; graph: string; summary: string; out: string; policyManifest?: string }) {
  const repo = fileURLToPath(new URL('../../../../..', import.meta.url));
  if ([args.source, args.graph, args.summary, ...(args.policyManifest ? [args.policyManifest] : []), path.join(repo, 'crates'), path.join(repo, 'scripts'), path.join(repo, 'dev-tools')]
    .some(input => contains(args.out, input) || contains(input, args.out))) throw new Error('Comparison output must be separate from source/code inputs');
  await mkdir(args.out, { recursive: true });
  // A failed repeat must not leave a previous report looking like this run's result.
  await writeFile(path.join(args.out, 'comparison.json'), JSON.stringify({ status: 'incomplete', runtimeAdmission: 'not-executed' }) + '\n');
  const graph = JSON.parse(await readFile(args.graph, 'utf8')) as TypeGraph;
  const summary = JSON.parse(await readFile(args.summary, 'utf8')) as ExtractionSummary;
  const source = await sourceIdentity(args.source);
  if (graph.format !== 'atlas-source-type-graph/v1' || summary.format !== 'atlas-source-extraction/v1'
    || !graph.complete || !summary.complete || graph.typescript !== summary.typescript_version
    || source.source_digest !== summary.source.source_digest || graph.source?.source_digest !== source.source_digest)
    throw new Error('Comparison requires complete extraction of the same source bytes');
  const policy = args.policyManifest ? await loadGenerationInput(args.policyManifest) : undefined;
  if (policy && policy.source.source_digest !== source.source_digest) throw new Error('Policy manifest must describe the same source bytes');
  const openTraitArrays = [...new Set([...(policy?.openTraitArrays ?? []), ...documentTraitArrays(graph)])].sort();
  // Items first establish reusable owners before Actor embedded-item references.
  const ordered = [...graph.roots.filter(root => root.documentKind === 'Item'),
    ...graph.roots.filter(root => root.documentKind === 'Actor'),
    ...graph.roots.filter(root => root.documentKind && !['Actor','Item'].includes(root.documentKind)),
    ...graph.roots.filter(root => root.ruleKey)];
  if (!['Actor','Item'].every(kind => ordered.some(root => root.documentKind === kind))) throw new Error('Actor and Item roots are required');
  const roots = ordered.map((root,index) => ({ key: root.ruleKey ?? root.documentKind!, reference: root.ref!, name: root.ruleKey ? `${root.ruleKey}Rule` : root.name, module: `portfolio/root${index}` }));
  const selection = roots.map(root => ({ name: root.name, declaration: root.reference, valueRef: root.reference, module: root.module, fields: [], deferred: [] }));
  const packetFile = path.join(args.out, 'packets.ndjson');
  const digest = createHash('sha256'), contexts: Context[] = [];
  await pipeline(Readable.from((async function* () {
    for await (const { source: raw, ...packet } of sampleDocuments(args.source)) {
      const text = JSON.stringify({ ...packet, source: raw }) + '\n';
      digest.update(text); contexts.push(packet);
      yield text;
    }
  })()), createWriteStream(packetFile));
  const results = await sourceProbe({ graph, input: { source, nodes: graph.nodes, selection, openTraitArrays },
    roots: roots.filter(root => ['Actor','Item'].includes(root.key)), packets: packetFile, out: path.join(args.out, 'probe'), target: path.join(args.out, 'target') });
  const comparison = compareDocumentResults(contexts, results);
  const unobservedFamilies = (graph.portfolio?.families ?? []).flatMap(family => family.registered
    .filter(name => !comparison.families[`${family.documentKind}/${name}`]).map(name => `${family.documentKind}/${name}`)).sort();
  const report = { status: 'complete', source, corpusDigest: digest.digest('hex'), graphDigest: createHash('sha256').update(await readFile(args.graph)).digest('hex'),
    typescript: graph.typescript, openTraitArrays, generatedRoots: roots.length,
    roots, unobservedFamilies, runtimeAdmission: 'not-executed',
    profile: 'persisted-declarations-before-defaults', ...comparison };
  await writeFile(path.join(args.out, 'comparison.json'), JSON.stringify(report, null, 2) + '\n');
  return { report, exitCode: report.counts.rejected || report.counts.fidelityFailures ? 1 : 0 };
}
