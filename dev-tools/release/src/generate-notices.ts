import { compare, object, type Json } from './release-data.js';

export function notices(metadata: Json, pf2eNotices: string, sourcePin: Json): string {
  if (!pf2eNotices.trim()) throw new Error('PF2e notices are empty; restore web/atlas-ui/public/third-party-notices.txt');
  const pin = object(sourcePin);
  if (typeof pin.commit !== 'string' || typeof pin.systemVersion !== 'string') throw new Error('Expected PF2e source commit and systemVersion');
  if (!pf2eNotices.includes(`Source commit: ${pin.commit} (PF2e ${pin.systemVersion})`)) {
    throw new Error('PF2e metadata notice does not match source-contracts/source-pin.json; update its source commit and system version');
  }
  const packages = object(metadata).packages;
  if (!Array.isArray(packages)) throw new Error('Expected Cargo packages');
  const sorted = packages.map((value) => {
    const item = object(value);
    if (typeof item.name !== 'string' || typeof item.version !== 'string') throw new Error('Expected package name and version');
    return { name: item.name, version: item.version, license: item.license || 'UNKNOWN', source: item.source || 'workspace' };
  }).sort((a, b) => compare(a.name.toLowerCase(), b.name.toLowerCase()) || compare(a.version, b.version));
  return [
    '# Third-Party Notices', '', 'This file is generated for PF2e Atlas release artifacts.',
    'It is intended as release hygiene and is not legal advice.', '', '## Project', '',
    '- PF2e Atlas: MIT', '', '## PF2e-derived Source and Bundled Web Assets', '',
    pf2eNotices.trimEnd(), '', '## Rust Dependencies', '',
    ...sorted.map((item) => `- ${item.name} ${item.version}: ${item.license} (${item.source})`),
    '', '## Bundled Native Libraries', '',
    'PF2e Atlas uses the `ort` Rust crate, which may copy ONNX Runtime native',
    'libraries into release build outputs. ONNX Runtime is published by Microsoft',
    'under the MIT License.', '', '- ONNX Runtime: MIT',
    '  - Project: https://github.com/microsoft/onnxruntime',
    '  - License: https://github.com/microsoft/onnxruntime/blob/main/LICENSE',
    '  - Third-party notices: https://github.com/microsoft/onnxruntime/blob/main/ThirdPartyNotices.txt',
    '', 'If future release artifacts bundle additional native libraries, update this',
    'section before publishing the release.', '',
  ].join('\n');
}
