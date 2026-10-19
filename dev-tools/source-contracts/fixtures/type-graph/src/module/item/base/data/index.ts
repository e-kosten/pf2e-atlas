export type ItemSourcePF2e = { type: 'equipment'; rules: { key: string; [field: string]: unknown }[] }
  | { type: 'spell'; damage: { formula: string } };
