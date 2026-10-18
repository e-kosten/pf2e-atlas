import { useEffect, useRef } from 'react';
import { projectedDice } from './projected-dice';

export type DiceMotion = 'tumble' | 'spin' | 'rock';
export type DiceFinish = 'plain' | 'shaded';

const outlines: Record<number, string> = {
  4: 'M24 3 L46 43 L2 43 Z',
  6: 'M4 4 H44 V44 H4 Z',
  8: 'M24 2 L46 24 L24 46 L2 24 Z',
  10: 'M24 2 L46 23 L24 46 L2 23 Z',
  12: 'M24 2 L46 18 L37 44 L11 44 L2 18 Z',
  20: 'M24 2 L44 13 L44 35 L24 46 L4 35 L4 13 Z',
};

// Minimal recognition cues; keep the result area clear and omit shading.
const iconEdges: Record<number, string> = {
  8: 'M24 2 V12 M24 36 V46',
  10: 'M24 2 V10 M24 37 V46',
  12: 'M24 10 L38 20 L33 36 L15 36 L10 20 Z M24 2 V10',
  20: 'M24 2 L4 35 H44 Z',
};

const motions: Record<DiceMotion, Keyframe[]> = {
  tumble: [{transform:'none'}, {transform:'translateY(-4px) rotate(-12deg)',offset:.25}, {transform:'translateY(-2px) rotate(14deg)',offset:.6}, {transform:'none'}],
  spin: [{transform:'rotate(0deg)'}, {transform:'rotate(300deg)',offset:.65}, {transform:'rotate(360deg)'}],
  rock: [{transform:'none'}, {transform:'translateY(-5px) rotate(-18deg) scale(.96)',offset:.25}, {transform:'translateY(1px) rotate(8deg) scale(1.03)',offset:.65}, {transform:'none'}],
};

export function DieFace({ sides, value, pending, animate, motion, finish, unrolled }: {
  sides: number; value?: number; pending: boolean; animate: boolean;
  motion: DiceMotion; finish: DiceFinish; unrolled: boolean;
}) {
  const ref = useRef<HTMLSpanElement>(null);
  const running = useRef<Animation | null>(null);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    if (!animate) { running.current?.cancel(); running.current = null; return; }
    if (pending) {
      running.current?.cancel();
      running.current = el.animate(motions[motion], {duration:650,easing:'cubic-bezier(.25,.1,.25,1)'});
    } else if (running.current?.playState === 'running') {
      // If a response/failure arrives early, settle from the displayed pose.
      const transform = getComputedStyle(el).transform;
      running.current.cancel();
      running.current = el.animate([{transform},{transform:'none'}], {duration:160,easing:'ease-out'});
    }
  }, [pending, animate, motion]);
  useEffect(() => () => running.current?.cancel(), []);
  const solid = projectedDice[sides] ?? projectedDice[20];
  const display = pending ? motion === 'spin' ? '' : '·' : value ?? '';
  return <span ref={ref} className={`die-face finish-${finish} ${unrolled ? 'unrolled' : ''}`}>
    <svg viewBox="0 0 48 48" focusable="false">
      {finish === 'plain' ? (
        <>
          <path d={outlines[sides] ?? outlines[20]} />
          {iconEdges[sides] && <path className="icon-edges" d={iconEdges[sides]} />}
        </>
      ) : <>
        {solid.faces.map((face,i) => <g key={i}>
          <polygon points={face.points} />
          <polygon className="projected-shade" points={face.points} style={{fillOpacity:face.shade}} />
        </g>)}
        <text className="facet-value" x={solid.label[0]} y={solid.label[1]} textAnchor="middle" dominantBaseline="central">{display}</text>
      </>}
    </svg>
    {finish === 'plain' && <span>{display}</span>}
  </span>;
}
