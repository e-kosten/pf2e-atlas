export type ActorSourcePF2e = { type: 'npc'; items: import('../../item/base/data/index.js').ItemSourcePF2e[] }
  | { type: 'character'; details: { level: number } };
