import { useEffect, useRef, useState } from 'react';
import { Button, Select } from 'antd';
import { RollResult, type Result } from './rolling';

const presets = [
  { label: 'Initiative · d20 + 5', formula: '1d20+5', sides: [20], modifier: 5 },
  { label: 'Damage · 2d6 + 4', formula: '2d6+4', sides: [6, 6], modifier: 4 },
  { label: 'Mixed dice · d8 + d6 + 4', formula: '1d8+1d6+4', sides: [8, 6], modifier: 4 },
  { label: 'Keep higher · 2d20 + 8', formula: '2d20K1+8', sides: [20, 20], modifier: 8, keep: 'high' },
  { label: 'Keep lower · 2d20 + 8', formula: '2d20k1+8', sides: [20, 20], modifier: 8, keep: 'low' },
  { label: 'Large pool · 8d6', formula: '8d6', sides: Array<number>(8).fill(6), modifier: 0 },
];
type Engine = 'custom' | 'three' | 'box';
type State = { pending: boolean; result?: Result; elapsed?: number; error?: string };
type Face = { sides: number; value: number };
// Bounded presentation for these six fixtures only, not a second expression parser.
function browserResult(faces: Face[], preset: typeof presets[number]): Result {
  if (faces.length !== preset.sides.length || faces.some((f, i) => f.sides !== preset.sides[i] || !Number.isInteger(f.value) || f.value < 1 || f.value > f.sides)) throw new Error('Unexpected dice returned by renderer');
  const keepIndex = preset.keep ? faces.findIndex(f => f.value === (preset.keep === 'high' ? Math.max : Math.min)(...faces.map(f => f.value))) : -1;
  const dice = faces.map((f, i) => ({ sides: f.sides, value: f.value, kept: !preset.keep || keepIndex === i }));
  const total = dice.reduce((sum, d) => sum + (d.kept ? d.value : 0), preset.modifier);
  return { label: preset.label, formula: preset.formula, dice, modifier: preset.modifier, total, breakdown: dice.map(d => `d${d.sides}: ${d.value}${d.kept ? '' : ' (dropped)'}`).join(' · ') + `; modifier +${preset.modifier}; total ${total}` };
}
export function PhysicsComparison({ reduced }: { reduced: boolean }) {
  const [selected, setSelected] = useState(1);
  const [ready, setReady] = useState({ three: false, box: false });
  const [states, setStates] = useState<Record<Engine, State>>({ custom: { pending: false }, three: { pending: false }, box: { pending: false } });
  const three = useRef<HTMLIFrameElement>(null);
  const box = useRef<HTMLIFrameElement>(null);
  const current = useRef<{ id: string; started: number; preset: typeof presets[number]; backend?: Result } | undefined>(undefined);
  const timers = useRef<number[]>([]);
  const busy = Object.values(states).some(s => s.pending);
  const update = (engine: Engine, state: State) => setStates(s => ({ ...s, [engine]: state }));
  useEffect(() => { setReady({ three: false, box: false }); }, [reduced]);
  useEffect(() => {
    const receive = (event: MessageEvent) => {
      if (event.origin !== location.origin || event.data?.channel !== 'dice-preview') return;
      const engine = event.source === three.current?.contentWindow ? 'three' : event.source === box.current?.contentWindow ? 'box' : undefined;
      if (!engine) return;
      const data = event.data;
      if (data.kind === 'ready') { setReady(s => ({ ...s, [engine]: true })); return; }
      const request = current.current;
      if (data.kind === 'error') { update(engine, { pending: false, error: 'Renderer unavailable. Reload the comparison to retry.' }); return; }
      if (!request || data.id !== request.id || data.kind !== 'result') return;
      try {
        // Actual library result contracts, checked against the requested dice.
        const faces: Face[] = engine === 'three'
          ? data.result.sets.flatMap((s: { rolls: Face[] }) => s.rolls)
          : [...data.result].sort((a, b) => a.groupId - b.groupId || a.rollId - b.rollId).map((d: Face) => ({ sides: Number(d.sides), value: d.value }));
        const result = browserResult(faces, request.preset);
        if (engine === 'three' && JSON.stringify(result.dice) !== JSON.stringify(request.backend?.dice)) throw new Error('Supplied outcomes did not match');
        update(engine, { pending: false, result: engine === 'three' ? request.backend : result, elapsed: performance.now() - request.started });
      } catch { update(engine, { pending: false, error: 'The renderer returned unexpected faces. No result accepted.' }); }
    };
    window.addEventListener('message', receive);
    return () => { window.removeEventListener('message', receive); timers.current.forEach(clearTimeout); current.current = undefined; };
  }, []);
  async function roll() {
    if (busy) return;
    timers.current.forEach(clearTimeout);
    const preset = presets[selected];
    const id = crypto.randomUUID();
    current.current = { id, preset, started: performance.now() };
    setStates({ custom: { pending: true }, three: { pending: !reduced }, box: { pending: !reduced } });
    if (!reduced) box.current?.contentWindow?.postMessage({ channel: 'dice-preview', id, notation: preset.sides.map(sides => ({ qty: 1, sides })) }, location.origin);
    timers.current = [window.setTimeout(() => {
      setStates(s => Object.fromEntries(Object.entries(s).map(([k, v]) => [k, v.pending ? { pending: false, error: 'Roll timed out. Reload the comparison to retry.' } : v])) as Record<Engine, State>);
    }, 30000)];
    try {
      const response = await fetch('/roll', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ id, groups: [{ label: preset.label, formula: preset.formula }] }) });
      if (!response.ok) throw new Error('Roll failed');
      const result: Result = (await response.json()).groups[0];
      if (current.current?.id !== id) return;
      current.current.backend = result;
      if (!reduced) three.current?.contentWindow?.postMessage({ channel: 'dice-preview', id, notation: `${preset.sides.map(s => `1d${s}`).join('+')}@${result.dice.map(d => d.value).join(',')}` }, location.origin);
      timers.current.push(window.setTimeout(() => update('custom', { pending: false, result, elapsed: performance.now() - current.current!.started }), reduced ? 0 : 700));
    } catch {
      update('custom', { pending: false, error: 'Backend unavailable. Check the prototype server.' });
      update('three', { pending: false, error: 'Backend unavailable. No supplied outcomes.' });
    }
  }
  return <section>
    <div className="section-heading">
      <div><h2>Animation comparison</h2><p className="muted">Same dice request. Custom and ThreeJS share a result; Dice Box rolls independently.</p></div>
      <div className="comparison-controls"><Select aria-label="Comparison dice" value={selected} disabled={busy} onChange={setSelected} options={presets.map((p, i) => ({ label: p.label, value: i }))} /><Button type="primary" disabled={busy || (!reduced && (!ready.three || !ready.box))} onClick={roll}>Roll all</Button></div>
    </div>
    {reduced && <p>Reduced motion is enabled. The custom result remains available; physical animation comparisons are paused.</p>}
    <div className="comparison-grid">
      {(['custom', 'three', 'box'] as const).map(engine => <article className="dice-example" key={engine} data-engine={engine}>
        <h3>{{ custom: 'Lightweight inline', three: 'Dice Box ThreeJS', box: 'Dice Box' }[engine]}</h3>
        <p className="muted">{engine === 'box' ? 'Browser-generated faces · Babylon / Ammo' : engine === 'three' ? 'Backend-supplied faces · ThreeJS / Cannon' : 'Backend-supplied faces · custom SVG / CSS'}</p>
        <div className="comparison-stage">
          {engine === 'custom' ? <RollResult pending={states.custom.pending} result={states.custom.result} sides={presets[selected].sides} animate={!reduced} /> : !reduced ? <iframe ref={engine === 'three' ? three : box} title={`${engine} dice animation`} src={`/renderer.html?engine=${engine}`} /> : null}
        </div>
        {engine !== 'custom' && <div className="comparison-receipt" role="status">{states[engine].result ? <><strong>Total {states[engine].result!.total}</strong><p>{states[engine].result!.breakdown}</p></> : <span className="muted">{states[engine].pending ? 'Rolling…' : ready[engine] ? 'Ready' : reduced ? 'Animation paused' : 'Loading renderer…'}</span>}</div>}
        {states[engine].elapsed !== undefined && <small className="muted">{states[engine].result?.formula} · {(states[engine].elapsed! / 1000).toFixed(2)} s to settled result</small>}
        {states[engine].error && <p role="alert" className="local-error">{states[engine].error}</p>}
      </article>)}
    </div>
    <p className="comparison-note muted">These are real library animations in bounded panels, with sound off. Backend requests and renderer loading add separate costs; times here measure click to settled result, not just calculation. No results are saved to encounters.</p>
  </section>;
}
