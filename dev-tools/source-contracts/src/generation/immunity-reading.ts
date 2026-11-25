import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import ts from 'typescript';
import { conditionId, type ConditionCatalog } from './condition-definitions.js';
import type { ReferenceCatalog } from './reference-definitions.js';
import type { TraitDefinition } from './trait-definitions.js';

export interface ImmunityReading {
  conditions: string[];
  traits: { immunity: string; trait: string }[];
  swarmRuleKey: string;
  swarmRecordKey: string;
}

// Accepted reading targets from IWR.describe. Time has a predicate but its text
// describes time creatures. A trait appearing in only one alternative (Mental,
// for example) does not establish a target for the entire immunity.
const sameTrait = [
  'air', 'alchemical', 'auditory', 'disease', 'earth', 'light', 'metal', 'olfactory',
  'prediction', 'radiation', 'visual', 'water', 'wood',
  'arcane', 'divine', 'occult', 'primal', 'holy', 'unholy', 'emotion',
];
const traitTargets = [
  ...sameTrait.map(trait => ({ immunity: trait, trait })),
  { immunity: 'fear-effects', trait: 'fear' },
  // ActorPF2e.applyDamage checks this immunity in the instantDeath calculation.
  { immunity: 'death-effects', trait: 'death' },
];

async function methodBody(source: string, fileName: string, className: string, methodName: string) {
  const text = await readFile(path.join(source, fileName), 'utf8');
  const file = ts.createSourceFile(fileName, text, ts.ScriptTarget.Latest, true);
  const owner = file.statements.filter(ts.isClassDeclaration).find(node => node.name?.text === className);
  const body = owner?.members.filter(ts.isMethodDeclaration).find(node => node.name.getText(file) === methodName)?.body;
  if (!body) throw new Error(`Missing immunity reading evidence: ${className}.${methodName}`);
  return { file, body };
}

function verifyEvidence(node: ts.Node, file: ts.SourceFile, expected: string, evidence: string) {
  const printer = ts.createPrinter({ removeComments: true, newLine: ts.NewLineKind.LineFeed });
  const text = printer.printNode(ts.EmitHint.Unspecified, node, file);
  const digest = createHash('sha256').update(text).digest('hex');
  if (digest !== expected) throw new Error(`Review immunity reading targets: ${evidence} changed (${digest})`);
}

export async function immunityReading(source: string, references: ReferenceCatalog, conditions: ConditionCatalog,
  traits: readonly TraitDefinition[]): Promise<ImmunityReading> {
  const iwr = await methodBody(source, 'src/module/actor/data/iwr.ts', 'IWR', 'describe');
  verifyEvidence(iwr.body, iwr.file, '23d07017df59ad72d0094970423fbf02526308cfb2074a0107002dd560419ea9', 'IWR.describe');
  const actor = await methodBody(source, 'src/module/actor/base.ts', 'ActorPF2e', 'applyDamage');
  const death = actor.body.statements.filter(ts.isVariableStatement)
    .flatMap(statement => [...statement.declarationList.declarations])
    .find(declaration => declaration.name.getText(actor.file) === 'instantDeath')?.initializer;
  if (!death) throw new Error('Missing ActorPF2e.applyDamage instantDeath calculation');
  verifyEvidence(death, actor.file, '9535f7f79a272e422fc76f041455d61c880291397cd481b0bb452cfafd6e4636',
    'ActorPF2e.applyDamage instantDeath');

  const immunityTypes = new Set(references.definitions.filter(entry => entry.namespace === 'immunity_type')
    .map(entry => entry.identifier));
  const readableTraits = new Set(traits.filter(trait => trait.descriptionKey).map(trait => trait.identifier));
  for (const target of traitTargets) {
    if (!immunityTypes.has(target.immunity) || !readableTraits.has(target.trait))
      throw new Error(`Missing immunity or trait definition: ${target.immunity} -> ${target.trait}`);
  }
  const glossary = references.definitions.find(entry => entry.namespace === 'rule'
    && entry.identifier === 'pf2e:PF2E.NPC.Abilities.Glossary.SwarmMind');
  if (!immunityTypes.has('swarm-mind') || !glossary?.recordKey || glossary.excluded || glossary.expectedFamily !== 'action')
    throw new Error('Missing Swarm Mind glossary mapping');

  return {
    conditions: conditions.conditions.map(entry => entry.slug).filter(slug => immunityTypes.has(slug)),
    traits: traitTargets,
    swarmRuleKey: 'rule:' + glossary.identifier,
    swarmRecordKey: glossary.recordKey,
  };
}

export function generateImmunityReading(reading: ImmunityReading, header: string, source: { repository: string; commit: string }) {
  const quote = (value: string) => JSON.stringify(value);
  return `${header}
// Reviewed immunity reading targets for ${source.repository.replace(/\.git$/, '')}/tree/${source.commit}.
// Evidence: IWR.describe and ActorPF2e.applyDamage (instantDeath).
use super::condition_definitions::ConditionId;

pub const SWARM_MIND_RULE_KEY: &str = ${quote(reading.swarmRuleKey)};
pub const SWARM_MIND_RECORD_KEY: &str = ${quote(reading.swarmRecordKey)};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Condition(&'static str),
    Trait(&'static str),
    SwarmMind,
}

pub fn target(immunity: &str) -> Option<Target> {
    Some(match immunity {
        "swarm-mind" => Target::SwarmMind,
${reading.conditions.map(slug => `        ${quote(slug)} => Target::Condition(${conditionId(slug)}.definition().record_key),`).join('\n')}
${reading.traits.map(({ immunity, trait }) => `        ${quote(immunity)} => Target::Trait(${quote(trait)}),`).join('\n')}
        _ => return None,
    })
}
`;
}
