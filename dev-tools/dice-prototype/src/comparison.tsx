import { useState } from 'react';
import { Button, Select, Segmented } from 'antd';
import { PhysicsComparison } from './physics-comparison';
import { RollResult, useRoll, type Group, type Receipt } from './rolling';

const examples: { label: string; formula: string; sides: number[]; groups?: Group[] }[] = [
  { label: 'Initiative · d20 + 5', formula: '1d20+5', sides: [20] },
  { label: 'Damage · 2d6 + 4', formula: '2d6+4', sides: [6, 6] },
  { label: 'Mixed dice · d8 + d6 + 4', formula: '1d8+1d6+4', sides: [8, 6] },
  { label: 'Fortune · keep higher', formula: '2d20K1+8', sides: [20, 20] },
  { label: 'Misfortune · keep lower', formula: '2d20k1+8', sides: [20, 20] },
  { label: 'Large pool · 8d6', formula: '8d6', sides: Array<number>(8).fill(6) },
  { label: 'All dice types · d4 to d20', formula: '1d4+1d6+1d8+1d10+1d12+1d20', sides: [4,6,8,10,12,20],
    groups: [4,6,8,10,12,20].map(s => ({label: `d${s}`, formula: `1d${s}`})) },
  ...[4, 6, 8, 10, 12, 20].map(s => ({ label: `Single d${s}`, formula: `1d${s}`, sides: [s] })),
];

export function AnimationComparison({ reduced }: { reduced: boolean }) {
  const [mode, setMode] = useState('inline');
  return <>
    <div className="comparison-mode"><Segmented aria-label="Animation family" value={mode} onChange={setMode} options={[{label:'Inline animations',value:'inline'},{label:'Earlier tray experiments',value:'physics'}]} /></div>
    {mode === 'inline' ? <InlineComparison reduced={reduced} /> : <PhysicsComparison reduced={reduced} />}
  </>;
}
function InlineComparison({ reduced }: { reduced: boolean }) {
  const [selected, setSelected] = useState(() => examples.findIndex(example => example.groups !== undefined));
  const [receipt, setReceipt] = useState<Receipt>();
  const preset = examples[selected];
  const op = useRoll({ animate: !reduced, reduced, delay: 0, fail: false }, setReceipt);
  const result = receipt && (receipt.groups.length === 1 ? receipt.groups[0] : {
    label: preset.label, formula: preset.formula, total: receipt.total,
    dice: receipt.groups.flatMap(group => group.dice),
    breakdown: receipt.groups.map(group => group.breakdown).join('; '),
  });
  const busy = op.pending;
  return <section>
    <div className="section-heading">
      <div><h2>Inline animations</h2><p className="muted">One backend roll, six presentations.</p></div>
      <div className="comparison-controls">
        <Select aria-label="Comparison dice" disabled={busy} value={selected} onChange={value => { setSelected(value); setReceipt(undefined); }} options={examples.map((e,i) => ({label:e.label,value:i}))} />
        <Button type="primary" disabled={busy} onClick={() => op.roll(preset.groups ?? [{ label:preset.label, formula:preset.formula }])}>Roll all</Button>
      </div>
    </div>
    <div className="comparison-grid inline-grid">
      {(['plain','shaded'] as const).flatMap(finish =>
        (['spin','tumble','rock'] as const).map(motion =>
          <article key={`${finish}-${motion}`} className="dice-example" data-engine={`${finish}-${motion}`}>
            <h3>{motion[0].toUpperCase() + motion.slice(1)}</h3>
            <p className="muted">{finish === 'plain' ? 'Simple' : 'Shaded polyhedron'}</p>
            <RollResult pending={op.pending} result={result} sides={preset.sides} animate={!reduced} slow={op.slow} motion={motion} finish={finish} />
          </article>
        )
      )}
    </div>
    {op.error && <div role="alert" className="local-error">{op.error} <Button onClick={() => op.roll([],true)}>Retry</Button></div>}
    <p className="comparison-note muted">All six options show the same backend result. Immediate/reduced motion skips animation.</p>
  </section>;
}
